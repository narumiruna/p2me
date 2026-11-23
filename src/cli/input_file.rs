use std::{
    fs::{File, Metadata, OpenOptions},
    io::{self, ErrorKind},
    path::Path,
};

use serde_json::Value;

pub fn read_json(path: &Path) -> Result<Value, &'static str> {
    let value: Value = if path == Path::new("-") {
        serde_json::from_reader(io::stdin().lock())
    } else {
        let (file, _) = open(path).map_err(|error| match error.kind() {
            ErrorKind::InvalidInput => "the JSON file must be a regular file",
            _ => "could not open the JSON file",
        })?;
        serde_json::from_reader(io::BufReader::new(file))
    }
    .map_err(|_| "could not read a JSON object from input")?;
    if !value.is_object() {
        return Err("JSON input must be an object");
    }
    Ok(value)
}

pub fn open(path: &Path) -> io::Result<(File, Metadata)> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK | libc::O_NOCTTY);
    }

    let file = options.open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "not a regular file",
        ));
    }
    Ok((file, metadata))
}
