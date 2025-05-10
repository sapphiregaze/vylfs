use std::{fs, io, path::PathBuf};

pub fn view() -> io::Result<()> {
    let log_path = PathBuf::from("/tmp/vylfs.out");

    match fs::read_to_string(&log_path) {
        Ok(contents) => print!("{contents}"),
        Err(ref err) if err.kind() == io::ErrorKind::NotFound => {
            println!("no log available");
        }
        Err(err) => return Err(err),
    }

    Ok(())
}
