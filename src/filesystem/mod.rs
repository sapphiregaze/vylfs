mod inode;
pub mod mount;
pub mod unmount;

use std::{
    path::Path,
    time::{Duration, SystemTime},
};

use fuser::{FileAttr, FileType, Filesystem, ReplyAttr};
use libc::{getegid, geteuid};
use tracing::info;

use inode::generate;

pub struct VylFs {
    ttl: Duration,
    root_attr: FileAttr,
}

impl VylFs {
    pub fn new(mount_point: &Path) -> Self {
        let ino = generate(mount_point);
        let uid = unsafe { geteuid() };
        let gid = unsafe { getegid() };

        let root_attr: FileAttr = FileAttr {
            ino,
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
        }
    }
}

impl Filesystem for VylFs {
    fn init(
        &mut self,
        _req: &fuser::Request<'_>,
        _config: &mut fuser::KernelConfig,
    ) -> Result<(), i32> {
        info!("Filesystem initialized");
        Ok(())
    }

    fn destroy(&mut self) {
        info!("Filesystem destroyed");
    }

    fn lookup(
        &mut self,
        _req: &fuser::Request<'_>,
        _parent: u64,
        _name: &std::ffi::OsStr,
        _reply: fuser::ReplyEntry,
    ) {
        info!("Lookup called");
    }

    fn getattr(&mut self, _req: &fuser::Request<'_>, ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
        match ino {
            1 => reply.attr(&self.ttl, &self.root_attr),
            _ => reply.error(libc::ENOENT),
        }
    }
}
