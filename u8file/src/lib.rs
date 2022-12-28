use std::{
    borrow::{Borrow, BorrowMut},
    io::{Cursor, Seek, SeekFrom, Write},
    ops::Neg,
};

use byteorder::{ReadBytesExt, WriteBytesExt, BE};

#[derive(Debug)]
pub enum Entry {
    DirEntry {
        name: String,
        files: Vec<Entry>,
    },
    FileEntry {
        name: String,
        data: FileEntry
    },
}

#[derive(Debug)]
pub enum FileEntry {
    Ref {
        offset: u32,
        length: u32,
    },
    Data(Vec<u8>),
}

pub struct U8File {
    data: Vec<u8>,
    root: Vec<Entry>,
}

#[derive(thiserror::Error, Debug)]
pub enum U8ParseError {
    #[error("unexpected EOF")]
    UnexpectedEoF,
    #[error("invalid node")]
    InvalidNode,
    #[error("invalid magic")]
    InvalidMagic,
    #[error("invalid node decoding")]
    InvalidNodeDecoding,
}

impl From<std::io::Error> for U8ParseError {
    fn from(_: std::io::Error) -> Self {
        U8ParseError::UnexpectedEoF
    }
}

#[derive(Debug)]
enum RawNode {
    RawFileNode {
        string_offset: u32,
        data_start: u32,
        data_size: u32,
    },
    RawDirNode {
        string_offset: u32,
        next_parent_index: u32,
    },
}

impl RawNode {
    const SIZE: u32 = 0xC;

    fn get_string_offset(&self) -> u32 {
        match self {
            RawNode::RawFileNode { string_offset, .. } => *string_offset,
            RawNode::RawDirNode { string_offset, .. } => *string_offset,
        }
    }
}

fn read_raw_node(data: &mut Cursor<Vec<u8>>, pos: u32) -> Result<RawNode, U8ParseError> {
    data.seek(SeekFrom::Start(pos.into()))?;
    let node_type = data.read_u8()?;
    if node_type == 0 {
        // 0 is data file
        let string_offset = data.read_u24::<BE>()?;
        let data_start = data.read_u32::<BE>()?;
        let data_size = data.read_u32::<BE>()?;
        Ok(RawNode::RawFileNode {
            string_offset,
            data_start,
            data_size,
        })
    } else if node_type == 1 {
        // 1 is dir
        let string_offset = data.read_u24::<BE>()?;
        let _parent_index = data.read_u32::<BE>()?;
        let next_parent_index = data.read_u32::<BE>()?;
        Ok(RawNode::RawDirNode {
            string_offset,
            next_parent_index,
        })
    } else {
        Err(U8ParseError::InvalidNode)
    }
}

fn read_ascii(data: &mut Cursor<Vec<u8>>, pos: u32) -> Result<String, U8ParseError> {
    data.seek(SeekFrom::Start(pos.into()))?;
    let mut buf = Vec::new();
    loop {
        let read = data.read_u8()?;
        if read == 0 {
            break;
        }
        buf.push(read);
    }

    if !buf.is_ascii() {
        return Err(U8ParseError::InvalidNodeDecoding);
    }

    // we check that the buffer is ascii directly above
    Ok(String::from_utf8(buf).unwrap())
}

impl Entry {
    pub fn is_dir(&self) -> bool {
        matches!(self, Entry::DirEntry { .. })
    }

    pub fn is_ref(&self) -> bool {
        matches!(self, Entry::FileEntry { data: FileEntry::Ref {..}, .. })
    }

    pub fn is_data(&self) -> bool {
        matches!(self, Entry::FileEntry { data: FileEntry::Data(..), .. })
    }

    pub fn get_name(&self) -> &String {
        match self {
            Self::DirEntry { name, .. } => name,
            Self::FileEntry { name, .. } => name,
        }
    }
}

pub const MAGIC_HEADER: u32 = 0x55AA382D;

