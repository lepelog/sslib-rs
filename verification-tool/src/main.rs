use std::{path::PathBuf, fs, io::Cursor, ffi::OsString};

use nlzss11::decompress;
use u8file::U8File;

fn main() {
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

