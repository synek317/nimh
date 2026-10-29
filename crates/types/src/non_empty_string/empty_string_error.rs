use std::error::Error;
use std::fmt;

/// The error returned when input for one of the non-empty string types is empty or consists only
/// of whitespace.
///
/// It keeps the original, untrimmed input. The message shows the input with `{:?}`, so
/// invisible characters such as `"\t\n"` are visible.
///
/// # Examples
///
/// ```
/// use nimh_types::NonEmptyString;
///
/// let err = NonEmptyString::new("\t\n").unwrap_err();
/// assert_eq!(
///     err.to_string(),
///     r#"string is empty or contains only whitespace: "\t\n""#,
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmptyStringError {
    input: String,
}

impl EmptyStringError {
    pub(crate) fn new(input: String) -> Self {
        Self { input }
    }

    /// Returns the rejected input, exactly as it was given (before trimming).
    ///
    /// # Examples
    ///
    /// ```
    /// use nimh_types::NonEmptyString;
    ///
    /// let err = NonEmptyString::new(" \u{3000} ").unwrap_err();
    /// assert_eq!(err.input(), " \u{3000} ");
    /// ```
    #[must_use]
    pub fn input(&self) -> &str {
        &self.input
    }

    /// Consumes the error and returns the rejected input, exactly as it was given.
    ///
    /// # Examples
    ///
    /// ```
    /// use nimh_types::NonEmptyString;
    ///
    /// let err = NonEmptyString::new(String::from("  ")).unwrap_err();
    /// let input: String = err.into_input();
    /// assert_eq!(input, "  ");
    /// ```
    #[must_use]
    pub fn into_input(self) -> String {
        self.input
    }
}

impl fmt::Display for EmptyStringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "string is empty or contains only whitespace: {:?}",
            self.input
        )
    }
}

impl Error for EmptyStringError {}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("", r#""""#; "empty")]
    #[test_case("\t\n\r", r#""\t\n\r""#; "control whitespace")]
    #[test_case("  ", r#""  ""#; "spaces")]
    fn message_shows_the_input_escaped(input: &str, shown: &str) {
        let err = EmptyStringError::new(input.to_owned());

        assert_eq!(
            err.to_string(),
            format!("string is empty or contains only whitespace: {shown}")
        );
    }

    #[test]
    fn keeps_the_original_input() {
        let err = EmptyStringError::new(" \u{00A0} ".to_owned());

        assert_eq!(err.input(), " \u{00A0} ");
        assert_eq!(err.into_input(), " \u{00A0} ");
    }

    #[test]
    fn is_a_std_error() {
        let err: Box<dyn Error + Send + Sync> = Box::new(EmptyStringError::new(String::new()));

        assert!(err.source().is_none());
    }
}
