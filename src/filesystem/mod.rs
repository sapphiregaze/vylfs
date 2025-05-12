mod directory;
pub mod mount;
pub mod unmount;

use std::{
    collections::HashMap,
    ffi::OsStr,
    time::{Duration, SystemTime},
};

use fuser::{
    FileAttr, FileType, Filesystem, KernelConfig, ReplyAttr, ReplyCreate, ReplyData,
    ReplyDirectory, ReplyEmpty, ReplyEntry, ReplyWrite, Request, FUSE_ROOT_ID,
};
use libc::{getegid, geteuid};
use tracing::info;

pub struct VylFs {
    ttl: Duration,
    inode_counter: u64,
    inodes: HashMap<u64, FileAttr>,
    entries: HashMap<(u64, String), u64>,
    data: HashMap<u64, Vec<u8>>,
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

        let mut fs = Self {
            ttl,
            inode_counter: FUSE_ROOT_ID + 1,
            inodes: HashMap::new(),
            entries: HashMap::new(),
            data: HashMap::new(),
        };

        fs.inodes.insert(FUSE_ROOT_ID, root_attr);
        fs
    }

    pub fn add_entry(&mut self, parent: u64, name: &str, attr: FileAttr) {
        let ino = attr.ino;
        self.inodes.insert(ino, attr);
        self.entries.insert((parent, name.to_string()), ino);
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
        if ino != FUSE_ROOT_ID {
            reply.error(libc::ENOENT);
            return;
        }

        let mut entries = vec![
            (FUSE_ROOT_ID, FileType::Directory, ".".to_string()),
            (FUSE_ROOT_ID, FileType::Directory, "..".to_string()),
        ];

        for ((parent, name), &child_ino) in &self.entries {
            if *parent == ino {
                if let Some(attr) = self.inodes.get(&child_ino) {
                    entries.push((attr.ino, attr.kind, name.clone()));
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
        self.data.insert(ino, Vec::new());

        reply.created(&self.ttl, &attr, 0, 0, 0);
    }

    fn setattr(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _mode: Option<u32>,
        _uid: Option<u32>,
        _gid: Option<u32>,
        _size: Option<u64>,
        atime: Option<fuser::TimeOrNow>,
        mtime: Option<fuser::TimeOrNow>,
        _ctime: Option<SystemTime>,
        _fh: Option<u64>,
        _crtime: Option<SystemTime>,
        _chgtime: Option<SystemTime>,
        _bkuptime: Option<SystemTime>,
        _flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        if let Some(attr) = self.inodes.get_mut(&ino) {
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

            reply.attr(&self.ttl, attr);
        } else {
            reply.error(libc::ENOENT);
        }
    }

    fn unlink(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        let name_str = match name.to_str() {
            Some(s) => s.to_string(),
            None => {
                reply.error(libc::EINVAL);
                return;
            }
        };

        let key = (parent, name_str.clone());

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
        if let Some(content) = self.data.get(&ino) {
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
        if let Some(buffer) = self.data.get_mut(&ino) {
            let offset = offset as usize;
            let end = offset + data.len();

            if buffer.len() < end {
                buffer.resize(end, 0);
            }

            buffer[offset..end].copy_from_slice(data);

            if let Some(attr) = self.inodes.get_mut(&ino) {
                attr.size = buffer.len() as u64;
                attr.mtime = SystemTime::now();
            }

            reply.written(data.len() as u32);
        } else {
            reply.error(libc::ENOENT);
        }
    }
}
