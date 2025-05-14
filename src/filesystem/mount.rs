use std::error::Error;
use std::fs::File;
use std::path::Path;

use daemonize::Daemonize;
use fuser::{MountOption, mount2};
use tracing::info;

use crate::filesystem::VylFs;
use crate::filesystem::directory::validate_dir;

/// Mounts the encrypted filesystem in a background daemon process.
pub fn mount(root_dir: &Path, mount_point: &Path) -> Result<(), Box<dyn Error>> {
    validate_dir(root_dir)?;
    validate_dir(mount_point)?;

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

    let fs = VylFs::new();
    mount2(fs, mount_point, &options)?;
    info!("Unmounted '{}' and exiting daemon", mount_point.display());

    Ok(())
}
