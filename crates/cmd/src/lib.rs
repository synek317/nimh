//! Running external commands for [NiMh](https://github.com/synek317/nimh), also available
//! through `nimh::cmd`.
//!
//! `NiMh`: [crates.io](https://crates.io/crates/nimh) / [docs.rs](https://docs.rs/nimh) / [github](https://github.com/synek317/nimh).
//!
//! `std::process::Command` is precise and verbose. Starting a program, checking how it ended
//! and decoding what it printed are three separate steps, and forgetting the second one is an
//! easy mistake to make. `Cmd` folds them into one call that gives either the result or an
//! error that tells the whole story:
//!
//! ```text
//! `git rev-parse HEAD` failed (exit status: 128):
//!   fatal: not a git repository (or any of the parent directories): .git
//! ```
//!
//! # Three ways to run a command
//!
//! | The command is run to | Call | What it prints |
//! | --- | --- | --- |
//! | do some work | `run` | is shown as it appears |
//! | produce a value | `read` | is captured and returned |
//! | answer a yes/no question | `succeeds` | is discarded |
//!
//! ```no_run
//! use nimh_cmd::Cmd;
//!
//! Cmd::new("cargo")
//!     .args(["build", "--release"])
//!     .dir("/srv/app")
//!     .run()?;
//!
//! let commit = Cmd::new("git").args(["rev-parse", "HEAD"]).read()?;
//!
//! if !Cmd::new("git").args(["diff", "--quiet"]).succeeds()? {
//!     println!("{commit} has uncommitted changes");
//! }
//! # Ok::<(), nimh_cmd::CmdError>(())
//! ```
//!
//! For a program with a list of arguments and nothing else, `nimh_cmd::run` and
//! `nimh_cmd::read` are shorter still:
//!
//! ```no_run
//! nimh_cmd::run("git", ["fetch", "--all"])?;
//! let branch = nimh_cmd::read("git", ["branch", "--show-current"])?;
//! # Ok::<(), nimh_cmd::CmdError>(())
//! ```
//!
//! # No shell
//!
//! Arguments reach the program exactly as they were given. No shell reads them on the way,
//! so spaces, quotes and `$` need no escaping and cannot smuggle in another command. Errors
//! and logs do show the command the way a shell would expect it, quotes included, so that it
//! can be copied and tried by hand.
//!
//! # Logs
//!
//! Every command is announced at the debug level of the `log` facade before it starts.
//! Values of environment variables given to a command never reach the logs, because they
//! are often secrets.

use std::ffi::OsString;

pub use cmd::Cmd;
pub use cmd_error::CmdError;
pub use command_line::CommandLine;

mod cmd;
mod cmd_error;
mod command_line;
mod output;
mod quote;
mod stderr_excerpt;

/// Runs `program` with `args`, showing what it prints, and waits until it ends.
///
/// A shorter spelling of [`Cmd::run`]. Use [`Cmd`] to run a program without arguments, in
/// another directory or with extra environment variables.
///
/// # Errors
///
/// Returns [`CmdError`] when the program cannot be started or ends unsuccessfully.
///
/// # Examples
///
/// ```no_run
/// nimh_cmd::run("git", ["fetch", "--all"])?;
/// # Ok::<(), nimh_cmd::CmdError>(())
/// ```
pub fn run<I>(program: impl Into<OsString>, args: I) -> Result<(), CmdError>
where
    I: IntoIterator,
    I::Item: Into<OsString>,
{
    Cmd::new(program).args(args).run()
}

/// Runs `program` with `args` and returns what it printed to stdout.
///
/// A shorter spelling of [`Cmd::read`]. Use [`Cmd`] to run a program without arguments, in
/// another directory or with extra environment variables.
///
/// # Errors
///
/// Returns [`CmdError`] when the program cannot be started, ends unsuccessfully or prints
/// something that is not valid UTF-8.
///
/// # Examples
///
/// ```
/// # #[cfg(unix)]
/// # {
/// let greeting = nimh_cmd::read("echo", ["hello", "world"])?;
///
/// assert_eq!(greeting, "hello world");
/// # }
/// # Ok::<(), nimh_cmd::CmdError>(())
/// ```
pub fn read<I>(program: impl Into<OsString>, args: I) -> Result<String, CmdError>
where
    I: IntoIterator,
    I::Item: Into<OsString>,
{
    Cmd::new(program).args(args).read()
}
