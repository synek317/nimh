use std::error::Error;
use std::fmt;
use std::iter;

/// An error shown together with everything that caused it.
///
/// Usually created with [`DisplayChain::display_chain`](crate::DisplayChain::display_chain).
/// The format string picks the layout:
///
/// - `{}` puts the chain on one line, `outer: cause: root cause`, which suits logs;
/// - `{:#}` gives every cause its own line, which suits a terminal.
///
/// Many errors already quote their cause in their own message, for example
/// *"cannot read 'app.toml': No such file or directory"*. A cause that its parent has
/// already quoted is skipped, so nothing is said twice.
///
/// # Examples
///
/// ```
/// use nimh_error::ErrorChain;
///
/// let error = "http".parse::<u16>().unwrap_err();
///
/// assert_eq!(
///     ErrorChain::new(&error).to_string(),
///     "invalid digit found in string",
/// );
/// ```
#[derive(Debug)]
pub struct ErrorChain<'a, E: ?Sized>(&'a E);

impl<'a, E: Error + ?Sized> ErrorChain<'a, E> {
    /// Wraps `error` so that it is displayed together with its causes.
    #[must_use]
    pub fn new(error: &'a E) -> Self {
        Self(error)
    }
}

impl<E: Error + ?Sized> fmt::Display for ErrorChain<'_, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parent = self.0.to_string();
        f.write_str(&parent)?;

        for cause in causes(self.0) {
            let message = cause.to_string();
            if !parent.contains(&message) {
                write_cause(f, &message)?;
            }
            parent = message;
        }
        Ok(())
    }
}

/// Everything behind `error`, from its direct cause down to the root cause.
fn causes<E: Error + ?Sized>(error: &E) -> impl Iterator<Item = &(dyn Error + 'static)> {
    iter::successors(error.source(), |&cause| cause.source())
}

fn write_cause(f: &mut fmt::Formatter<'_>, message: &str) -> fmt::Result {
    if f.alternate() {
        write_cause_on_its_own_line(f, message)
    } else {
        write!(f, ": {message}")
    }
}

/// A message can span several lines itself, for example when it quotes the output of a
/// command. The lines after the first one are indented, so that they stay visibly attached
/// to their cause.
fn write_cause_on_its_own_line(f: &mut fmt::Formatter<'_>, message: &str) -> fmt::Result {
    let mut lines = message.lines();
    write!(f, "\n  caused by: {}", lines.next().unwrap_or_default())?;
    lines.try_for_each(|line| write!(f, "\n    {line}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layered_error::LayeredError;
    use test_case::test_case;

    #[test_case(&["a"], "a"; "error without a cause")]
    #[test_case(&["a", "b"], "a: b"; "one cause")]
    #[test_case(&["a", "b", "c"], "a: b: c"; "chain of causes")]
    #[test_case(
        &["cannot read 'app.toml': not found", "not found"],
        "cannot read 'app.toml': not found";
        "cause quoted by its parent is skipped"
    )]
    #[test_case(
        &["a: b", "b", "c"],
        "a: b: c";
        "cause behind a skipped one is still shown"
    )]
    #[test_case(&["a", ""], "a"; "cause without a message is skipped")]
    fn default_format_puts_the_chain_on_one_line(messages: &[&'static str], expected: &str) {
        let error = LayeredError::chain(messages);

        assert_eq!(ErrorChain::new(&error).to_string(), expected);
    }

    #[test_case(&["a"], "a"; "error without a cause")]
    #[test_case(
        &["a", "b", "c"],
        "a\n  caused by: b\n  caused by: c";
        "chain of causes"
    )]
    #[test_case(
        &["a: b", "b", "c"],
        "a: b\n  caused by: c";
        "cause quoted by its parent is skipped"
    )]
    #[test_case(
        &["deploy failed", "`git fetch` failed:\n  fatal: no route to host"],
        "deploy failed\n  caused by: `git fetch` failed:\n      fatal: no route to host";
        "lines of a multi-line cause stay attached to it"
    )]
    fn alternate_format_gives_every_cause_its_own_line(messages: &[&'static str], expected: &str) {
        let error = LayeredError::chain(messages);

        assert_eq!(format!("{:#}", ErrorChain::new(&error)), expected);
    }

    #[test]
    fn accepts_unsized_errors() {
        let error: Box<dyn Error + Send + Sync> = Box::new(LayeredError::chain(&["a", "b"]));

        assert_eq!(ErrorChain::new(error.as_ref()).to_string(), "a: b");
    }
}
