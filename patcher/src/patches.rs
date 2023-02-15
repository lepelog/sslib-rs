use std::fmt::Display;

use bzs::structs::{BzsEntries, OBJ, SOBJ};

use crate::actor_params::{NewSobjShim, ScChangOpts};

pub trait ByIdExt {
    type Out;
    fn remove_by_id(&mut self, id: u16) -> Result<Self::Out, InvalidPatchError>;
    fn modify_by_id(&mut self, id: u16) -> Result<&mut Self::Out, InvalidPatchError>;
}

impl ByIdExt for Vec<OBJ> {
    type Out = OBJ;
    fn remove_by_id(&mut self, id: u16) -> Result<OBJ, InvalidPatchError> {
        let index = self
            .iter()
            .position(|obj| obj.id == id)
            .ok_or(InvalidPatchError::IdNotFound(id))?;
        Ok(self.remove(index))
    }
    fn modify_by_id(&mut self, id: u16) -> Result<&mut OBJ, InvalidPatchError> {
        self.iter_mut()
            .find(|obj| obj.id == id)
            .ok_or(InvalidPatchError::IdNotFound(id))
    }
}

impl ByIdExt for Vec<SOBJ> {
    type Out = SOBJ;
    fn remove_by_id(&mut self, id: u16) -> Result<SOBJ, InvalidPatchError> {
        let index = self
            .iter()
            .position(|obj| obj.id == id)
            .ok_or(InvalidPatchError::IdNotFound(id))?;
        Ok(self.remove(index))
    }
    fn modify_by_id(&mut self, id: u16) -> Result<&mut SOBJ, InvalidPatchError> {
        self.iter_mut()
            .find(|obj| obj.id == id)
            .ok_or(InvalidPatchError::IdNotFound(id))
    }
}

/// Replace new_value in value, by applying the mask after the shift
pub fn mask_shift_set(value: u32, mask: u32, shift: u32, new_value: u32) -> u32 {
    let new_value = new_value & mask;
    (value & !(mask << shift)) | (new_value << shift)
}

pub trait ObjExt {
    fn set_NPC_trigstoryfid(&mut self, flag: u16);
    fn set_NPC_untrigstoryfid(&mut self, flag: u16);
}

pub fn zero_pad<const N: usize>(input: &[u8]) -> [u8; N] {
    let mut out = [0; N];
    for (o, i) in out.iter_mut().zip(input) {
        *o = *i;
    }
    out
}

