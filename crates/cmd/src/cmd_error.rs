use std::error::Error;
use std::fmt;
use std::io;
use std::process::ExitStatus;

use crate::CommandLine;
use crate::stderr_excerpt::StderrExcerpt;

/// The error returned when a command does not do what it was run for.
///
/// The message always shows the command, written the way it could be typed into a shell,
/// and everything that is known about the failure:
///
/// ```text
/// `git rev-parse HEAD` failed (exit status: 128):
///   fatal: not a git repository (or any of the parent directories): .git
/// ```
///
/// # Examples
///
/// ```
/// use nimh_cmd::{Cmd, CmdError};
///
/// let error = Cmd::new("nimh-docs-no-such-program").run().unwrap_err();
///
/// assert!(matches!(error, CmdError::CannotRun { .. }));
/// assert_eq!(
///     error.to_string(),
///     "cannot run `nimh-docs-no-such-program`: program 'nimh-docs-no-such-program' was not found",
/// );
/// ```
#[derive(Debug)]
#[non_exhaustive]
pub enum CmdError {
    /// The command could not be started, or its outcome could not be collected.
    CannotRun {
        /// The command that was to be run.
        command: CommandLine,
        /// What went wrong. A missing program and a missing working directory are told
        /// apart here, which the operating system does not do.
        reason: io::Error,
    },

    /// The command ran and ended unsuccessfully: with a non-zero exit code or by a signal.
    Failed {
        /// The command that was run.
        command: CommandLine,
        /// How the command ended.
        status: ExitStatus,
        /// What the command printed to stderr, without surrounding whitespace. It is empty
        /// when stderr was not captured, as in [`Cmd::run`](crate::Cmd::run).
        stderr: String,
    },

    /// The command succeeded, but what it printed to stdout is not valid UTF-8.
    NotUtf8 {
        /// The command that was run.
        command: CommandLine,
    },
}

impl CmdError {
    /// Returns the command the error is about.
    #[must_use]
    pub fn command(&self) -> &CommandLine {
        match self {
            Self::CannotRun { command, .. }
            | Self::Failed { command, .. }
            | Self::NotUtf8 { command } => command,
        }
    }
}

impl fmt::Display for CmdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CannotRun { command, reason } => write!(f, "cannot run `{command}`: {reason}"),
            Self::Failed {
                command,
                status,
                stderr,
            } => {
                write!(f, "`{command}` failed ({status})")?;
                if stderr.is_empty() {
                    Ok(())
                } else {
                    write!(f, ":{}", StderrExcerpt(stderr))
                }
            }
            Self::NotUtf8 { command } => {
                write!(f, "`{command}` printed text that is not valid UTF-8")
            }
        }
    }
}

impl Error for CmdError {
    /// The reason a command could not be run is already part of the message, so it is not
    /// repeated as the source. What caused that reason, if anything, is.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CannotRun { reason, .. } => reason.source(),
            Self::Failed { .. } | Self::NotUtf8 { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{OsStr, OsString};

    fn command() -> CommandLine {
        let args = [OsString::from("rev-parse"), OsString::from("HEAD")];
        CommandLine::new(OsStr::new("git"), &args)
    }

    #[test]
    fn cannot_run_shows_the_command_and_the_reason() {
        let error = CmdError::CannotRun {
            command: command(),
            reason: io::Error::new(io::ErrorKind::NotFound, "program 'git' was not found"),
        };

        assert_eq!(
            error.to_string(),
            "cannot run `git rev-parse HEAD`: program 'git' was not found"
        );
        assert!(error.source().is_none());
    }

    #[test]
    fn not_utf_8_shows_the_command() {
        let error = CmdError::NotUtf8 { command: command() };

        assert_eq!(
            error.to_string(),
            "`git rev-parse HEAD` printed text that is not valid UTF-8"
        );
    }

    #[test]
    fn every_error_knows_its_command() {
        let error = CmdError::NotUtf8 { command: command() };

        assert_eq!(error.command().as_str(), "git rev-parse HEAD");
    }

    #[test]
    fn can_cross_threads_as_a_boxed_error() {
        let error: Box<dyn Error + Send + Sync> =
            Box::new(CmdError::NotUtf8 { command: command() });

        assert!(error.to_string().contains("git rev-parse HEAD"));
    }

    #[cfg(unix)]
    mod failed {
        use super::*;
        use std::os::unix::process::ExitStatusExt;

        /// The raw wait status of a process that exited with `code`.
        fn exited_with(code: i32) -> ExitStatus {
            ExitStatus::from_raw(code << 8)
        }

        /// The raw wait status of a process that was stopped by `signal`.
        fn stopped_by(signal: i32) -> ExitStatus {
            ExitStatus::from_raw(signal)
        }

        fn failed(status: ExitStatus, stderr: &str) -> CmdError {
            CmdError::Failed {
                command: command(),
                status,
                stderr: stderr.to_owned(),
            }
        }

        #[test]
        fn shows_the_exit_code() {
            let error = failed(exited_with(128), "");

            assert_eq!(
                error.to_string(),
                "`git rev-parse HEAD` failed (exit status: 128)"
            );
        }

        #[test]
        fn shows_the_signal() {
            let error = failed(stopped_by(9), "");

            assert_eq!(
                error.to_string(),
                "`git rev-parse HEAD` failed (signal: 9 (SIGKILL))"
            );
        }

        #[test]
        fn shows_stderr_below_the_message() {
            let error = failed(
                exited_with(128),
                "fatal: not a git repository\nhint: run git init",
            );

            assert_eq!(
                error.to_string(),
                "`git rev-parse HEAD` failed (exit status: 128):\n  fatal: not a git repository\n  hint: run git init"
            );
        }
    }
}
