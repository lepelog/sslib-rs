use std::{
    borrow::Cow,
    collections::BTreeMap,
    io::{self, Cursor, Seek, SeekFrom, Write},
};

use binrw::{BinReaderExt, BinWriterExt};
use structs::{RawRelocation, RelHeader, RelocationKind};

pub mod structs;

fn try_read_data<'a>(
    bytes: &'a [u8],
    subject: &'static str,
    offset: u32,
    length: u32,
) -> Result<&'a [u8], RelReadError> {
    bytes
        .get(offset as usize..(offset + length) as usize)
        .ok_or(RelReadError::OobRead {
            subject,
            position: offset,
        })
}

#[derive(thiserror::Error, Debug)]
pub enum RelReadError {
    #[error("{0}")]
    Binrw(#[from] binrw::Error),
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("Trying to read {subject}: Oob at {position}")]
    OobRead {
        subject: &'static str,
        position: u32,
    },
}

#[derive(Debug)]
pub struct Relocation {
    pub offset: u32,
    pub typ: RelocationKind,
    pub symbol_section: u8,
    pub symbol_offset: u32,
}

#[derive(Debug)]
pub enum Section<'a> {
    Bss {
        size: u32,
    },
    Empty,
    Data {
        data: Cow<'a, [u8]>,
        offset: u32,
        is_executable: bool,
    },
}

impl<'a> Section<'a> {
    pub fn read(section_head: &'a [u8; 8], data: &'a [u8]) -> Result<Section<'a>, RelReadError> {
        let mut offset = u32::from_be_bytes(section_head[..4].try_into().unwrap());
        let length = u32::from_be_bytes(section_head[4..].try_into().unwrap());
        if length == 0 {
            return Ok(Section::Empty);
        }
        let is_executable = (offset & 1) != 0;
        offset &= !1;
        if offset == 0 {
            assert!(!is_executable);
            return Ok(Section::Bss { size: length });
        }
        let section_data = try_read_data(data, "section data", offset, length)?;
        Ok(Section::Data {
            data: Cow::Borrowed(section_data),
            offset,
            is_executable,
        })
    }
    pub fn into_owned(self) -> Section<'static> {
        match self {
            Self::Data {
                data,
                offset,
                is_executable,
            } => Section::Data {
                data: Cow::Owned(data.into_owned()),
                offset,
                is_executable,
            },
            Self::Bss { size } => Section::Bss { size },
            Self::Empty => Section::Empty,
        }
    }

    pub fn write<WS: Write + Seek>(
        &self,
        w: &mut WS,
        section_info_offset: u64,
        end_offset: &mut u64,
    ) -> binrw::BinResult<()> {
        let (first_val, second_val) = match self {
            Self::Empty => (0, 0),
            Section::Bss { size } => (0, *size),
            Section::Data {
                data,
                offset,
                is_executable,
            } => {
                w.seek(SeekFrom::Start(*offset as u64))?;
                w.write_all(data.as_ref())?;
                let mut first_val = *offset;
                if *is_executable {
                    first_val |= 1;
                }
                *end_offset = *offset as u64 + data.len() as u64;
                (first_val as u32, data.len() as u32)
            }
        };
        w.seek(SeekFrom::Start(section_info_offset))?;
        w.write_be(&first_val)?;
        w.write_be(&second_val)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Rel<'a> {
    header: RelHeader,
    sections: Vec<Section<'a>>,
    // Map<OtherRelIdOrMailDol, Map<LocalSection, Vec<Relocations>>>
    relocations: BTreeMap<u32, BTreeMap<u8, Vec<Relocation>>>,
}

