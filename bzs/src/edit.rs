//! utilities to patch bzs files
//! 
//! 

use crate::{structs::{BzsEntries, OBJ, SOBJ}, actor_params::{NewObj, NewSobj, SaveObjOpts}};



#[derive(Debug, thiserror::Error)]
pub enum InvalidPatchError {
    #[error("Could not find id 0x{0:X}")]
    IdNotFound(u16),
}

pub trait ByIdExt {
    type Out;
    type NewObj<'a> where Self: 'a;
    fn remove_by_id(&mut self, id: u16) -> Result<Self::Out, InvalidPatchError>;
    fn modify_by_id(&mut self, id: u16) -> Result<&mut Self::Out, InvalidPatchError>;
    fn get_by_id(&self, id: u16) -> Result<&Self::Out, InvalidPatchError>;
    fn create<'a>(&'a mut self, next_id: &mut u16) -> Self::NewObj<'a>;
}

impl ByIdExt for Vec<OBJ> {
    type Out = OBJ;
    type NewObj<'a> = NewObj<'a>;
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
    fn get_by_id(&self, id: u16) -> Result<&OBJ, InvalidPatchError> {
        self.iter()
            .find(|obj| obj.id == id)
            .ok_or(InvalidPatchError::IdNotFound(id))
    }
    fn create<'a>(&'a mut self, next_id: &mut u16) -> NewObj<'a> {
        self.push(OBJ::default());
        *next_id += 1;
        let obj = self.last_mut().unwrap();
        obj.id = *next_id;
        obj.params1 = 0xFFFFFFFF;
        obj.params2 = 0xFFFFFFFF;
        NewObj(obj)
    }
}

impl ByIdExt for Vec<SOBJ> {
    type Out = SOBJ;
    type NewObj<'a> = NewSobj<'a>;
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
    fn get_by_id(&self, id: u16) -> Result<&SOBJ, InvalidPatchError> {
        self.iter()
            .find(|obj| obj.id == id)
            .ok_or(InvalidPatchError::IdNotFound(id))
    }
    fn create<'a>(&'a mut self, next_id: &mut u16) -> NewSobj<'a> {
        self.push(SOBJ::default());
        *next_id += 1;
        let sobj = self.last_mut().unwrap();
        sobj.id = *next_id;
        sobj.params1 = 0xFFFFFFFF;
        sobj.params2 = 0xFFFFFFFF;
        sobj.set_size(1f32, 1f32, 1f32);
        NewSobj(sobj)
    }
}

pub trait ObjActorExt {
    fn as_obj(&mut self) -> &mut OBJ;
    fn set_pos(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.as_obj().posx = x;
        self.as_obj().posy = y;
        self.as_obj().posz = z;
        self
    }
    fn set_angle(&mut self, x: u16, y: u16, z: u16) -> &mut Self {
        self.as_obj().anglex = x;
        self.as_obj().angley = y;
        self.as_obj().anglez = z;
        self
    }
    fn set_params1(&mut self, params1: u32) -> &mut Self {
        self.as_obj().params1 = params1;
        self
    }
    fn set_params2(&mut self, params2: u32) -> &mut Self {
        self.as_obj().params2 = params2;
        self
    }
    fn set_name(&mut self, name: [u8; 8]) -> &mut Self {
        self.as_obj().name = name;
        self
    }
    fn set_id(&mut self, id: u16) -> &mut Self {
        self.as_obj().id = (self.as_obj().id & !0xFC00) | (id & 0xFC00);
        self
    }
    fn as_save_obj<'a>(&'a mut self) -> SaveObjOpts<'a> {
        SaveObjOpts(self.as_obj())
    }
}

impl ObjActorExt for OBJ {
    fn as_obj(&mut self) -> &mut OBJ {
        self
    }
}

pub trait SobjActorExt {
    fn as_sobj(&mut self) -> &mut SOBJ;
    fn set_pos(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.as_sobj().posx = x;
        self.as_sobj().posy = y;
        self.as_sobj().posz = z;
        self
    }
    fn set_size(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        self.as_sobj().sizex = x;
        self.as_sobj().sizey = y;
        self.as_sobj().sizez = z;
        self
    }
    fn set_angle(&mut self, x: u16, y: u16, z: u16) -> &mut Self {
        self.as_sobj().anglex = x;
        self.as_sobj().angley = y;
        self.as_sobj().anglez = z;
        self
    }
    fn set_params1(&mut self, params1: u32) -> &mut Self {
        self.as_sobj().params1 = params1;
        self
    }
    fn set_params2(&mut self, params2: u32) -> &mut Self {
        self.as_sobj().params2 = params2;
        self
    }
    fn set_name(&mut self, name: [u8; 8]) -> &mut Self {
        self.as_sobj().name = name;
        self
    }
    fn set_id(&mut self, id: u16) -> &mut Self {
        self.as_sobj().id = (self.as_sobj().id & !0xFC00) | (id & 0xFC00);
        self
    }
}

impl SobjActorExt for SOBJ {
    fn as_sobj(&mut self) -> &mut SOBJ {
        self
    }
}

/// Replace new_value in value, by applying the mask after the shift
pub fn mask_shift_set(value: u32, mask: u32, shift: u32, new_value: u32) -> u32 {
    let new_value = new_value & mask;
    (value & !(mask << shift)) | (new_value << shift)
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

pub fn zero_pad<const N: usize>(input: &[u8]) -> [u8; N] {
    let mut out = [0; N];
    for (o, i) in out.iter_mut().zip(input) {
        *o = *i;
    }
    out
}
