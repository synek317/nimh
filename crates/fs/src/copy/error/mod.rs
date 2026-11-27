use std::io;
use std::path::PathBuf;

use thiserror::Error;

use super::EntryKind;

/// An error produced while recursively copying a file-system entry.
#[derive(Debug, Error)]
pub enum CopyError {
    /// A path could not be inspected.
    #[error("cannot inspect '{}'", path.display())]
    Inspect {
        /// The path being inspected.
        path: PathBuf,
        /// The underlying file-system error.
        #[source]
        source: io::Error,
    },

    /// An existing destination was rejected by its policy.
    #[error("destination '{}' already exists as {kind}", path.display())]
    DestinationExists {
        /// The destination path.
        path: PathBuf,
        /// The existing entry type.
        kind: EntryKind,
    },

    /// A nested collision was rejected by its policy.
    #[error(
        "cannot copy {source_kind} '{}' because '{}' already exists as {destination_kind}",
        source_path.display(),
        destination_path.display()
    )]
    EntryExists {
        /// The source path.
        source_path: PathBuf,
        /// The destination path.
        destination_path: PathBuf,
        /// The source entry type.
        source_kind: EntryKind,
        /// The destination entry type.
        destination_kind: EntryKind,
    },

    /// Source and destination types cannot be combined by the selected policy.
    #[error(
        "cannot copy {source_kind} '{}' onto {destination_kind} '{}'",
        source_path.display(),
        destination_path.display()
    )]
    TypeMismatch {
        /// The source path.
        source_path: PathBuf,
        /// The destination path.
        destination_path: PathBuf,
        /// The source entry type.
        source_kind: EntryKind,
        /// The destination entry type.
        destination_kind: EntryKind,
    },

    /// Source and destination resolve to the same path.
    #[error("source and destination are the same path: '{}'", path.display())]
    SamePath {
        /// The shared path.
        path: PathBuf,
    },

    /// The destination is contained by the source directory.
    #[error("destination '{}' is inside source directory '{}'", destination.display(), source_path.display())]
    DestinationInsideSource {
        /// The source directory.
        source_path: PathBuf,
        /// The destination path.
        destination: PathBuf,
    },

    /// Replacing the destination would also remove the source.
    #[error("source '{}' is inside destination '{}' which would be replaced", source_path.display(), destination.display())]
    SourceInsideDestination {
        /// The source path.
        source_path: PathBuf,
        /// The destination path.
        destination: PathBuf,
    },

    /// The source has no final path component for `CopyInside`.
    #[error("source '{}' has no file name", path.display())]
    SourceWithoutFileName {
        /// The source path.
        path: PathBuf,
    },

    /// A symbolic-link cycle was found while following links.
    #[error("symbolic-link cycle at '{}'", path.display())]
    SymlinkCycle {
        /// The path at which the cycle was detected.
        path: PathBuf,
    },

    /// The source entry type cannot be copied.
    #[error("unsupported source entry at '{}'", path.display())]
    UnsupportedSource {
        /// The unsupported path.
        path: PathBuf,
    },

    /// A directory could not be read.
    #[error("cannot read directory '{}'", path.display())]
    ReadDirectory {
        /// The directory path.
        path: PathBuf,
        /// The underlying file-system error.
        #[source]
        source: io::Error,
    },

    /// A symbolic link target could not be read.
    #[error("cannot read symbolic link '{}'", path.display())]
    ReadSymlink {
        /// The link path.
        path: PathBuf,
        /// The underlying file-system error.
        #[source]
        source: io::Error,
    },

    /// An entry could not be removed.
    #[error("cannot remove {kind} '{}'", path.display())]
    Remove {
        /// The entry path.
        path: PathBuf,
        /// The entry type.
        kind: EntryKind,
        /// The underlying file-system error.
        #[source]
        source: io::Error,
    },

    /// A directory could not be created.
    #[error("cannot create directory '{}'", path.display())]
    CreateDirectory {
        /// The directory path.
        path: PathBuf,
        /// The underlying file-system error.
        #[source]
        source: io::Error,
    },

    /// A regular file could not be copied.
    #[error("cannot copy file '{}' to '{}'", source_path.display(), destination_path.display())]
    CopyFile {
        /// The source file.
        source_path: PathBuf,
        /// The destination file.
        destination_path: PathBuf,
        /// The underlying file-system error.
        #[source]
        source: io::Error,
    },

    /// A symbolic link could not be created.
    #[error("cannot create symbolic link '{}'", path.display())]
    CreateSymlink {
        /// The new link path.
        path: PathBuf,
        /// The underlying file-system error.
        #[source]
        source: io::Error,
    },

    /// Directory permissions could not be copied.
    #[error("cannot copy permissions from '{}' to '{}'", source_path.display(), destination_path.display())]
    SetPermissions {
        /// The source directory.
        source_path: PathBuf,
        /// The destination directory.
        destination_path: PathBuf,
        /// The underlying file-system error.
        #[source]
        source: io::Error,
    },

    /// A symbolic link target cannot be relocated between the paths.
    #[error("cannot relocate symbolic link '{}' to '{}'", source_path.display(), destination_path.display())]
    RelocateSymlink {
        /// The source link.
        source_path: PathBuf,
        /// The destination link.
        destination_path: PathBuf,
        /// The underlying path error.
        #[source]
        source: io::Error,
    },
}

/// The result type used by recursive copying.
pub type CopyResult<T> = Result<T, CopyError>;
