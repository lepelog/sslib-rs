use binrw::{BinReaderExt, BinWriterExt};
use encoding::all::WINDOWS_31J;
use encoding::{DecoderTrap, Encoding};
use std::io::Write;
use std::ops::{BitAnd, Not};
use std::{
    collections::HashMap,
    io::{Read, Seek, SeekFrom},
    ops::Neg,
};
use structs::RawFlw3;

mod structs;

#[derive(Default, Debug)]
pub struct TextSegment {
    pub atr: Vec<u8>,
    pub text: Vec<u16>,
}

#[derive(Debug)]
pub struct Msbt {
    pub lbl: HashMap<String, u32>,
    pub text: Vec<TextSegment>,
}

#[derive(Debug)]
pub struct Msbf {
    pub flows: Vec<FlowEntry>,
    pub entrypoints: HashMap<String, u32>,
}

#[derive(Debug)]
pub enum FlowEntry {
    Start {
        next: i16,
    },
    Text {
        file: u16,
        line: u16,
        next: i16,
    },
    Flow {
        subtype: u8,
        param1: u16,
        param2: u16,
        next: i16,
        param3: i16,
    },
    Switch {
        subtype: u8,
        param1: u16,
        param2: u16,
        param3: i16,
        branches: Vec<i16>,
    },
}

#[inline]
fn align_next(num: u64, alignment: u64) -> u64 {
    num.wrapping_add(alignment - 1)
        .bitand((alignment - 1).not())
}

fn entrypoint_hash(bytes: &[u8], entries: usize) -> usize {
    let mut hash: u32 = 0;
    for b in bytes {
        hash = hash.wrapping_mul(0x492).wrapping_add(*b as u32);
    }
    hash as usize % entries
}

pub fn parse_lbl<R: Read + Seek>(
    r: &mut R,
    cur_seg_start: u64,
) -> binrw::BinResult<HashMap<String, u32>> {
    let mut lbl_entries = HashMap::new();
    let group_count = r.read_be::<u32>()?;
    for i in 0..group_count {
        r.seek(SeekFrom::Start(cur_seg_start + 4 + i as u64 * 8))?;
        let count = r.read_be::<u32>()?;
        let group_offset = r.read_be::<u32>()?;
        r.seek(SeekFrom::Start(cur_seg_start + group_offset as u64))?;
        for _ in 0..count {
            let str_len = r.read_be::<u8>()?;
            let mut str_buf = vec![0; str_len.into()];
            r.read_exact(&mut str_buf)?;
            // TODO: this shouldn't panic
            let str = WINDOWS_31J.decode(&str_buf, DecoderTrap::Strict).unwrap();
            let value = r.read_be::<u32>()?;
            lbl_entries.insert(str, value);
        }
    }
    Ok(lbl_entries)
}

pub fn parse_msbt<R: Read + Seek>(r: &mut R) -> binrw::BinResult<Msbt> {
    let file_end = r.seek(SeekFrom::End(0))?;
    let mut next_seg = 0x20;
    let mut lbl_entries = HashMap::new();
    let mut atr1 = Vec::new();
    let mut txt2 = Vec::new();
    while next_seg < file_end {
        r.seek(SeekFrom::Start(next_seg))?;
        let seg_id = r.read_be::<u32>()?;
        let seg_len = r.read_be::<u32>()?;
        r.seek(SeekFrom::Current(8))?; // seek to start of section data
        let cur_seg_start = next_seg + 0x10;
        next_seg += 0x10;
        next_seg += u64::from(seg_len);
        next_seg += (next_seg as isize).neg().rem_euclid(0x10) as u64;
        match seg_id {
            0x4c424c31 /* LBL1 */ => {
                lbl_entries = parse_lbl(r, cur_seg_start)?;
            },
            0x41545231 /* ATR1 */ => {
                let count = r.read_be::<u32>()?;
                let dimension = r.read_be::<u32>()?;
                for _ in 0..count {
                    let mut current_arr = Vec::new();
                    for _ in 0..dimension {
                        current_arr.push(r.read_be::<u8>()?);
                    }
                    atr1.push(current_arr);
                }
            },
            0x54585432 /* TXT2 */ => {
                let count = r.read_be::<u32>()?;
                for i in 0..count {
                    r.seek(SeekFrom::Start(cur_seg_start + 4 + i as u64 * 4))?;
                    let str_offset = r.read_be::<u32>()?;
                    r.seek(SeekFrom::Start(cur_seg_start + str_offset as u64))?;
                    let mut utf16buf = Vec::new();
                    loop {
                        let val = r.read_be::<u16>()?;
                        if val == 0 {
                            break;
                        }
                        utf16buf.push(val);
                    }
                    txt2.push(utf16buf);
                }
            },
            // TODO: proper error
            _ => panic!("unknown seg: {:X}", seg_id),
        };
    }
    // add atr to txt
    assert_eq!(atr1.len(), txt2.len());
    let text = txt2
        .into_iter()
        .zip(atr1.into_iter())
        .map(|(text, atr)| TextSegment { atr, text })
        .collect();
    Ok(Msbt {
        lbl: lbl_entries,
        text,
    })
}

