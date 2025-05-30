use std::io;
use std::path::Path;

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

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_validate_dir_valid_directory() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let path = temp_dir.path();

        let result = validate_dir(path);
        assert!(result.is_ok(), "Expected Ok(()), but got {:?}", result);

        Ok(())
    }

    #[test]
    fn test_validate_dir_is_file() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test_file.txt");
        fs::File::create(&file_path)?;

        let result = validate_dir(&file_path);
        assert!(result.is_err(), "Expected an error, but got Ok(())");

        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("is not a directory"));

        Ok(())
    }

    #[test]
    fn test_validate_dir_does_not_exist() {
        let non_existent_path = Path::new("this/path/definitely/does/not/exist_123456789");

        let result = validate_dir(non_existent_path);
        assert!(result.is_err(), "Expected an error, but got Ok(())");

        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
        assert!(err.to_string().contains("does not exist"));
    }

    #[test]
    fn test_validate_dir_empty_path_does_not_exist() {
        let empty_path = Path::new("");

        let result = validate_dir(empty_path);
        assert!(result.is_err(), "Expected an error, but got Ok(())");

        let err = result.unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
        assert!(err.to_string().contains("does not exist"));
    }
}