impl U8File {
    /// reads a byte Vector into an U8File or returns an Error
    pub fn from_vec(v: Vec<u8>) -> Result<Self, U8ParseError> {
        let mut c = Cursor::new(v);
        let header = c.read_u32::<BE>()?;
        if MAGIC_HEADER != header {
            return Err(U8ParseError::InvalidMagic);
        }
        let first_node_offset = c.read_u32::<BE>()?;

        let first_node = read_raw_node(&mut c, first_node_offset)?;

        let total_node_count = match first_node {
            RawNode::RawDirNode {
                next_parent_index, ..
            } => next_parent_index,
            _ => return Err(U8ParseError::InvalidNode),
        };

        let string_pool_offset = first_node_offset + total_node_count * 12;

        // let root_node_name = read_ascii(&mut c, string_pool_offset)?;

        let root = read_nodes_recursive(
            &mut c,
            1,
            total_node_count,
            first_node_offset,
            string_pool_offset,
        )?;

        Ok(U8File {
            root,
            data: c.into_inner(),
        })
    }

    pub fn get_root_entry(&self) -> &Vec<Entry> {
        &self.root
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn get_entry_data<'a>(&'a self, path: &str) -> Option<&[u8]> {
        self.get_entry(path)
            .and_then(|entry| self.get_data_from_entry(entry))
    }

    pub fn set_entry_data(&mut self, path: &str, new_data: Vec<u8>) -> bool {
        self.get_entry_mut(path).map_or(false, |entry| {
            match entry {
                Entry::DirEntry { .. } => false,
                Entry::FileEntry { data, .. } => {
                    *data = FileEntry::Data(new_data);
                    true
                }
            }
        })
    }

    pub fn get_data_from_offset_len(&self, offset: u32, length: u32) -> &[u8] {
        &self.data[offset as usize..][..length as usize]
    }

    pub fn get_data_from_entry<'a>(&'a self, entry: &'a Entry) -> Option<&'a [u8]> {
        match entry {
            Entry::DirEntry { .. } => None,
            Entry::FileEntry { data: FileEntry::Data(data), .. } => Some(data),
            &Entry::FileEntry { data: FileEntry::Ref { offset, length }, .. } => {
                Some(&self.data[offset as usize..][..length as usize])
            }
        }
    }

