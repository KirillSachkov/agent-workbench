//! File helpers shared by the commands: hashing, writing with parents and removing with pruning.

use std::fs;
use std::io;
use std::path::Path;

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};

/// A file's content as the harness source holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blob {
    pub bytes: Vec<u8>,
    pub executable: bool,
}

pub fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Reads a file, returning `None` when it does not exist.
pub fn read_optional(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).with_context(|| format!("read {}", path.display())),
    }
}

pub fn write(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, bytes).with_context(|| format!("write {}", path.display()))
}

pub fn write_blob(path: &Path, blob: &Blob) -> Result<()> {
    write(path, &blob.bytes)?;
    set_executable(path, blob.executable)
}

#[cfg(unix)]
pub fn set_executable(path: &Path, executable: bool) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)?.permissions();
    let mode = permissions.mode();
    let wanted = if executable {
        mode | 0o111
    } else {
        mode & !0o111
    };
    if wanted != mode {
        permissions.set_mode(wanted);
        fs::set_permissions(path, permissions)
            .with_context(|| format!("chmod {}", path.display()))?;
    }
    Ok(())
}

#[cfg(not(unix))]
pub fn set_executable(_path: &Path, _executable: bool) -> Result<()> {
    Ok(())
}

/// Removes a file and every directory above it that became empty, up to `stop` (exclusive).
pub fn remove_and_prune(path: &Path, stop: &Path) -> Result<()> {
    fs::remove_file(path).with_context(|| format!("remove {}", path.display()))?;
    let mut dir = path.parent();
    while let Some(current) = dir {
        if current == stop || !current.starts_with(stop) {
            break;
        }
        if fs::remove_dir(current).is_err() {
            break;
        }
        dir = current.parent();
    }
    Ok(())
}
