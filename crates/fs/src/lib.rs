//! Small file system helpers modelled after familiar Unix commands.
//!
//! [`cp_r`] and [`cp_rl`] provide convenient command-like behavior. Use
//! [`CopyOptions`] when every collision must be explicit:
//!
//! ```no_run
//! use nimh_fs::{
//!     ConflictPolicy, CopyOptions, DestinationDirectory, DestinationPolicy,
//!     DirectoryConflict, FileConflict,
//! };
//!
//! let options = CopyOptions::new()
//!     .with_destination(DestinationPolicy {
//!         directory: DestinationDirectory::Merge,
//!         ..DestinationPolicy::new()
//!     })
//!     .with_conflicts(ConflictPolicy {
//!         directory: DirectoryConflict::Merge,
//!         file: FileConflict::Overwrite,
//!         ..ConflictPolicy::new()
//!     });
//!
//! options.copy("source", "destination")?;
//! # Ok::<(), nimh_fs::CopyError>(())
//! ```

mod copy;
mod symlink;

pub use copy::{
    ConflictPolicy, CopyError, CopyOptions, CopyResult, DestinationDirectory, DestinationFile,
    DestinationOther, DestinationPolicy, DestinationSymlink, DirectoryConflict, EntryKind,
    FileConflict, SymlinkBehavior, SymlinkConflict, TypeMismatch, copy_recursively,
    copy_with_options, cp_r, cp_rl,
};
pub use symlink::ln_sfn;
