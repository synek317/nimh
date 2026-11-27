use super::SymlinkBehavior;

/// Controls a recursive copy operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CopyOptions {
    /// Controls symbolic links found in the source tree.
    pub symlinks: SymlinkBehavior,
    /// Controls an already existing destination passed by the caller.
    pub destination: DestinationPolicy,
    /// Controls collisions below the destination root.
    pub conflicts: ConflictPolicy,
}

impl CopyOptions {
    /// Returns options which reject every existing destination entry.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            symlinks: SymlinkBehavior::PreservePath,
            destination: DestinationPolicy::new(),
            conflicts: ConflictPolicy::new(),
        }
    }

    /// Sets symbolic-link behavior.
    #[must_use]
    pub const fn with_symlinks(mut self, symlinks: SymlinkBehavior) -> Self {
        self.symlinks = symlinks;
        self
    }

    /// Sets the policy for the destination passed by the caller.
    #[must_use]
    pub const fn with_destination(mut self, destination: DestinationPolicy) -> Self {
        self.destination = destination;
        self
    }

    /// Sets policies for collisions below the destination root.
    #[must_use]
    pub const fn with_conflicts(mut self, conflicts: ConflictPolicy) -> Self {
        self.conflicts = conflicts;
        self
    }

    /// Returns options matching [`crate::cp_r`], with configurable symlinks.
    #[must_use]
    pub const fn cp_r(symlinks: SymlinkBehavior) -> Self {
        Self {
            symlinks,
            destination: DestinationPolicy::cp_r(),
            conflicts: ConflictPolicy::cp_r(),
        }
    }
}

impl Default for CopyOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Controls an already existing destination passed to the copy operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DestinationPolicy {
    /// Behavior when the destination is a regular file.
    pub file: DestinationFile,
    /// Behavior when the destination is a directory.
    pub directory: DestinationDirectory,
    /// Behavior when the destination is a symbolic link.
    pub symlink: DestinationSymlink,
    /// Behavior for other file-system entry types.
    pub other: DestinationOther,
}

impl DestinationPolicy {
    /// Returns a policy which rejects every existing destination.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            file: DestinationFile::Error,
            directory: DestinationDirectory::Error,
            symlink: DestinationSymlink::Error,
            other: DestinationOther::Error,
        }
    }

    const fn cp_r() -> Self {
        Self {
            file: DestinationFile::Replace,
            directory: DestinationDirectory::CopyInside,
            symlink: DestinationSymlink::Replace,
            other: DestinationOther::Error,
        }
    }
}

impl Default for DestinationPolicy {
    fn default() -> Self {
        Self::new()
    }
}

/// Controls collisions below the destination root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConflictPolicy {
    /// Behavior when both entries are regular files.
    pub file: FileConflict,
    /// Behavior when both entries are directories.
    pub directory: DirectoryConflict,
    /// Behavior when both entries are symbolic links.
    pub symlink: SymlinkConflict,
    /// Behavior when the source and destination have different types.
    pub type_mismatch: TypeMismatch,
}

impl ConflictPolicy {
    /// Returns a policy which rejects every collision.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            file: FileConflict::Error,
            directory: DirectoryConflict::Error,
            symlink: SymlinkConflict::Error,
            type_mismatch: TypeMismatch::Error,
        }
    }

    /// Replaces entries only when their types match.
    #[must_use]
    pub const fn replace_same_type() -> Self {
        Self {
            file: FileConflict::Replace,
            directory: DirectoryConflict::Replace,
            symlink: SymlinkConflict::Replace,
            type_mismatch: TypeMismatch::Error,
        }
    }

    const fn cp_r() -> Self {
        Self {
            file: FileConflict::Overwrite,
            directory: DirectoryConflict::Merge,
            symlink: SymlinkConflict::Replace,
            type_mismatch: TypeMismatch::Error,
        }
    }
}

impl Default for ConflictPolicy {
    fn default() -> Self {
        Self::new()
    }
}

/// Behavior for an existing destination file.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DestinationFile {
    /// Returns an error.
    #[default]
    Error,
    /// Removes the file and copies the source in its place.
    Replace,
}

/// Behavior for an existing destination directory.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DestinationDirectory {
    /// Returns an error.
    #[default]
    Error,
    /// Removes the directory and copies the source in its place.
    Replace,
    /// Copies the source below the directory, using the source file name.
    CopyInside,
    /// Merges a source directory directly into the destination directory.
    Merge,
}

/// Behavior for an existing destination symbolic link.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DestinationSymlink {
    /// Returns an error.
    #[default]
    Error,
    /// Removes the link and copies the source in its place.
    Replace,
}

/// Behavior for an existing destination of an unsupported type.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DestinationOther {
    /// Returns an error.
    #[default]
    Error,
    /// Removes the entry and copies the source in its place.
    Replace,
}

/// Behavior when both colliding entries are regular files.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FileConflict {
    /// Returns an error.
    #[default]
    Error,
    /// Writes into the existing file, preserving its identity.
    Overwrite,
    /// Removes the existing file before copying.
    Replace,
}

/// Behavior when both colliding entries are directories.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DirectoryConflict {
    /// Returns an error.
    #[default]
    Error,
    /// Removes the existing directory tree before copying.
    Replace,
    /// Copies source children into the existing directory.
    Merge,
}

/// Behavior when both colliding entries are symbolic links.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SymlinkConflict {
    /// Returns an error.
    #[default]
    Error,
    /// Removes the existing link before copying.
    Replace,
}

/// Behavior when colliding entries have different types.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TypeMismatch {
    /// Returns an error.
    #[default]
    Error,
    /// Removes the destination entry before copying.
    Replace,
}
