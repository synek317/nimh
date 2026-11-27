//! Integration tests that run real commands.

#![cfg(unix)]

use std::fs;

use nimh_cmd::{Cmd, CmdError};
use tempfile::tempdir;

/// A command that runs `script` in a shell, for behavior no single program provides.
fn sh(script: &str) -> Cmd {
    Cmd::new("sh").args(["-c", script])
}

#[test]
fn run_succeeds_when_the_command_does() -> Result<(), CmdError> {
    Cmd::new("true").run()
}

#[test]
fn run_reports_a_non_zero_exit_code() {
    let error = Cmd::new("false").run().unwrap_err();

    assert!(matches!(error, CmdError::Failed { .. }), "{error}");
    assert_eq!(error.to_string(), "`false` failed (exit status: 1)");
}

#[test]
fn run_reports_a_signal() {
    let error = sh("kill -9 $$").run().unwrap_err();

    assert_eq!(
        error.to_string(),
        "`sh -c 'kill -9 $$'` failed (signal: 9 (SIGKILL))"
    );
}

#[test]
fn run_reports_a_missing_program_by_name() {
    let error = Cmd::new("nimh-tests-no-such-program")
        .arg("--version")
        .run()
        .unwrap_err();

    assert!(matches!(error, CmdError::CannotRun { .. }), "{error}");
    assert_eq!(
        error.to_string(),
        "cannot run `nimh-tests-no-such-program --version`: \
         program 'nimh-tests-no-such-program' was not found"
    );
}

#[test]
fn run_reports_a_missing_working_directory() -> std::io::Result<()> {
    let temp = tempdir()?;
    let missing = temp.path().join("missing");

    let error = Cmd::new("true").dir(&missing).run().unwrap_err();

    assert_eq!(
        error.to_string(),
        format!(
            "cannot run `true`: working directory '{}' does not exist",
            missing.display()
        )
    );
    Ok(())
}

#[test]
fn read_returns_stdout_without_the_final_line_break() -> Result<(), CmdError> {
    let output = sh("echo '  first'; echo second; echo").read()?;

    assert_eq!(output, "  first\nsecond");
    Ok(())
}

#[test]
fn read_passes_arguments_to_the_program_untouched() -> Result<(), CmdError> {
    let output = Cmd::new("printf")
        .args(["%s|%s|%s", "two words", "$HOME", "it's; rm -rf *"])
        .read()?;

    assert_eq!(output, "two words|$HOME|it's; rm -rf *");
    Ok(())
}

#[test]
fn read_puts_stderr_into_the_error() {
    let error = sh("echo partial; echo 'fatal: boom' >&2; exit 3")
        .read()
        .unwrap_err();

    let CmdError::Failed { status, stderr, .. } = &error else {
        panic!("unexpected error: {error}");
    };
    assert_eq!(status.code(), Some(3));
    assert_eq!(stderr, "fatal: boom");
    assert_eq!(
        error.to_string(),
        "`sh -c 'echo partial; echo '\\''fatal: boom'\\'' >&2; exit 3'` failed (exit status: 3):\n  fatal: boom"
    );
}

#[test]
fn read_ignores_stderr_of_a_successful_command() -> Result<(), CmdError> {
    let output = sh("echo 'warning: careful' >&2; echo done").read()?;

    assert_eq!(output, "done");
    Ok(())
}

#[test]
fn read_rejects_output_that_is_not_utf_8() {
    let error = sh(r"printf '\377'").read().unwrap_err();

    assert!(matches!(error, CmdError::NotUtf8 { .. }), "{error}");
}

#[test]
fn read_does_not_let_the_command_wait_for_input() -> Result<(), CmdError> {
    let output = Cmd::new("cat").read()?;

    assert_eq!(output, "");
    Ok(())
}

#[test]
fn succeeds_answers_with_the_exit_code() -> Result<(), CmdError> {
    assert!(Cmd::new("true").succeeds()?);
    assert!(!Cmd::new("false").succeeds()?);
    assert!(!sh("echo noise; echo noise >&2; exit 7").succeeds()?);
    Ok(())
}

#[test]
fn succeeds_reports_a_command_that_gave_no_answer() {
    let error = sh("kill -9 $$").succeeds().unwrap_err();

    assert!(matches!(error, CmdError::Failed { .. }), "{error}");
}

#[test]
fn succeeds_reports_a_missing_program() {
    let error = Cmd::new("nimh-tests-no-such-program")
        .succeeds()
        .unwrap_err();

    assert!(matches!(error, CmdError::CannotRun { .. }), "{error}");
}

#[test]
fn dir_sets_the_working_directory() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempdir()?;
    fs::write(temp.path().join("marker.txt"), "")?;

    let listing = Cmd::new("ls").dir(temp.path()).read()?;

    assert_eq!(listing, "marker.txt");
    Ok(())
}

#[test]
fn env_adds_a_variable_to_the_inherited_ones() -> Result<(), CmdError> {
    let output = sh(r#"printf '%s %s' "$GREETING" "${PATH:+inherited}""#)
        .env("GREETING", "hello")
        .read()?;

    assert_eq!(output, "hello inherited");
    Ok(())
}

#[test]
fn a_command_can_be_run_many_times() -> Result<(), CmdError> {
    let echo = Cmd::new("echo").arg("again");

    assert_eq!(echo.read()?, "again");
    assert_eq!(echo.clone().arg("and again").read()?, "again and again");
    assert_eq!(echo.read()?, "again");
    Ok(())
}

#[test]
fn free_functions_run_a_program_with_arguments() -> Result<(), CmdError> {
    nimh_cmd::run("test", ["-d", "/"])?;

    assert_eq!(nimh_cmd::read("echo", ["hello", "world"])?, "hello world");
    Ok(())
}