pub fn do_custom_obj_patch(obj: &mut OBJ, key: &str, value: u32) {
    if obj.name.starts_with(b"Npc") {
        if key == "trigstoryfid" {
            obj.params1 = mask_shift_set(obj.params1, 0x7FF, 10, value);
        } else if key == "untrigstoryfid" {
            obj.params1 = mask_shift_set(obj.params1, 0x7FF, 21, value);
        } else if key == "talk_behaviour" {
            obj.anglez = value as u16
        } else if obj.name.starts_with(b"NpcTke") {
            if key == "trigscenefid" {
                obj.anglex = mask_shift_set(obj.anglex.into(), 0xFF, 0, value) as u16;
            } else if key == "untrigscenefid" {
                obj.anglex = mask_shift_set(obj.anglex.into(), 0xFF, 8, value) as u16;
            } else if key == "subtype" {
                obj.params1 = mask_shift_set(obj.params1, 0xFF, 0, value);
            } else {
                panic!(
                    "ERROR: unsupported key '{}' to patch for object {:?}'",
                    key, obj.name
                );
            }
        } else {
            panic!(
                "ERROR: unsupported key '{}' to patch for object {:?}'",
                key, obj.name
            );
        }
    } else if obj.name.starts_with(b"TBox") {
        if key == "spawnscenefid" {
            obj.params1 = mask_shift_set(obj.params1, 0xFF, 20, value);
        } else if key == "setscenefid" {
            obj.anglex = mask_shift_set(obj.anglex.into(), 0xFF, 0, value) as u16;
        } else if key == "itemid" {
            obj.anglez = mask_shift_set(obj.anglez.into(), 0x1FF, 0, value) as u16;
        } else {
            panic!(
                "ERROR: unsupported key '{}' to patch for object {:?}'",
                key, obj.name
            );
        }
    } else if obj.name.starts_with(b"EvntTag") {
        if key == "trigscenefid" {
            obj.params1 = mask_shift_set(obj.params1, 0xFF, 16, value);
        } else if key == "setscenefid" {
            obj.params1 = mask_shift_set(obj.params1, 0xFF, 8, value);
        } else if key == "event" {
            obj.params1 = mask_shift_set(obj.params1, 0xFF, 0, value);
        } else {
            panic!(
                "ERROR: unsupported key '{}' to patch for object {:?}'",
                key, obj.name
            )
        }
    } else if obj.name.starts_with(b"EvfTag") {
        if key == "trigstoryfid" {
            obj.params1 = mask_shift_set(obj.params1, 0x7FF, 19, value);
        } else if key == "setstoryfid" {
            obj.params1 = mask_shift_set(obj.params1, 0x7FF, 8, value);
        } else if key == "event" {
            obj.params1 = mask_shift_set(obj.params1, 0xFF, 0, value);
        } else {
            panic!(
                "ERROR: unsupported key '{}' to patch for object {:?}'",
                key, obj.name
            );
        }
    } else if obj.name.starts_with(b"ScChang") {
        if key == "trigstoryfid" {
            obj.anglex = mask_shift_set(obj.anglex.into(), 0x7FF, 0, value) as u16;
        } else if key == "untrigstoryfid" {
            obj.anglez = mask_shift_set(obj.anglez.into(), 0x7FF, 0, value) as u16;
        } else if key == "scen_link" {
            obj.params1 = mask_shift_set(obj.params1, 0xFF, 0, value);
        } else if key == "trigscenefid" {
            obj.params1 = mask_shift_set(obj.params1, 0xFF, 24, value);
        } else {
            panic!(
                "ERROR: unsupported key '{}' to patch for object {:?}'",
                key, obj.name
            );
        }
    } else if obj.name.starts_with(b"SwAreaT") {
        if key == "setstoryfid" {
            obj.anglex = mask_shift_set(obj.anglex.into(), 0x7FF, 0, value) as u16;
        } else if key == "unsetstoryfid" {
            obj.anglez = mask_shift_set(obj.anglez.into(), 0x7FF, 0, value) as u16;
        } else if key == "setscenefid" {
            obj.params1 = mask_shift_set(obj.params1, 0xFF, 0, value);
        } else if key == "unsetscenefid" {
            obj.params1 = mask_shift_set(obj.params1, 0xFF, 8, value);
        } else {
            panic!(
                "ERROR: unsupported key '{}' to patch for object {:?}'",
                key, obj.name
            );
        }
    } else {
        panic!(
            "ERROR: unsupported key '{}' to patch for object {:?}'",
            key, obj.name
        );
    }
}

