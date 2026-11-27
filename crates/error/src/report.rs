use std::error::Error;
use std::io::{self, Write};
use std::process::ExitCode;

use crate::{DisplayChain, Reportable};

/// Ends the program in a readable way: prints the error, if there is one, and picks the
/// exit code.
///
/// On failure the error is written to stderr together with everything that caused it, one
/// cause per line, and the exit code is [`ExitCode::FAILURE`]:
///
/// ```text
/// error: cannot load the configuration
///   caused by: cannot read 'app.toml'
///   caused by: No such file or directory (os error 2)
/// ```
///
/// The error can have any of the shapes a program usually ends with: a type that implements
/// [`Error`], `Box<dyn Error>` with or without `Send + Sync`, `anyhow::Error` or a plain
/// string. See [`Reportable`] for the full list.
///
/// # Examples
///
/// ```no_run
/// use std::error::Error;
/// use std::process::ExitCode;
///
/// fn main() -> ExitCode {
///     nimh_error::report(run())
/// }
///
/// fn run() -> Result<(), Box<dyn Error>> {
///     let port: u16 = "http".parse()?;
///     println!("listening on port {port}");
///     Ok(())
/// }
/// ```
pub fn report<E, Shape>(result: Result<(), E>) -> ExitCode
where
    E: Reportable<Shape>,
{
    let Err(error) = result else {
        return ExitCode::SUCCESS;
    };
    print_failure(error.into_boxed_error().as_ref());
    ExitCode::FAILURE
}

fn print_failure(error: &dyn Error) {
    // When stderr itself is gone there is nobody left to tell, so a failed write is ignored.
    let _ = writeln!(io::stderr(), "{}", failure_message(error));
}

fn failure_message(error: &dyn Error) -> String {
    format!("error: {:#}", error.display_chain())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layered_error::LayeredError;

    /// `ExitCode` cannot be compared, but its `Debug` form can.
    fn shown(exit_code: ExitCode) -> String {
        format!("{exit_code:?}")
    }

    #[test]
    fn success_gives_the_success_exit_code() {
        let exit_code = report(Ok::<(), LayeredError>(()));

        assert_eq!(shown(exit_code), shown(ExitCode::SUCCESS));
    }

    #[test]
    fn failure_gives_the_failure_exit_code() {
        let exit_code = report(Err(LayeredError::chain(&["a", "b"])));

        assert_eq!(shown(exit_code), shown(ExitCode::FAILURE));
    }

    #[test]
    fn accepts_the_usual_results_of_a_program() {
        let boxed: Result<(), Box<dyn Error>> = Err("boxed".into());
        let thread_safe: Result<(), Box<dyn Error + Send + Sync>> = Err("thread-safe".into());
        let string: Result<(), String> = Err(String::from("string"));

        assert_eq!(shown(report(boxed)), shown(ExitCode::FAILURE));
        assert_eq!(shown(report(thread_safe)), shown(ExitCode::FAILURE));
        assert_eq!(shown(report(string)), shown(ExitCode::FAILURE));
    }

    #[test]
    fn failure_message_lists_every_cause_on_its_own_line() {
        let error = LayeredError::chain(&["cannot start", "cannot read 'app.toml'", "not found"]);

        assert_eq!(
            failure_message(&error),
            "error: cannot start\n  caused by: cannot read 'app.toml'\n  caused by: not found"
        );
    }
}
