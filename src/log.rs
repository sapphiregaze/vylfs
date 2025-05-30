use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

/// Prints the `vylfs` log file from `/tmp/vylfs.out` if it exists.
pub fn view() -> io::Result<()> {
    view_file(&PathBuf::from("/tmp/vylfs.out"), &mut io::stdout())
}

fn view_file<W: io::Write>(file_path: &Path, writer: &mut W) -> io::Result<()> {
    match fs::read_to_string(file_path) {
        Ok(content) => write!(writer, "{content}")?,
        Err(ref err) if err.kind() == io::ErrorKind::NotFound => {
            writeln!(writer, "no logs available")?;
        }
        Err(err) => return Err(err),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Cursor;
    use std::io::{self};

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_view_log_exists() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let log_file_path = temp_dir.path().join("vylfs.out");
        let expected_contents = "This is a log entry.\nAnother line.";
        fs::write(&log_file_path, expected_contents)?;

        let mut buffer = Cursor::new(Vec::new());
        let result = view_file(&log_file_path, &mut buffer);
        assert!(result.is_ok(), "Expected Ok(()), but got {:?}", result);

        let output = String::from_utf8(buffer.into_inner()).unwrap();
        assert_eq!(output, expected_contents);

        Ok(())
    }

    #[test]
    fn test_view_log_not_found() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let non_existent_log_path = temp_dir.path().join("non_existent_log.out");

        let mut buffer = Cursor::new(Vec::new());
        let result = view_file(&non_existent_log_path, &mut buffer);
        assert!(result.is_ok(), "Expected Ok(()), but got {:?}", result);

        let output = String::from_utf8(buffer.into_inner()).unwrap();
        assert_eq!(output.trim(), "no logs available");

        Ok(())
    }

    #[test]
    fn test_view_log_permission_denied() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let denied_path = temp_dir.path().join("denied.out");

        fs::create_dir(&denied_path)?;

        let mut buffer = Cursor::new(Vec::new());
        let result = view_file(&denied_path, &mut buffer);
        assert!(result.is_err(), "Expected an error, but got Ok(())");

        let err = result.unwrap_err();
        assert_ne!(err.kind(), io::ErrorKind::NotFound);
        let output = String::from_utf8(buffer.into_inner()).unwrap();
        assert!(
            output.is_empty(),
            "Buffer should be empty on error before println!"
        );

        Ok(())
    }
}
