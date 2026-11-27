use std::io;
use std::path::{Path, PathBuf};

use std::fs;

use crate::symlink;

use super::{CopyError, CopyResult, EntryKind};

pub(super) fn inspect(path: &Path) -> CopyResult<Option<EntryKind>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(EntryKind::from_metadata(&metadata))),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(CopyError::Inspect {
            path: path.to_owned(),
            source,
        }),
    }
}

pub(super) fn followed_kind(path: &Path) -> CopyResult<EntryKind> {
    fs::metadata(path)
        .map(|metadata| EntryKind::from_metadata(&metadata))
        .map_err(|source| CopyError::Inspect {
            path: path.to_owned(),
            source,
        })
}

pub(super) fn remove(path: &Path, kind: EntryKind) -> CopyResult<()> {
    let result = match kind {
        EntryKind::Directory => fs::remove_dir_all(path),
        EntryKind::Symlink => symlink::remove(path),
        EntryKind::File | EntryKind::Other => fs::remove_file(path),
    };
    result.map_err(|source| CopyError::Remove {
        path: path.to_owned(),
        kind,
        source,
    })
}

pub(super) fn canonicalize(path: &Path) -> CopyResult<PathBuf> {
    fs::canonicalize(path).map_err(|source| CopyError::Inspect {
        path: path.to_owned(),
        source,
    })
}
