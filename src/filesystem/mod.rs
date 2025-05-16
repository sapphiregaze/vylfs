mod directory;
pub mod mount;
pub mod unmount;

use std::collections::HashMap;
use std::ffi::OsStr;
use std::time::Duration;
use std::time::SystemTime;

use fuser::FUSE_ROOT_ID;
use fuser::FileAttr;
use fuser::FileType;
use fuser::Filesystem;
use fuser::KernelConfig;
use fuser::ReplyAttr;
use fuser::ReplyCreate;
use fuser::ReplyData;
use fuser::ReplyDirectory;
use fuser::ReplyEmpty;
use fuser::ReplyEntry;
use fuser::ReplyWrite;
use fuser::Request;
use libc::getegid;
use libc::geteuid;
use tracing::info;

pub struct VylFs {
    ttl: Duration,
    inode_counter: u64,
    inodes: HashMap<u64, FileAttr>,
    entries: HashMap<(u64, String), u64>,
    file_data: HashMap<u64, Vec<u8>>,
}

impl VylFs {
    pub fn new() -> Self {
        let uid = unsafe { geteuid() };
        let gid = unsafe { getegid() };
        let ttl = Duration::from_secs(1);
        let root_attr: FileAttr = FileAttr {
            ino: FUSE_ROOT_ID,
            size: 4096,
            blocks: 8,
            atime: SystemTime::now(),
            mtime: SystemTime::now(),
            ctime: SystemTime::now(),
            crtime: SystemTime::now(),
            kind: FileType::Directory,
            perm: 0o755,
            nlink: 2,
            uid,
            gid,
            rdev: 0,
            blksize: 4096,
            flags: 0,
        };

        Self {
            ttl,
            inode_counter: FUSE_ROOT_ID + 1,
            inodes: HashMap::from([(FUSE_ROOT_ID, root_attr)]),
            entries: HashMap::new(),
            file_data: HashMap::new(),
        }
    }

    pub fn add_entry(&mut self, parent: u64, name: &str, attr: FileAttr) {
        let ino = attr.ino;
        self.inodes.insert(ino, attr);
        self.entries.insert((parent, name.to_string()), ino);
    }

    pub fn remove_entry(&mut self, ino: &u64, key: &(u64, String)) {
        self.inodes.remove(ino);
        self.entries.remove(key);
    }
}

impl Filesystem for VylFs {
    fn init(&mut self, _req: &Request<'_>, _config: &mut KernelConfig) -> Result<(), i32> {
        info!("Filesystem initialized");
        Ok(())
    }

    fn destroy(&mut self) {
        info!("Filesystem destroyed");
    }

    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let name_str = match name.to_str() {
            Some(s) => s,
            None => {
                reply.error(libc::EINVAL);
                return;
            }
        };

