use std::error::Error;
use std::ffi::OsString;
use std::fmt;

/// The error returned when an environment variable cannot be read as the requested type.
///
/// The message always names the variable, and shows the offending value with `{:?}`, so
/// that invisible characters such as a trailing `\n` are visible.
///
/// Values are part of the message. Keep that in mind before logging an error about a
/// variable that holds a secret.
///
/// # Examples
///
/// ```
/// let error = nimh_env::var::<u16>("NIMH_DOCS_UNSET").unwrap_err();
///
/// assert_eq!(error.name(), "NIMH_DOCS_UNSET");
/// assert_eq!(
///     error.to_string(),
///     "environment variable `NIMH_DOCS_UNSET` is not set",
/// );
/// ```
#[derive(Debug)]
#[non_exhaustive]
pub enum EnvError {
    /// The name is empty or contains `=` or a NUL character, so no variable can have it.
    InvalidName {
        /// The rejected name.
        name: String,
    },

    /// The variable is required, but it is not set.
    NotSet {
        /// The name of the variable.
        name: String,
    },

    /// The variable is set to something that is not valid Unicode.
    NotUnicode {
        /// The name of the variable.
        name: String,
        /// The value, exactly as the operating system returned it.
        value: OsString,
    },

    /// The variable is set, but its value does not parse as the requested type.
    InvalidValue {
        /// The name of the variable.
        name: String,
        /// The value that failed to parse.
        value: String,
        /// The error reported by the parser, usually [`FromStr::Err`](std::str::FromStr::Err).
        reason: Box<dyn Error + Send + Sync>,
    },
}

impl EnvError {
    /// Returns the name of the variable the error is about.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::InvalidName { name }
            | Self::NotSet { name }
            | Self::NotUnicode { name, .. }
            | Self::InvalidValue { name, .. } => name,
        }
    }
}

impl fmt::Display for EnvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName { name } => write!(
                f,
                "invalid environment variable name {name:?}: \
                 a name must not be empty or contain `=` or NUL"
            ),
            Self::NotSet { name } => write!(f, "environment variable `{name}` is not set"),
            Self::NotUnicode { name, value } => write!(
                f,
                "environment variable `{name}` is not valid Unicode: {value:?}"
            ),
            Self::InvalidValue {
                name,
                value,
                reason,
            } => write!(
                f,
                "environment variable `{name}` has invalid value {value:?}: {reason}"
            ),
        }
    }
}

impl Error for EnvError {
    /// The reason a value did not parse is already part of the message, so it is not
    /// repeated as the source. What caused that reason, if anything, is.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidValue { reason, .. } => reason.source(),
            Self::InvalidName { .. } | Self::NotSet { .. } | Self::NotUnicode { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    fn name() -> String {
        String::from("PORT")
    }

    fn invalid_value(value: &str) -> EnvError {
        EnvError::InvalidValue {
            name: name(),
            value: value.to_owned(),
            reason: value.parse::<u16>().unwrap_err().into(),
        }
    }

    #[test_case(
        &EnvError::InvalidName { name: String::from("PO=RT") },
        r#"invalid environment variable name "PO=RT": a name must not be empty or contain `=` or NUL"#;
        "invalid name"
    )]
    #[test_case(
        &EnvError::NotSet { name: name() },
        "environment variable `PORT` is not set";
        "not set"
    )]
    #[test_case(
        &EnvError::NotUnicode { name: name(), value: OsString::from("80") },
        r#"environment variable `PORT` is not valid Unicode: "80""#;
        "not Unicode"
    )]
    #[test_case(
        &invalid_value("http"),
        r#"environment variable `PORT` has invalid value "http": invalid digit found in string"#;
        "invalid value"
    )]
    #[test_case(
        &invalid_value("80\n"),
        r#"environment variable `PORT` has invalid value "80\n": invalid digit found in string"#;
        "invisible characters are escaped"
    )]
    #[test_case(
        &invalid_value(""),
        r#"environment variable `PORT` has invalid value "": cannot parse integer from empty string"#;
        "empty value"
    )]
    fn message_names_the_variable_and_the_problem(error: &EnvError, expected: &str) {
        assert_eq!(error.to_string(), expected);
        assert!(error.to_string().contains(error.name()));
    }

    #[test]
    fn reason_is_in_the_message_and_not_repeated_as_the_source() {
        let error = invalid_value("http");

        assert!(error.source().is_none());
    }

    #[test]
    fn can_cross_threads_as_a_boxed_error() {
        let error: Box<dyn Error + Send + Sync> = Box::new(invalid_value("http"));

        assert!(error.to_string().contains("`PORT`"));
    }
}
