use std::fmt;
use std::fs::Metadata;

/// A file-system entry type relevant to copying.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntryKind {
    /// A regular file.
    File,
    /// A directory.
    Directory,
    /// A symbolic link.
    Symlink,
    /// A socket, pipe, device, or another unsupported entry.
    Other,
}

impl EntryKind {
    pub(super) fn from_metadata(metadata: &Metadata) -> Self {
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            Self::Symlink
        } else if file_type.is_dir() {
            Self::Directory
        } else if file_type.is_file() {
            Self::File
        } else {
            Self::Other
        }
    }
}

impl fmt::Display for EntryKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::File => "file",
            Self::Directory => "directory",
            Self::Symlink => "symbolic link",
            Self::Other => "unsupported entry",
        })
    }
}