        match self.entries.get(&(parent, name_str.to_string())) {
            Some(&child_ino) => {
                if let Some(attr) = self.inodes.get(&child_ino) {
                    reply.entry(&self.ttl, attr, 0);
                } else {
                    reply.error(libc::ENOENT);
                }
            }
            None => reply.error(libc::ENOENT),
        }
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
        match self.inodes.get(&ino) {
            Some(attr) => reply.attr(&self.ttl, attr),
            None => reply.error(libc::ENOENT),
        }
    }

    fn readdir(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        let Some(dir_attr) = self.inodes.get(&ino) else {
            reply.error(libc::ENOENT);
            return;
        };

        if dir_attr.kind != FileType::Directory {
            reply.error(libc::ENOTDIR);
            return;
        }

        let parent_ino = if ino == FUSE_ROOT_ID {
            FUSE_ROOT_ID
        } else {
            self.entries
                .iter()
                .find_map(
                    |((parent, _name), &child)| if child == ino { Some(*parent) } else { None },
                )
                .unwrap_or(FUSE_ROOT_ID)
        };

        let mut entries = vec![
            (ino, FileType::Directory, ".".to_string()),
            (parent_ino, FileType::Directory, "..".to_string()),
        ];

        for ((parent, name), &child_ino) in &self.entries {
            if *parent == ino {
                if let Some(attr) = self.inodes.get(&child_ino) {
                    entries.push((attr.ino, attr.kind, name.to_string()));
                }
            }
        }

        for (i, (child_ino, file_type, name)) in
            entries.into_iter().enumerate().skip(offset as usize)
        {
            let full = reply.add(child_ino, (i + 1) as i64, file_type, name);
            if full {
                break;
            }
        }

        reply.ok();
    }

    fn create(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        mode: u32,
        _umask: u32,
        _flags: i32,
        reply: ReplyCreate,
    ) {
        let name_str = match name.to_str() {
            Some(s) => s,
            None => {
                reply.error(libc::EINVAL);
                return;
            }
        };

        if self.entries.contains_key(&(parent, name_str.to_string())) {
            reply.error(libc::EEXIST);
            return;
        }

        let ino = self.inode_counter;
        self.inode_counter += 1;

        let uid = unsafe { geteuid() };
        let gid = unsafe { getegid() };
        let attr = FileAttr {
            ino,
            size: 0,
            blocks: 0,
            atime: SystemTime::now(),
            mtime: SystemTime::now(),
            ctime: SystemTime::now(),
            crtime: SystemTime::now(),
            kind: FileType::RegularFile,
            perm: (mode & 0o7777) as u16,
            nlink: 1,
            uid,
            gid,
            rdev: 0,
            blksize: 4096,
            flags: 0,
        };

        self.add_entry(parent, name_str, attr);
        self.file_data.insert(ino, Vec::new());

        reply.created(&self.ttl, &attr, 0, 0, 0);
    }

    fn setattr(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        size: Option<u64>,
        atime: Option<fuser::TimeOrNow>,
        mtime: Option<fuser::TimeOrNow>,
        _ctime: Option<SystemTime>,
        _fh: Option<u64>,
        crtime: Option<SystemTime>,
        chgtime: Option<SystemTime>,
        _bkuptime: Option<SystemTime>,
        flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        if let Some(attr) = self.inodes.get_mut(&ino) {
            if let Some(new_mode) = mode {
                attr.perm = new_mode as u16;
            }

            if let Some(new_uid) = uid {
                attr.uid = new_uid;
            }

            if let Some(new_gid) = gid {
                attr.gid = new_gid;
            }

            if let Some(new_size) = size {
                attr.size = new_size;
                attr.blocks = new_size.div_ceil(attr.blksize as u64);
            }

            if let Some(a) = atime {
                attr.atime = match a {
                    fuser::TimeOrNow::SpecificTime(t) => t,
                    fuser::TimeOrNow::Now => SystemTime::now(),
                };
            }

            if let Some(m) = mtime {
                attr.mtime = match m {
                    fuser::TimeOrNow::SpecificTime(t) => t,
                    fuser::TimeOrNow::Now => SystemTime::now(),
                };
            }

            if let Some(c) = crtime {
                attr.crtime = c;
            }

            if let Some(c) = chgtime {
                attr.ctime = c;
            }

            if let Some(f) = flags {
                attr.flags = f;
            }

            reply.attr(&self.ttl, attr);
        } else {
            reply.error(libc::ENOENT);
        }
    }

    fn unlink(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        let name_str = match name.to_str() {
            Some(s) => s,
            None => {
                reply.error(libc::EINVAL);
                return;
            }
        };

        let key = (parent, name_str.to_string());
        match self.entries.remove(&key) {
            Some(ino) => {
                self.inodes.remove(&ino);
                reply.ok();
            }
            None => {
                reply.error(libc::ENOENT);
            }
        }
    }

    fn read(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        if let Some(content) = self.file_data.get(&ino) {
            let start = offset as usize;
            let end = (offset as usize + size as usize).min(content.len());

            if start > content.len() {
                reply.data(&[]);
            } else {
                reply.data(&content[start..end]);
            }
        } else {
            reply.error(libc::ENOENT);
        }
    }

    fn write(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        data: &[u8],
        _write_flags: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyWrite,
    ) {
        if let Some(buffer) = self.file_data.get_mut(&ino) {
            let offset = offset as usize;
            let end = offset + data.len();

            if buffer.len() < end {
                buffer.resize(end, 0);
            }

            buffer[offset..end].copy_from_slice(data);

            if let Some(attr) = self.inodes.get_mut(&ino) {
                attr.size = buffer.len() as u64;
                attr.blocks = if attr.size == 0 {
                    0
                } else {
                    std::cmp::max(8, attr.size.div_ceil(512))
                };
                attr.mtime = SystemTime::now();
            }

            reply.written(data.len() as u32);
        } else {
            reply.error(libc::ENOENT);
        }
    }

    fn mkdir(
        &mut self,
        _req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        mode: u32,
        _umask: u32,
        reply: ReplyEntry,
    ) {
        let name_str = match name.to_str() {
            Some(s) => s,
            None => {
                reply.error(libc::EINVAL);
                return;
            }
        };

        if self.entries.contains_key(&(parent, name_str.to_string())) {
            reply.error(libc::EEXIST);
            return;
        }

        let ino = self.inode_counter;
        self.inode_counter += 1;

        let uid = unsafe { geteuid() };
        let gid = unsafe { getegid() };
        let attr = FileAttr {
            ino,
            size: 4096,
            blocks: 8,
            atime: SystemTime::now(),
            mtime: SystemTime::now(),
            ctime: SystemTime::now(),
            crtime: SystemTime::now(),
            kind: FileType::Directory,
            perm: (mode & 0o7777) as u16,
            nlink: 2,
            uid,
            gid,
            rdev: 0,
            blksize: 4096,
            flags: 0,
        };

        self.add_entry(parent, name_str, attr);
        reply.entry(&self.ttl, &attr, 0);
    }

    fn rmdir(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        let name_str = match name.to_str() {
            Some(s) => s,
            None => {
                reply.error(libc::EINVAL);
                return;
            }
        };

        let key = (parent, name_str.to_string());
        let Some(&ino) = self.entries.get(&key) else {
            reply.error(libc::ENOENT);
            return;
        };

        let Some(attr) = self.inodes.get(&ino) else {
            reply.error(libc::ENOENT);
            return;
        };

        if attr.kind != FileType::Directory {
            reply.error(libc::ENOTDIR);
            return;
        }

        if self.entries.keys().any(|(p, _)| *p == ino) {
            reply.error(libc::ENOTEMPTY);
            return;
        }

        self.remove_entry(&ino, &key);
        reply.ok();
    }
}
