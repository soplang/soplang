//! Owned staging directories for package fetches and immutable cache publication.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::{io_reason, manifest_error};
use crate::error::SoplangError;

pub(super) struct TempDirectory {
    pub path: PathBuf,
}

impl TempDirectory {
    pub fn new(parent: &Path, label: &str) -> Result<Self, SoplangError> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        fs::create_dir_all(parent).map_err(|failure| {
            manifest_error(
                format!("Galka kaydka lama abuurin. {}", io_reason(&failure)),
                1,
            )
        })?;
        for _ in 0..128 {
            let path = parent.join(format!(
                ".{label}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(failure) if failure.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(failure) => {
                    return Err(manifest_error(
                        format!(
                            "Galka ku-meelgaarka ah lama abuurin. {}",
                            io_reason(&failure)
                        ),
                        1,
                    ))
                }
            }
        }
        Err(manifest_error("Magac gal ku-meelgaar ah lama helin.", 1))
    }
}
impl Drop for TempDirectory {
    fn drop(&mut self) {
        #[cfg(windows)]
        clear_readonly(&self.path);
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub(super) fn create_lock(dir: &Path, bytes: &[u8]) -> Result<bool, SoplangError> {
    use std::io::Write;
    let temp = TempDirectory::new(dir, "lock")?;
    let staged = temp.path.join("lock");
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&staged)
        .map_err(|failure| manifest_error(io_reason(&failure), 1))?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|failure| manifest_error(io_reason(&failure), 1))?;
    drop(file);
    match fs::hard_link(&staged, dir.join(super::LOCK_FILE)) {
        Ok(()) => Ok(true),
        Err(failure) if failure.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(failure) => Err(manifest_error(
            format!("soplang.lock lama daabici karo. {}", io_reason(&failure)),
            1,
        )),
    }
}

pub(super) struct PublicationLock {
    _file: fs::File,
}
impl PublicationLock {
    pub fn acquire(parent: &Path, key: &str) -> Result<Self, SoplangError> {
        let locks = parent.join(".locks");
        fs::create_dir_all(&locks).map_err(|failure| manifest_error(io_reason(&failure), 1))?;
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(locks.join(format!("{key}.lock")))
            .map_err(|failure| manifest_error(io_reason(&failure), 1))?;
        let start = std::time::Instant::now();
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(Self { _file: file }),
                Err(fs::TryLockError::WouldBlock)
                    if start.elapsed() < std::time::Duration::from_secs(60) =>
                {
                    std::thread::sleep(std::time::Duration::from_millis(5))
                }
                Err(fs::TryLockError::WouldBlock) => {
                    return Err(manifest_error(
                        "Sugitaanka kaydka xirmada wuu dheeraaday.",
                        1,
                    ))
                }
                Err(fs::TryLockError::Error(failure)) => {
                    return Err(manifest_error(io_reason(&failure), 1))
                }
            }
        }
    }
}

#[cfg(windows)]
fn clear_readonly(path: &Path) {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return;
    };
    if metadata.file_type().is_symlink() {
        return;
    }
    if metadata.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                clear_readonly(&entry.path());
            }
        }
    } else if metadata.permissions().readonly() {
        let mut permissions = metadata.permissions();
        permissions.set_readonly(false);
        let _ = fs::set_permissions(path, permissions);
    }
}
