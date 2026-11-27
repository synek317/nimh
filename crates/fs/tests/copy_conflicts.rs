//! Tests for collisions below the destination root.

#![cfg(unix)]

mod support;

use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use nimh_fs::{
    ConflictPolicy, CopyOptions, DestinationDirectory, DestinationPolicy, DirectoryConflict,
    EntryKind, FileConflict, SymlinkConflict, TypeMismatch,
};
use support::{TestResult, create_entry, entry_kind};
use tempfile::tempdir;
use test_case::test_case;

#[test_case(DirectoryConflict::Error, true, true; "error")]
#[test_case(DirectoryConflict::Replace, false, false; "replace")]
#[test_case(DirectoryConflict::Merge, false, true; "merge")]
fn existing_nested_directory(
    policy: DirectoryConflict,
    should_error: bool,
    keeps_old_file: bool,
) -> TestResult {
    let temp = tempdir()?;
    let (source, destination) = create_roots(temp.path())?;
    fs::create_dir(source.join("nested"))?;
    fs::create_dir(destination.join("nested"))?;
    fs::write(source.join("nested/new.txt"), "new")?;
    fs::write(destination.join("nested/old.txt"), "old")?;
    let options = merge_options(ConflictPolicy {
        directory: policy,
        ..ConflictPolicy::new()
    });

    let result = options.copy(&source, &destination);

    assert_eq!(result.is_err(), should_error);
    if !should_error {
        assert_eq!(
            fs::read_to_string(destination.join("nested/new.txt"))?,
            "new"
        );
    }
    assert_eq!(destination.join("nested/old.txt").exists(), keeps_old_file);
    Ok(())
}

#[test_case(FileConflict::Error, true, "old"; "error")]
#[test_case(FileConflict::Overwrite, false, "new"; "overwrite preserves identity")]
#[test_case(FileConflict::Replace, false, "old"; "replace creates a new file")]
fn existing_nested_file(
    policy: FileConflict,
    should_error: bool,
    hard_link_content: &str,
) -> TestResult {
    let temp = tempdir()?;
    let (source, destination) = create_roots(temp.path())?;
    fs::write(source.join("item"), "new")?;
    fs::write(destination.join("item"), "old")?;
    fs::hard_link(destination.join("item"), temp.path().join("alias"))?;
    let options = merge_options(ConflictPolicy {
        file: policy,
        ..ConflictPolicy::new()
    });

    let result = options.copy(&source, &destination);

    assert_eq!(result.is_err(), should_error);
    assert_eq!(
        fs::read_to_string(temp.path().join("alias"))?,
        hard_link_content
    );
    if !should_error {
        assert_eq!(fs::read_to_string(destination.join("item"))?, "new");
    }
    Ok(())
}

#[test_case(SymlinkConflict::Error, true, "old"; "error")]
#[test_case(SymlinkConflict::Replace, false, "new"; "replace")]
fn existing_nested_symlink(
    policy: SymlinkConflict,
    should_error: bool,
    expected_target: &str,
) -> TestResult {
    let temp = tempdir()?;
    let (source, destination) = create_roots(temp.path())?;
    symlink("new", source.join("link"))?;
    symlink("old", destination.join("link"))?;
    let options = merge_options(ConflictPolicy {
        symlink: policy,
        ..ConflictPolicy::new()
    });

    let result = options.copy(&source, &destination);

    assert_eq!(result.is_err(), should_error);
    assert_eq!(
        fs::read_link(destination.join("link"))?,
        Path::new(expected_target)
    );
    Ok(())
}

#[test_case(EntryKind::File, EntryKind::Directory; "file over directory")]
#[test_case(EntryKind::Directory, EntryKind::File; "directory over file")]
#[test_case(EntryKind::Symlink, EntryKind::File; "symlink over file")]
#[test_case(EntryKind::File, EntryKind::Symlink; "file over symlink")]
fn type_mismatch_can_error_or_replace(
    source_kind: EntryKind,
    destination_kind: EntryKind,
) -> TestResult {
    let temp = tempdir()?;
    let (source, destination) = create_roots(temp.path())?;
    create_entry(&source.join("item"), source_kind, "source")?;
    create_entry(&destination.join("item"), destination_kind, "destination")?;
    let error_options = merge_options(ConflictPolicy::new());

    assert!(
        error_options.copy(&source, &destination).is_err(),
        "mismatched types should be rejected"
    );

    let replace_options = merge_options(ConflictPolicy {
        type_mismatch: TypeMismatch::Replace,
        ..ConflictPolicy::new()
    });
    replace_options.copy(&source, &destination)?;
    assert_eq!(entry_kind(&destination.join("item"))?, source_kind);
    Ok(())
}

#[test]
fn replace_same_type_rejects_type_mismatch() -> TestResult {
    let temp = tempdir()?;
    let (source, destination) = create_roots(temp.path())?;
    fs::write(source.join("item"), "new")?;
    fs::create_dir(destination.join("item"))?;
    let options = merge_options(ConflictPolicy::replace_same_type());

    assert!(options.copy(source, destination).is_err());
    Ok(())
}

#[test_case(DirectoryConflict::Error, true; "nested destination error")]
#[test_case(DirectoryConflict::Merge, false; "nested destination merge")]
fn copy_inside_uses_nested_conflict_policy(
    directory: DirectoryConflict,
    should_error: bool,
) -> TestResult {
    let temp = tempdir()?;
    let source = temp.path().join("source");
    let destination = temp.path().join("destination");
    fs::create_dir(&source)?;
    fs::create_dir(&destination)?;
    fs::create_dir(destination.join("source"))?;
    fs::write(source.join("new"), "new")?;
    let options = CopyOptions::new()
        .with_destination(DestinationPolicy {
            directory: DestinationDirectory::CopyInside,
            ..DestinationPolicy::new()
        })
        .with_conflicts(ConflictPolicy {
            directory,
            ..ConflictPolicy::new()
        });

    let result = options.copy(&source, &destination);

    assert_eq!(result.is_err(), should_error);
    assert_eq!(destination.join("source/new").exists(), !should_error);
    Ok(())
}

fn create_roots(root: &Path) -> std::io::Result<(std::path::PathBuf, std::path::PathBuf)> {
    let source = root.join("source");
    let destination = root.join("destination");
    fs::create_dir(&source)?;
    fs::create_dir(&destination)?;
    Ok((source, destination))
}

fn merge_options(conflicts: ConflictPolicy) -> CopyOptions {
    CopyOptions::new()
        .with_destination(DestinationPolicy {
            directory: DestinationDirectory::Merge,
            ..DestinationPolicy::new()
        })
        .with_conflicts(conflicts)
}
