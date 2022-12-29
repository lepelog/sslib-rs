use binrw::binrw;

#[binrw]
#[derive(Debug, Clone)]
pub struct RelHeader {
    pub id: u32,
    // filled at runtime
    pub next: u32,
    // filled at runtime
    pub prev: u32,
    pub num_sections: u32,
    pub section_info_offset: u32,
    pub name_offset: u32,
    pub name_size: u32,
    pub version: u32,
    pub bss_size: u32,
    pub rel_offset: u32,
    pub imp_offset: u32,
    pub imp_size: u32,
    pub prolog_section: u8,
    pub epilog_section: u8,
    pub unresolved_section: u8,
    // filled at runtime
    pub bss_section: u8,
    pub prolog_offset: u32,
    pub epilog_offset: u32,
    pub unresolved_offset: u32,
    pub align: u32,
    pub bss_align: u32,
    pub fix_size: u32,
}

#[binrw]
#[br(repr = u8)]
#[bw(repr = u8)]
#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum RelocationKind {
    R_PPC_NONE = 0x00,
    R_PPC_ADDR32 = 0x01,
    R_PPC_ADDR24 = 0x02,
    R_PPC_ADDR16 = 0x03,
    R_PPC_ADDR16_LO = 0x04,
    R_PPC_ADDR16_HI = 0x05,
    R_PPC_ADDR16_HA = 0x06,
    R_PPC_ADDR14 = 0x07,
    R_PPC_ADDR14_BRTAKEN = 0x08,
    R_PPC_ADDR14_BRNTAKEN = 0x09,
    R_PPC_REL24 = 0x0A,
    R_PPC_REL14 = 0x0B,
    R_PPC_REL14_BRTAKEN = 0x0C,
    R_PPC_REL14_BRNTAKEN = 0x0D,

    R_DOLPHIN_NOP = 0xC9,
    R_DOLPHIN_SECTION = 0xCA,
    R_DOLPHIN_END = 0xCB,
    R_DOLPHIN_MRKREF = 0xCC,
}

#[binrw]
#[derive(Debug)]
pub(crate) struct RawRelocation {
    pub offset_from_prev: u16,
    pub typ: RelocationKind,
    pub section: u8,
    pub symbol_offset: u32,
}