    /// returns a reference to the entry specified by the path
    /// a starting "/" is ignored
    pub fn get_entry<'a>(&'a self, path: &str) -> Option<&'a Entry> {
        let mut parts_iter = path.split('/').peekable();
        // allow starting with leading slash or not
        if parts_iter.peek() == Some(&"") {
            parts_iter.next();
        }
        // get the first entry
        let first_part = parts_iter.next()?;
        let mut entry = self
            .root
            .iter()
            .find(|entry| entry.get_name() == first_part)?;
        for part in parts_iter {
            entry = match entry {
                Entry::DirEntry { files, .. } => {
                    files.iter().find(|entry| entry.get_name() == part)?
                }
                _ => return None,
            }
        }
        Some(entry.borrow())
    }

    /// returns a reference to the entry specified by the path
    /// a starting "/" is ignored
    pub fn get_entry_mut<'a>(&'a mut self, path: &str) -> Option<&'a mut Entry> {
        let mut parts_iter = path.split('/').peekable();
        // allow starting with leading slash or not
        if parts_iter.peek() == Some(&"") {
            parts_iter.next();
        }
        // get the first entry
        let first_part = parts_iter.next()?;
        let mut entry = self
            .root
            .iter_mut()
            .find(|entry| entry.get_name() == first_part)?;
        for part in parts_iter {
            entry = match entry {
                Entry::DirEntry { files, .. } => {
                    files.iter_mut().find(|entry| entry.get_name() == part)?
                }
                _ => return None,
            }
        }
        Some(entry.borrow_mut())
    }

    /// returns all full paths as a Vector
    pub fn get_all_paths(&self) -> Vec<String> {
        let mut result = Vec::new();
        Self::collect_paths_rec("", &self.root, &mut result);
        result
    }

    fn collect_paths_rec(dir_stack: &str, files: &[Entry], collector: &mut Vec<String>) {
        for entry in files.iter() {
            let mut full_name = dir_stack.to_string();
            full_name.push('/');
            full_name.push_str(entry.get_name());
            match entry {
                Entry::DirEntry { files, .. } => {
                    Self::collect_paths_rec(&full_name, files, collector);
                }
                _ => {
                    collector.push(full_name);
                }
            }
        }
    }

    pub fn write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        let mut rebuild_entries = Vec::new();
        let mut string_pool = Vec::new();
        // root node
        rebuild_entries.push(RebuildEntry::Dir {
            parent: 0,
            str_offset: 0,
            next_parent: 0, // filled in later
        });
        string_pool.push(0);

        // build structure for other nodes
        self.do_rebuild_rec(
            &self.root,
            0,
            &mut 0,
            &mut rebuild_entries,
            &mut string_pool,
        );
        let next_parent_pos = rebuild_entries.len() as u32;
        match rebuild_entries.get_mut(0).unwrap() {
            RebuildEntry::Dir { next_parent, .. } => {
                *next_parent = next_parent_pos;
            }
            _ => unreachable!(),
        }

        // actually write the data
        w.write_u32::<BE>(MAGIC_HEADER)?;
        // first node offset
        w.write_u32::<BE>(0x20)?;
        // size of nodes and string pool
        let node_and_string_pool_size =
            rebuild_entries.len() as u32 * RawNode::SIZE + string_pool.len() as u32;
        w.write_u32::<BE>(node_and_string_pool_size)?;
        let unpadded_data_offset = 0x20 + node_and_string_pool_size;
        let data_offset =
            unpadded_data_offset + (unpadded_data_offset as isize).neg().rem_euclid(0x20) as u32;
        w.write_u32::<BE>(data_offset)?;
        // pad until node section
        w.write_all(&[0; 16])?;
        // let string_pool_offset = 0x20 + rebuild_entries.len() as u32 * RawNode::SIZE;

        // nodes
        for node in rebuild_entries.iter() {
            match node {
                &RebuildEntry::Dir {
                    str_offset,
                    parent,
                    next_parent,
                } => {
                    w.write_u8(1)?;
                    w.write_u24::<BE>(str_offset)?;
                    w.write_u32::<BE>(parent)?;
                    w.write_u32::<BE>(next_parent)?;
                }
                &RebuildEntry::FileRef {
                    str_offset,
                    length,
                    new_offset,
                    ..
                } => {
                    w.write_u8(0)?;
                    w.write_u24::<BE>(str_offset)?;
                    w.write_u32::<BE>(data_offset + new_offset)?;
                    w.write_u32::<BE>(length)?;
                }
                RebuildEntry::FileData {
                    str_offset,
                    new_offset,
                    data,
                    ..
                } => {
                    w.write_u8(0)?;
                    w.write_u24::<BE>(*str_offset)?;
                    w.write_u32::<BE>(data_offset + *new_offset)?;
                    w.write_u32::<BE>(data.len() as u32)?;
                }
            }
        }

        // string pool
        w.write_all(&string_pool)?;

        // actual data
        let mut current_pos = unpadded_data_offset;
        let padding = [0; 32];
        for node in rebuild_entries.iter() {
            let needed_padding = (current_pos as isize).neg().rem_euclid(0x20) as u32;
            w.write_all(&padding[..needed_padding as usize])?;
            current_pos += needed_padding;
            match node {
                RebuildEntry::Dir { .. } => continue,
                RebuildEntry::FileData {
                    data, new_offset, ..
                } => {
                    assert_eq!(current_pos, *new_offset + data_offset);
                    w.write_all(data)?;
                    current_pos += data.len() as u32;
                }
                &RebuildEntry::FileRef {
                    offset,
                    length,
                    new_offset,
                    ..
                } => {
                    assert_eq!(current_pos, new_offset + data_offset);
                    w.write_all(&self.data[offset as usize..][..length as usize])?;
                    current_pos += length;
                }
            }
        }

        Ok(())
    }

    fn do_rebuild_rec<'a>(
        &'a self,
        files: &'a [Entry],
        parent: u32,
        data_offset: &mut u32,
        rebuild_entries: &mut Vec<RebuildEntry<'a>>,
        string_pool: &mut Vec<u8>,
    ) {
        for entry in files.iter() {
            let str_offset = string_pool.len() as u32;
            string_pool.extend(entry.get_name().as_bytes());
            string_pool.push(0);
            match entry {
                Entry::DirEntry {
                    files: sub_files, ..
                } => {
                    let current_entry_pos = rebuild_entries.len();
                    rebuild_entries.push(RebuildEntry::Dir {
                        next_parent: 0, // filled in later
                        parent,
                        str_offset,
                    });
                    self.do_rebuild_rec(
                        sub_files,
                        current_entry_pos as u32,
                        data_offset,
                        rebuild_entries,
                        string_pool,
                    );
                    let next_parent_pos = rebuild_entries.len() as u32;
                    match rebuild_entries.get_mut(current_entry_pos).unwrap() {
                        RebuildEntry::Dir { next_parent, .. } => {
                            *next_parent = next_parent_pos;
                        }
                        _ => unreachable!(),
                    }
                }
                &Entry::FileEntry { data: FileEntry::Ref { offset, length }, .. } => {
                    rebuild_entries.push(RebuildEntry::FileRef {
                        length,
                        offset,
                        str_offset,
                        new_offset: *data_offset,
                    });
                    *data_offset += length;
                    *data_offset += (*data_offset as isize).neg().rem_euclid(0x20) as u32;
                }
                Entry::FileEntry { data: FileEntry::Data(data), .. } => {
                    rebuild_entries.push(RebuildEntry::FileData {
                        str_offset,
                        data,
                        new_offset: *data_offset,
                    });
                    *data_offset += data.len() as u32;
                    *data_offset += (*data_offset as isize).neg().rem_euclid(0x20) as u32;
                }
            }
        }
    }
}

