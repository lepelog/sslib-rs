use std::collections::HashSet;

use bzs::{
    edit::{find_highest_used_id, ByIdExt, ObjActorExt, zero_pad, InvalidPatchError},
    structs::BzsEntries,
};
use patcher_lib::{handle, stages::Stage, PatcherFunctions};

pub fn main() -> anyhow::Result<()> {
    struct Test;

    // breakpoint 802eeffc
    impl PatcherFunctions for Test {
        fn stagepatch(
            &self,
            stage: Stage,
            room: Option<u8>,
            bzs: &mut BzsEntries,
            oarc_add: &mut HashSet<(u8, &'static str)>,
            oarc_delete: &mut HashSet<(u8, &'static str)>,
        ) -> Result<bool, InvalidPatchError> {
            Ok(match (stage, room) {
                (Stage::F000, Some(0)) => {
                    let mut next_id = find_highest_used_id(bzs);
                    for i in 0..3 {
                        bzs.lay[0]
                            .obj
                            .create(&mut next_id)
                            .as_tubo()
                            .set_subtype(0)
                            .set_drop(0x26)
                            .set_pos(-5222f32, 1238f32 + i as f32 * 20f32, -6627f32)
                            .set_angle(0, 1345, 0);
                    }
                    let save_obj_name: [u8; 8] = zero_pad(b"saveObj");
                    for (i, obj) in bzs.lay[0]
                        .objs
                        .iter_mut()
                        .filter(|o| o.name == save_obj_name)
                        .enumerate()
                    {
                        println!("found obj: {obj:?}");
                        obj.as_save_obj().set_exit(i as u8).set_subtype(1);
                        obj.angley = obj.angley.wrapping_add(u16::MAX / 2);
                    }
                    true
                }
                _ => false,
            })
        }
    }
    handle(Test)
}