fn write_relocations_for_module<WS: Write + Seek>(
    relocations: &BTreeMap<u8, Vec<Relocation>>,
    module_id: u32,
    imp_entry_offset: &mut u32,
    relocations_offset: &mut u32,
    w: &mut WS,
) -> binrw::BinResult<()> {
    w.seek(SeekFrom::Start(*imp_entry_offset as u64))?;
    w.write_be(&module_id)?;
    w.write_be(relocations_offset)?;
    *imp_entry_offset += 8;
    w.seek(SeekFrom::Start(*relocations_offset as u64))?;
    for (section, relocs) in relocations {
        if relocs.is_empty() {
            continue;
        }
        // set section
        let mut current_section_offset = 0;
        w.write_be(&RawRelocation {
            offset_from_prev: 0,
            section: *section,
            symbol_offset: 0,
            typ: RelocationKind::R_DOLPHIN_SECTION,
        })?;
        for reloc in relocs {
            while reloc.offset - current_section_offset > 0xFFFF {
                w.write_be(&RawRelocation {
                    offset_from_prev: 0xFFFF,
                    section: 0,
                    symbol_offset: 0,
                    typ: RelocationKind::R_DOLPHIN_NOP,
                })?;
                current_section_offset += 0xFFFF;
            }
            w.write_be(&RawRelocation {
                offset_from_prev: (reloc.offset - current_section_offset)
                    .try_into()
                    .expect("should have been handled above"),
                section: reloc.symbol_section,
                symbol_offset: reloc.symbol_offset,
                typ: reloc.typ,
            })?;
            current_section_offset = reloc.offset;
        }
    }
    // mark end
    w.write_be(&RawRelocation {
        offset_from_prev: 0,
        section: 0,
        symbol_offset: 0,
        typ: RelocationKind::R_DOLPHIN_END,
    })?;
    *relocations_offset = w.stream_position()? as u32;
    Ok(())
}