enum RebuildEntry<'a> {
    // str_offset is without the base stringpool offset
    // data offset is without the base data offset
    Dir {
        str_offset: u32,
        parent: u32,
        next_parent: u32,
    },
    FileRef {
        str_offset: u32,
        offset: u32,
        length: u32,
        new_offset: u32,
    },
    FileData {
        str_offset: u32,
        data: &'a Vec<u8>,
        new_offset: u32,
    },
}

fn read_nodes_recursive(
    data: &mut Cursor<Vec<u8>>,
    start_idx: u32,
    end_index: u32,
    first_node_offset: u32,
    string_pool_offset: u32,
) -> Result<Vec<Entry>, U8ParseError> {
    let mut files = Vec::new();
    let mut cur_idx = start_idx;
    while cur_idx < end_index {
        let node = read_raw_node(data, first_node_offset + cur_idx * 12)?;
        let node_name = read_ascii(data, string_pool_offset + node.get_string_offset())?;

        match node {
            RawNode::RawDirNode {
                next_parent_index, ..
            } => {
                files.push(Entry::DirEntry {
                    name: node_name,
                    files: read_nodes_recursive(
                        data,
                        cur_idx + 1,
                        next_parent_index,
                        first_node_offset,
                        string_pool_offset,
                    )?,
                });
                cur_idx = next_parent_index;
            }
            RawNode::RawFileNode {
                data_size,
                data_start,
                ..
            } => {
                files.push(Entry::FileEntry {
                    name: node_name,
                    data: FileEntry::Ref { 
                        offset: data_start,
                        length: data_size,
                    }
                });
                cur_idx += 1;
            }
        }
    }
    Ok(files)
}
