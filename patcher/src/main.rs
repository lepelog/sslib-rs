use std::{
    collections::HashSet,
    fs::{self, File},
    io::{Cursor, Read},
    path::Path,
    time::{Duration, Instant},
};

use anyhow::{bail, Context};
use bzs::{structs::{parse_bzs_file, write_bzs}, edit::InvalidPatchError};
use patcher_lib::{PatcherFunctions, handle};
use u8file::{Entry, U8File};

use crate::patches_gen::base_stage_patches;

mod checks;
mod checks_gen;
mod eventpatches;
mod options;
// mod patches;
mod patches_gen;
fn main() -> anyhow::Result<()> {
    struct Base;

    impl PatcherFunctions for Base {
        fn stagepatch(&self, stage: patcher_lib::stages::Stage, room: Option<u8>,
                bzs: &mut bzs::structs::BzsEntries,
                oarc_add: &mut HashSet<(u8, &'static str)>,
                oarc_delete: &mut HashSet<(u8, &'static str)>) -> Result<bool, InvalidPatchError> {
            base_stage_patches(&format!("{stage:?}"), room, bzs, oarc_add, oarc_delete).map(|_| true)
        }
    }

    handle(Base)
}
