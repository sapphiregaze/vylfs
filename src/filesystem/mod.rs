mod directory;
pub mod mount;
pub mod unmount;

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use fuser::{
    FileAttr, FileType, Filesystem, KernelConfig, ReplyAttr, ReplyDirectory, ReplyEntry, Request,
    FUSE_ROOT_ID,
};
use libc::{getegid, geteuid};
use tracing::info;

pub struct VylFs {
    ttl: Duration,
    root_attr: FileAttr,
    _root_path: PathBuf,
}

impl VylFs {
    pub fn new(mount_point: &Path) -> Self {
        let uid = unsafe { geteuid() };
        let gid = unsafe { getegid() };

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
            ttl: Duration::from_secs(1),
            root_attr,
            _root_path: mount_point.to_path_buf(),
        }
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

    fn lookup(&mut self, _req: &Request<'_>, _parent: u64, _name: &OsStr, _reply: ReplyEntry) {
        info!("Lookup called");
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
        match ino {
            FUSE_ROOT_ID => reply.attr(&self.ttl, &self.root_attr),
            _ => reply.error(libc::ENOENT),
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

        let entries = vec![
            (FUSE_ROOT_ID, FileType::Directory, "."),
            (FUSE_ROOT_ID, FileType::Directory, ".."),
        ];

        for (i, (inode, kind, name)) in entries.into_iter().enumerate().skip(offset as usize) {
            let offset = (i + 1) as i64;
            if reply.add(inode, offset, kind, name) {
                break;
            }
        }

        reply.ok();
    }
}
