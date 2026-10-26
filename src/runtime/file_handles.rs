//! Explicit file handles, bounded reads, seeking, and atomic replacement.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::atomic::{AtomicI64, Ordering};

use super::context::{RuntimeLocal, RuntimeState};
use super::value::Value;
use crate::error::{codes, io_reason, runtime_error_ex, ErrorMeta, SoplangError};

const STATE: RuntimeLocal<RuntimeState> = RuntimeLocal::new(|state| state);

const MAX_READ: usize = 16 * 1024 * 1024;
const MAX_LINE: usize = 1024 * 1024;

static NEXT_ID: AtomicI64 = AtomicI64::new(1);

pub(super) struct OpenFile {
    file: File,
    mode: String,
}

fn error(message: impl Into<String>) -> SoplangError {
    runtime_error_ex(
        message,
        0,
        0,
        ErrorMeta::default().with_code(codes::E058_FILE),
    )
}

pub(super) fn open(path: &str, mode: &str) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        let mut options = OpenOptions::new();
        match mode {
            "akhris" => {
                options.read(true);
            }
            "qor" => {
                options.read(true).write(true).create(true).truncate(true);
            }
            "kudar" => {
                options.read(true).append(true).create(true);
            }
            "akhris_qor" => {
                options.read(true).write(true).create(true);
            }
            _ => {
                return Err(error(
                    "Habka faylka waa inuu noqdaa akhris, qor, kudar, ama akhris_qor.",
                ))
            }
        }
        let file = options.open(path).map_err(|failure| {
            error(format!(
                "Faylka '{path}' lama furin. {}",
                io_reason(&failure)
            ))
        })?;
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        state
            .files
            .borrow_mut()
            .get_or_insert_with(HashMap::new)
            .insert(
                id,
                OpenFile {
                    file,
                    mode: mode.to_string(),
                },
            );
        let mut fields = HashMap::new();
        fields.insert("aqoonsi".to_string(), Value::Int(id));
        fields.insert("nooc".to_string(), Value::Str("fayl".to_string()));
        fields.insert("hab".to_string(), Value::Str(mode.to_string()));
        Ok(Value::Object(std::rc::Rc::new(std::cell::RefCell::new(
            fields,
        ))))
    })
}

pub(super) fn read_bytes(handle: &Value, limit: usize) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        if limit == 0 || limit > MAX_READ {
            return Err(error(
                "Cabbirka akhriska faylka waa inuu u dhaxeeyaa 1 iyo 16777216.",
            ));
        }
        let id = handle_id(handle)?;
        let mut files = state.files.borrow_mut();
        let open = files
            .as_mut()
            .and_then(|files| files.get_mut(&id))
            .ok_or_else(|| error("Faylka waa la xidhay."))?;
        let mut buffer = vec![0u8; limit];
        let read = open.file.read(&mut buffer).map_err(|failure| {
            error(format!(
                "Akhriska faylka wuu fashilmay. {}",
                io_reason(&failure)
            ))
        })?;
        buffer.truncate(read);
        Ok(super::bytes::from_raw_vec(buffer))
    })
}

pub(super) fn read_line(handle: &Value) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        let id = handle_id(handle)?;
        let mut files = state.files.borrow_mut();
        let open = files
            .as_mut()
            .and_then(|files| files.get_mut(&id))
            .ok_or_else(|| error("Faylka waa la xidhay."))?;
        let mut bytes = Vec::new();
        let mut buf = [0u8; 1];
        loop {
            let read = open.file.read(&mut buf).map_err(|failure| {
                error(format!(
                    "Akhriska faylka wuu fashilmay. {}",
                    io_reason(&failure)
                ))
            })?;
            if read == 0 {
                if bytes.is_empty() {
                    return Ok(Value::Null);
                }
                break;
            }
            if buf[0] == b'\n' {
                break;
            }
            bytes.push(buf[0]);
            if bytes.len() > MAX_LINE {
                return Err(error("Xariiqda faylka aad ayay u dheer tahay."));
            }
        }
        if bytes.last() == Some(&b'\r') {
            bytes.pop();
        }
        let text = String::from_utf8(bytes)
            .map_err(|_| error("Xariiqda faylka ma aha qoraal UTF-8 ah."))?;
        Ok(Value::Str(text))
    })
}

