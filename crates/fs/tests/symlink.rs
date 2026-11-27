#![cfg(unix)]

//! Integration tests for symbolic-link replacement.

use std::fs;
use std::io;
use std::path::Path;

use nimh_fs::ln_sfn;
use tempfile::tempdir;

#[test]
fn ln_sfn_creates_and_replaces_symlink() -> io::Result<()> {
    let temp = tempdir()?;
    let link = temp.path().join("current");

    ln_sfn("first", &link)?;
    assert_eq!(fs::read_link(&link)?, Path::new("first"));

    ln_sfn("second", &link)?;
    assert_eq!(fs::read_link(&link)?, Path::new("second"));
    Ok(())
}

#[test]
fn ln_sfn_replaces_broken_symlink() -> io::Result<()> {
    let temp = tempdir()?;
    let link = temp.path().join("current");
    std::os::unix::fs::symlink("missing", &link)?;

    ln_sfn("replacement", &link)?;

    assert_eq!(fs::read_link(&link)?, Path::new("replacement"));
    Ok(())
}

#[test]
fn ln_sfn_does_not_replace_real_directory() -> io::Result<()> {
    let temp = tempdir()?;
    let link = temp.path().join("directory");
    fs::create_dir(&link)?;

    let error = ln_sfn("target", &link).unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::IsADirectory);
    assert!(link.is_dir());
    Ok(())
}
