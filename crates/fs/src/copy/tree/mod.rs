use std::collections::HashSet;
use std::path::{Path, PathBuf};

use fs_err as fs;

use super::entry;
use super::operations;
use super::relocation::Relocation;
use super::{
    CopyError, CopyOptions, CopyResult, DirectoryConflict, EntryKind, FileConflict,
    SymlinkBehavior, SymlinkConflict, TypeMismatch,
};
use crate::symlink;

pub(super) struct TreeCopier {
    options: CopyOptions,
    relocation: Relocation,
    active_directories: HashSet<PathBuf>,
}

impl TreeCopier {
    pub(super) fn new(options: CopyOptions, source: &Path, destination: &Path) -> CopyResult<Self> {
        let relocation = Relocation::new(source, destination).map_err(|source_error| {
            CopyError::RelocateSymlink {
                source_path: source.to_owned(),
                destination_path: destination.to_owned(),
                source: source_error,
            }
        })?;
        Ok(Self {
            options,
            relocation,
            active_directories: HashSet::new(),
        })
    }

    pub(super) fn copy_root(
        &mut self,
        source: &Path,
        destination: &Path,
        merge: bool,
    ) -> CopyResult<()> {
        if merge {
            return self.copy_directory(source, destination, false);
        }
        self.copy_entry(source, destination)
    }

    fn copy_entry(&mut self, source: &Path, destination: &Path) -> CopyResult<()> {
        let Some(source_kind) = source_kind(source, self.options.symlinks)? else {
            return Ok(());
        };
        let Some(destination_kind) = entry::inspect(destination)? else {
            return self.copy_new(source, source_kind, destination);
        };

        if source_kind == destination_kind {
            self.resolve_same_type(source, source_kind, destination)
        } else {
            self.resolve_type_mismatch(source, source_kind, destination, destination_kind)
        }
    }

    fn resolve_same_type(
        &mut self,
        source: &Path,
        kind: EntryKind,
        destination: &Path,
    ) -> CopyResult<()> {
        match kind {
            EntryKind::File => self.resolve_file(source, destination),
            EntryKind::Directory => self.resolve_directory(source, destination),
            EntryKind::Symlink => self.resolve_symlink(source, destination),
            EntryKind::Other => Err(CopyError::UnsupportedSource {
                path: source.to_owned(),
            }),
        }
    }

    fn resolve_file(&mut self, source: &Path, destination: &Path) -> CopyResult<()> {
        match self.options.conflicts.file {
            FileConflict::Error => Err(entry_exists(
                source,
                EntryKind::File,
                destination,
                EntryKind::File,
            )),
            FileConflict::Overwrite => operations::copy_file(source, destination),
            FileConflict::Replace => {
                entry::remove(destination, EntryKind::File)?;
                operations::copy_file(source, destination)
            }
        }
    }

    fn resolve_directory(&mut self, source: &Path, destination: &Path) -> CopyResult<()> {
        match self.options.conflicts.directory {
            DirectoryConflict::Error => Err(entry_exists(
                source,
                EntryKind::Directory,
                destination,
                EntryKind::Directory,
            )),
            DirectoryConflict::Replace => {
                entry::remove(destination, EntryKind::Directory)?;
                self.copy_directory(source, destination, true)
            }
            DirectoryConflict::Merge => self.copy_directory(source, destination, false),
        }
    }

    fn resolve_symlink(&mut self, source: &Path, destination: &Path) -> CopyResult<()> {
        match self.options.conflicts.symlink {
            SymlinkConflict::Error => Err(entry_exists(
                source,
                EntryKind::Symlink,
                destination,
                EntryKind::Symlink,
            )),
            SymlinkConflict::Replace => {
                entry::remove(destination, EntryKind::Symlink)?;
                self.copy_symlink(source, destination)
            }
        }
    }

    fn resolve_type_mismatch(
        &mut self,
        source: &Path,
        source_kind: EntryKind,
        destination: &Path,
        destination_kind: EntryKind,
    ) -> CopyResult<()> {
        match self.options.conflicts.type_mismatch {
            TypeMismatch::Error => Err(type_mismatch(
                source,
                source_kind,
                destination,
                destination_kind,
            )),
            TypeMismatch::Replace => {
                entry::remove(destination, destination_kind)?;
                self.copy_new(source, source_kind, destination)
            }
        }
    }

