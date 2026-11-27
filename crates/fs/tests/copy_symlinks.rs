//! Tests for source symbolic-link behavior.

#![cfg(unix)]

mod support;

use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use nimh_fs::{CopyOptions, EntryKind, SymlinkBehavior, cp_r, cp_rl};
use support::{TestResult, entry_kind};
use tempfile::tempdir;
use test_case::test_case;

#[test_case(SymlinkBehavior::PreservePath, Some(EntryKind::Symlink); "preserve path")]
#[test_case(SymlinkBehavior::Relocate, Some(EntryKind::Symlink); "relocate")]
#[test_case(SymlinkBehavior::Follow, Some(EntryKind::File); "follow")]
#[test_case(SymlinkBehavior::Skip, None; "skip")]
fn source_symlink_behavior(
    behavior: SymlinkBehavior,
    expected_kind: Option<EntryKind>,
) -> TestResult {
    let temp = tempdir()?;
    let source = temp.path().join("source");
    let destination = temp.path().join("destination");
    fs::create_dir(&source)?;
    fs::write(source.join("target"), "content")?;
    symlink("target", source.join("link"))?;

    CopyOptions::cp_r(behavior).copy(&source, &destination)?;

    let copied_link = destination.join("link");
    assert_eq!(copied_link.exists(), expected_kind.is_some());
    if let Some(expected_kind) = expected_kind {
        assert_eq!(entry_kind(&copied_link)?, expected_kind);
    }
    Ok(())
}

#[test]
fn relocation_redirects_internal_links_to_copied_targets() -> TestResult {
    let temp = tempdir()?;
    let source = temp.path().join("source");
    let destination = temp.path().join("destination");
    fs::create_dir(&source)?;
    fs::write(source.join("bar"), "content")?;
    symlink("bar", source.join("relative"))?;
    symlink(source.join("bar"), source.join("absolute"))?;
    symlink(source.join("missing"), source.join("broken"))?;

    CopyOptions::cp_r(SymlinkBehavior::Relocate).copy(&source, &destination)?;

    assert_eq!(
        fs::read_link(destination.join("relative"))?,
        Path::new("bar")
    );
    assert_eq!(
        fs::read_link(destination.join("absolute"))?,
        destination.join("bar")
    );
    assert_eq!(
        fs::read_link(destination.join("broken"))?,
        destination.join("missing")
    );
    Ok(())
}

#[test]
fn relocation_keeps_external_target() -> TestResult {
    let temp = tempdir()?;
    let source = temp.path().join("source");
    let destination = temp.path().join("nested/destination");
    fs::create_dir(&source)?;
    fs::create_dir(temp.path().join("nested"))?;
    fs::write(temp.path().join("shared"), "content")?;
    symlink("../shared", source.join("link"))?;

    CopyOptions::cp_r(SymlinkBehavior::Relocate).copy(&source, &destination)?;

    assert_eq!(
        fs::read_link(destination.join("link"))?,
        Path::new("../../shared")
    );
    Ok(())
}

#[test]
fn cp_r_copies_inside_directory_and_preserves_links() -> TestResult {
    let temp = tempdir()?;
    let source = temp.path().join("source");
    let destination = temp.path().join("destination");
    fs::create_dir(&source)?;
    fs::create_dir(&destination)?;
    fs::write(source.join("target"), "content")?;
    symlink("target", source.join("link"))?;

    cp_r(&source, &destination)?;

    assert_eq!(
        fs::read_link(destination.join("source/link"))?,
        Path::new("target")
    );
    Ok(())
}

#[test]
fn cp_rl_follows_directory_links_and_detects_cycles() -> TestResult {
    let temp = tempdir()?;
    let source = temp.path().join("source");
    fs::create_dir(&source)?;
    fs::create_dir(source.join("real"))?;
    fs::write(source.join("real/file"), "content")?;
    symlink("real", source.join("link"))?;
    cp_rl(&source, temp.path().join("copy"))?;
    assert_eq!(
        fs::read_to_string(temp.path().join("copy/link/file"))?,
        "content"
    );

    symlink(".", source.join("cycle"))?;
    assert!(cp_rl(&source, temp.path().join("cycle-copy")).is_err());
    Ok(())
}
