use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};

use crate::{CmdError, CommandLine, output};

/// An external command: a program, its arguments and the conditions it runs in.
///
/// Building a `Cmd` starts nothing. One of three methods runs it, depending on what the
/// command is for:
///
/// - [`run`](Self::run) for a command that does some work;
/// - [`read`](Self::read) for a command that produces a value;
/// - [`succeeds`](Self::succeeds) for a command that answers a yes/no question.
///
/// A `Cmd` is a plain value. It can be stored, cloned and run any number of times.
///
/// # Examples
///
/// ```
/// use nimh_cmd::Cmd;
///
/// let commit = Cmd::new("git")
///     .args(["commit", "--message", "Fix the build"])
///     .dir("/srv/app");
///
/// assert_eq!(
///     commit.command_line().as_str(),
///     "git commit --message 'Fix the build'",
/// );
/// ```
#[derive(Debug, Clone)]
#[must_use = "a command does nothing until `run`, `read` or `succeeds` is called"]
pub struct Cmd {
    program: OsString,
    args: Vec<OsString>,
    dir: Option<PathBuf>,
    envs: Vec<(OsString, OsString)>,
}

impl Cmd {
    /// Creates a command that runs `program`, either a name looked up in `PATH` or a path.
    pub fn new(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            dir: None,
            envs: Vec::new(),
        }
    }

    /// Adds one argument.
    ///
    /// The program receives it exactly as given. No shell is involved, so nothing needs to
    /// be quoted or escaped.
    pub fn arg(mut self, arg: impl Into<OsString>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Adds several arguments.
    pub fn args<I>(mut self, args: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<OsString>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    /// Sets the directory to run the command in, instead of the current one.
    pub fn dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.dir = Some(dir.into());
        self
    }

    /// Sets an environment variable for the command, on top of the inherited ones.
    pub fn env(mut self, name: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.envs.push((name.into(), value.into()));
        self
    }

    /// Returns the command as text, the way it could be typed into a shell.
    #[must_use]
    pub fn command_line(&self) -> CommandLine {
        CommandLine::new(&self.program, &self.args)
    }

    /// Runs the command and waits until it ends.
    ///
    /// The command shares the terminal with the application: what it prints is shown as it
    /// appears, and it can read from stdin.
    ///
    /// # Errors
    ///
    /// Returns [`CmdError`] when the command cannot be started or ends unsuccessfully.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nimh_cmd::Cmd;
    ///
    /// Cmd::new("cargo").args(["build", "--release"]).run()?;
    /// # Ok::<(), nimh_cmd::CmdError>(())
    /// ```
    pub fn run(&self) -> Result<(), CmdError> {
        let status = self.start(Command::status)?;
        if status.success() {
            Ok(())
        } else {
            Err(self.failed(status, String::new()))
        }
    }

    /// Runs the command and returns what it printed to stdout, without the line break at
    /// the end.
    ///
    /// What the command prints to stderr is captured as well. If the command fails, it
    /// becomes part of the error. If the command succeeds, it is logged at the debug level.
    ///
    /// # Errors
    ///
    /// Returns [`CmdError`] when the command cannot be started, ends unsuccessfully or
    /// prints something that is not valid UTF-8.
    ///
    /// # Examples
    ///
    /// ```
    /// use nimh_cmd::Cmd;
    ///
    /// # #[cfg(unix)]
    /// # {
    /// let greeting = Cmd::new("echo").arg("hello").read()?;
    ///
    /// assert_eq!(greeting, "hello");
    /// # }
    /// # Ok::<(), nimh_cmd::CmdError>(())
    /// ```
    pub fn read(&self) -> Result<String, CmdError> {
        let output = self.start(Command::output)?;
        let stderr = output::diagnostics(&output.stderr);
        if !output.status.success() {
            return Err(self.failed(output.status, stderr));
        }

        self.log_diagnostics(&stderr);
        output::text(output.stdout).ok_or_else(|| CmdError::NotUtf8 {
            command: self.command_line(),
        })
    }

    /// Runs the command for its exit code alone: `true` for zero, `false` for anything else.
    ///
    /// This is for commands that answer a question through their exit code, such as
    /// `git diff --quiet`. What the command prints is discarded.
    ///
    /// # Errors
    ///
    /// Returns [`CmdError`] when the command cannot be started, or when it was stopped by a
    /// signal and therefore gave no answer.
    ///
    /// # Examples
    ///
    /// ```
    /// use nimh_cmd::Cmd;
    ///
    /// # #[cfg(unix)]
    /// # {
    /// assert!(Cmd::new("test").args(["-d", "/"]).succeeds()?);
    /// assert!(!Cmd::new("test").args(["-d", "/no/such/directory"]).succeeds()?);
    /// # }
    /// # Ok::<(), nimh_cmd::CmdError>(())
    /// ```
    pub fn succeeds(&self) -> Result<bool, CmdError> {
        let status =
            self.start(|command| command.stdout(Stdio::null()).stderr(Stdio::null()).status())?;
        match status.code() {
            Some(code) => Ok(code == 0),
            None => Err(self.failed(status, String::new())),
        }
    }

    /// Announces the command, launches it the way the caller needs and waits for the
    /// outcome.
    fn start<T>(&self, launch: impl FnOnce(&mut Command) -> io::Result<T>) -> Result<T, CmdError> {
        self.announce();
        launch(&mut self.to_command()).map_err(|error| CmdError::CannotRun {
            command: self.command_line(),
            reason: self.explain(error),
        })
    }

    fn announce(&self) {
        match &self.dir {
            Some(dir) => log::debug!("running `{}` in '{}'", self.command_line(), dir.display()),
            None => log::debug!("running `{}`", self.command_line()),
        }
    }

    fn to_command(&self) -> Command {
        let mut command = Command::new(&self.program);
        command.args(&self.args);
        command.envs(self.envs.iter().map(|(name, value)| (name, value)));
        if let Some(dir) = &self.dir {
            command.current_dir(dir);
        }
        command
    }

    /// Replaces "No such file or directory" with what is actually missing.
    ///
    /// The operating system answers the same way whether the program or the working
    /// directory does not exist, and names neither.
    fn explain(&self, error: io::Error) -> io::Error {
        if error.kind() != io::ErrorKind::NotFound {
            return error;
        }
        let missing = match &self.dir {
            Some(dir) if !dir.exists() => {
                format!("working directory '{}' does not exist", dir.display())
            }
            _ => format!("program '{}' was not found", self.program.to_string_lossy()),
        };
        io::Error::new(io::ErrorKind::NotFound, missing)
    }

    fn failed(&self, status: ExitStatus, stderr: String) -> CmdError {
        CmdError::Failed {
            command: self.command_line(),
            status,
            stderr,
        }
    }

    /// A command that succeeded may still have warned about something. That is worth
    /// keeping, but not worth more than the debug level: many programs, `git` among them,
    /// report ordinary progress on stderr.
    fn log_diagnostics(&self, stderr: &str) {
        if !stderr.is_empty() {
            log::debug!("`{}` wrote to stderr:\n{stderr}", self.command_line());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn collects_arguments_in_the_order_they_were_added() {
        let cmd = Cmd::new("git")
            .arg("commit")
            .args(["--message", "Fix the build"])
            .arg(Path::new("src"));

        assert_eq!(
            cmd.command_line().as_str(),
            "git commit --message 'Fix the build' src"
        );
    }

    #[test]
    fn accepts_borrowed_and_owned_values() {
        let cmd = Cmd::new(String::from("git"))
            .arg(String::from("status"))
            .args(vec![PathBuf::from("src")])
            .dir(Path::new("/srv/app"))
            .env("LANG", String::from("C"));

        assert_eq!(cmd.command_line().as_str(), "git status src");
    }

    #[test]
    fn environment_variables_are_not_part_of_the_command_line() {
        let cmd = Cmd::new("deploy").env("TOKEN", "secret");

        assert_eq!(cmd.command_line().as_str(), "deploy");
    }

    #[test]
    fn errors_other_than_not_found_are_kept_as_they_are() {
        let cmd = Cmd::new("deploy");
        let denied = io::Error::new(io::ErrorKind::PermissionDenied, "permission denied");

        let explained = cmd.explain(denied);

        assert_eq!(explained.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(explained.to_string(), "permission denied");
    }

    #[test]
    fn not_found_is_blamed_on_the_program_when_the_directory_exists() {
        let cmd = Cmd::new("deploy").dir(env!("CARGO_MANIFEST_DIR"));

        let explained = cmd.explain(io::ErrorKind::NotFound.into());

        assert_eq!(explained.kind(), io::ErrorKind::NotFound);
        assert_eq!(explained.to_string(), "program 'deploy' was not found");
    }

    #[test]
    fn not_found_is_blamed_on_the_directory_when_it_does_not_exist() {
        let cmd = Cmd::new("deploy").dir("/nimh/no/such/directory");

        let explained = cmd.explain(io::ErrorKind::NotFound.into());

        assert_eq!(
            explained.to_string(),
            "working directory '/nimh/no/such/directory' does not exist"
        );
    }
}