    fn copy_new(&mut self, source: &Path, kind: EntryKind, destination: &Path) -> CopyResult<()> {
        match kind {
            EntryKind::File => operations::copy_file(source, destination),
            EntryKind::Directory => self.copy_directory(source, destination, true),
            EntryKind::Symlink => self.copy_symlink(source, destination),
            EntryKind::Other => Err(CopyError::UnsupportedSource {
                path: source.to_owned(),
            }),
        }
    }

    fn copy_symlink(&self, source: &Path, destination: &Path) -> CopyResult<()> {
        let target =
            symlink::read_target(source).map_err(|source_error| CopyError::ReadSymlink {
                path: source.to_owned(),
                source: source_error,
            })?;
        let target = self.relocated_target(source, destination, target)?;
        symlink::create_from_source(&target, destination, source).map_err(|source_error| {
            CopyError::CreateSymlink {
                path: destination.to_owned(),
                source: source_error,
            }
        })
    }

    fn relocated_target(
        &self,
        source: &Path,
        destination: &Path,
        target: PathBuf,
    ) -> CopyResult<PathBuf> {
        if self.options.symlinks == SymlinkBehavior::PreservePath {
            return Ok(target);
        }
        self.relocation
            .target(source, destination, &target)
            .map_err(|source_error| CopyError::RelocateSymlink {
                source_path: source.to_owned(),
                destination_path: destination.to_owned(),
                source: source_error,
            })
    }

    fn copy_directory(
        &mut self,
        source: &Path,
        destination: &Path,
        create: bool,
    ) -> CopyResult<()> {
        let canonical_source = entry::canonicalize(source)?;
        if !self.active_directories.insert(canonical_source.clone()) {
            return Err(CopyError::SymlinkCycle {
                path: source.to_owned(),
            });
        }

        let result = self.copy_directory_contents(source, destination, create);
        self.active_directories.remove(&canonical_source);
        result
    }

    fn copy_directory_contents(
        &mut self,
        source: &Path,
        destination: &Path,
        create: bool,
    ) -> CopyResult<()> {
        if create {
            operations::create_directory(destination)?;
        }
        self.copy_children(source, destination)?;
        operations::copy_permissions(source, destination)
    }

    fn copy_children(&mut self, source: &Path, destination: &Path) -> CopyResult<()> {
        let entries = fs::read_dir(source).map_err(|source_error| CopyError::ReadDirectory {
            path: source.to_owned(),
            source: source_error,
        })?;
        for entry in entries {
            let entry = entry.map_err(|source_error| CopyError::ReadDirectory {
                path: source.to_owned(),
                source: source_error,
            })?;
            self.copy_entry(&entry.path(), &destination.join(entry.file_name()))?;
        }
        Ok(())
    }
}

pub(super) fn source_kind(
    source: &Path,
    symlinks: SymlinkBehavior,
) -> CopyResult<Option<EntryKind>> {
    let Some(kind) = entry::inspect(source)? else {
        return Err(missing_source(source));
    };
    if kind != EntryKind::Symlink {
        return Ok(Some(kind));
    }
    match symlinks {
        SymlinkBehavior::Skip => Ok(None),
        SymlinkBehavior::Follow => entry::followed_kind(source).map(Some),
        SymlinkBehavior::PreservePath | SymlinkBehavior::Relocate => Ok(Some(kind)),
    }
}

fn entry_exists(
    source_path: &Path,
    source_kind: EntryKind,
    destination_path: &Path,
    destination_kind: EntryKind,
) -> CopyError {
    CopyError::EntryExists {
        source_path: source_path.to_owned(),
        destination_path: destination_path.to_owned(),
        source_kind,
        destination_kind,
    }
}

fn type_mismatch(
    source_path: &Path,
    source_kind: EntryKind,
    destination_path: &Path,
    destination_kind: EntryKind,
) -> CopyError {
    CopyError::TypeMismatch {
        source_path: source_path.to_owned(),
        destination_path: destination_path.to_owned(),
        source_kind,
        destination_kind,
    }
}

fn missing_source(path: &Path) -> CopyError {
    CopyError::Inspect {
        path: path.to_owned(),
        source: std::io::Error::new(std::io::ErrorKind::NotFound, "source does not exist"),
    }
}