impl<'a> Rel<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, RelReadError> {
        let mut bytes_reader = Cursor::new(bytes);
        let header: RelHeader = bytes_reader.read_be()?;
        // println!("{header:?}");
        let mut sections = Vec::with_capacity(header.num_sections as usize);
        for i in 0..header.num_sections {
            let section_head_bytes =
                try_read_data(bytes, "section head", header.section_info_offset + i * 8, 8)?
                    .try_into()
                    .unwrap();
            sections.push(Section::read(section_head_bytes, bytes)?);
        }

        // process relocations
        let mut relocations = BTreeMap::new();
        let mut imp_offset = header.imp_offset;
        while imp_offset < header.imp_offset + header.imp_size {
            bytes_reader.seek(SeekFrom::Start(imp_offset.into()))?;
            imp_offset += 8;
            // reloc header
            let module_num: u32 = bytes_reader.read_be()?;
            let relocation_data_offset: u32 = bytes_reader.read_be()?;

            let mut current_reloc_offset: u32 = 0;

            let mut section_relocations: BTreeMap<u8, Vec<Relocation>> = BTreeMap::new();
            let mut current_section_relocs = section_relocations.entry(0).or_default();
            bytes_reader.seek(SeekFrom::Start(relocation_data_offset.into()))?;
            loop {
                let raw_reloc: RawRelocation = bytes_reader.read_be()?;
                current_reloc_offset += u32::from(raw_reloc.offset_from_prev);
                match raw_reloc.typ {
                    RelocationKind::R_DOLPHIN_NOP => continue,
                    RelocationKind::R_DOLPHIN_SECTION => {
                        current_reloc_offset = 0;
                        current_section_relocs =
                            section_relocations.entry(raw_reloc.section).or_default();
                        continue;
                    }
                    RelocationKind::R_DOLPHIN_END => {
                        break;
                    }
                    _ => (),
                }
                current_section_relocs.push(Relocation {
                    offset: current_reloc_offset,
                    typ: raw_reloc.typ,
                    symbol_section: raw_reloc.section,
                    symbol_offset: raw_reloc.symbol_offset,
                });
            }
            relocations.insert(module_num, section_relocations);
        }

        Ok(Self {
            header,
            sections,
            relocations,
        })
    }

    pub fn write<WS: Write + Seek>(&self, w: &mut WS) -> binrw::BinResult<()> {
        // leave 0x4C space for the rel header
        // leave 8 * sectioncount space for the section table
        // w.seek(SeekFrom::Start((self.sections.len() as u64 * 8 + 0x4C)))?;
        let mut header = self.header.clone();
        header.section_info_offset = 0x4C;
        // this shouldn't be necessary, I don't think it makes sense to change this
        header.num_sections = self.sections.len() as u32;
        let mut section_offset = self.sections.len() as u64 * 8 + 0x4C;
        for (i, section) in self.sections.iter().enumerate() {
            section.write(w, i as u64 * 8 + 0x4C, &mut section_offset)?;
        }
        header.imp_offset = section_offset as u32;
        // relocations, first reserve space for the imp table,
        // 8 bytes per referenced module
        header.imp_size = self.relocations.len() as u32 * 8;
        let mut imp_entry_offset = header.imp_offset;
        let mut relocations_offset = imp_entry_offset + header.imp_size;
        header.rel_offset = relocations_offset;
        for (module_id, relocations) in &self.relocations {
            // relocations against oneself and main.dol are done last
            if *module_id != self.header.id && *module_id != 0 {
                write_relocations_for_module(
                    relocations,
                    *module_id,
                    &mut imp_entry_offset,
                    &mut relocations_offset,
                    w,
                )?;
            }
        }
        // set fixed size now, before own relocs and main dol relocs
        // this is because need to only be processed once and can then be overwritten
        header.fix_size = relocations_offset;
        if let Some(own_relocs) = self.relocations.get(&self.header.id) {
            write_relocations_for_module(
                own_relocs,
                self.header.id,
                &mut imp_entry_offset,
                &mut relocations_offset,
                w,
            )?;
        }
        if let Some(main_relocs) = self.relocations.get(&0) {
            write_relocations_for_module(
                main_relocs,
                0,
                &mut imp_entry_offset,
                &mut relocations_offset,
                w,
            )?;
        }
        // finally, write the header
        w.seek(SeekFrom::Start(0))?;
        w.write_be(&header)?;
        Ok(())
    }

    pub fn get_header(&self) -> &RelHeader {
        &self.header
    }

    pub fn get_relocations(&self) -> &BTreeMap<u32, BTreeMap<u8, Vec<Relocation>>> {
        &self.relocations
    }

    pub fn get_sections(&self) -> &Vec<Section> {
        &self.sections
    }

    pub fn offset_in_rel_to_section_offset(&self, offset: u32) -> Option<(u8, u32)> {
        let mut offset_after_last_section = 0;
        for (i, section) in self.sections.iter().enumerate() {
            let (start, end) = match section {
                Section::Bss { size } => {
                    (offset_after_last_section, offset_after_last_section + *size)
                }
                Section::Empty => continue,
                Section::Data { data, offset, .. } => (*offset, *offset + data.len() as u32),
            };
            offset_after_last_section = end;
            if offset >= start && offset < end {
                return Some((i as u8, offset - start));
            }
        }
        None
    }

    pub fn get_section_data<'b>(&'b self, section: u8) -> Option<&'b [u8]> {
        match self.sections.get(section as usize) {
            Some(Section::Data { data, .. }) => Some(data.as_ref()),
            _ => None,
        }
    }

    pub fn get_section_data_mut<'b>(&'b mut self, section: u8) -> Option<&'b mut [u8]> {
        match self.sections.get_mut(section as usize) {
            Some(Section::Data { data, .. }) => Some(data.to_mut()),
            _ => None,
        }
    }

    pub fn get_section_data_for_rel_offset<'b>(&'b self, offset: u32) -> Option<&'b [u8]> {
        let (section, offset_in_section) = self.offset_in_rel_to_section_offset(offset)?;
        match self.sections.get(section as usize) {
            Some(Section::Data { data, .. }) => data.as_ref().get(offset_in_section as usize..),
            _ => None,
        }
    }

    pub fn get_section_data_for_rel_offset_mut<'b>(
        &'b mut self,
        offset: u32,
    ) -> Option<&'b mut [u8]> {
        let (section, offset_in_section) = self.offset_in_rel_to_section_offset(offset)?;
        match self.sections.get_mut(section as usize) {
            Some(Section::Data { data, .. }) => data.to_mut().get_mut(offset_in_section as usize..),
            _ => None,
        }
    }
}
