use std::io::{Seek, SeekFrom, Write};

use binrw::{binrw, BinRead, BinReaderExt, BinWriterExt, Endian, ReadOptions};
use sslib_proc::derive_patch_match_struct;

use crate::encoding::{write_nul_term_shift_jis, NulTermShiftJis};

#[binrw]
#[derive_patch_match_struct]
#[derive(Debug, Clone, Default)]
pub struct FILE {
    pub unk: i16,
    pub dummy: i16,
}

#[binrw]
#[derive_patch_match_struct]
#[derive(Debug, Clone, Default)]
pub struct SCEN {
    pub name: [u8; 32],
    pub room: u8,
    pub layer: u8,
    pub entrance: u8,
    pub night: u8,
    pub byte5: u8,
    pub flag6: u8,
    // always 0
    pub zero: u8,
    pub saveprompt: u8,
}

#[binrw]
#[derive_patch_match_struct]
#[derive(Debug, Clone, Default)]
pub struct CAM {
    pub unk1: u32,
    pub posx: f32,
    pub posy: f32,
    pub posz: f32,
    pub angle: f32,
    pub unk2: [u8; 8],
    pub name: [u8; 16],
}

#[binrw]
#[derive_patch_match_struct]
#[derive(Debug, Clone, Default)]
pub struct PATH {
    pub unk1: [u8; 2],
    pub pnt_start_idx: u16,
    pub pnt_total_count: u16,
    pub unk2: [u8; 6],
}

#[binrw]
#[derive_patch_match_struct]
#[derive(Debug, Clone, Default)]
pub struct SPTH {
    pub unk1: [u8; 2],
    pub pnt_start_idx: u16,
    pub pnt_total_count: u16,
    pub unk2: [u8; 6],
}

#[binrw]
#[derive_patch_match_struct]
#[derive(Debug, Clone, Default)]
pub struct PNT {
    posx: f32,
    posy: f32,
    posz: f32,
    unk: u32,
}

#[binrw]
#[derive_patch_match_struct]
#[derive(Debug, Clone, Default)]
pub struct SPNT {
    posx: f32,
    posy: f32,
    posz: f32,
    unk: u32,
}

#[binrw]
#[derive_patch_match_struct]
#[derive(Debug, Clone, Default)]
pub struct BPNT {
    pos1x: f32,
    pos1y: f32,
    pos1z: f32,
    pos2x: f32,
    pos2y: f32,
    pos2z: f32,
    pos3x: f32,
    pos3y: f32,
    pos3z: f32,
    unk: [u8; 4],
}

#[binrw]
#[derive_patch_match_struct]
#[derive(Debug, Clone, Default)]
pub struct AREA {
    pub posx: f32,
    pub posy: f32,
    pub posz: f32,
    pub sizex: f32,
    pub sizey: f32,
    pub sizez: f32,
    pub angley: u16,
    pub area_link: i16,
    pub unk3: u8,
    pub dummy: [u8; 3],
}

#[binrw]
#[derive_patch_match_struct]
#[derive(Debug, Clone, Default)]
pub struct EVNT {
    pub unk1: [u8; 2],
    pub storyflag1: i16,
    pub storyflag2: i16,
    pub unk2: [u8; 3],
    pub exit_id: u8,
    pub unk3: u8,
    pub sceneflag1: u8,
    pub sceneflag2: u8,
    pub skipflag: u8,
    pub dummy1: i16,
    pub item: i16,
    pub dummy2: i16,
    pub name: [u8; 32],
}

#[binrw]
#[derive_patch_match_struct]
#[derive(Debug, Clone, Default)]
pub struct PLY {
    pub storyflag: i16,
    pub play_cutscene: i8,
    pub byte4: u8,
    pub posx: f32,
    pub posy: f32,
    pub posz: f32,
    pub anglex: f32,
    pub angley: f32,
    pub anglez: f32,
    pub entrance_id: i16,
}

#[binrw]
#[derive_patch_match_struct]
#[derive(Debug, Clone, Default)]
pub struct LYSE {
    pub storyflag: i16,
    pub night: i8,
    pub layer: i8,
}

#[binrw]
#[derive_patch_match_struct]
#[derive(Debug, Clone, Default)]
pub struct STIF {
    pub wtf1: f32,
    pub wtf2: f32,
    pub wtf3: f32,
    pub byte1: i8,
    pub flagindex: i8,
    pub byte3: i8,
    pub byte4: i8,
    pub unk1: [u8; 2],
    pub map_name_id: i8,
    pub unk2: u8,
}

