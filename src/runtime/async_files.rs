//! Whole-file I/O tasks. Only owned paths and byte buffers cross to Tokio.

use std::io;
use std::path::PathBuf;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::stdlib::{expect_arity, expect_bytes_contents, expect_text_contents, expect_text_path};
use super::tasks::{self, ExternalKind, OwnedFailure, OwnedResult};
use super::value::Value;
use crate::error::{runtime_error, SoplangError};

const MAX_FILE_BYTES: usize = 16 * 1024 * 1024;

fn read(args: Vec<Value>, name: &str, text: bool) -> Result<Value, SoplangError> {
    expect_arity(&args, name, 1)?;
    let path = PathBuf::from(expect_text_path(&args[0], name)?);
    tasks::spawn_external(
        async move {
            let file = tokio::fs::File::open(&path)
                .await
                .map_err(|error| OwnedFailure::Read(path.clone(), error))?;
            let mut limited = file.take((MAX_FILE_BYTES + 1) as u64);
            let mut bytes = Vec::new();
            limited
                .read_to_end(&mut bytes)
                .await
                .map_err(|error| OwnedFailure::Read(path.clone(), error))?;
            if bytes.len() > MAX_FILE_BYTES {
                return Err(OwnedFailure::FileTooLarge(path));
            }
            if text {
                String::from_utf8(bytes)
                    .map(OwnedResult::Text)
                    .map_err(|error| {
                        OwnedFailure::Read(path, io::Error::new(io::ErrorKind::InvalidData, error))
                    })
            } else {
                Ok(OwnedResult::Bytes(bytes))
            }
        },
        ExternalKind::File,
    )
}

fn write(args: Vec<Value>, name: &str, text: bool, append: bool) -> Result<Value, SoplangError> {
    expect_arity(&args, name, 2)?;
    let path = PathBuf::from(expect_text_path(&args[0], name)?);
    let bytes = if text {
        expect_text_contents(&args[1])?.into_bytes()
    } else {
        expect_bytes_contents(&args[1], name)?
    };
    if bytes.len() > MAX_FILE_BYTES {
        return Err(runtime_error(
            "Xogta faylka la qorayo waxay ka weyn tahay 16 MiB.",
            0,
            0,
        ));
    }
    tasks::spawn_external(
        async move {
            if append {
                let mut file = tokio::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                    .await
                    .map_err(|error| OwnedFailure::Write(path.clone(), error))?;
                file.write_all(&bytes)
                    .await
                    .map_err(|error| OwnedFailure::Write(path, error))?;
            } else {
                tokio::fs::write(&path, bytes)
                    .await
                    .map_err(|error| OwnedFailure::Write(path, error))?;
            }
            Ok(OwnedResult::Unit)
        },
        ExternalKind::File,
    )
}

pub(super) fn read_text(args: Vec<Value>) -> Result<Value, SoplangError> {
    read(args, "akhrifayl_shaqo", true)
}

pub(super) fn read_bytes(args: Vec<Value>) -> Result<Value, SoplangError> {
    read(args, "akhrif_baytyo_shaqo", false)
}

pub(super) fn write_text(args: Vec<Value>) -> Result<Value, SoplangError> {
    write(args, "qor_fayl_shaqo", true, false)
}

pub(super) fn append_text(args: Vec<Value>) -> Result<Value, SoplangError> {
    write(args, "kudar_fayl_shaqo", true, true)
}

pub(super) fn write_bytes(args: Vec<Value>) -> Result<Value, SoplangError> {
    write(args, "qor_baytyo_shaqo", false, false)
}

pub(super) fn append_bytes(args: Vec<Value>) -> Result<Value, SoplangError> {
    write(args, "kudar_baytyo_shaqo", false, true)
}
