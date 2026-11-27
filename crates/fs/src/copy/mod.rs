mod destination;
mod entry;
mod entry_kind;
mod error;
mod operations;
mod options;
mod relocation;
mod symlink_behavior;
mod tree;

use std::path::Path;

pub use entry_kind::EntryKind;
pub use error::{CopyError, CopyResult};
pub use options::{
    ConflictPolicy, CopyOptions, DestinationDirectory, DestinationFile, DestinationOther,
    DestinationPolicy, DestinationSymlink, DirectoryConflict, FileConflict, SymlinkConflict,
    TypeMismatch,
};
pub use symlink_behavior::SymlinkBehavior;

use destination::DestinationPlan;
use tree::TreeCopier;

impl CopyOptions {
    /// Copies `source` recursively according to these options.
    ///
    /// # Errors
    ///
    /// Returns [`CopyError`] when a selected policy rejects a collision or a
    /// file-system operation fails.
    pub fn copy(self, source: impl AsRef<Path>, destination: impl AsRef<Path>) -> CopyResult<()> {
        copy_with_options(source.as_ref(), destination.as_ref(), self)
    }
}

/// Recursively copies `source` with explicit options.
///
/// # Errors
///
/// Returns [`CopyError`] when a selected policy rejects a collision or a
/// file-system operation fails.
pub fn copy_with_options(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
    options: CopyOptions,
) -> CopyResult<()> {
    let source = source.as_ref();
    let Some(source_kind) = tree::source_kind(source, options.symlinks)? else {
        return Ok(());
    };
    let plan = DestinationPlan::build(
        source,
        source_kind,
        destination.as_ref(),
        options.destination,
    )?;
    plan.validate(source, source_kind)?;
    let mut copier = TreeCopier::new(options, source, plan.path())?;
    plan.prepare()?;
    copier.copy_root(source, plan.path(), plan.merges_root())
}

/// Recursively copies `source` using `cp_r` collision behavior.
///
/// `symlink_behavior` controls symbolic links. If the destination is an
/// existing directory, the source is copied inside it.
///
/// # Errors
///
/// Returns [`CopyError`] when copying fails.
pub fn copy_recursively(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
    symlink_behavior: SymlinkBehavior,
) -> CopyResult<()> {
    CopyOptions::cp_r(symlink_behavior).copy(source, destination)
}

/// Recursively copies `source`, preserving symlink paths.
///
/// # Errors
///
/// Returns [`CopyError`] when copying fails.
pub fn cp_r(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> CopyResult<()> {
    copy_recursively(source, destination, SymlinkBehavior::PreservePath)
}

/// Recursively copies `source`, following symbolic links.
///
/// # Errors
///
/// Returns [`CopyError`] when copying fails.
pub fn cp_rl(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> CopyResult<()> {
    copy_recursively(source, destination, SymlinkBehavior::Follow)
}