#[binrw]
#[derive_patch_match_struct]
#[derive(Debug, Clone, Default)]
pub struct PCAM {
    pub pos1x: f32,
    pub pos1y: f32,
    pub pos1z: f32,
    pub pos2x: f32,
    pub pos2y: f32,
    pub pos2z: f32,
    pub angle: f32,
    pub wtf: f32,
    pub unk: [u8; 4],
}

#[binrw]
#[derive_patch_match_struct]
#[derive(Debug, Clone, Default)]
pub struct LYLT {
    pub layer: i8,
    pub demo_high: i8,
    pub demo_low: i8,
    pub dummy: i8,
}

#[binrw]
#[derive_patch_match_struct]
#[derive(Debug, Clone, Default)]
pub struct SOBJ {
    pub params1: u32,
    pub params2: u32,
    pub posx: f32,
    pub posy: f32,
    pub posz: f32,
    pub sizex: f32,
    pub sizey: f32,
    pub sizez: f32,
    pub anglex: u16,
    pub angley: u16,
    pub anglez: u16,
    pub id: u16,
    pub name: [u8; 8],
}

#[binrw]
#[derive_patch_match_struct]
#[derive(Debug, Clone, Default)]
pub struct OBJ {
    pub params1: u32,
    pub params2: u32,
    pub posx: f32,
    pub posy: f32,
    pub posz: f32,
    pub anglex: u16,
    pub angley: u16,
    pub anglez: u16,
    pub id: u16,
    pub name: [u8; 8],
}

#[derive(Debug, Default, Clone)]
pub struct RMPL {
    room: u8,
    data: Vec<u16>,
}

