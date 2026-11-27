use std::io;
use std::path::{Path, PathBuf};

use path_clean::PathClean;

pub(super) struct Relocation {
    source_root: PathBuf,
    destination_root: PathBuf,
}

impl Relocation {
    pub(super) fn new(source: &Path, destination: &Path) -> io::Result<Self> {
        Ok(Self {
            source_root: absolute(source)?,
            destination_root: absolute(destination)?,
        })
    }

    pub(super) fn target(
        &self,
        source_link: &Path,
        destination_link: &Path,
        target: &Path,
    ) -> io::Result<PathBuf> {
        let target_path = resolve_target(source_link, target)?;
        let relocated_path = self.relocate(target_path);
        format_target(destination_link, target, relocated_path)
    }

    fn relocate(&self, target: PathBuf) -> PathBuf {
        let Ok(relative_target) = target.strip_prefix(&self.source_root) else {
            return target;
        };
        self.destination_root.join(relative_target)
    }
}

fn resolve_target(link: &Path, target: &Path) -> io::Result<PathBuf> {
    if target.is_absolute() {
        return Ok(target.clean());
    }
    Ok(absolute(parent(link))?.join(target).clean())
}

fn format_target(link: &Path, original: &Path, target: PathBuf) -> io::Result<PathBuf> {
    if original.is_absolute() {
        return Ok(target);
    }

    let link_parent = absolute(parent(link))?;
    let target = pathdiff::diff_paths(target, link_parent).ok_or_else(relocation_error)?;
    Ok(non_empty(target))
}

fn absolute(path: &Path) -> io::Result<PathBuf> {
    std::path::absolute(path).map(|path| path.clean())
}

fn parent(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn non_empty(path: PathBuf) -> PathBuf {
    if path.as_os_str().is_empty() {
        return PathBuf::from(".");
    }
    path
}

fn relocation_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        "cannot relocate symlink across these paths",
    )
}
