use std::{path::PathBuf, fs, io::Cursor, ffi::OsString};

use bzs::structs::{parse_bzs_file, write_bzs};
use nlzss11::{decompress, compress};
use u8file::U8File;

fn main() {
    skyloft_fun();
}

fn skyloft_fun() {
    let mut args = std::env::args().skip(1);
    let vanilla_skyloft_arc = args.next().expect("first arg missing");
    let patched_skyloft_arc = args.next().expect("second arg missing");
    let compr_data = fs::read(&vanilla_skyloft_arc).unwrap();
    let data = decompress(&compr_data).unwrap();
    let mut arc = U8File::from_vec(data).unwrap();
    let mut room_arc = U8File::from_vec(arc.get_entry_data("/rarc/F000_r00.arc").unwrap().to_vec()).unwrap();
    let room_bzs_data = room_arc.get_entry_data("/dat/room.bzs").unwrap();
    let mut room_bzs = parse_bzs_file(&mut Cursor::new(room_bzs_data)).unwrap();
    let mut found_statue = None;
    let lay0objs = &mut room_bzs.lay[0].objs;
    for objs in lay0objs.iter() {
        if objs.name == *b"saveObj\0" {
            found_statue = Some(objs.clone());
            break;
        }
    }
    println!("{found_statue:?}");
    if let Some(mut statue) = found_statue {
        for _ in 0..30 {
            statue.posy += 100f32;
            statue.angley = statue.angley.wrapping_add(0x1000);
            lay0objs.push(statue.clone());
        }
    }
    let mut room_bzs_buf = Vec::new();
    write_bzs(&room_bzs, &mut Cursor::new(&mut room_bzs_buf)).unwrap();
    assert!(room_arc.set_entry_data("/dat/room.bzs", room_bzs_buf));
    let mut room_arc_buf = Vec::new();
    room_arc.write(&mut Cursor::new(&mut room_arc_buf)).unwrap();
    arc.set_entry_data("/rarc/F000_r00.arc", room_arc_buf);
    let mut arc_buf = Vec::new();
    arc.write(&mut Cursor::new(&mut arc_buf)).unwrap();
    let compressed_arc = compress(&arc_buf);
    fs::write(patched_skyloft_arc, &compressed_arc).unwrap();
}

fn roundtrip_all() {
    let game_root = std::env::args().skip(1).next().expect("first argument is the path to the game root");
    let game_root_path = PathBuf::from(game_root);
    let stages_path = game_root_path.join("files/Stage");
    for stage_dir in stages_path.read_dir().expect("iter dir fail") {
        let stage_dir = stage_dir.expect("iter dir fail");
        let dir = stage_dir.file_name();
        println!("doing {dir:?}");
        let mut os_path = dir.clone();
        os_path.push("/");
        os_path.push(&dir);
        os_path.push("_stg_l0.arc.LZ");
        let arc_path = stages_path.join(os_path);
        let compr_data = fs::read(&arc_path).expect("failed to open file");
        let arc_data = decompress(&compr_data).expect("error decompressing");
        let arc = U8File::from_vec(arc_data).expect("failed to parse arc");
        let bzs_data = arc.get_entry_data("dat/stage.bzs").expect("no stage.bzs");
        roundtrip_bzs(bzs_data, &dir);
    }
}

fn roundtrip_bzs(data: &[u8], stage: &OsString) {
    let bzs = bzs::structs::parse_bzs_file(&mut Cursor::new(data)).expect("couldn't read bzs");
    let mut out_buf = Vec::new();
    bzs::structs::write_bzs(&bzs, &mut Cursor::new(&mut out_buf)).expect("failed to write bzs");
    if data != out_buf {
        println!("missmatch for {stage:?}!");
        if let Some(stage) = stage.to_str() {
            fs::write(format!("{stage}-orig.bzs"), data).unwrap();
            fs::write(format!("{stage}-roun.bzs"), out_buf).unwrap();
        }
    }
}

