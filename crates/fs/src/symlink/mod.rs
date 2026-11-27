use std::io;
use std::path::{Path, PathBuf};

use fs_err as fs;

/// Creates a symbolic link, replacing an existing file or symbolic link.
///
/// This is equivalent to `ln -sfn target link` for a link path that is not a
/// real directory. An existing real directory is never removed.
///
/// # Errors
///
/// Returns an error when the existing path cannot be inspected or removed,
/// when the link cannot be created, or when `link` is a real directory.
pub fn ln_sfn(target: impl AsRef<Path>, link: impl AsRef<Path>) -> io::Result<()> {
    let target = target.as_ref();
    let link = link.as_ref();
    remove_existing_link(link)?;
    create_for_target(target, link)
}

pub(crate) fn read_target(source: &Path) -> io::Result<PathBuf> {
    fs::read_link(source)
}

#[cfg(unix)]
pub(crate) fn create_from_source(target: &Path, link: &Path, _source: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
pub(crate) fn create_from_source(target: &Path, link: &Path, source: &Path) -> io::Result<()> {
    use std::os::windows::fs::FileTypeExt;

    let file_type = fs::symlink_metadata(source)?.file_type();
    if file_type.is_symlink_dir() {
        std::os::windows::fs::symlink_dir(target, link)
    } else {
        std::os::windows::fs::symlink_file(target, link)
    }
}

fn remove_existing_link(link: &Path) -> io::Result<()> {
    match fs::symlink_metadata(link) {
        Ok(metadata) if metadata.is_dir() => Err(io::Error::new(
            io::ErrorKind::IsADirectory,
            format!("cannot replace directory '{}'", link.display()),
        )),
        Ok(_) => fs::remove_file(link),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

#[cfg(unix)]
fn create_for_target(target: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_for_target(target: &Path, link: &Path) -> io::Result<()> {
    let resolved_target = link.parent().unwrap_or(Path::new("")).join(target);
    if fs::metadata(resolved_target).is_ok_and(|metadata| metadata.is_dir()) {
        std::os::windows::fs::symlink_dir(target, link)
    } else {
        std::os::windows::fs::symlink_file(target, link)
    }
}
