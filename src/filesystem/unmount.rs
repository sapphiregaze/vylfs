use std::{ffi::CString, io, path::Path};

use libc::{geteuid, umount2, MNT_FORCE};

pub fn unmount(mount_point: &Path) -> io::Result<()> {
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
