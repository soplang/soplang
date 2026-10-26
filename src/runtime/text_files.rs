//! Convenience UTF-8 text file operations for the 3.3 initial file stage.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use crate::error::{file_read_error, file_write_error, SoplangError};

pub fn read_utf8(path: &str) -> Result<String, SoplangError> {
    let path = Path::new(path);
    fs::read_to_string(path).map_err(|error| file_read_error(path, &error))
}

pub fn write_utf8(path: &str, contents: &str, append: bool) -> Result<(), SoplangError> {
    let path = Path::new(path);
    if append {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|error| file_write_error(path, &error))?;
        file.write_all(contents.as_bytes())
            .map_err(|error| file_write_error(path, &error))?;
        return Ok(());
    }
    fs::write(path, contents).map_err(|error| file_write_error(path, &error))
}
