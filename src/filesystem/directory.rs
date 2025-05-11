use std::{io, path::Path};

/// Ensures the given path exists and is a directory.
pub fn validate_dir(path: &Path) -> io::Result<()> {
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("'{}' does not exist", path.display()),
        ));
    }
    if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("'{}' is not a directory", path.display()),
        ));
    }
    Ok(())
}

