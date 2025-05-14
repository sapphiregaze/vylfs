use std::{ffi::CString, io, path::Path};

use libc::{MNT_FORCE, geteuid, umount2};

use crate::filesystem::directory::validate_dir;

/// Unmounts the given mount point using `umount2`, requires root privileges.
pub fn unmount(mount_point: &Path) -> io::Result<()> {
    validate_dir(mount_point)?;

    let is_root = unsafe { geteuid() == 0 };
    if !is_root {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "must be root to unmount with umount2",
        ));
    }

    let c_path = CString::new(mount_point.to_str().unwrap()).unwrap();
    let res = unsafe { umount2(c_path.as_ptr(), MNT_FORCE) };
    if res != 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}
