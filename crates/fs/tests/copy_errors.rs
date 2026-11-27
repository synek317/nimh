//! Tests for structured copy errors and destructive-path guards.

mod support;

use std::error::Error;
use std::fs;
use std::io;

use nimh_fs::{
    CopyError, CopyOptions, DestinationDirectory, DestinationFile, DestinationPolicy, EntryKind,
};
use support::TestResult;
use tempfile::tempdir;

#[test]
fn destination_error_contains_path_and_kind() -> TestResult {
    let temp = tempdir()?;
    let source = temp.path().join("source");
    let destination = temp.path().join("destination");
    fs::write(&source, "new")?;
    fs::write(&destination, "old")?;

    let error = CopyOptions::new().copy(&source, &destination).unwrap_err();

    assert!(matches!(
        error,
        CopyError::DestinationExists {
            kind: EntryKind::File,
            ..
        }
    ));
    let message = error.to_string();
    assert!(message.contains(destination.to_string_lossy().as_ref()));
    assert!(message.contains("file"));
    Ok(())
}

#[test]
fn io_error_keeps_its_source() -> TestResult {
    let temp = tempdir()?;
    let missing = temp.path().join("missing");

    let error = CopyOptions::new()
        .copy(&missing, temp.path().join("destination"))
        .unwrap_err();

    let source = error.source().expect("I/O cause should be retained");
    let io_error = source
        .downcast_ref::<io::Error>()
        .expect("cause should be an io::Error");
    assert_eq!(io_error.kind(), io::ErrorKind::NotFound);
    Ok(())
}

#[test]
fn copying_path_onto_itself_is_rejected_before_mutation() -> TestResult {
    let temp = tempdir()?;
    let path = temp.path().join("file");
    fs::write(&path, "content")?;
    let options = CopyOptions::new().with_destination(DestinationPolicy {
        file: DestinationFile::Replace,
        ..DestinationPolicy::new()
    });

    let error = options.copy(&path, &path).unwrap_err();

    assert!(matches!(error, CopyError::SamePath { .. }));
    assert_eq!(fs::read_to_string(path)?, "content");
    Ok(())
}

#[test]
fn destination_inside_source_is_rejected_before_mutation() -> TestResult {
    let temp = tempdir()?;
    let source = temp.path().join("source");
    let destination = source.join("copy");
    fs::create_dir(&source)?;

    let error = CopyOptions::new().copy(&source, &destination).unwrap_err();

    assert!(
        matches!(error, CopyError::DestinationInsideSource { .. }),
        "unexpected error: {error:?}"
    );
    assert!(!destination.exists());
    Ok(())
}

#[test]
fn replacing_destination_containing_source_is_rejected() -> TestResult {
    let temp = tempdir()?;
    let destination = temp.path().join("destination");
    let source = destination.join("source");
    fs::create_dir(&destination)?;
    fs::create_dir(&source)?;
    fs::write(source.join("file"), "content")?;
    let options = CopyOptions::new().with_destination(DestinationPolicy {
        directory: DestinationDirectory::Replace,
        ..DestinationPolicy::new()
    });

    let error = options.copy(&source, &destination).unwrap_err();

    assert!(matches!(error, CopyError::SourceInsideDestination { .. }));
    assert_eq!(fs::read_to_string(source.join("file"))?, "content");
    Ok(())
}
