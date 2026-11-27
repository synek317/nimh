//! Integration tests for symbolic-link replacement.

#![cfg(unix)]

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

#[test]
fn ln_sfn_replaces_the_link_and_not_the_directory_it_points_to() -> io::Result<()> {
    let temp = tempdir()?;
    let old = temp.path().join("old");
    let link = temp.path().join("current");
    fs::create_dir(&old)?;
    std::os::unix::fs::symlink(&old, &link)?;

    ln_sfn("new", &link)?;

    assert_eq!(fs::read_link(&link)?, Path::new("new"));
    assert_eq!(fs::read_dir(&old)?.count(), 0);
    Ok(())
}

#[test]
fn ln_sfn_leaves_no_temporary_links_behind() -> io::Result<()> {
    let temp = tempdir()?;
    let link = temp.path().join("current");

    for target in ["first", "second", "third"] {
        ln_sfn(target, &link)?;
    }

    let names: Vec<_> = fs::read_dir(temp.path())?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<io::Result<_>>()?;
    assert_eq!(names, ["current"]);
    Ok(())
}

#[test]
fn ln_sfn_names_the_directory_it_refuses_to_replace() -> io::Result<()> {
    let temp = tempdir()?;
    let directory = temp.path().join("directory");
    fs::create_dir(&directory)?;

    let error = ln_sfn("target", &directory).unwrap_err();

    assert!(
        error.to_string().contains(&*directory.to_string_lossy()),
        "{error}"
    );
    Ok(())
}

#[test]
fn ln_sfn_reports_a_missing_parent_directory_with_its_path() -> io::Result<()> {
    let temp = tempdir()?;
    let link = temp.path().join("missing/current");

    let error = ln_sfn("target", &link).unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::NotFound);
    assert!(error.to_string().contains("missing"), "{error}");
    Ok(())
}
