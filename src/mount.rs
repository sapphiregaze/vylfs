use std::error::Error;
use std::fs::File;
use std::path::Path;

use daemonize::Daemonize;
use fuser::{mount2, Filesystem, MountOption};
use tracing::info;

struct VylFs;

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
}

pub fn mount(root_dir: &Path, mount_point: &Path) -> Result<(), Box<dyn Error>> {
    if !root_dir.exists() {
        return Err(format!("Root directory '{}' does not exist", root_dir.display()).into());
    }
    if !root_dir.is_dir() {
        return Err(format!("Root path '{}' is not a directory", root_dir.display()).into());
    }

    if !mount_point.exists() {
        return Err(format!("Mount point '{}' does not exist", mount_point.display()).into());
    }
    if !mount_point.is_dir() {
        return Err(format!("Mount point '{}' is not a directory", mount_point.display()).into());
    }

    let stdout = File::create("/tmp/vylfs.out")?;
    let stderr = File::create("/tmp/vylfs.err")?;

    Daemonize::new()
        .stdout(stdout)
        .stderr(stderr)
        .working_directory(".")
        .start()
        .map_err(|e| format!("Failed to daemonize: {e}"))?;

    let options = vec![
        MountOption::FSName("vylfs".to_string()),
        MountOption::AutoUnmount,
        MountOption::AllowRoot,
    ];

    mount2(VylFs, mount_point, &options)?;
    info!("Unmounted '{}' and exiting daemon", mount_point.display());

    Ok(())
}
