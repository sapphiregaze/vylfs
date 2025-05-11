use std::error::Error;
use std::fs::File;
use std::path::Path;

use daemonize::Daemonize;
use fuser::{mount2, MountOption};
use tracing::info;

use crate::filesystem::VylFs;

pub fn mount(root_dir: &Path, mount_point: &Path) -> Result<(), Box<dyn Error>> {
    if !root_dir.exists() {
        return Err(format!("root directory '{}' does not exist", root_dir.display()).into());
    }
    if !root_dir.is_dir() {
        return Err(format!("root path '{}' is not a directory", root_dir.display()).into());
    }

    if !mount_point.exists() {
        return Err(format!("mount point '{}' does not exist", mount_point.display()).into());
    }
    if !mount_point.is_dir() {
        return Err(format!("mount point '{}' is not a directory", mount_point.display()).into());
    }

    let stdout = File::create("/tmp/vylfs.out")?;
    let stderr = File::create("/tmp/vylfs.err")?;

    Daemonize::new()
        .stdout(stdout)
        .stderr(stderr)
        .working_directory(".")
        .start()
        .map_err(|e| format!("failed to daemonize: {e}"))?;

    let options = vec![
        MountOption::FSName("vylfs".to_string()),
        MountOption::AutoUnmount,
        MountOption::AllowRoot,
    ];

    let fs = VylFs::new(mount_point);
    mount2(fs, mount_point, &options)?;
    info!("Unmounted '{}' and exiting daemon", mount_point.display());

    Ok(())
}
