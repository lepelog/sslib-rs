use std::{
    fs::{self, File},
    io::{Cursor, Read},
    path::Path,
};

use anyhow::{bail, Context};
use bzs::structs::{parse_bzs_file, write_bzs};
use u8file::{Entry, U8File};

mod checks;
mod eventpatches;
mod options;
mod patches;
mod patches_gen;
mod actor_params;

fn main() -> anyhow::Result<()> {
    let actual_extract_path = Path::new("../../sslib/actual-extract/");
    let modified_extract_path = Path::new("../../sslib/modified-extract/");
    // let oarc_cache = Path::new("../../sslib/oarc/");

    let mut buf = Vec::new();

    // iterate through all layer 0
    for dir in actual_extract_path
        .join("DATA/files/Stage")
        .read_dir()
        .context("Iterating stage failed")?
    {
        let dir = dir.context("unwrap dir failed")?;
        let name = dir.file_name();
        let name_str = name.to_str().unwrap();
        let l0_path = dir.path().join(format!("{}_stg_l0.arc.LZ", name_str));
        println!("Working on {name_str}...");

        // read arc
        buf.clear();
        let mut f = File::open(&l0_path).with_context(|| format!("open {:?} failed", &l0_path))?;
        f.read_to_end(&mut buf)
            .with_context(|| format!("read {:?} failed", &l0_path))?;
        let decompressed = nlzss11::decompress(&buf)
            .with_context(|| format!("decompress {} failed", &name_str))?;
        let mut arc = u8file::U8File::read(&decompressed)
            .with_context(|| format!("read arc {} failed", &name_str))?;

        // first, process the bzs without rooms
        let bzs_data = arc
            .get_entry_data("dat/stage.bzs")
            .with_context(|| format!("stage not found in {}", &name_str))?;
        let mut bzs = parse_bzs_file(&mut Cursor::new(&bzs_data))
            .with_context(|| format!("faied to parse stage bzs {}", &name_str))?;
        patches_gen::base_stage_patches(name_str, None, &mut bzs)
            .with_context(|| format!("failed patched for {:?}", &name_str))?;

        buf.clear();
        write_bzs(&bzs, &mut Cursor::new(&mut buf))
            .with_context(|| format!("writing bzs stage failed {:?}", &l0_path))?;
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
            let mut room_arc = U8File::read(room_arc_data)
                .with_context(|| format!("failed to parse arc room {room_id} {name_str}"))?;
            let room_bzs_data = room_arc
                .get_entry_data("dat/room.bzs")
                .with_context(|| format!("failed to find room bzs in {room_id} {name_str}"))?;
            let mut room_bzs = parse_bzs_file(&mut Cursor::new(&room_bzs_data))
                .with_context(|| format!("failed to parse room bzs in {room_id} {name_str}"))?;

            // patch
            patches_gen::base_stage_patches(name_str, Some(*room_id), &mut room_bzs)
                .with_context(|| format!("patches for {name_str} {room_id} failed"))?;

            // write back
            buf.clear();
            write_bzs(&room_bzs, &mut Cursor::new(&mut buf))
                .with_context(|| format!("writing bzs for {name_str} {room_id} failed"))?;
            room_arc.set_entry_data("dat/room.bzs", buf.clone());

            buf.clear();
            room_arc
                .write(&mut Cursor::new(&mut buf))
                .with_context(|| format!("writing arc for {name_str} {room_id} failed"))?;

            arc.set_entry_data(&room_filename, buf.clone());
        }

        // write arc
        buf.clear();
        arc.write(&mut Cursor::new(&mut buf))
            .with_context(|| format!("writing arc for {name_str} failed!"))?;
        let compressed = nlzss11::compress(&buf);
        let out_path = modified_extract_path.join(format!(
            "DATA/files/Stage/{name_str}/{name_str}_stg_l0.arc.LZ"
        ));
        fs::write(&out_path, &compressed)
            .with_context(|| format!("writing {:?} failed", &out_path))?;
    }
    Ok(())
}
