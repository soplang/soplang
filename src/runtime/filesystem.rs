//! Paths, metadata, and filesystem management for the 3.3 stage.

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::UNIX_EPOCH;

use super::value::Value;
use crate::error::{external_error, path_io_error, type_error, SoplangError};

pub fn path_exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn path_info(path: &str) -> Result<Value, SoplangError> {
    let path = Path::new(path);
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(simple_object(&[("jir", Value::Bool(false))]));
        }
        Err(error) => return Err(path_io_error(path, &error, "Akhriska xogta jidka")),
    };
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

pub(crate) fn replace_bytes(path: &Path, bytes: &[u8]) -> Result<(), SoplangError> {
    replace_bytes_with_permissions(path, bytes, None)
}

pub(crate) fn replace_bytes_with_permissions(
    path: &Path,
    bytes: &[u8],
    permissions: Option<fs::Permissions>,
) -> Result<(), SoplangError> {
    let (temp, mut file) = create_temp_beside(path, |temp| {
        OpenOptions::new().write(true).create_new(true).open(temp)
    })
    .map_err(|error| path_io_error(path, &error, "Beddelka faylka"))?;
    let write_result = (|| {
        if let Some(permissions) = permissions {
            file.set_permissions(permissions)
        } else {
            Ok(())
        }
    })()
    .and_then(|_| file.write_all(bytes))
    .and_then(|_| file.sync_all());
    drop(file); // Windows also requires the temporary handle to be closed.
    if let Err(error) = write_result.and_then(|_| atomic_rename(&temp, path)) {
        let _ = fs::remove_file(&temp);
        return Err(path_io_error(path, &error, "Beddelka faylka"));
    }
    Ok(())
}

fn destination_present(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}

// Creation reserves the name atomically. Never truncate or remove a path
// belonging to an earlier operation or another process.
fn create_temp_beside<T>(
    path: &Path,
    mut create: impl FnMut(&Path) -> io::Result<T>,
) -> io::Result<(PathBuf, T)> {
    static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
    for _ in 0..128 {
        let mut name = std::ffi::OsString::from(".");
        name.push(
            path.file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("fayl")),
        );
        name.push(format!(
            ".{}.{}.sopliin",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        let temp = path.with_file_name(name);
        match create(&temp) {
            Ok(value) => return Ok((temp, value)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::from(io::ErrorKind::AlreadyExists))
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
        let (temp, ()) = create_temp_beside(to, |temp| place_symlink(&target, temp))
            .map_err(|error| path_io_error(to, &error, "Wareejinta xidhiidhka"))?;
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

pub fn contained_read(root: &str, request_path: &str) -> Result<Option<Vec<u8>>, SoplangError> {
    if root.is_empty() || request_path.contains('\0') {
        return Err(external_error(
            "Jidka faylka ma saxna.",
            "invalid static path".to_string(),
        ));
    }
    let root_path = Path::new(root);
    let root_canon = fs::canonicalize(root_path)
        .map_err(|error| path_io_error(root_path, &error, "Akhriska xididka"))?;
    if !root_canon.is_dir() {
        return Err(external_error(
            "Xididka faylka waa inuu noqdaa gal.",
            "static root".to_string(),
        ));
    }
    let relative = request_path.trim_start_matches(['/', '\\']);
    if relative.split(['/', '\\']).any(|part| part == "..") {
        return Err(external_error(
            "Jidka faylka wuxuu ka baxayaa xididka.",
            "path escape".to_string(),
        ));
    }
    let joined = root_canon.join(relative);
    let metadata = match fs::symlink_metadata(&joined) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(path_io_error(&joined, &error, "Akhriska faylka")),
    };
    if metadata.is_dir() {
        return Ok(None);
    }
    let canon = fs::canonicalize(&joined)
        .map_err(|error| path_io_error(&joined, &error, "Akhriska faylka"))?;
    if !canon.starts_with(&root_canon) || !canon.is_file() {
        return Err(external_error(
            "Jidka faylka wuxuu ka baxayaa xididka.",
            "path escape".to_string(),
        ));
    }
    fs::read(&canon)
        .map(Some)
        .map_err(|error| path_io_error(&canon, &error, "Akhriska faylka"))
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
    fn replacement_preserves_neighbors_and_cleans_failed_writes() {
        let dir = temp_dir();
        let dest = dir.join("note.txt");
        let neighbor = dir.join(".note.txt.sopliin");
        fs::write(&dest, "old").unwrap();
        fs::write(&neighbor, "unrelated").unwrap();
        replace_bytes(&dest, b"new").unwrap();
        assert_eq!(fs::read(&dest).unwrap(), b"new");
        assert_eq!(fs::read(&neighbor).unwrap(), b"unrelated");
        let folder = dir.join("folder");
        fs::create_dir(&folder).unwrap();
        assert!(replace_bytes(&folder, b"must fail").is_err());
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 3);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn replacement_reserves_independent_temporaries_for_concurrent_writers() {
        let dir = temp_dir();
        let dest = dir.join("shared.txt");
        let mut threads = Vec::new();
        for byte in 1..=8 {
            let dest = dest.clone();
            threads.push(std::thread::spawn(move || {
                for _ in 0..8 {
                    replace_bytes(&dest, &vec![byte; 8192]).unwrap();
                }
            }));
        }
        for thread in threads {
            thread.join().unwrap();
        }
        let bytes = fs::read(dest).unwrap();
        assert_eq!(bytes.len(), 8192);
        assert!(bytes.iter().all(|byte| *byte == bytes[0]));
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn replacement_never_follows_a_neighbor_or_destination_symlink() {
        let dir = temp_dir();
        let target = dir.join("target.txt");
        let dest = dir.join("note.txt");
        let neighbor = dir.join(".note.txt.sopliin");
        fs::write(&target, "protected").unwrap();
        std::os::unix::fs::symlink(&target, &neighbor).unwrap();
        std::os::unix::fs::symlink(&target, &dest).unwrap();
        replace_bytes(&dest, b"new").unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"protected");
        assert_eq!(fs::read(&dest).unwrap(), b"new");
        assert!(fs::symlink_metadata(neighbor)
            .unwrap()
            .file_type()
            .is_symlink());
        let dangling = dir.join("dangling");
        std::os::unix::fs::symlink(dir.join("missing"), &dangling).unwrap();
        let Value::Object(info) = path_info(dangling.to_str().unwrap()).unwrap() else {
            panic!("metadata")
        };
        assert_eq!(info.borrow()["jir"], Value::Bool(true));
        assert_eq!(info.borrow()["nooc"], Value::Str("xidhiidh".into()));
        fs::remove_dir_all(dir).unwrap();
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
