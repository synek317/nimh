use std::ffi::{OsStr, OsString};
use std::fmt;

use crate::quote::quote;

/// A command as text: the program followed by its arguments.
///
/// It is written the way a shell expects it. Arguments that a shell would split or
/// interpret are quoted, so the text can be copied from an error or a log and tried by hand.
///
/// # Examples
///
/// ```
/// use nimh_cmd::Cmd;
///
/// let command_line = Cmd::new("grep")
///     .args(["--regexp", "it's here", "notes.txt"])
///     .command_line();
///
/// assert_eq!(command_line.to_string(), r"grep --regexp 'it'\''s here' notes.txt");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandLine(String);

impl CommandLine {
    pub(crate) fn new(program: &OsStr, args: &[OsString]) -> Self {
        let mut text = quote(program).into_owned();
        for arg in args {
            text.push(' ');
            text.push_str(&quote(arg));
        }
        Self(text)
    }

    /// Returns the command line as a `&str`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CommandLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("git", &[], "git"; "program alone")]
    #[test_case("git", &["status", "--short"], "git status --short"; "plain arguments")]
    #[test_case(
        "git",
        &["commit", "-m", "Fix the build"],
        "git commit -m 'Fix the build'";
        "argument with spaces"
    )]
    #[test_case("/opt/my tools/run", &[""], "'/opt/my tools/run' ''"; "quoted program and empty argument")]
    fn joins_the_program_and_its_arguments(program: &str, args: &[&str], expected: &str) {
        let args: Vec<OsString> = args.iter().map(OsString::from).collect();

        let command_line = CommandLine::new(OsStr::new(program), &args);

        assert_eq!(command_line.as_str(), expected);
        assert_eq!(command_line.to_string(), expected);
    }
}
