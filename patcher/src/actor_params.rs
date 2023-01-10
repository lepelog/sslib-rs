use bzs::structs::SOBJ;

use crate::patches::mask_shift_set;

pub struct MaskValue<T> {
    pub mask: T,
    pub value: T,
}

impl MaskValue<u32> {
    pub fn apply(&self, out_value: &mut u32) {
        *out_value = (*out_value & !self.mask) | self.value
    }
    pub fn check(&self, value: u32) -> bool {
        (value & self.mask) == self.value
    }
}

impl <T: Default> Default for MaskValue<T> {
    fn default() -> Self {
        Self { mask: Default::default(), value: Default::default() }
    }
}

impl MaskValue<u16> {
    pub fn apply(&self, out_value: &mut u16) {
        *out_value = (*out_value & !self.mask) | self.value
    }
    pub fn check(&self, value: u16) -> bool {
        (value & self.mask) == self.value
    }
}

#[derive(Default)]
pub struct SOBJParamPatch {
    params1: MaskValue<u32>,
    params2: MaskValue<u32>,
    posx: Option<f32>,
    posy: Option<f32>,
    posz: Option<f32>,
    sizex: Option<f32>,
    sizey: Option<f32>,
    sizez: Option<f32>,
    anglex: MaskValue<u16>,
    angley: MaskValue<u16>,
    anglez: MaskValue<u16>,
    id: MaskValue<u16>,
    name: Option<[u8; 8]>,
}

impl SOBJParamPatch {
    pub fn apply(&self, sobj: &mut SOBJ) {
        self.params1.apply(&mut sobj.params1);
        self.params2.apply(&mut sobj.params2);
        if let Some(val) = self.posx {
            sobj.posx = val;
        }
        if let Some(val) = self.posy {
            sobj.posy = val;
        }
        if let Some(val) = self.posz {
            sobj.posz = val;
        }
        if let Some(val) = self.sizex {
            sobj.sizex = val;
        }
        if let Some(val) = self.sizey {
            sobj.sizey = val;
        }
        if let Some(val) = self.sizez {
            sobj.sizez = val;
        }
        self.anglex.apply(&mut sobj.anglex);
        self.angley.apply(&mut sobj.angley);
        self.anglez.apply(&mut sobj.anglez);
        self.id.apply(&mut sobj.id);
        if let Some(val) = self.name {
            sobj.name = val;
        }
    }
}

pub struct ScChangOpts<'a> {
    pub sobj: &'a mut SOBJ,
}

impl<'a> SobjActorExt for ScChangOpts<'a> {
    fn as_sobj(&mut self) -> &mut SOBJ {
        self.sobj
    }
}

impl<'a> ScChangOpts<'a> {
    pub fn set_scen_link(&mut self, scen_link: u8) -> &mut Self {
        self.sobj.params1 = mask_shift_set(self.sobj.params1, 0xFF, 0, scen_link.into());
        self
    }
    pub fn set_trigscenefid(&mut self, flag: u8) -> &mut Self {
        self.sobj.params1 = mask_shift_set(self.sobj.params1, 0xFF, 24, flag.into());
        self
    }
    pub fn set_untrigstoryfid(&mut self, flag: u16) -> &mut Self {
        self.sobj.anglex = mask_shift_set(self.sobj.anglex as u32, 0x7FF, 0, flag.into()) as u16;
        self
    }
    pub fn set_trigstoryfid(&mut self, flag: u16) -> &mut Self {
        self.sobj.anglez = mask_shift_set(self.sobj.anglez as u32, 0x7FF, 0, flag.into()) as u16;
        self
    }
}

pub struct NewSobjShim<'a> {
    pub sobj: &'a mut SOBJ,
}

impl<'a> NewSobjShim<'a> {
    pub fn create_sc_chang(self) -> ScChangOpts<'a> {
        ScChangOpts { sobj: self.sobj }
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
    fn as_sc_chang(&mut self) -> ScChangOpts {
        
        ScChangOpts { sobj: self.as_sobj() }
    }
}

impl SobjActorExt for SOBJ {
    fn as_sobj(&mut self) -> &mut SOBJ {
        self
    }
}