pub(super) fn write(handle: &Value, data: &[u8]) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        let id = handle_id(handle)?;
        let mut files = state.files.borrow_mut();
        let open = files
            .as_mut()
            .and_then(|files| files.get_mut(&id))
            .ok_or_else(|| error("Faylka waa la xidhay."))?;
        if open.mode == "akhris" {
            return Err(error("Faylkan waa akhris keliya."));
        }
        open.file.write_all(data).map_err(|failure| {
            error(format!(
                "Qorista faylka way fashilantay. {}",
                io_reason(&failure)
            ))
        })?;
        Ok(Value::Null)
    })
}

pub(super) fn flush(handle: &Value) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        let id = handle_id(handle)?;
        let mut files = state.files.borrow_mut();
        let open = files
            .as_mut()
            .and_then(|files| files.get_mut(&id))
            .ok_or_else(|| error("Faylka waa la xidhay."))?;
        open.file.flush().map_err(|failure| {
            error(format!(
                "Nadiifinta faylka way fashilantay. {}",
                io_reason(&failure)
            ))
        })?;
        Ok(Value::Null)
    })
}

pub(super) fn close(handle: &Value) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        let id = handle_id(handle)?;
        let removed = state
            .files
            .borrow_mut()
            .get_or_insert_with(HashMap::new)
            .remove(&id);
        if removed.is_none() {
            return Err(error("Faylka waa la xidhay."));
        }
        Ok(Value::Null)
    })
}

pub(super) fn seek(handle: &Value, offset: i64, origin: &str) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        let from = match origin {
            "bilow" if offset < 0 => return Err(error("Raadinta bilowga ma noqon karto taban.")),
            "bilow" => SeekFrom::Start(offset as u64),
            "hadda" => SeekFrom::Current(offset),
            "dhamaad" => SeekFrom::End(offset),
            _ => {
                return Err(error(
                    "Asalka raadinta waa inuu noqdaa bilow, hadda, ama dhamaad.",
                ))
            }
        };
        let id = handle_id(handle)?;
        let mut files = state.files.borrow_mut();
        let open = files
            .as_mut()
            .and_then(|files| files.get_mut(&id))
            .ok_or_else(|| error("Faylka waa la xidhay."))?;
        let position = open.file.seek(from).map_err(|failure| {
            error(format!(
                "Raadinta faylka way fashilantay. {}",
                io_reason(&failure)
            ))
        })?;
        i64::try_from(position)
            .map(Value::Int)
            .map_err(|_| error("Meesha faylka aad ayay u weyn tahay."))
    })
}

pub(super) fn position(handle: &Value) -> Result<Value, SoplangError> {
    STATE.with(|state| {
        let id = handle_id(handle)?;
        let mut files = state.files.borrow_mut();
        let open = files
            .as_mut()
            .and_then(|files| files.get_mut(&id))
            .ok_or_else(|| error("Faylka waa la xidhay."))?;
        let position = open.file.stream_position().map_err(|failure| {
            error(format!(
                "Meesha faylka lama aqoonin. {}",
                io_reason(&failure)
            ))
        })?;
        i64::try_from(position)
            .map(Value::Int)
            .map_err(|_| error("Meesha faylka aad ayay u weyn tahay."))
    })
}

fn handle_id(value: &Value) -> Result<i64, SoplangError> {
    let Value::Object(object) = value else {
        return Err(error("Faylka waa inuu noqdaa walax."));
    };
    let object = object
        .try_borrow()
        .map_err(|_| error("Walaxda hadda lama akhriyi karo."))?;
    match object.get("nooc") {
        Some(Value::Str(kind)) if kind == "fayl" => {}
        _ => return Err(error("Nooca faylka ma saxna.")),
    }
    match object.get("aqoonsi") {
        Some(Value::Int(id)) => Ok(*id),
        _ => Err(error("Aqoonsiga faylka ma saxna.")),
    }
}
