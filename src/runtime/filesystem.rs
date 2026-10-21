//! Paths, metadata, and filesystem management for the 3.3 stage.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::UNIX_EPOCH;

use super::value::Value;
use crate::error::{path_io_error, type_error, SoplangError};

pub fn path_exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn path_info(path: &str) -> Result<Value, SoplangError> {
    let path = Path::new(path);
    if !path.exists() {
        return Ok(simple_object(&[("jir", Value::Bool(false))]));
    }
    let metadata = fs::symlink_metadata(path).map_err(|error| path_io_error(path, &error, "Akhriska xogta jidka"))?;
    let kind = if metadata.file_type().is_symlink() {
        "xidhiidh"
    } else if metadata.is_dir() {
        "gal"
    } else {
        "fayl"
    };
    let size = metadata.len() as i64;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    Ok(simple_object(&[
        ("jir", Value::Bool(true)),
        ("nooc", Value::Str(kind.to_string())),
        ("cabbir", Value::Int(size)),
        ("wax_laga_beddelay", Value::Int(modified)),
    ]))
}

pub fn join_paths(segments: &[String]) -> Result<String, SoplangError> {
    if segments.is_empty() {
        return Err(type_error(
            "isku_xidh() wuxuu u baahan yahay ugu yaraan hal qayb.",
            0,
            0,
        ));
    }
    let mut joined = PathBuf::from(&segments[0]);
    for segment in segments.iter().skip(1) {
        joined.push(segment);
    }
    Ok(joined.to_string_lossy().into_owned())
}

pub fn absolute_path(path: &str) -> Result<String, SoplangError> {
    let path = Path::new(path);
    let absolute = std::path::absolute(path)
        .map_err(|error| path_io_error(path, &error, "Baabi'idda jidka"))?;
    Ok(absolute.to_string_lossy().into_owned())
}

pub fn copy_path(from: &str, to: &str, overwrite: bool) -> Result<(), SoplangError> {
    let from = Path::new(from);
    let to = Path::new(to);
    if to.exists() && !overwrite {
        return Err(crate::error::external_error(
            format!("Fayl ama gal ayaa hore ugu jira '{to}'.", to = to.display()),
            "destination already exists".to_string(),
        ));
    }
    if let Some(parent) = to.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|error| path_io_error(to, &error, "Nuqulka faylka"))?;
        }
    }
    fs::copy(from, to).map_err(|error| path_io_error(from, &error, "Nuqulka faylka"))?;
    Ok(())
}

pub fn move_path(from: &str, to: &str) -> Result<(), SoplangError> {
    let from = Path::new(from);
    let to = Path::new(to);
    fs::rename(from, to).map_err(|error| path_io_error(from, &error, "Wareejinta faylka"))
}

pub fn delete_file(path: &str) -> Result<(), SoplangError> {
    let path = Path::new(path);
    fs::remove_file(path).map_err(|error| path_io_error(path, &error, "Tirtirka faylka"))
}

pub fn create_directory(path: &str) -> Result<(), SoplangError> {
    let path = Path::new(path);
    fs::create_dir_all(path).map_err(|error| path_io_error(path, &error, "Abuurista galka"))
}

pub fn list_directory(path: &str) -> Result<Vec<String>, SoplangError> {
    let path = Path::new(path);
    let mut names = fs::read_dir(path)
        .map_err(|error| path_io_error(path, &error, "Tirinta galka"))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    names.sort();
    Ok(names)
}

pub fn remove_directory(path: &str, recursive: bool) -> Result<(), SoplangError> {
    let path = Path::new(path);
    if recursive {
        fs::remove_dir_all(path).map_err(|error| path_io_error(path, &error, "Tirtirka galka"))
    } else {
        fs::remove_dir(path).map_err(|error| path_io_error(path, &error, "Tirtirka galka"))
    }
}

fn simple_object(fields: &[(&str, Value)]) -> Value {
    let mut map = HashMap::new();
    for (key, value) in fields {
        map.insert((*key).to_string(), value.clone());
    }
    Value::Object(Rc::new(std::cell::RefCell::new(map)))
}
