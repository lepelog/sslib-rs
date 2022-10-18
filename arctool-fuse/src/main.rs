use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::File,
    io::{self, Read, Seek},
    os::unix::prelude::MetadataExt,
    path::PathBuf,
    time::{Duration, SystemTime},
};

use clap::Parser;
use fuser::{mount2, FileAttr, FileType, Filesystem, MountOption};
use nlzss11::DecompressError;
use u8file::{Entry, U8File};

#[derive(Parser)]
/// program to mount .arc and .arc.LZ files as a fuse filesystem
struct Args {
    /// path to the archive
    archive: PathBuf,
    /// path where the mount should be
    mount: PathBuf,
}

#[derive(thiserror::Error, Debug)]
enum MyError {
    #[error("io error: {0:?}")]
    Io(#[from] io::Error),
    #[error("unknown filetype")]
    UnknownFile,
    #[error("decompress error {0:?}")]
    DecompressError(#[from] DecompressError),
    #[error("arc parse error {0:?}")]
    U8ParseError(#[from] u8file::U8ParseError),
}

// map "inodes" to entries:
// 1 is the root node, then perform depth first recursive lookup

struct InodeSupplier {
    state: u64,
}

impl InodeSupplier {
    pub fn new() -> Self {
        Self { state: 0 }
    }

    pub fn next(&mut self) -> u64 {
        self.state += 1;
        self.state
    }
}

struct U8Filesystem {
    u8file: U8File,
    inode_entires: HashMap<u64, InodeEntry>,
    common_info: CommonInfo,
}

struct CommonInfo {
    gid: u32,
    uid: u32,
    atime: SystemTime,
    crtime: SystemTime,
    ctime: SystemTime,
    mtime: SystemTime,
}

enum InodeEntry {
    DirEntry {
        name: String,
        files: Vec<u64>,
    },
    FileEntry {
        name: String,
        offset: u32,
        size: u32,
    },
}

fn construct_inode_map(
    name: &str,
    entries: &[Entry],
    inode_supplier: &mut InodeSupplier,
    inode_map: &mut HashMap<u64, InodeEntry>,
) -> u64 {
    let cur_inode = inode_supplier.next();
    let child_inodes = entries
        .iter()
        .map(|entry| match entry {
            Entry::DirEntry { name, files } => {
                construct_inode_map(name, files, inode_supplier, inode_map)
            }
            Entry::FileRefEntry {
                name,
                offset,
                length,
            } => {
                let file_inode = inode_supplier.next();
                inode_map.insert(
                    file_inode,
                    InodeEntry::FileEntry {
                        name: name.clone(),
                        offset: *offset,
                        size: *length,
                    },
                );
                file_inode
            }
            Entry::FileDataEntry { .. } => unreachable!(),
        })
        .collect();
    inode_map.insert(
        cur_inode,
        InodeEntry::DirEntry {
            name: name.to_string(),
            files: child_inodes,
        },
    );
    cur_inode
}

impl InodeEntry {
    pub fn get_name(&self) -> &String {
        match self {
            Self::DirEntry { name, .. } => name,
            Self::FileEntry { name, .. } => name,
        }
    }

    pub fn get_name_os(&self) -> &OsStr {
        OsStr::new(self.get_name())
    }

    pub fn get_file_type(&self) -> FileType {
        match self {
            Self::DirEntry { .. } => FileType::Directory,
            Self::FileEntry { .. } => FileType::RegularFile,
        }
    }

    pub fn to_file_attr(
        &self,
        ino: u64,
        common: &CommonInfo,
        inode_map: &HashMap<u64, InodeEntry>,
    ) -> FileAttr {
        match self {
            InodeEntry::DirEntry { files, .. } => FileAttr {
                ino,
                atime: common.atime,
                crtime: common.crtime,
                ctime: common.ctime,
                mtime: common.mtime,
                blksize: 512,
                blocks: files.len() as u64 / 512,
                size: files.len() as u64,
                perm: 0o555,
                gid: common.gid,
                uid: common.uid,
                flags: 0,
                rdev: 0,
                kind: FileType::Directory,
                nlink: 2 + files
                    .iter()
                    .filter(|f| matches!(inode_map.get(f), Some(InodeEntry::DirEntry { .. })))
                    .count() as u32,
            },
            InodeEntry::FileEntry { size, .. } => FileAttr {
                ino,
                atime: common.atime,
                crtime: common.crtime,
                ctime: common.ctime,
                mtime: common.mtime,
                blksize: 512,
                blocks: *size as u64 / 512,
                size: *size as u64,
                perm: 0o444,
                gid: common.gid,
                uid: common.uid,
                flags: 0,
                rdev: 0,
                kind: FileType::RegularFile,
                nlink: 1,
            },
        }
    }
}

impl Filesystem for U8Filesystem {
    fn getattr(&mut self, _req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyAttr) {
        if let Some(entry) = self.inode_entires.get(&ino) {
            reply.attr(
                &Duration::MAX,
                &entry.to_file_attr(ino, &self.common_info, &self.inode_entires),
            );
        } else {
            reply.error(libc::ENOENT)
        }
    }

    fn lookup(
        &mut self,
        req: &fuser::Request<'_>,
        parent: u64,
        name: &std::ffi::OsStr,
        reply: fuser::ReplyEntry,
    ) {
        if let Some(entry) = self.inode_entires.get(&parent) {
            match entry {
                InodeEntry::DirEntry { files, .. } => {
                    for inode in files.iter() {
                        if let Some(fileentry) = self.inode_entires.get(inode) {
                            if OsStr::new(fileentry.get_name()) == name {
                                reply.entry(
                                    &Duration::MAX,
                                    &fileentry.to_file_attr(
                                        *inode,
                                        &self.common_info,
                                        &self.inode_entires,
                                    ),
                                    req.unique(),
                                );
                                return;
                            }
                        }
                    }
                    reply.error(libc::ENOENT);
                }
                _ => reply.error(libc::ENOTDIR),
            }
        } else {
            reply.error(libc::ENOENT);
        }
    }

    fn opendir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        flags: i32,
        reply: fuser::ReplyOpen,
    ) {
        if let Some(entry) = self.inode_entires.get(&ino) {
            if matches!(entry, InodeEntry::DirEntry { .. }) {
                match flags & libc::O_ACCMODE {
                    libc::O_RDONLY => (),
                    _ => {
                        reply.error(libc::EACCES);
                        return;
                    }
                }
                reply.opened(0, 0);
            } else {
                reply.error(libc::ENOTDIR);
            }
        } else {
            reply.error(libc::ENOENT);
        }
    }

    fn readdir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: fuser::ReplyDirectory,
    ) {
        if let Some(entry) = self.inode_entires.get(&ino) {
            match entry {
                InodeEntry::DirEntry { files, .. } => {
                    for (i, file_ino) in files.iter().skip(offset as usize).enumerate() {
                        if let Some(file_entry) = self.inode_entires.get(file_ino) {
                            if reply.add(
                                *file_ino,
                                offset + i as i64 + 1,
                                file_entry.get_file_type(),
                                file_entry.get_name_os(),
                            ) {
                                // buffer is full
                                return;
                            }
                        }
                    }
                    reply.ok();
                }
                _ => reply.error(libc::ENOTDIR),
            }
        } else {
            reply.error(libc::ENOENT);
        }
    }

    fn open(&mut self, _req: &fuser::Request<'_>, ino: u64, flags: i32, reply: fuser::ReplyOpen) {
        if let Some(entry) = self.inode_entires.get(&ino) {
            if matches!(entry, InodeEntry::FileEntry { .. }) {
                match flags & libc::O_ACCMODE {
                    libc::O_RDONLY => (),
                    _ => {
                        reply.error(libc::EACCES);
                        return;
                    }
                }
                reply.opened(0, 0);
            } else {
                reply.error(libc::EISDIR);
            }
        } else {
            reply.error(libc::ENOENT);
        }
    }

    fn read(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _fh: u64,
        offset_in_file: i64,
        request_size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: fuser::ReplyData,
    ) {
        if let Some(entry) = self.inode_entires.get(&ino) {
            match entry {
                InodeEntry::FileEntry { offset, size, .. } => {
                    let data = self.u8file.get_data_from_offset_len(*offset, *size);
                    reply.data(
                        data.get(
                            offset_in_file as usize
                                ..offset_in_file as usize + request_size as usize,
                        )
                        .unwrap_or(&[]),
                    );
                }
                _ => reply.error(libc::EISDIR),
            }
        } else {
            reply.error(libc::ENOENT);
        }
    }
}

fn main() -> Result<(), MyError> {
    env_logger::init();
    let args = Args::parse();
    let mut f = File::open(&args.archive)?;
    let meta = f.metadata()?;

    let mut probe = [0; 4];
    f.read_exact(&mut probe)?;
    f.seek(io::SeekFrom::Start(0))?;
    let arc_data = if probe[0] == 0x11 {
        // probably a nlzss11 compressed file
        let mut compressed_bytes = Vec::new();
        f.read_to_end(&mut compressed_bytes)?;
        nlzss11::decompress(&compressed_bytes)?
    } else if u32::from_be_bytes(probe) == u8file::MAGIC_HEADER {
        let mut bytes = Vec::new();
        f.read_to_end(&mut bytes)?;
        bytes
    } else {
        return Err(MyError::UnknownFile);
    };
    let u8file = u8file::U8File::from_vec(arc_data)?;
    for file in u8file.get_all_paths() {
        println!("{file}");
    }
    let mut inode_entires = HashMap::new();
    construct_inode_map(
        ".",
        u8file.get_root_entry(),
        &mut InodeSupplier::new(),
        &mut inode_entires,
    );
    let filesystem = U8Filesystem {
        u8file,
        common_info: CommonInfo {
            gid: meta.gid(),
            uid: meta.uid(),
            atime: meta.accessed()?,
            crtime: meta.created()?,
            ctime: meta.created()?,
            mtime: meta.modified()?,
        },
        inode_entires,
    };
    mount2(
        filesystem,
        &args.mount,
        &[
            MountOption::AutoUnmount,
            MountOption::FSName("arc-fs".to_string()),
            MountOption::RO,
        ],
    )?;
    Ok(())
}
