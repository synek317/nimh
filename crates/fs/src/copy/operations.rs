use std::path::Path;

use std::fs;

use super::{CopyError, CopyResult};

pub(super) fn create_directory(path: &Path) -> CopyResult<()> {
    fs::create_dir(path).map_err(|source| CopyError::CreateDirectory {
        path: path.to_owned(),
        source,
    })
}

pub(super) fn copy_file(source_path: &Path, destination_path: &Path) -> CopyResult<()> {
    fs::copy(source_path, destination_path)
        .map(|_| ())
        .map_err(|source| CopyError::CopyFile {
            source_path: source_path.to_owned(),
            destination_path: destination_path.to_owned(),
            source,
        })
}

pub(super) fn copy_permissions(source_path: &Path, destination_path: &Path) -> CopyResult<()> {
    let permissions = fs::metadata(source_path)
        .map_err(|source| CopyError::Inspect {
            path: source_path.to_owned(),
            source,
        })?
        .permissions();
    fs::set_permissions(destination_path, permissions).map_err(|source| CopyError::SetPermissions {
        source_path: source_path.to_owned(),
        destination_path: destination_path.to_owned(),
        source,
    })
}
