use std::{error::Error, fs, io, path::PathBuf};

pub fn view() -> Result<(), Box<dyn Error>> {
    let log_path = PathBuf::from("/tmp/vylfs.out");

    match fs::read_to_string(&log_path) {
        Ok(contents) => {
            print!("{contents}");
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            println!("No log available.");
        }
        Err(err) => {
            return Err(err.into());
        }
    }

    Ok(())
}