impl BinRead for RMPL {
    // base_addr
    type Args = u64;

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        options: &binrw::ReadOptions,
        base_addr: Self::Args,
    ) -> binrw::BinResult<Self> {
        let rmpl_id = 0;
        reader.read_exact(&mut [rmpl_id])?;
        let count = 0;
        reader.read_exact(&mut [count])?;
        let offset: u16 = reader.read_type(options.endian())?;
        let pos = reader.stream_position()?;

        reader.seek(SeekFrom::Start(base_addr + offset as u64))?;
        let mut data = Vec::with_capacity(count as usize);
        for _ in 0..count {
            data.push(reader.read_type::<u16>(options.endian())?);
        }

        // return to where we finished reading the RMPL info
        reader.seek(SeekFrom::Start(pos))?;
        Ok(RMPL {
            room: rmpl_id,
            data,
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct BzsEntries {
    pub file: Option<FILE>,
    pub stif: Option<STIF>,
    pub arcn: Vec<String>,
    pub objn: Vec<String>,
    pub lylt: Vec<LYLT>,
    pub lyse: Vec<LYSE>,
    pub scen: Vec<SCEN>,
    pub cam: Vec<CAM>,
    pub pcam: Vec<PCAM>,
    pub path: Vec<PATH>,
    pub pnt: Vec<PNT>,
    pub spnt: Vec<SPNT>,
    pub bpnt: Vec<BPNT>,
    pub spth: Vec<SPTH>,
    pub area: Vec<AREA>,
    pub evnt: Vec<EVNT>,
    pub ply: Vec<PLY>,
    pub rmpl: Vec<RMPL>,
    pub objs: Vec<OBJ>,
    pub obj: Vec<OBJ>,
    pub door: Vec<OBJ>,
    pub sobj: Vec<SOBJ>,
    pub sobs: Vec<SOBJ>,
    pub stas: Vec<SOBJ>,
    pub stag: Vec<SOBJ>,
    pub sndt: Vec<SOBJ>,
    pub lay: Vec<BzsEntries>,
}

#[derive(Debug)]
pub enum BzsEntry {
    FILE(FILE),
    STIF(STIF),
    ARCN(Vec<String>),
    OBJN(Vec<String>),
    LYLT(Vec<LYLT>),
    LYSE(Vec<LYSE>),
    SCEN(Vec<SCEN>),
    CAM(Vec<CAM>),
    PCAM(Vec<PCAM>),
    PATH(Vec<PATH>),
    PNT(Vec<PNT>),
    SPNT(Vec<SPNT>),
    BPNT(Vec<BPNT>),
    SPTH(Vec<SPTH>),
    AREA(Vec<AREA>),
    EVNT(Vec<EVNT>),
    PLY(Vec<PLY>),
    RMPL(Vec<RMPL>),
    OBJS(Vec<OBJ>),
    OBJ(Vec<OBJ>),
    DOOR(Vec<OBJ>),
    SOBJ(Vec<SOBJ>),
    SOBS(Vec<SOBJ>),
    STAS(Vec<SOBJ>),
    STAG(Vec<SOBJ>),
    SNDT(Vec<SOBJ>),
    // layers
    LAY(Vec<Vec<BzsEntry>>),
}

#[binrw]
struct BzsSuperEntry {
    name: u32,
    count: u16,
    ff: u16,
    offset: u32,
}

#[binrw]
struct LayEntry {
    count: u16,
    ff: u16,
    offset: u32,
}

#[binrw]
struct RmplDef {
    rmpl_id: u8,
    count: u8,
    offset: u16,
}

pub fn parse_bzs_file<R: std::io::Read + std::io::Seek>(
    reader: &mut R,
) -> binrw::BinResult<BzsEntries> {
    let root_entry: BzsSuperEntry = reader.read_be()?;
    parse_bzs_entries2(reader, root_entry.count as u64, root_entry.offset as u64)
}

pub fn parse_bzs_entries2<R: std::io::Read + std::io::Seek>(
    reader: &mut R,
    entrycount: u64,
    offset: u64,
) -> binrw::BinResult<BzsEntries> {
    let mut entries = BzsEntries::default();
    for i in 0..entrycount {
        let pos = offset + i * 12;
        reader.seek(SeekFrom::Start(pos))?;
        let entrydef: BzsSuperEntry = reader.read_be()?;
        let binrw_error = |s: String| binrw::Error::Custom {
            pos,
            err: Box::new(s),
        };
        reader.seek(SeekFrom::Start(pos + entrydef.offset as u64))?;
        match entrydef.name {
            0x46494c45 /* FILE */ => {
                if entrydef.count != 1 {
                    return Err(binrw_error(format!("expected one FILE entry, got {}", entrydef.count)));
                }
                entries.file = Some(reader.read_be()?);
            },
            0x5343454e /* SCEN */ => {
                entries.scen = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
            },
            0x43414d20 /* CAM  */ => {
                entries.cam = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
            },
            0x50415448 /* PATH */ => {
                entries.path = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
            },
            0x504e5420 /* PNT  */ => {
                entries.pnt = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
            },
            0x53504e54 /* SPNT */ => {
                entries.spnt = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
            },
            0x42504e54 /* BPNT */ => {
                entries.bpnt = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
            },
            0x53505448 /* SPTH */ => {
                entries.spth = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
            },
            0x41524541 /* AREA */ => {
                entries.area = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
            },
            0x45564e54 /* EVNT */ => {
                entries.evnt = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
            },
            0x504c5920 /* PLY  */ => {
                entries.ply = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
            },
            0x4f424a53 /* OBJS */ => {
                entries.objs = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
            },
            0x4f424a20 /* OBJ  */ => {
                entries.obj = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
            },
            0x534f4253 /* SOBS */ => {
                entries.sobs = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
            },
            0x534f424a /* SOBJ */ => {
                entries.sobj = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
            },
            0x53544153 /* STAS */ => {
                entries.stas = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
            },
            0x53544147 /* STAG */ => {
                entries.stag = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
            },
            0x534e4454 /* SNDT */ => {
                entries.sndt = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
            },
            0x444f4f52 /* DOOR */ => {
                entries.door = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
            },
            0x4c595345 /* LYSE */ => {
                entries.lyse = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
            },
            0x53544946 /* STIF */ => {
                if entrydef.count != 1 {
                    return Err(binrw_error(format!("expected one STIF entry, got {}", entrydef.count)));
                }
                entries.stif = reader.read_be()?;
            },
            0x5043414d /* PCAM */ => {
                entries.pcam = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
            },
            0x4c594c54 /* LYLT */ => {
                entries.lylt = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
            },
            0x4c415920 /* LAY  */ => {
                if entrydef.count != 29 {
                    return Err(binrw_error(format!("expected 29 LAY entries, got {}", entrydef.count)));
                }
                let mut layers = Vec::with_capacity(29);
                for i in 0..entrydef.count {
                    let baseoff = pos + entrydef.offset as u64 + i as u64 * 8;
                    reader.seek(SeekFrom::Start(baseoff))?;
                    let layentry: LayEntry = reader.read_be()?;
                    if layentry.count == 0 {
                        layers.push(BzsEntries::default());
                    } else {
                        layers.push(parse_bzs_entries2(reader, layentry.count as u64, baseoff + layentry.offset as u64)?);
                    }
                }
                entries.lay = layers;
            }
            0x4152434e /* ARCN */ => {
                let offsets: Vec<u16> = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
                let strings = offsets.iter().map(|off| {
                    reader.seek(SeekFrom::Start(pos + entrydef.offset as u64 + *off as u64))?;
                    let s: NulTermShiftJis = reader.read_be()?;
                    Ok(s.data)
                }).collect::<Result<_, binrw::Error>>()?;
                entries.arcn = strings;
            },
            0x4f424a4e /* OBJN */ => {
                let offsets: Vec<u16> = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
                let strings = offsets.iter().map(|off| {
                    reader.seek(SeekFrom::Start(pos + entrydef.offset as u64 + *off as u64))?;
                    let s: NulTermShiftJis = reader.read_be()?;
                    Ok(s.data)
                }).collect::<Result<_, binrw::Error>>()?;
                entries.objn = strings;
            },
            0x524d504c /* RMPL */ => {
                let mut rmpls = Vec::with_capacity(entrydef.count as usize);
                for i in 0..entrydef.count {
                    let basepos = pos + entrydef.offset as u64 + i as u64 * 4;
                    reader.seek(SeekFrom::Start(basepos))?;
                    let rmpl_def: RmplDef = reader.read_be()?;
                    reader.seek(SeekFrom::Start(basepos + rmpl_def.offset as u64))?;
                    let data: Vec<u16> = binrw::count(rmpl_def.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
                    rmpls.push(RMPL { room: rmpl_def.rmpl_id, data })
                }
                entries.rmpl = rmpls;
            }
            _ => {
                return Err(binrw_error(format!("unknown bzs type: {}", entrydef.name)));
            },
        }
    }
    Ok(entries)
}

pub fn parse_bzs_entries<R: std::io::Read + std::io::Seek>(
    reader: &mut R,
    entrycount: u64,
    offset: u64,
) -> binrw::BinResult<Vec<BzsEntry>> {
    let mut entries = Vec::with_capacity(entrycount as usize);
    for i in 0..entrycount {
        let pos = offset + i * 12;
        reader.seek(SeekFrom::Start(pos))?;
        let entrydef: BzsSuperEntry = reader.read_be()?;
        let binrw_error = |s: String| binrw::Error::Custom {
            pos,
            err: Box::new(s),
        };
        reader.seek(SeekFrom::Start(pos + entrydef.offset as u64))?;
        entries.push(match entrydef.name {
            0x46494c45 /* FILE */ => {
                if entrydef.count != 1 {
                    return Err(binrw_error(format!("expected one FILE entry, got {}", entrydef.count)));
                }
                BzsEntry::FILE(reader.read_be()?)
            },
            0x5343454e /* SCEN */ => {
                BzsEntry::SCEN(binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?)
            },
            0x43414d20 /* CAM  */ => {
                BzsEntry::CAM(binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?)
            },
            0x50415448 /* PATH */ => {
                BzsEntry::PATH(binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?)
            },
            0x504e5420 /* PNT  */ => {
                BzsEntry::PNT(binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?)
            },
            0x53504e54 /* SPNT */ => {
                BzsEntry::SPNT(binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?)
            },
            0x42504e54 /* BPNT */ => {
                BzsEntry::BPNT(binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?)
            },
            0x53505448 /* SPTH */ => {
                BzsEntry::SPTH(binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?)
            },
            0x41524541 /* AREA */ => {
                BzsEntry::AREA(binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?)
            },
            0x45564e54 /* EVNT */ => {
                BzsEntry::EVNT(binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?)
            },
            0x504c5920 /* PLY  */ => {
                BzsEntry::PLY(binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?)
            },
            0x4f424a53 /* OBJS */ => {
                BzsEntry::OBJS(binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?)
            },
            0x4f424a20 /* OBJ  */ => {
                BzsEntry::OBJ(binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?)
            },
            0x534f4253 /* SOBS */ => {
                BzsEntry::SOBS(binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?)
            },
            0x534f424a /* SOBJ */ => {
                BzsEntry::SOBJ(binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?)
            },
            0x53544153 /* STAS */ => {
                BzsEntry::STAS(binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?)
            },
            0x53544147 /* STAG */ => {
                BzsEntry::STAG(binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?)
            },
            0x534e4454 /* SNDT */ => {
                BzsEntry::SNDT(binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?)
            },
            0x444f4f52 /* DOOR */ => {
                BzsEntry::DOOR(binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?)
            },
            0x4c595345 /* LYSE */ => {
                BzsEntry::LYSE(binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?)
            },
            0x53544946 /* STIF */ => {
                if entrydef.count != 1 {
                    return Err(binrw_error(format!("expected one STIF entry, got {}", entrydef.count)));
                }
                BzsEntry::STIF(reader.read_be()?)
            },
            0x5043414d /* PCAM */ => {
                BzsEntry::PCAM(binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?)
            },
            0x4c594c54 /* LYLT */ => {
                BzsEntry::LYLT(binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?)
            },
            0x4c415920 /* LAY  */ => {
                if entrydef.count != 29 {
                    return Err(binrw_error(format!("expected 29 LAY entries, got {}", entrydef.count)));
                }
                let mut layers = Vec::with_capacity(29);
                for i in 0..entrydef.count {
                    let baseoff = pos + entrydef.offset as u64 + i as u64 * 8;
                    reader.seek(SeekFrom::Start(baseoff))?;
                    let layentry: LayEntry = reader.read_be()?;
                    if layentry.count == 0 {
                        layers.push(vec![]);
                    } else {
                        layers.push(parse_bzs_entries(reader, layentry.count as u64, baseoff + layentry.offset as u64)?);
                    }
                }
                BzsEntry::LAY(layers)
            }
            0x4152434e /* ARCN */ => {
                let offsets: Vec<u16> = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
                let strings = offsets.iter().map(|off| {
                    reader.seek(SeekFrom::Start(pos + entrydef.offset as u64 + *off as u64))?;
                    let s: NulTermShiftJis = reader.read_be()?;
                    Ok(s.data)
                }).collect::<Result<_, binrw::Error>>()?;
                BzsEntry::ARCN(strings)
            },
            0x4f424a4e /* OBJN */ => {
                let offsets: Vec<u16> = binrw::count(entrydef.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
                let strings = offsets.iter().map(|off| {
                    reader.seek(SeekFrom::Start(pos + entrydef.offset as u64 + *off as u64))?;
                    let s: NulTermShiftJis = reader.read_be()?;
                    Ok(s.data)
                }).collect::<Result<_, binrw::Error>>()?;
                BzsEntry::OBJN(strings)
            },
            0x524d504c /* RMPL */ => {
                let mut rmpls = Vec::with_capacity(entrydef.count as usize);
                for i in 0..entrydef.count {
                    let basepos = pos + entrydef.offset as u64 + i as u64 * 4;
                    reader.seek(SeekFrom::Start(basepos))?;
                    let rmpl_def: RmplDef = reader.read_be()?;
                    reader.seek(SeekFrom::Start(basepos + rmpl_def.offset as u64))?;
                    let data: Vec<u16> = binrw::count(rmpl_def.count as usize)(reader, &ReadOptions::new(Endian::Big), ())?;
                    rmpls.push(RMPL { room: rmpl_def.rmpl_id, data })
                }
                BzsEntry::RMPL(rmpls)
            }
            _ => {
                return Err(binrw_error(format!("unknown bzs type: {}", entrydef.name)));
            },
        });
    }
    Ok(entries)
}

pub fn write_bzs<WS: Write + Seek>(entries: &BzsEntries, writer: &mut WS) -> binrw::BinResult<()> {
    let entry_count = write_bzs_entries(entries, writer, 12)?;
    // pad to 0x20 bytes
    let stream_pos = writer.stream_position()?;
    let pad_bytes = ((stream_pos + 0x1F) & !0x1F) - stream_pos;
    for _ in 0..pad_bytes {
        writer.write_all(&[0xFF])?;
    }
    writer.seek(SeekFrom::Start(0))?;
    writer.write_all(b"V001")?;
    writer.write_be(&LayEntry {
        count: entry_count as u16,
        ff: u16::MAX,
        offset: 12,
    })?;
    Ok(())
}

fn write_bzs_entries<WS: Write + Seek>(
    entries: &BzsEntries,
    writer: &mut WS,
    offset: u64,
) -> binrw::BinResult<usize> {
    // how many header entries are needed?
    // we skip empty objects here
    let mut count = 0;
    if entries.file.is_some() {
        count += 1;
    }
    if entries.stif.is_some() {
        count += 1;
    }
    if entries.arcn.len() > 0 {
        count += 1
    }
    if entries.objn.len() > 0 {
        count += 1;
    }
    if entries.lylt.len() > 0 {
        count += 1;
    }
    if entries.lyse.len() > 0 {
        count += 1;
    }
    if entries.scen.len() > 0 {
        count += 1;
    }
    if entries.cam.len() > 0 {
        count += 1;
    }
    if entries.pcam.len() > 0 {
        count += 1;
    }
    if entries.path.len() > 0 {
        count += 1;
    }
    if entries.pnt.len() > 0 {
        count += 1;
    }
    if entries.spnt.len() > 0 {
        count += 1;
    }
    if entries.bpnt.len() > 0 {
        count += 1;
    }
    if entries.spth.len() > 0 {
        count += 1;
    }
    if entries.area.len() > 0 {
        count += 1;
    }
    if entries.evnt.len() > 0 {
        count += 1;
    }
    if entries.ply.len() > 0 {
        count += 1;
    }
    if entries.rmpl.len() > 0 {
        count += 1;
    }
    if entries.objs.len() > 0 {
        count += 1;
    }
    if entries.obj.len() > 0 {
        count += 1;
    }
    if entries.door.len() > 0 {
        count += 1;
    }
    if entries.sobj.len() > 0 {
        count += 1;
    }
    if entries.sobs.len() > 0 {
        count += 1;
    }
    if entries.stas.len() > 0 {
        count += 1;
    }
    if entries.stag.len() > 0 {
        count += 1;
    }
    if entries.sndt.len() > 0 {
        count += 1;
    }
    if entries.lay.len() > 0 {
        count += 1;
    }
    // we now know the length of the headers, so the data offset
    let mut current_data_offset = offset + count as u64 * 12;
    let mut header_index = 0;
    let mut write_header = |name: &[u8; 4],
                            count: usize,
                            current_data_offset: u64,
                            writer: &mut WS|
     -> binrw::BinResult<()> {
        let entry_pos = offset + header_index as u64 * 12;
        header_index += 1;
        writer.seek(SeekFrom::Start(entry_pos))?;
        writer.write_be(&BzsSuperEntry {
            count: count as u16,
            ff: 0xFFFF,
            name: u32::from_be_bytes(*name),
            offset: (current_data_offset - entry_pos) as u32,
        })?;
        Ok(())
    };
    let align_4 = |current_data_offset: &mut u64, writer: &mut WS| -> std::io::Result<()> {
        let pad_needed = ((*current_data_offset + 3) & !3) - *current_data_offset;
        if pad_needed > 0 {
            writer.seek(SeekFrom::Start(*current_data_offset))?;
            for _ in 0..pad_needed {
                writer.write_all(&[0xFF])?;
            }
        }
        *current_data_offset += pad_needed;
        Ok(())
    };
    if entries.lyse.len() > 0 {
        write_header(b"LYSE", entries.lyse.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        for lyse in &entries.lyse {
            writer.write_be(lyse)?;
        }
        current_data_offset = writer.stream_position()?;
    }
    if let Some(stif) = &entries.stif {
        write_header(b"STIF", 1, current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        writer.write_be(stif)?;
        current_data_offset = writer.stream_position()?;
    }
    if entries.objn.len() > 0 {
        write_header(b"OBJN", entries.objn.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        // first, calculate the space needed for all the string offsets
        // which is the offset (relativ to the current data offset)
        // for the first string
        let mut strings_offset = entries.objn.len() as u64 * 2;
        for (i, string) in entries.objn.iter().enumerate() {
            writer.seek(SeekFrom::Start(current_data_offset + i as u64 * 2))?;
            writer.write_be(&(strings_offset as u16))?;
            writer.seek(SeekFrom::Start(current_data_offset + strings_offset))?;
            write_nul_term_shift_jis(string, writer)?;
            strings_offset = writer.stream_position()? - current_data_offset;
        }
        current_data_offset += strings_offset;
        align_4(&mut current_data_offset, writer)?;
    }
    if entries.arcn.len() > 0 {
        write_header(b"ARCN", entries.arcn.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        // first, calculate the space needed for all the string offsets
        // which is the offset (relativ to the current data offset)
        // for the first string
        let mut strings_offset = entries.arcn.len() as u64 * 2;
        for (i, string) in entries.arcn.iter().enumerate() {
            writer.seek(SeekFrom::Start(current_data_offset + i as u64 * 2))?;
            writer.write_be(&(strings_offset as u16))?;
            writer.seek(SeekFrom::Start(current_data_offset + strings_offset))?;
            write_nul_term_shift_jis(string, writer)?;
            strings_offset = writer.stream_position()? - current_data_offset;
        }
        current_data_offset = writer.stream_position()?;
        align_4(&mut current_data_offset, writer)?;
    }
    if let Some(file) = &entries.file {
        write_header(b"FILE", 1, current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        writer.write_be(file)?;
        current_data_offset = writer.stream_position()?;
    }
    if entries.scen.len() > 0 {
        write_header(b"SCEN", entries.scen.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        for scen in &entries.scen {
            writer.write_be(scen)?;
        }
        current_data_offset = writer.stream_position()?;
    }
    if entries.rmpl.len() > 0 {
        write_header(b"RMPL", entries.rmpl.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        let mut rmpl_data_offset = entries.rmpl.len() as u64 * 4;
        for (i, entry) in entries.rmpl.iter().enumerate() {
            writer.seek(SeekFrom::Start(current_data_offset + i as u64 * 4))?;
            writer.write_be(&RmplDef {
                count: entry.data.len() as u8,
                offset: (rmpl_data_offset - i as u64 * 4) as u16,
                rmpl_id: entry.room,
            })?;
            writer.seek(SeekFrom::Start(current_data_offset + rmpl_data_offset))?;
            writer.write_be(&entry.data)?;
            rmpl_data_offset = writer.stream_position()? - current_data_offset;
        }
        current_data_offset += rmpl_data_offset;
        align_4(&mut current_data_offset, writer)?;
    }
    if entries.cam.len() > 0 {
        write_header(b"CAM ", entries.cam.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        for entry in &entries.cam {
            writer.write_be(entry)?;
        }
        current_data_offset = writer.stream_position()?;
    }
    if entries.pcam.len() > 0 {
        write_header(b"PCAM", entries.pcam.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        for entry in &entries.pcam {
            writer.write_be(entry)?;
        }
        current_data_offset = writer.stream_position()?;
    }
    if entries.path.len() > 0 {
        write_header(b"PATH", entries.path.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        for entry in &entries.path {
            writer.write_be(entry)?;
        }
        current_data_offset = writer.stream_position()?;
    }
    if entries.pnt.len() > 0 {
        write_header(b"PNT ", entries.pnt.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        for entry in &entries.pnt {
            writer.write_be(entry)?;
        }
        current_data_offset = writer.stream_position()?;
    }
    if entries.spnt.len() > 0 {
        write_header(b"SPNT", entries.spnt.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        for entry in &entries.spnt {
            writer.write_be(entry)?;
        }
        current_data_offset = writer.stream_position()?;
    }
    if entries.bpnt.len() > 0 {
        write_header(b"BPNT", entries.bpnt.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        for entry in &entries.bpnt {
            writer.write_be(entry)?;
        }
        current_data_offset = writer.stream_position()?;
    }
    if entries.spth.len() > 0 {
        write_header(b"SPTH", entries.spth.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        for entry in &entries.spth {
            writer.write_be(entry)?;
        }
        current_data_offset = writer.stream_position()?;
    }
    if entries.area.len() > 0 {
        write_header(b"AREA", entries.area.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        for entry in &entries.area {
            writer.write_be(entry)?;
        }
        current_data_offset = writer.stream_position()?;
    }
    if entries.evnt.len() > 0 {
        write_header(b"EVNT", entries.evnt.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        for entry in &entries.evnt {
            writer.write_be(entry)?;
        }
        current_data_offset = writer.stream_position()?;
    }
    if entries.ply.len() > 0 {
        write_header(b"PLY ", entries.ply.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        for entry in &entries.ply {
            writer.write_be(entry)?;
        }
        current_data_offset = writer.stream_position()?;
    }
    if entries.objs.len() > 0 {
        write_header(b"OBJS", entries.objs.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        for entry in &entries.objs {
            writer.write_be(entry)?;
        }
        current_data_offset = writer.stream_position()?;
    }
    if entries.obj.len() > 0 {
        write_header(b"OBJ ", entries.obj.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        for entry in &entries.obj {
            writer.write_be(entry)?;
        }
        current_data_offset = writer.stream_position()?;
    }
    if entries.sobj.len() > 0 {
        write_header(b"SOBJ", entries.sobj.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        for entry in &entries.sobj {
            writer.write_be(entry)?;
        }
        current_data_offset = writer.stream_position()?;
    }
    if entries.sobs.len() > 0 {
        write_header(b"SOBS", entries.sobs.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        for entry in &entries.sobs {
            writer.write_be(entry)?;
        }
        current_data_offset = writer.stream_position()?;
    }
    if entries.door.len() > 0 {
        write_header(b"DOOR", entries.door.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        for entry in &entries.door {
            writer.write_be(entry)?;
        }
        current_data_offset = writer.stream_position()?;
    }
    if entries.stas.len() > 0 {
        write_header(b"STAS", entries.stas.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        for entry in &entries.stas {
            writer.write_be(entry)?;
        }
        current_data_offset = writer.stream_position()?;
    }
    if entries.stag.len() > 0 {
        write_header(b"STAG", entries.stag.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        for entry in &entries.stag {
            writer.write_be(entry)?;
        }
        current_data_offset = writer.stream_position()?;
    }
    if entries.sndt.len() > 0 {
        write_header(b"SNDT", entries.sndt.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        for entry in &entries.sndt {
            writer.write_be(entry)?;
        }
        current_data_offset = writer.stream_position()?;
    }
    if entries.lylt.len() > 0 {
        write_header(b"LYLT", entries.lylt.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;
        for lylt in &entries.lylt {
            writer.write_be(lylt)?;
        }
        current_data_offset = writer.stream_position()?;
    }
    if entries.lay.len() > 0 {
        write_header(b"LAY ", entries.lay.len(), current_data_offset, writer)?;
        writer.seek(SeekFrom::Start(current_data_offset))?;

        let mut lay_data_rel_offset = entries.lay.len() as u64 * 8;
        // let mut new_current_data_offset = 0;
        for (i, lay) in entries.lay.iter().enumerate() {
            // TODO: is this seek needed?
            writer.seek(SeekFrom::Start(current_data_offset + lay_data_rel_offset))?;
            let entry_count =
                write_bzs_entries(lay, writer, current_data_offset + lay_data_rel_offset)?;
            let lay_entry = LayEntry {
                count: entry_count as u16,
                ff: u16::MAX,
                offset: if entry_count > 0 {
                    (lay_data_rel_offset - i as u64 * 8) as u32
                } else {
                    0
                },
            };
            lay_data_rel_offset = writer.stream_position()? - current_data_offset;
            writer.seek(SeekFrom::Start(current_data_offset + i as u64 * 8))?;
            writer.write_be(&lay_entry)?;
        }

        current_data_offset += lay_data_rel_offset;
        writer.seek(SeekFrom::Start(current_data_offset))?;
    }
    Ok(count)
}

// 0x46494c45 /* FILE */ => {}
// 0x5343454e /* SCEN */ => {}
// 0x43414d20 /* CAM  */ => {}
// 0x50415448 /* PATH */ => {}
// 0x504e5420 /* PNT  */ => {}
// 0x53504e54 /* SPNT */ => {}
// 0x42504e54 /* BPNT */ => {}
// 0x53505448 /* SPTH */ => {}
// 0x41524541 /* AREA */ => {}
// 0x45564e54 /* EVNT */ => {}
// 0x504c5920 /* PLY  */ => {}
// 0x4f424a53 /* OBJS */ => {}
// 0x4f424a20 /* OBJ  */ => {}
// 0x534f4253 /* SOBS */ => {}
// 0x534f424a /* SOBJ */ => {}
// 0x53544153 /* STAS */ => {}
// 0x53544147 /* STAG */ => {}
// 0x534e4454 /* SNDT */ => {}
// 0x444f4f52 /* DOOR */ => {}
// 0x4c595345 /* LYSE */ => {}
// 0x53544946 /* STIF */ => {}
// 0x5043414d /* PCAM */ => {}
// 0x4c594c54 /* LYLT */ => {}

#[cfg(test)]
mod test {
    use std::{
        fs::{self, File},
        io::Cursor,
    };

    use crate::structs::write_bzs;

    use super::{parse_bzs_file, AREA};

    #[test]
    pub fn test_parse() {
        let mut f = File::open("../../../ss-extract/D000_stg_l0.d/dat/stage.bzs").unwrap();
        let bzs = parse_bzs_file(&mut f).unwrap();
        println!("{:?}", bzs);
        let mut buf = Vec::new();
        write_bzs(&bzs, &mut Cursor::new(&mut buf)).unwrap();
        fs::write("out.bzs", &buf).unwrap();
    }
}