pub fn parse_msbf<R: Read + Seek>(r: &mut R) -> binrw::BinResult<Msbf> {
    let file_end = r.seek(SeekFrom::End(0))?;
    let mut next_seg = 0x20;
    let mut entrypoints = HashMap::new();
    let mut flows = Vec::new();
    // let mut txt2 = Vec::new();
    while next_seg < file_end {
        r.seek(SeekFrom::Start(next_seg))?;
        let seg_id = r.read_be::<u32>()?;
        let seg_len = r.read_be::<u32>()?;
        r.seek(SeekFrom::Current(8))?; // seek to start of section data
        let cur_seg_start = next_seg + 0x10;
        next_seg += 0x10;
        next_seg += u64::from(seg_len);
        next_seg = align_next(next_seg, 0x10);
        match seg_id {
            0x46454E31 /* FEN1 */ => {
                entrypoints = parse_lbl(r, cur_seg_start)?;
            },
            0x464C5733 /* FLW3 */ => {
                let flow_count = r.read_be::<u16>()?;
                let _branch_count = r.read_be::<u16>()?;
                let flow_start = cur_seg_start + 0x10;
                let branch_start = flow_start + 0x10 * flow_count as u64;
                for i in 0..flow_count as u64 {
                    r.seek(SeekFrom::Start(flow_start + i * 0x10))?;
                    let raw_flw = r.read_be::<RawFlw3>()?;
                    match raw_flw.typ {
                        1 => {
                            flows.push(FlowEntry::Text{
                                file: raw_flw.param3 as u16,
                                line: raw_flw.param4 as u16,
                                next: raw_flw.next,
                            });
                        },
                        2 => {
                            let mut branches = Vec::new();
                            let branch_offset = raw_flw.param5;
                            let branch_count = raw_flw.param4;
                            r.seek(SeekFrom::Start(branch_start + 2 * branch_offset as u64))?;
                            for _ in 0..branch_count {
                                branches.push(r.read_be::<i16>()?);
                            }
                            flows.push(FlowEntry::Switch{
                                param1: raw_flw.param1,
                                param2: raw_flw.param2,
                                param3: raw_flw.param3,
                                subtype: raw_flw.sub_type,
                                branches
                            });
                        },
                        3 => {
                            flows.push(FlowEntry::Flow{
                                subtype: raw_flw.sub_type,
                                param1: raw_flw.param1,
                                param2: raw_flw.param2,
                                param3: raw_flw.param3,
                                next: raw_flw.next,
                            });
                        },
                        4 => {
                            flows.push(FlowEntry::Start{next: raw_flw.next});
                        }
                        _ => panic!("unknown type: {:X}", raw_flw.typ),
                    }
                }
            },
            // TODO: proper error
            _ => panic!("unknown seg: {:X}", seg_id),
        };
    }
    Ok(Msbf { entrypoints, flows })
}

// stream position is at the end after it finished
fn write_lbl_fen<WS: Write + Seek>(
    map: &HashMap<String, u32>,
    section_name: u32,
    bucket_count: usize,
    ws: &mut WS,
) -> binrw::BinResult<()> {
    let section_start = ws.stream_position()?;
    // FEN1/LBL1
    // we don't know the total section size yet
    // but we know there will be 30 groups here
    ws.seek(SeekFrom::Start(section_start))?;
    ws.write_be(&section_name)?;
    // length will be filled in later
    ws.write_be(&0u32)?;
    ws.write_all(&[0u8; 8])?;

    let mut buckets: Vec<Vec<(&[u8], u32)>> = vec![vec![]; bucket_count];
    ws.write_be(&(bucket_count as u32))?;
    // sort entries to specific lists
    // it's fine to iterate the hash map here, it will be sorted later
    for (entrypoint, idx) in map {
        // idk if this actually has to be the case, try for now
        assert!(entrypoint.is_ascii());
        let encoded_entrypoint = entrypoint.as_bytes();
        let bucket = entrypoint_hash(encoded_entrypoint, bucket_count);
        buckets[bucket].push((encoded_entrypoint, *idx));
    }
    for bucket in &mut buckets {
        // sort by referenced index, *in theory* they might be not unique,
        // but in that case the order doesn't matter
        bucket.sort_unstable_by_key(|e| e.1);
    }

    let section_data_start = section_start + 16 /* header */;

    // offset is relative to FEN1 section start after header
    let mut seg_data_offset: u32 = buckets.len() as u32 * 8 + 4;
    for (i, bucket) in buckets.iter().enumerate() {
        // absolute offset
        let bucket_header_offset = section_data_start + i as u64 * 8 + 4 /* bucket count */;
        // write data
        ws.seek(SeekFrom::Start(section_data_start + seg_data_offset as u64))?;
        for (lbl, idx) in bucket {
            // first string len
            ws.write_all(&[lbl.len() as u8])?;
            // then string (no explicit 0 byte at the end)
            ws.write_all(lbl)?;
            // then index (unaligned)
            ws.write_be(idx)?;
        }
        let new_seg_data_offset = (ws.stream_position()? - section_data_start) as u32;
        ws.seek(SeekFrom::Start(bucket_header_offset))?;
        // placeholder for length
        ws.write_be(&(bucket.len() as u32))?;
        // actual data offset
        ws.write_be(&seg_data_offset)?;
        seg_data_offset = new_seg_data_offset;
    }

    // write FEN1 length
    ws.seek(SeekFrom::Start(section_start + 4))?;
    ws.write_be(&seg_data_offset)?;

    // section_start is aligned, so we can only use seg_data_offset to determine the padding
    let pads = (seg_data_offset as isize).neg().rem_euclid(16);

    ws.seek(SeekFrom::Start(section_data_start + seg_data_offset as u64))?;

    for _ in 0..pads {
        ws.write_all(&[0xAB])?;
    }
    Ok(())
}

