//! Paths, metadata, and filesystem management for the 3.3 stage.

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::UNIX_EPOCH;

use super::value::Value;
use crate::error::{external_error, path_io_error, type_error, SoplangError};

pub fn path_exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn path_info(path: &str) -> Result<Value, SoplangError> {
    let path = Path::new(path);
    if !path.exists() {
        return Ok(simple_object(&[("jir", Value::Bool(false))]));
    }
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| path_io_error(path, &error, "Akhriska xogta jidka"))?;
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
    if destination_present(to) && !overwrite {
        return Err(external_error(
            format!("Fayl ama gal ayaa hore ugu jira '{to}'.", to = to.display()),
            "destination already exists".to_string(),
        ));
    }
    if let Some(parent) = to.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|error| path_io_error(to, &error, "Nuqulka faylka"))?;
        }
    }
    let metadata = fs::symlink_metadata(from)
        .map_err(|error| path_io_error(from, &error, "Nuqulka faylka"))?;
    if metadata.file_type().is_symlink() {
        if destination_present(to) {
            fs::remove_file(to).map_err(|error| path_io_error(to, &error, "Nuqulka xidhiidhka"))?;
        }
        let target = fs::read_link(from)
            .map_err(|error| path_io_error(from, &error, "Nuqulka xidhiidhka"))?;
        return place_symlink(&target, to)
            .map_err(|error| path_io_error(to, &error, "Nuqulka xidhiidhka"));
    }
    fs::copy(from, to).map_err(|error| path_io_error(from, &error, "Nuqulka faylka"))?;
    Ok(())
}

pub fn move_path(from: &str, to: &str) -> Result<(), SoplangError> {
    let from = Path::new(from);
    let to = Path::new(to);
    match fs::rename(from, to) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::CrossesDevices => move_across(from, to),
        Err(error) => Err(path_io_error(from, &error, "Wareejinta faylka")),
    }
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
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| path_io_error(path, &error, "Tirtirka galka"))?;
    if metadata.file_type().is_symlink() {
        return fs::remove_file(path)
            .map_err(|error| path_io_error(path, &error, "Tirtirka xidhiidhka"));
    }
    if recursive {
        fs::remove_dir_all(path).map_err(|error| path_io_error(path, &error, "Tirtirka galka"))
    } else {
        fs::remove_dir(path).map_err(|error| path_io_error(path, &error, "Tirtirka galka"))
    }
}

pub(super) fn replace_bytes(path: &Path, bytes: &[u8]) -> Result<(), SoplangError> {
    let temp = temp_beside(path);
    let write_temp = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        Ok(())
    })();
    if let Err(error) = write_temp {
        let _ = fs::remove_file(&temp);
        return Err(path_io_error(path, &error, "Beddelka faylka"));
    }
    if let Err(error) = atomic_rename(&temp, path) {
        let _ = fs::remove_file(&temp);
        return Err(path_io_error(path, &error, "Beddelka faylka"));
    }
    Ok(())
}

fn destination_present(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

fn temp_beside(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("fayl");
    let file_name = format!(".{name}.sopliin");
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join(file_name),
        _ => PathBuf::from(file_name),
    }
}

fn move_across(from: &Path, to: &Path) -> Result<(), SoplangError> {
    let metadata = fs::symlink_metadata(from)
        .map_err(|error| path_io_error(from, &error, "Wareejinta faylka"))?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        return Err(external_error(
            format!(
                "Gal lama wareejin karo marka diskigu kala duwan yahay '{}'.",
                from.display()
            ),
            "cross-device directory move".to_string(),
        ));
    }
    if metadata.file_type().is_symlink() {
        let target = fs::read_link(from)
            .map_err(|error| path_io_error(from, &error, "Wareejinta xidhiidhka"))?;
        let temp = temp_beside(to);
        if let Err(error) = place_symlink(&target, &temp) {
            let _ = fs::remove_file(&temp);
            return Err(path_io_error(to, &error, "Wareejinta xidhiidhka"));
        }
        if let Err(error) = atomic_rename(&temp, to) {
            let _ = fs::remove_file(&temp);
            return Err(path_io_error(to, &error, "Wareejinta xidhiidhka"));
        }
        return fs::remove_file(from)
            .map_err(|error| path_io_error(from, &error, "Wareejinta xidhiidhka"));
    }
    let bytes = fs::read(from).map_err(|error| path_io_error(from, &error, "Wareejinta faylka"))?;
    replace_bytes(to, &bytes)?;
    fs::remove_file(from).map_err(|error| path_io_error(from, &error, "Wareejinta faylka"))
}

fn place_symlink(target: &Path, dest: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, dest)
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::{symlink_dir, symlink_file};
        let parent = dest
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        if parent.join(target).is_dir() {
            symlink_dir(target, dest)
        } else {
            symlink_file(target, dest)
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (target, dest);
        Err(io::Error::new(io::ErrorKind::Unsupported, "symlink"))
    }
}

fn atomic_rename(from: &Path, to: &Path) -> io::Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        #[link(name = "kernel32")]
        extern "system" {
            fn MoveFileExW(existing: *const u16, new: *const u16, flags: u32) -> i32;
        }
        fn wide(path: &Path) -> Vec<u16> {
            path.as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect()
        }
        let from = wide(from);
        let to = wide(to);
        if unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), 1 | 8) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
    #[cfg(not(windows))]
    {
        fs::rename(from, to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "soplang_fs_unit_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn copy_fallback_moves_a_file_and_refuses_a_directory() {
        let dir = temp_dir();
        let from = dir.join("from.txt");
        let to = dir.join("to.txt");
        fs::write(&from, "abc").unwrap();
        move_across(&from, &to).unwrap();
        assert!(!from.exists());
        assert_eq!(fs::read_to_string(&to).unwrap(), "abc");
        let folder = dir.join("folder");
        fs::create_dir(&folder).unwrap();
        let error = move_across(&folder, &dir.join("other")).unwrap_err();
        assert!(error.to_string().contains("diskigu kala duwan"));
        let _ = fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    #[test]
    fn copying_a_symlink_duplicates_the_link() {
        let dir = temp_dir();
        let target = dir.join("target.txt");
        let link = dir.join("link.txt");
        let copy = dir.join("copy.txt");
        fs::write(&target, "nux").unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();
        copy_path(link.to_str().unwrap(), copy.to_str().unwrap(), false).unwrap();
        assert!(fs::symlink_metadata(&copy)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(fs::read_link(&copy).unwrap(), target);
        let linked_dir = dir.join("linked-dir");
        std::os::unix::fs::symlink(&dir, &linked_dir).unwrap();
        remove_directory(linked_dir.to_str().unwrap(), true).unwrap();
        assert!(dir.exists());
        assert!(!linked_dir.exists());
        delete_file(link.to_str().unwrap()).unwrap();
        assert!(target.exists());
        let _ = fs::remove_dir_all(dir);
    }
}

fn simple_object(fields: &[(&str, Value)]) -> Value {
    let mut map = HashMap::new();
    for (key, value) in fields {
        map.insert((*key).to_string(), value.clone());
    }
    Value::Object(Rc::new(std::cell::RefCell::new(map)))
}
