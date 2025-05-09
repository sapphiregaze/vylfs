use std::error::Error;
use std::path::Path;

pub fn mount(root_dir: &Path, mount_point: &Path) -> Result<(), Box<dyn Error>> {
    if !root_dir.exists() {
        return Err(format!("Root directory '{}' does not exist.", root_dir.display()).into());
    }
    if !root_dir.is_dir() {
        return Err(format!("Root path '{}' is not a directory.", root_dir.display()).into());
    }

    if !mount_point.exists() {
        return Err(format!("Mount point '{}' does not exist.", mount_point.display()).into());
    }
    if !mount_point.is_dir() {
        return Err(format!(
            "Mount point '{}' is not a directory.",
            mount_point.display()
        )
        .into());
    }

    Ok(())
}
