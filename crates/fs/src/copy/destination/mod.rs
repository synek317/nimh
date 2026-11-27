use std::path::{Path, PathBuf};

use path_clean::PathClean;

use super::entry;
use super::{
    CopyError, CopyResult, DestinationDirectory, DestinationFile, DestinationOther,
    DestinationPolicy, DestinationSymlink, EntryKind,
};

pub(super) struct DestinationPlan {
    path: PathBuf,
    action: RootAction,
}

impl DestinationPlan {
    pub(super) fn build(
        source: &Path,
        source_kind: EntryKind,
        requested: &Path,
        policy: DestinationPolicy,
    ) -> CopyResult<Self> {
        let Some(destination_kind) = entry::inspect(requested)? else {
            return Ok(Self::copy_to(requested));
        };

        match destination_kind {
            EntryKind::File => plan_file(requested, policy.file),
            EntryKind::Directory => {
                plan_directory(source, source_kind, requested, policy.directory)
            }
            EntryKind::Symlink => plan_symlink(requested, policy.symlink),
            EntryKind::Other => plan_other(requested, policy.other),
        }
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    pub(super) fn merges_root(&self) -> bool {
        self.action == RootAction::Merge
    }

    pub(super) fn validate(&self, source: &Path, source_kind: EntryKind) -> CopyResult<()> {
        let source = comparable_path(source, source_kind)?;
        let destination = comparable_destination(&self.path)?;
        if source == destination {
            return Err(CopyError::SamePath { path: source });
        }
        if source_kind == EntryKind::Directory && destination.starts_with(&source) {
            return Err(CopyError::DestinationInsideSource {
                source_path: source,
                destination,
            });
        }
        if self.replaces() && source.starts_with(&destination) {
            return Err(CopyError::SourceInsideDestination {
                source_path: source,
                destination,
            });
        }
        Ok(())
    }

    pub(super) fn prepare(&self) -> CopyResult<()> {
        let RootAction::Replace(kind) = self.action else {
            return Ok(());
        };
        entry::remove(&self.path, kind)
    }

    fn copy_to(path: &Path) -> Self {
        Self {
            path: path.to_owned(),
            action: RootAction::Copy,
        }
    }

    fn replace(path: &Path, kind: EntryKind) -> Self {
        Self {
            path: path.to_owned(),
            action: RootAction::Replace(kind),
        }
    }

    fn replaces(&self) -> bool {
        matches!(self.action, RootAction::Replace(_))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RootAction {
    Copy,
    Replace(EntryKind),
    Merge,
}

fn plan_file(path: &Path, policy: DestinationFile) -> CopyResult<DestinationPlan> {
    match policy {
        DestinationFile::Error => destination_exists(path, EntryKind::File),
        DestinationFile::Replace => Ok(DestinationPlan::replace(path, EntryKind::File)),
    }
}

fn plan_directory(
    source: &Path,
    source_kind: EntryKind,
    path: &Path,
    policy: DestinationDirectory,
) -> CopyResult<DestinationPlan> {
    match policy {
        DestinationDirectory::Error => destination_exists(path, EntryKind::Directory),
        DestinationDirectory::Replace => Ok(DestinationPlan::replace(path, EntryKind::Directory)),
        DestinationDirectory::CopyInside => copy_inside(source, path),
        DestinationDirectory::Merge => merge(source, source_kind, path),
    }
}

fn plan_symlink(path: &Path, policy: DestinationSymlink) -> CopyResult<DestinationPlan> {
    match policy {
        DestinationSymlink::Error => destination_exists(path, EntryKind::Symlink),
        DestinationSymlink::Replace => Ok(DestinationPlan::replace(path, EntryKind::Symlink)),
    }
}

fn plan_other(path: &Path, policy: DestinationOther) -> CopyResult<DestinationPlan> {
    match policy {
        DestinationOther::Error => destination_exists(path, EntryKind::Other),
        DestinationOther::Replace => Ok(DestinationPlan::replace(path, EntryKind::Other)),
    }
}

fn copy_inside(source: &Path, destination: &Path) -> CopyResult<DestinationPlan> {
    let name = source
        .file_name()
        .ok_or_else(|| CopyError::SourceWithoutFileName {
            path: source.to_owned(),
        })?;
    Ok(DestinationPlan::copy_to(&destination.join(name)))
}

fn merge(source: &Path, source_kind: EntryKind, destination: &Path) -> CopyResult<DestinationPlan> {
    if source_kind != EntryKind::Directory {
        return Err(type_mismatch(source, source_kind, destination));
    }
    Ok(DestinationPlan {
        path: destination.to_owned(),
        action: RootAction::Merge,
    })
}

fn destination_exists(path: &Path, kind: EntryKind) -> CopyResult<DestinationPlan> {
    Err(CopyError::DestinationExists {
        path: path.to_owned(),
        kind,
    })
}

fn type_mismatch(source: &Path, source_kind: EntryKind, destination: &Path) -> CopyError {
    CopyError::TypeMismatch {
        source_path: source.to_owned(),
        destination_path: destination.to_owned(),
        source_kind,
        destination_kind: EntryKind::Directory,
    }
}

fn comparable_path(path: &Path, kind: EntryKind) -> CopyResult<PathBuf> {
    if kind == EntryKind::Symlink {
        return absolute(path);
    }
    std::fs::canonicalize(path).map_err(|source| CopyError::Inspect {
        path: path.to_owned(),
        source,
    })
}

fn comparable_destination(path: &Path) -> CopyResult<PathBuf> {
    match entry::inspect(path)? {
        Some(EntryKind::Symlink) | None => canonical_parent_path(path),
        Some(_) => std::fs::canonicalize(path).map_err(|source| CopyError::Inspect {
            path: path.to_owned(),
            source,
        }),
    }
}

fn canonical_parent_path(path: &Path) -> CopyResult<PathBuf> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let parent = std::fs::canonicalize(parent).map_err(|source| CopyError::Inspect {
        path: parent.to_owned(),
        source,
    })?;
    let Some(name) = path.file_name() else {
        return absolute(path);
    };
    Ok(parent.join(name))
}

fn absolute(path: &Path) -> CopyResult<PathBuf> {
    std::path::absolute(path)
        .map(|path| path.clean())
        .map_err(|source| CopyError::Inspect {
            path: path.to_owned(),
            source,
        })
}
