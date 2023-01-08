use binrw::binrw;

#[binrw]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RawFlw3 {
    pub typ: u8,
    pub sub_type: u8,
    #[brw(pad_before = 2)]
    pub param1: u16,
    pub param2: u16,
    pub next: i16,
    pub param3: i16,
    pub param4: i16,
    pub param5: i16,
}
