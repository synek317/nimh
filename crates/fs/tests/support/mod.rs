#![allow(dead_code)]

use std::error::Error;
use std::fs;
use std::io;
use std::path::Path;

use nimh_fs::EntryKind;

pub type TestResult = Result<(), Box<dyn Error>>;

pub fn entry_kind(path: &Path) -> io::Result<EntryKind> {
    let file_type = fs::symlink_metadata(path)?.file_type();
    Ok(if file_type.is_symlink() {
        EntryKind::Symlink
    } else if file_type.is_dir() {
        EntryKind::Directory
    } else if file_type.is_file() {
        EntryKind::File
    } else {
        EntryKind::Other
    })
}

#[cfg(unix)]
pub fn create_entry(path: &Path, kind: EntryKind, content: &str) -> io::Result<()> {
    match kind {
        EntryKind::File => fs::write(path, content),
        EntryKind::Directory => {
            fs::create_dir(path)?;
            fs::write(path.join("content.txt"), content)
        }
        EntryKind::Symlink => std::os::unix::fs::symlink(content, path),
        EntryKind::Other => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "test helper does not create special entries",
        )),
    }
}
