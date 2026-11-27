#![cfg(unix)]

//! Tests for policies applied to the destination passed by the caller.

mod support;

use std::fs;
use std::os::unix::fs::symlink;
use std::os::unix::net::UnixListener;

use nimh_fs::{
    CopyError, CopyOptions, DestinationDirectory, DestinationFile, DestinationOther,
    DestinationPolicy, DestinationSymlink, EntryKind,
};
use support::TestResult;
use tempfile::tempdir;
use test_case::test_case;

#[test_case(DestinationFile::Error, true; "error")]
#[test_case(DestinationFile::Replace, false; "replace")]
fn existing_destination_file(policy: DestinationFile, should_error: bool) -> TestResult {
    let temp = tempdir()?;
    let source = temp.path().join("source");
    let destination = temp.path().join("destination");
    fs::create_dir(&source)?;
    fs::write(source.join("new.txt"), "new")?;
    fs::write(&destination, "old")?;
    let options = CopyOptions::new().with_destination(DestinationPolicy {
        file: policy,
        ..DestinationPolicy::new()
    });

    let result = options.copy(&source, &destination);

    assert_eq!(result.is_err(), should_error);
    if should_error {
        assert_eq!(fs::read_to_string(&destination)?, "old");
    } else {
        assert_eq!(fs::read_to_string(destination.join("new.txt"))?, "new");
    }
    Ok(())
}

#[test_case(DestinationDirectory::Error, true, false, true; "error")]
#[test_case(DestinationDirectory::Replace, false, true, false; "replace")]
#[test_case(DestinationDirectory::CopyInside, false, false, true; "copy inside")]
#[test_case(DestinationDirectory::Merge, false, true, true; "merge")]
fn existing_destination_directory(
    policy: DestinationDirectory,
    should_error: bool,
    file_at_root: bool,
    keeps_old_file: bool,
) -> TestResult {
    let temp = tempdir()?;
    let source = temp.path().join("source");
    let destination = temp.path().join("destination");
    fs::create_dir(&source)?;
    fs::create_dir(&destination)?;
    fs::write(source.join("new.txt"), "new")?;
    fs::write(destination.join("old.txt"), "old")?;
    let options = CopyOptions::new().with_destination(DestinationPolicy {
        directory: policy,
        ..DestinationPolicy::new()
    });

    let result = options.copy(&source, &destination);

    assert_eq!(result.is_err(), should_error);
    if !should_error {
        let copied = if file_at_root {
            destination.join("new.txt")
        } else {
            destination.join("source/new.txt")
        };
        assert_eq!(fs::read_to_string(copied)?, "new");
    }
    assert_eq!(destination.join("old.txt").exists(), keeps_old_file);
    Ok(())
}

#[test_case(DestinationSymlink::Error, true; "error")]
#[test_case(DestinationSymlink::Replace, false; "replace")]
fn existing_destination_symlink(policy: DestinationSymlink, should_error: bool) -> TestResult {
    let temp = tempdir()?;
    let source = temp.path().join("source.txt");
    let destination = temp.path().join("destination");
    fs::write(&source, "new")?;
    symlink("missing", &destination)?;
    let options = CopyOptions::new().with_destination(DestinationPolicy {
        symlink: policy,
        ..DestinationPolicy::new()
    });

    let result = options.copy(&source, &destination);

    assert_eq!(result.is_err(), should_error);
    if should_error {
        assert_eq!(support::entry_kind(&destination)?, EntryKind::Symlink);
    } else {
        assert_eq!(fs::read_to_string(&destination)?, "new");
    }
    Ok(())
}

#[test]
fn merge_rejects_non_directory_source() -> TestResult {
    let temp = tempdir()?;
    let source = temp.path().join("source.txt");
    let destination = temp.path().join("destination");
    fs::write(&source, "new")?;
    fs::create_dir(&destination)?;
    let options = CopyOptions::new().with_destination(DestinationPolicy {
        directory: DestinationDirectory::Merge,
        ..DestinationPolicy::new()
    });

    let error = options.copy(&source, &destination).unwrap_err();

    assert!(matches!(
        error,
        CopyError::TypeMismatch {
            source_kind: EntryKind::File,
            destination_kind: EntryKind::Directory,
            ..
        }
    ));
    Ok(())
}

#[test_case(DestinationOther::Error, true; "error")]
#[test_case(DestinationOther::Replace, false; "replace")]
fn existing_special_destination(policy: DestinationOther, should_error: bool) -> TestResult {
    let temp = tempdir()?;
    let source = temp.path().join("source");
    let destination = temp.path().join("destination.sock");
    fs::write(&source, "new")?;
    let _listener = UnixListener::bind(&destination)?;
    let options = CopyOptions::new().with_destination(DestinationPolicy {
        other: policy,
        ..DestinationPolicy::new()
    });

    let result = options.copy(&source, &destination);

    assert_eq!(result.is_err(), should_error);
    if !should_error {
        assert_eq!(fs::read_to_string(destination)?, "new");
    }
    Ok(())
}
