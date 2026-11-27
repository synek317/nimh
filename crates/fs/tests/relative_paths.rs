//! Paths relative to the current directory.
//!
//! The current directory belongs to the whole process, so this file holds a single test
//! and runs in a process of its own.

#![cfg(unix)]

use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;

use tempfile::tempdir;

#[test]
fn bare_file_names_are_resolved_against_the_current_directory() -> Result<(), Box<dyn Error>> {
    let temp = tempdir()?;
    env::set_current_dir(temp.path())?;
    fs::create_dir("source")?;
    fs::write("source/file.txt", "content")?;

    nimh_fs::cp_r("source", "copy")?;
    nimh_fs::cp_rl("source", "followed")?;
    nimh_fs::ln_sfn("copy", "current")?;

    assert_eq!(fs::read_to_string("copy/file.txt")?, "content");
    assert_eq!(fs::read_to_string("followed/file.txt")?, "content");
    assert_eq!(fs::read_link("current")?, Path::new("copy"));
    assert_eq!(fs::read_to_string("current/file.txt")?, "content");
    Ok(())
}