impl Msbf {
    pub fn write_msbf<WS: Write + Seek>(&self, ws: &mut WS) -> binrw::BinResult<()> {
        const HEADER: &[u8; 16] = b"MsgFlwBn\xFE\xFF\0\0\0\x03\0\x02";
        const EMPTY_HEADER: &[u8; 16] = &[0u8; 16];
        ws.write_all(HEADER)?;
        ws.write_all(EMPTY_HEADER)?;
        // write FLW3
        let flw_start = 32;
        // section header of 16
        ws.seek(SeekFrom::Current(16))?;
        // decompose flows into raw components
        let mut flows = Vec::new();
        let mut branch_points = Vec::new();
        for flow in &self.flows {
            let raw_flow = match flow {
                FlowEntry::Start { next } => RawFlw3 {
                    typ: 4,
                    sub_type: 0xFF,
                    next: *next,
                    param1: 0,
                    param2: 0,
                    param3: 0,
                    param4: 0,
                    param5: 0,
                },
                FlowEntry::Text { file, line, next } => RawFlw3 {
                    typ: 1,
                    sub_type: 0xFF,
                    next: *next,
                    param1: 0,
                    param2: 0,
                    param3: *file as i16,
                    param4: *line as i16,
                    param5: 0,
                },
                FlowEntry::Flow {
                    subtype,
                    param1,
                    param2,
                    next,
                    param3,
                } => RawFlw3 {
                    typ: 3,
                    sub_type: *subtype,
                    next: *next,
                    param1: *param1,
                    param2: *param2,
                    param3: *param3,
                    param4: 0,
                    param5: 0,
                },
                FlowEntry::Switch {
                    subtype,
                    param1,
                    param2,
                    param3,
                    branches,
                } => {
                    let branch_offset = branch_points.len() as i16;
                    branch_points.extend_from_slice(branches);
                    RawFlw3 {
                        typ: 2,
                        sub_type: *subtype,
                        next: -1,
                        param1: *param1,
                        param2: *param2,
                        param3: *param3,
                        param4: branches.len() as i16,
                        param5: branch_offset,
                    }
                }
            };
            flows.push(raw_flow);
        }
        ws.write_be(&(flows.len() as u16))?;
        ws.write_be(&(branch_points.len() as u16))?;
        ws.write_all(&[0u8; 12])?;
        for flow in &flows {
            ws.write_be(flow)?;
        }
        for branch_point in &branch_points {
            ws.write_be(branch_point)?;
        }
        // pad to 16 bytes
        let flw_end = ws.stream_position()?;
        let pads = (flw_end as isize).neg().rem_euclid(16);
        for _ in 0..pads {
            ws.write_all(&[0xAB])?;
        }
        let fen1_start = flw_end + pads as u64;
        // write FLW3 header
        ws.seek(SeekFrom::Start(flw_start))?;
        ws.write_all(b"FLW3")?;
        ws.write_be(&(flw_end as u32 - flw_start as u32 - 16/* header */))?;
        ws.write_all(&[0u8; 8])?;
        // FEN1
        ws.seek(SeekFrom::Start(fen1_start))?;
        write_lbl_fen(&self.entrypoints, u32::from_be_bytes(*b"FEN1"), 19, ws)?;
        // write file length
        let pos = ws.stream_position()?;
        ws.seek(SeekFrom::Start(0x12))?;
        ws.write_be(&(pos as u32))?;
        Ok(())
    }
}

impl FlowEntry {
    pub fn get_type(&self) -> &'static str {
        match self {
            Self::Text { .. } => "Text",
            Self::Flow { .. } => "Flow",
            Self::Switch { .. } => "Switch",
            Self::Start { .. } => "Start",
        }
    }
}
