//! Whole-file binary read and write for the 3.3 stage.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use super::bytes;
use super::value::Value;
use crate::error::{file_read_error, file_write_error, SoplangError};

pub fn read_file(path: &str) -> Result<Value, SoplangError> {
    let path = Path::new(path);
    let data = fs::read(path).map_err(|error| file_read_error(path, &error))?;
    Ok(bytes::from_raw_vec(data))
}

pub fn write_file(path: &str, data: &[u8], append: bool) -> Result<(), SoplangError> {
    let path = Path::new(path);
    if append {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|error| file_write_error(path, &error))?;
        file.write_all(data)
            .map_err(|error| file_write_error(path, &error))?;
        return Ok(());
    }
    fs::write(path, data).map_err(|error| file_write_error(path, &error))
}
