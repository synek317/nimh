use std::io;
use std::path::{Path, PathBuf};

use fs_err as fs;

/// Points the symbolic link `link` at `target`, like `ln -sfn target link`, replacing an
/// existing file or symbolic link in one atomic step.
///
/// The new link is created next to `link` under a temporary name and then renamed over it.
/// Whoever follows `link` at any moment finds either the old target or the new one, never
/// nothing, which makes it the right tool to switch a `current` link between releases.
///
/// `target` is stored exactly as given: a relative target is resolved against the directory
/// of `link` by whoever follows it, not against the current directory. An existing real
/// directory at `link` is never removed.
///
/// # Errors
///
/// Returns an error when `link` is a real directory, or when the link cannot be created or
/// renamed. The message names the paths involved.
///
/// # Examples
///
/// ```no_run
/// nimh_fs::ln_sfn("releases/v1.2.0", "current")?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn ln_sfn(target: impl AsRef<Path>, link: impl AsRef<Path>) -> io::Result<()> {
    let target = target.as_ref();
    let link = link.as_ref();
    reject_directory(link)?;
    replace_link(target, link)
}

pub(crate) fn read_target(source: &Path) -> io::Result<PathBuf> {
    std::fs::read_link(source)
}

#[cfg(unix)]
pub(crate) fn create_from_source(target: &Path, link: &Path, _source: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
pub(crate) fn create_from_source(target: &Path, link: &Path, source: &Path) -> io::Result<()> {
    use std::os::windows::fs::FileTypeExt;

    let file_type = std::fs::symlink_metadata(source)?.file_type();
    if file_type.is_symlink_dir() {
        std::os::windows::fs::symlink_dir(target, link)
    } else {
        std::os::windows::fs::symlink_file(target, link)
    }
}

fn reject_directory(link: &Path) -> io::Result<()> {
    match fs::symlink_metadata(link) {
        Ok(metadata) if metadata.is_dir() => Err(io::Error::new(
            io::ErrorKind::IsADirectory,
            format!(
                "cannot replace directory '{}' with a symbolic link",
                link.display()
            ),
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

/// `rename` replaces its destination atomically on Unix, so the link is never missing.
#[cfg(unix)]
fn replace_link(target: &Path, link: &Path) -> io::Result<()> {
    let temporary = create_temporary_link(target, link)?;
    fs::rename(&temporary, link).inspect_err(|_| {
        if let Err(error) = std::fs::remove_file(&temporary) {
            log_leftover(&temporary, &error);
        }
    })
}

/// Windows cannot rename over a symbolic link, so the old one is removed first.
#[cfg(windows)]
fn replace_link(target: &Path, link: &Path) -> io::Result<()> {
    match remove(link) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error),
        _ => {}
    }
    let resolved_target = parent(link).join(target);
    if std::fs::metadata(resolved_target).is_ok_and(|metadata| metadata.is_dir()) {
        fs::os::windows::fs::symlink_dir(target, link)
    } else {
        fs::os::windows::fs::symlink_file(target, link)
    }
}

/// Removes a symbolic link itself, never what it points to.
///
/// On Windows a link to a directory is a directory entry and is removed like one.
pub(crate) fn remove(link: &Path) -> io::Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::FileTypeExt;

        if fs::symlink_metadata(link)?.file_type().is_symlink_dir() {
            return fs::remove_dir(link);
        }
    }
    fs::remove_file(link)
}

/// Creates a link to `target` beside `link`, under a name no other call is using.
#[cfg(unix)]
fn create_temporary_link(target: &Path, link: &Path) -> io::Result<PathBuf> {
    loop {
        let temporary = temporary_path(link);
        match fs::os::unix::fs::symlink(target, &temporary) {
            Ok(()) => return Ok(temporary),
            // Left behind by a process that was killed halfway; another name will do.
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
}

#[cfg(unix)]
fn temporary_path(link: &Path) -> PathBuf {
    use std::process;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let name = link.file_name().unwrap_or_default().to_string_lossy();
    let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
    parent(link).join(format!(".{name}.nimh-{}-{unique}", process::id()))
}

#[cfg(unix)]
fn log_leftover(temporary: &Path, error: &io::Error) {
    log::warn!(
        "cannot remove temporary symbolic link '{}': {error}",
        temporary.display()
    );
}

fn parent(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}
