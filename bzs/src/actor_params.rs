use crate::{structs::{SOBJ, OBJ}, edit::{zero_pad, ObjActorExt, mask_shift_set}};

pub struct NewObj<'a>(pub &'a mut OBJ);

impl<'a> NewObj<'a> {
    pub fn as_tubo(self) -> TuboOpts<'a> {
        let obj = self.0;
        obj.set_id(0x04_00);
        obj.name = zero_pad(b"Tubo");
        TuboOpts(obj)
    }
    pub fn as_save_obj(self) -> SaveObjOpts<'a> {
        let obj = self.0;
        obj.set_id(0xFC_00);
        obj.name = zero_pad(b"saveObj");
        SaveObjOpts(obj)
    }
}

pub struct TuboOpts<'a>(pub &'a mut OBJ);

impl<'a> TuboOpts<'a> {
    pub fn set_subtype(&mut self, subtype: u8) -> &mut Self {
        self.0.params1 = mask_shift_set(self.0.params1, 0xF, 0, subtype.into());
        self
    }
    pub fn set_drop(&mut self, drop: u8) -> &mut Self {
        self.0.params2 = mask_shift_set(self.0.params2, 0xFF, 0x18, drop.into());
        self
    }
}

impl<'a> ObjActorExt for TuboOpts<'a> {
    fn as_obj(&mut self) -> &mut OBJ {
        self.0
    }
}

pub struct SaveObjOpts<'a>(pub &'a mut OBJ);

impl<'a> SaveObjOpts<'a> {
    pub fn set_subtype(&mut self, subtype: u8) -> &mut Self {
        self.0.params1 = mask_shift_set(self.0.params1, 0xFF, 8, subtype.into());
        self
    }
    pub fn set_exit(&mut self, exit: u8) -> &mut Self {
        self.0.params1 = mask_shift_set(self.0.params1, 0xFF, 0x10, exit.into());
        self
    }
}

impl<'a> ObjActorExt for SaveObjOpts<'a> {
    fn as_obj(&mut self) -> &mut OBJ {
        self.0
    }
}


pub struct NewSobj<'a>(pub &'a mut SOBJ);