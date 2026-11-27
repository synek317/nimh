/// Determines how recursive copying handles symbolic links.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SymlinkBehavior {
    /// Recreates each symbolic link with exactly the same target path, without following it.
    #[default]
    PreservePath,

    /// Recreates each symbolic link so that it still leads to the same place.
    ///
    /// A link into the copied tree is redirected to the copy. A link out of the tree keeps
    /// reaching its original target, which for a relative link means a rewritten path.
    Relocate,

    /// Copies the file or directory referenced by each symbolic link.
    Follow,

    /// Omits symbolic links from the copied tree.
    Skip,
}
