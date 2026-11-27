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
    /// Copies `source`, a file, a directory or a symbolic link, to `destination` according to
    /// these options.
    ///
    /// Before anything is changed, the copy is checked for the mistakes that would destroy
    /// data: copying a path onto itself, copying a directory into itself and replacing a
    /// destination that contains the source.
    ///
    /// # Errors
    ///
    /// Returns [`CopyError`] when the source does not exist, a policy rejects a collision or
    /// a file system operation fails. The entries copied before the failure stay in place.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nimh_fs::{CopyOptions, SymlinkBehavior};
    ///
    /// CopyOptions::cp_r(SymlinkBehavior::Relocate).copy("app", "/srv/app")?;
    /// # Ok::<(), nimh_fs::CopyError>(())
    /// ```
    pub fn copy(self, source: impl AsRef<Path>, destination: impl AsRef<Path>) -> CopyResult<()> {
        let source = source.as_ref();
        let Some(source_kind) = tree::source_kind(source, self.symlinks)? else {
            return Ok(());
        };
        let plan =
            DestinationPlan::build(source, source_kind, destination.as_ref(), self.destination)?;
        plan.validate(source, source_kind)?;
        let mut copier = TreeCopier::new(self, source, plan.path())?;
        plan.prepare()?;
        copier.copy_root(source, plan.path(), plan.merges_root())
    }
}

/// Copies `source` recursively, like `cp -R`, keeping symbolic links as they are.
///
/// It behaves like the command:
///
/// - an existing destination directory receives the source inside it, under the source's
///   name;
/// - an existing destination file or symbolic link is replaced;
/// - inside the tree, existing files are overwritten, directories are merged and symbolic
///   links are replaced;
/// - an entry that would replace one of another type, such as a file in place of a
///   directory, is an error.
///
/// Symbolic links are copied with exactly the same target path. Use [`cp_rl`] to copy what
/// they point to instead, and [`CopyOptions`] for any other combination.
///
/// # Errors
///
/// Returns [`CopyError`] when the source does not exist, two entries of different types
/// collide or a file system operation fails.
///
/// # Examples
///
/// ```no_run
/// nimh_fs::cp_r("target/release", "/srv/releases/v1.2.0")?;
/// # Ok::<(), nimh_fs::CopyError>(())
/// ```
pub fn cp_r(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> CopyResult<()> {
    CopyOptions::cp_r(SymlinkBehavior::PreservePath).copy(source, destination)
}

/// Copies `source` recursively, like `cp -RL`, copying what symbolic links point to.
///
/// Behaves like [`cp_r`] in everything else. A symbolic link that leads back into a
/// directory being copied is an error, instead of a copy that never ends.
///
/// # Errors
///
/// Returns [`CopyError`] when the source does not exist, a link is broken or forms a cycle,
/// two entries of different types collide or a file system operation fails.
///
/// # Examples
///
/// ```no_run
/// nimh_fs::cp_rl("config", "/etc/my_app")?;
/// # Ok::<(), nimh_fs::CopyError>(())
/// ```
pub fn cp_rl(source: impl AsRef<Path>, destination: impl AsRef<Path>) -> CopyResult<()> {
    CopyOptions::cp_r(SymlinkBehavior::Follow).copy(source, destination)
}
