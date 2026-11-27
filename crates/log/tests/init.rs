//! Integration test for installing the logger.
//!
//! A logger can be installed once per process, so the whole story is one test.

use nimh_log::{Level, LevelFilter, debug, log_enabled};

#[test]
fn the_first_initialization_decides() {
    nimh_log::init_with_level(LevelFilter::Debug);

    assert_eq!(log::max_level(), LevelFilter::Debug);
    assert!(log_enabled!(Level::Debug));
    assert!(!log_enabled!(Level::Trace));
    debug!("written by the logger under test");

    nimh_log::init_with_level(LevelFilter::Error);
    nimh_log::init();

    assert_eq!(log::max_level(), LevelFilter::Debug);
}