pub fn do_custom_sobj_patch(obj: &mut SOBJ, key: &str, value: u32) {
    if obj.name.starts_with(b"Npc") {
        if key == "trigstoryfid" {
            obj.params1 = mask_shift_set(obj.params1, 0x7FF, 10, value);
        } else if key == "untrigstoryfid" {
            obj.params1 = mask_shift_set(obj.params1, 0x7FF, 21, value);
        } else if key == "talk_behaviour" {
            obj.anglez = value as u16
        } else if obj.name.starts_with(b"NpcTke") {
            if key == "trigscenefid" {
                obj.anglex = mask_shift_set(obj.anglex.into(), 0xFF, 0, value) as u16;
            } else if key == "untrigscenefid" {
                obj.anglex = mask_shift_set(obj.anglex.into(), 0xFF, 8, value) as u16;
            } else if key == "subtype" {
                obj.params1 = mask_shift_set(obj.params1, 0xFF, 0, value);
            } else {
                panic!(
                    "ERROR: unsupported key '{}' to patch for object {:?}'",
                    key, obj.name
                );
            }
        } else {
            panic!(
                "ERROR: unsupported key '{}' to patch for object {:?}'",
                key, obj.name
            );
        }
    } else if obj.name.starts_with(b"TBox") {
        if key == "spawnscenefid" {
            obj.params1 = mask_shift_set(obj.params1, 0xFF, 20, value);
        } else if key == "setscenefid" {
            obj.anglex = mask_shift_set(obj.anglex.into(), 0xFF, 0, value) as u16;
        } else if key == "itemid" {
            obj.anglez = mask_shift_set(obj.anglez.into(), 0x1FF, 0, value) as u16;
        } else {
            panic!(
                "ERROR: unsupported key '{}' to patch for object {:?}'",
                key, obj.name
            );
        }
    } else if obj.name.starts_with(b"EvntTag") {
        if key == "trigscenefid" {
            obj.params1 = mask_shift_set(obj.params1, 0xFF, 16, value);
        } else if key == "setscenefid" {
            obj.params1 = mask_shift_set(obj.params1, 0xFF, 8, value);
        } else if key == "event" {
            obj.params1 = mask_shift_set(obj.params1, 0xFF, 0, value);
        } else {
            panic!(
                "ERROR: unsupported key '{}' to patch for object {:?}'",
                key, obj.name
            )
        }
    } else if obj.name.starts_with(b"EvfTag") {
        if key == "trigstoryfid" {
            obj.params1 = mask_shift_set(obj.params1, 0x7FF, 19, value);
        } else if key == "setstoryfid" {
            obj.params1 = mask_shift_set(obj.params1, 0x7FF, 8, value);
        } else if key == "event" {
            obj.params1 = mask_shift_set(obj.params1, 0xFF, 0, value);
        } else {
            panic!(
                "ERROR: unsupported key '{}' to patch for object {:?}'",
                key, obj.name
            );
        }
    } else if obj.name.starts_with(b"ScChang") {
        if key == "trigstoryfid" {
            obj.anglex = mask_shift_set(obj.anglex.into(), 0x7FF, 0, value) as u16;
        } else if key == "untrigstoryfid" {
            obj.anglez = mask_shift_set(obj.anglez.into(), 0x7FF, 0, value) as u16;
        } else if key == "scen_link" {
            obj.params1 = mask_shift_set(obj.params1, 0xFF, 0, value);
        } else if key == "trigscenefid" {
            obj.params1 = mask_shift_set(obj.params1, 0xFF, 24, value);
        } else {
            panic!(
                "ERROR: unsupported key '{}' to patch for object {:?}'",
                key, obj.name
            );
        }
    } else if obj.name.starts_with(b"SwAreaT") {
        if key == "setstoryfid" {
            obj.anglex = mask_shift_set(obj.anglex.into(), 0x7FF, 0, value) as u16;
        } else if key == "unsetstoryfid" {
            obj.anglez = mask_shift_set(obj.anglez.into(), 0x7FF, 0, value) as u16;
        } else if key == "setscenefid" {
            obj.params1 = mask_shift_set(obj.params1, 0xFF, 0, value);
        } else if key == "unsetscenefid" {
            obj.params1 = mask_shift_set(obj.params1, 0xFF, 8, value);
        } else {
            panic!(
                "ERROR: unsupported key '{}' to patch for object {:?}'",
                key, obj.name
            );
        }
    } else {
        panic!(
            "ERROR: unsupported key '{}' to patch for object {:?}'",
            key, obj.name
        );
    }
}

pub fn find_highest_used_id(bzs: &BzsEntries) -> u16 {
    let mut highest_id = 0;
    for obj in bzs.obj.iter().chain(&bzs.objs).chain(&bzs.door) {
        let id = obj.id & 0x3FF;
        if id != 0x3FF && id > highest_id {
            highest_id = id;
        }
    }
    for lay in &bzs.lay {
        for obj in lay.obj.iter().chain(&lay.objs).chain(&lay.door) {
            let id = obj.id & 0x3FF;
            if id != 0x3FF && id > highest_id {
                highest_id = id;
            }
        }
    }
    for sobj in bzs
        .sobj
        .iter()
        .chain(&bzs.sobs)
        .chain(&bzs.sndt)
        .chain(&bzs.stag)
        .chain(&bzs.stas)
    {
        let id = sobj.id & 0x3FF;
        if id != 0x3FF && id > highest_id {
            highest_id = id;
        }
    }
    for lay in &bzs.lay {
        for sobj in lay
            .sobj
            .iter()
            .chain(&lay.sobs)
            .chain(&lay.sndt)
            .chain(&lay.stag)
            .chain(&lay.stas)
        {
            let id = sobj.id & 0x3FF;
            if id != 0x3FF && id > highest_id {
                highest_id = id;
            }
        }
    }
    highest_id
}

struct Options {}
