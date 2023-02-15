use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::Cursor,
    path::Path,
    str::FromStr, mem::MaybeUninit,
};

use anyhow::{bail, Context};
use bzs::{structs::{parse_bzs_file, write_bzs, BzsEntries}, edit::{ByIdExt, find_highest_used_id, ObjActorExt, InvalidPatchError}};
use clap::Parser;
use disc_riider::{Fst, FstNode};
use log::info;
use stages::Stage;
use u8file::{Entry, U8File};

pub mod stages;
mod cli;

// this needs a better name
pub trait PatcherFunctions {
    /// will be called for every stage, room is none for the stage scoped bzs
    /// 
    /// the oarc_add and oarc_delete maps can be used to manipulate the oarcs of a stage on different layers
    fn stagepatch(&self, stage: Stage, room: Option<u8>,
        bzs: &mut BzsEntries,
        oarc_add: &mut HashSet<(u8, &'static str)>,
        oarc_delete: &mut HashSet<(u8, &'static str)>) -> Result<bool, InvalidPatchError> {
            Ok(false)
        }
}

pub fn handle<F: PatcherFunctions>(f: F) -> anyhow::Result<()> {
    let ctx = cli::Context::parse();
    match ctx {
        cli::Context::FullExtract { vanilla_iso, modified_extract_dir, free_space_bytes, single_stage } => {
            execute(&f, vanilla_iso, modified_extract_dir, free_space_bytes, single_stage)?;
        },
        cli::Context::SingleStage { vanilla_iso: vanilla_iso_path, stage: name_str, free_space_bytes } => {
            let _ = fs::create_dir_all("tmp");

            let mut vanilla_iso = disc_riider::WiiIsoReader::create(
                File::open(vanilla_iso_path).context("could not open vanilla ISO")?,
            )
            .context("could not read vanilla ISO")?;
            let mut data_section = vanilla_iso
                .open_partition_stream(&disc_riider::structs::WiiPartType::Data)
                .context("could not read DATA partition from ISO")?;
            let mut data_reader = data_section.open_encryption_reader();
            let section_header = data_reader
                .read_disc_header()
                .context("could not read vanilla ISO header, it might be corrupted?")?;
            let vanilla_fst = Fst::read(&mut data_reader, *section_header.fst_off)
                .context("couldn't read vanilla ISO filesystem")?;
            
            let layer_0_filename = format!("Stage/{name_str}/{name_str}_stg_l0.arc.LZ");

            let mut buf = Vec::new();
            let mut oarc_add = HashSet::new();
            let mut oarc_delete = HashSet::new();
            let Some(FstNode::File { offset: layer_0_data_offset, length: layer_0_data_length, .. }) = vanilla_fst.find_node_path(&layer_0_filename) else {
                bail!("couldn't find {layer_0_filename}");
            };
            info!("Working on {name_str}...");
            let Ok(stage) = Stage::from_str(&name_str) else {
                bail!("Stage {name_str} is not in the enum?!?!?");
            };
    
            // read arc
            buf.clear();
            data_reader
                .read_into_vec(
                    *layer_0_data_offset,
                    (*layer_0_data_length).into(),
                    &mut buf,
                )
                .context("read failed")?;

            let decompressed_l0 = nlzss11::decompress(&buf)
                .with_context(|| format!("decompress {} failed", name_str))?;
            
            let orig_len = decompressed_l0.len();
    
            let is_modified = handle_single_stage(&mut buf, &decompressed_l0, &name_str, stage, &mut oarc_add, &mut oarc_delete, &f)?;

            if !is_modified {
                bail!("stage is not modified, can't write it");
            }

            if (orig_len + free_space_bytes as usize) < buf.len() {
                bail!("new file is too big (needs {} bytes, has {} bytes)", buf.len(), orig_len + free_space_bytes as usize);
            }

            fs::write(Path::new("tmp/stage.arc"), &buf).context("failed to write tmp/stage.arc")?;
        }
    }
    Ok(())
}

// layer 0 as compressed data in buf
// outfile in buf (if something changed, otherwise the content is unspecified)
fn handle_single_stage<F: PatcherFunctions>(buf: &mut Vec<u8>, decompressed_l0: &[u8], name_str: &str, stage: Stage, oarc_add: &mut HashSet<(u8, &'static str)>, oarc_delete: &mut HashSet<(u8, &'static str)>, f: &F) -> anyhow::Result<bool> {
    let mut is_modified = false;

    let mut arc = u8file::U8File::read(&decompressed_l0)
        .with_context(|| format!("read arc {} failed", name_str))?;

    // first, process the bzs without rooms
    let bzs_data = arc
        .get_entry_data("dat/stage.bzs")
        .with_context(|| format!("stage not found in {}", &name_str))?;
    let mut bzs = parse_bzs_file(&mut Cursor::new(&bzs_data))
        .with_context(|| format!("failed to parse stage bzs {}", &name_str))?;

    if f.stagepatch(stage, None, &mut bzs, oarc_add, oarc_delete)
        .with_context(|| format!("failed patched for {:?}", &name_str))? {
        is_modified = true;
    }

    buf.clear();
    write_bzs(&bzs, &mut Cursor::new(&mut *buf))
        .with_context(|| format!("writing bzs stage failed {:?}", &name_str))?;
    arc.set_entry_data("dat/stage.bzs", buf.clone());

    // process rooms
    let room_dir_files = match arc
        .get_entry("rarc")
        .with_context(|| format!("no rarc in {name_str}"))?
    {
        Entry::DirEntry { files, .. } => files,
        _ => bail!("rarc is not a dir"),
    };
    let mut existing_rooms: Vec<u8> = Vec::with_capacity(10);
    for room_dir_file in room_dir_files {
        let name = room_dir_file.get_name();
        assert!(name.starts_with(name_str));
        // F000_r00.arc
        let room_no = &name[name_str.len() + 2..name.len() - 4];
        existing_rooms.push(
            room_no
                .parse()
                .with_context(|| format!("failed to parse {name} in {name_str}"))?,
        );
    }

    for room_id in &existing_rooms {
        // get bzs
        let room_filename = format!("rarc/{name_str}_r{room_id:02}.arc");
        let room_arc_data = arc.get_entry_data(&room_filename).unwrap();
        let mut room_arc = U8File::read(room_arc_data).with_context(|| {
            format!("failed to parse arc room {room_id} {name_str}")
        })?;
        let room_bzs_data =
            room_arc.get_entry_data("dat/room.bzs").with_context(|| {
                format!("failed to find room bzs in {room_id} {name_str}")
            })?;
        let mut room_bzs = parse_bzs_file(&mut Cursor::new(&room_bzs_data))
            .with_context(|| {
                format!("failed to parse room bzs in {room_id} {name_str}")
            })?;

        // patch
        if f.stagepatch(stage, Some(*room_id), &mut room_bzs, oarc_add, oarc_delete)
            .with_context(|| format!("patches for {name_str} {room_id} failed"))? {
            is_modified = true;
        }

        // write back
        buf.clear();
        write_bzs(&room_bzs, &mut Cursor::new(&mut *buf))
            .with_context(|| format!("writing bzs for {name_str} {room_id} failed"))?;
        room_arc.set_entry_data("dat/room.bzs", buf.clone());

        buf.clear();
        room_arc
            .write(&mut Cursor::new(&mut *buf))
            .with_context(|| format!("writing arc for {name_str} {room_id} failed"))?;

        arc.set_entry_data(&room_filename, buf.clone());
    }

    if is_modified {
        // write arc
        buf.clear();
        arc.write(&mut Cursor::new(&mut *buf))
            .with_context(|| format!("writing arc for {name_str} failed!"))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn execute<F: PatcherFunctions, V: AsRef<Path>, O: AsRef<Path>>(
    f: &F,
    vanilla_iso_path: V,
    out_modified_dir: O,
    free_space_bytes: u32,
    single_stage_name: Option<String>,
) -> anyhow::Result<()> {
    let single_stage = single_stage_name.map(|n| Stage::from_str(&n).with_context(|| format!("stagename {n} is invalid!"))).transpose()?;
    let modified_extract_path = out_modified_dir.as_ref();
    let mut vanilla_iso = disc_riider::WiiIsoReader::create(
        File::open(vanilla_iso_path.as_ref()).context("could not open vanilla ISO")?,
    )
    .context("could not read vanilla ISO")?;
    let mut data_section = vanilla_iso
        .open_partition_stream(&disc_riider::structs::WiiPartType::Data)
        .context("could not read DATA partition from ISO")?;
    let mut data_reader = data_section.open_encryption_reader();
    let section_header = data_reader
        .read_disc_header()
        .context("could not read vanilla ISO header, it might be corrupted?")?;
    let vanilla_fst = Fst::read(&mut data_reader, *section_header.fst_off)
        .context("couldn't read vanilla ISO filesystem")?;

    let FstNode::Directory { files: stage_dirs, .. } = vanilla_fst.find_node_path("Stage").context("can't find the Stage directory, is this a Skyward Sword ISO?")? else {
        bail!("stages not a directory");
    };
    // let oarc_cache = Path::new("../../sslib/oarc/");

    let mut buf = Vec::new();
    for stage_dir in stage_dirs {
        let mut oarc_add = HashSet::new();
        let mut oarc_delete = HashSet::new();
        let FstNode::Directory { name: name_str, files: stage_layer_files } = stage_dir else {
            bail!("no stage dir");
        };
        info!("Working on {name_str}...");
        let Ok(stage) = Stage::from_str(&name_str) else {
            bail!("Stage {name_str} is not in the enum?!?!?");
        };

        if let Some(single_stage) = single_stage {
            if stage != single_stage {
                continue;
            }
        }
        let l0_name = format!("{}_stg_l0.arc.LZ", name_str);
        let Some(FstNode::File { offset: layer_0_data_offset, length: layer_0_data_length, .. }) = stage_layer_files.iter().find(|f| f.get_name() == &l0_name) else {
            bail!("couldn't find l0");
        };

        // read arc
        buf.clear();
        data_reader
            .read_into_vec(
                *layer_0_data_offset,
                (*layer_0_data_length).into(),
                &mut buf,
            )
            .context("read failed")?;
        // let mut f = File::open(&l0_path).with_context(|| format!("open {:?} failed", &l0_path))?;
        // f.read_to_end(&mut buf)
        //     .with_context(|| format!("read {:?} failed", &l0_path))?;

        let decompressed_l0 = nlzss11::decompress(&buf)
            .with_context(|| format!("decompress {} failed", name_str))?;

        let is_modified = handle_single_stage(&mut buf, &decompressed_l0, name_str, stage, &mut oarc_add, &mut oarc_delete, f)?;

        // TODO: copying could be done cheaper, if that's needed in the future
        if is_modified {
            if free_space_bytes > 0 {
                buf.resize(buf.len() + free_space_bytes as usize, 0);
            }
            let mut compressed = nlzss11::compress(&buf);
            let out_path = modified_extract_path.join(format!(
                "DATA/files/Stage/{name_str}/{name_str}_stg_l0.arc.LZ"
            ));
            // if single_stage.is_some() {
            //     if out_path.exists() {
            //         let target_filesize = out_path.metadata().context("failed to get metadata of out stage")?.len() as usize;
            //         if target_filesize > compressed.len() {
            //             bail!("new file is bigger than the old one, not replacing in single stage mode");
            //         }
            //         compressed.resize(target_filesize, 0);
            //     }
            // }
            fs::write(&out_path, &compressed)
                .with_context(|| format!("writing {:?} failed", &out_path))?;
        }
    }
    Ok(())
}
