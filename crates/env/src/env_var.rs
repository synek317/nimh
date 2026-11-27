use std::env::{self, VarError};
use std::error::Error;
use std::fmt;
use std::str::FromStr;

use crate::EnvError;

/// The name of an environment variable, checked once and ready to be read.
///
/// A name is valid when it is not empty and contains neither `=` nor a NUL character. Those
/// are the names an operating system can actually hold; a lookup of any other name would
/// always come back empty, which looks like a missing variable and hides the real mistake.
///
/// # Examples
///
/// Declared as a `const`, the name is checked by the compiler:
///
/// ```
/// use nimh_env::EnvVar;
///
/// const WORKERS: EnvVar = EnvVar::new("NIMH_DOCS_WORKERS");
///
/// assert_eq!(WORKERS.name(), "NIMH_DOCS_WORKERS");
/// assert_eq!(WORKERS.get_or(4)?, 4);
/// # Ok::<(), nimh_env::EnvError>(())
/// ```
///
/// ```compile_fail
/// use nimh_env::EnvVar;
///
/// const BROKEN: EnvVar = EnvVar::new("WORKERS=4");
/// let _ = BROKEN;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnvVar<'a>(&'a str);

impl<'a> EnvVar<'a> {
    /// Creates an `EnvVar` from a name known to be valid, typically a literal.
    ///
    /// For names that are only known at run time use [`try_new`](Self::try_new).
    ///
    /// # Panics
    ///
    /// Panics if `name` is empty or contains `=` or a NUL character. In a `const` the panic
    /// happens during compilation.
    #[must_use]
    pub const fn new(name: &'a str) -> Self {
        assert!(
            is_valid(name),
            "the name of an environment variable must not be empty or contain `=` or NUL"
        );
        Self(name)
    }

    /// Creates an `EnvVar` from a name that may be invalid.
    ///
    /// # Errors
    ///
    /// Returns [`EnvError::InvalidName`] if `name` is empty or contains `=` or a NUL
    /// character.
    ///
    /// # Examples
    ///
    /// ```
    /// use nimh_env::EnvVar;
    ///
    /// let service = "billing";
    /// let name = format!("{}_PORT", service.to_uppercase());
    ///
    /// assert_eq!(EnvVar::try_new(&name)?.name(), "BILLING_PORT");
    /// assert!(EnvVar::try_new("BILLING=PORT").is_err());
    /// # Ok::<(), nimh_env::EnvError>(())
    /// ```
    pub fn try_new(name: &'a str) -> Result<Self, EnvError> {
        if is_valid(name) {
            Ok(Self(name))
        } else {
            Err(EnvError::InvalidName {
                name: name.to_owned(),
            })
        }
    }

    /// Returns the name of the variable.
    #[must_use]
    pub const fn name(self) -> &'a str {
        self.0
    }

    /// Reads the variable, which is required, and parses it.
    ///
    /// # Errors
    ///
    /// Returns [`EnvError`] when the variable is not set, is not valid Unicode or does not
    /// parse as `T`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use nimh_env::EnvVar;
    ///
    /// const PORT: EnvVar = EnvVar::new("PORT");
    ///
    /// let port: u16 = PORT.get()?;
    /// # Ok::<(), nimh_env::EnvError>(())
    /// ```
    pub fn get<T>(self) -> Result<T, EnvError>
    where
        T: FromStr,
        T::Err: Into<Box<dyn Error + Send + Sync>>,
    {
        self.get_opt()?.ok_or_else(|| EnvError::NotSet {
            name: self.0.to_owned(),
        })
    }

    /// Reads the variable and parses it, or returns `default` when it is not set.
    ///
    /// The default replaces a missing variable only. A variable that is set to something
    /// unusable is an error, so that a typo cannot hide behind the default.
    ///
    /// # Errors
    ///
    /// Returns [`EnvError`] when the variable is set but is not valid Unicode or does not
    /// parse as `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use nimh_env::EnvVar;
    /// use std::net::{IpAddr, Ipv4Addr};
    ///
    /// const BIND_ADDRESS: EnvVar = EnvVar::new("NIMH_DOCS_BIND_ADDRESS");
    ///
    /// let address = BIND_ADDRESS.get_or(IpAddr::V4(Ipv4Addr::LOCALHOST))?;
    /// assert_eq!(address.to_string(), "127.0.0.1");
    /// # Ok::<(), nimh_env::EnvError>(())
    /// ```
    pub fn get_or<T>(self, default: T) -> Result<T, EnvError>
    where
        T: FromStr,
        T::Err: Into<Box<dyn Error + Send + Sync>>,
    {
        if let Some(value) = self.get_opt()? {
            return Ok(value);
        }
        log::debug!("environment variable `{self}` is not set, using the default");
        Ok(default)
    }

    /// Reads the variable and parses it, or returns `None` when it is not set.
    ///
    /// # Errors
    ///
    /// Returns [`EnvError`] when the variable is set but is not valid Unicode or does not
    /// parse as `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use nimh_env::EnvVar;
    ///
    /// const PROXY_PORT: EnvVar = EnvVar::new("NIMH_DOCS_PROXY_PORT");
    ///
    /// match PROXY_PORT.get_opt::<u16>()? {
    ///     Some(port) => println!("connecting through the proxy on port {port}"),
    ///     None => println!("connecting directly"),
    /// }
    /// # Ok::<(), nimh_env::EnvError>(())
    /// ```
    pub fn get_opt<T>(self) -> Result<Option<T>, EnvError>
    where
        T: FromStr,
        T::Err: Into<Box<dyn Error + Send + Sync>>,
    {
        self.interpret(env::var(self.0))
    }

    /// Turns what the operating system said about the variable into a typed value.
    ///
    /// Kept apart from the lookup itself, so that every outcome can be tested without
    /// changing the environment of the test process.
    fn interpret<T>(self, lookup: Result<String, VarError>) -> Result<Option<T>, EnvError>
    where
        T: FromStr,
        T::Err: Into<Box<dyn Error + Send + Sync>>,
    {
        match lookup {
            Ok(value) => self.parse(value).map(Some),
            Err(VarError::NotPresent) => Ok(None),
            Err(VarError::NotUnicode(value)) => Err(EnvError::NotUnicode {
                name: self.0.to_owned(),
                value,
            }),
        }
    }

    fn parse<T>(self, value: String) -> Result<T, EnvError>
    where
        T: FromStr,
        T::Err: Into<Box<dyn Error + Send + Sync>>,
    {
        match value.parse() {
            Ok(parsed) => Ok(parsed),
            Err(reason) => Err(EnvError::InvalidValue {
                name: self.0.to_owned(),
                value,
                reason: T::Err::into(reason),
            }),
        }
    }
}

impl fmt::Display for EnvVar<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.0, f)
    }
}

/// A loop over bytes instead of `str::contains`, because this also runs at compile time.
const fn is_valid(name: &str) -> bool {
    let bytes = name.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if matches!(bytes[index], b'=' | b'\0') {
            return false;
        }
        index += 1;
    }
    !bytes.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use test_case::test_case;

    const PORT: EnvVar = EnvVar::new("PORT");
    const UNSET: EnvVar = EnvVar::new("NIMH_ENV_TESTS_UNSET");

    #[expect(
        clippy::unnecessary_wraps,
        reason = "stands for a successful lookup, which is a `Result`"
    )]
    fn set(value: &str) -> Result<String, VarError> {
        Ok(value.to_owned())
    }

    #[test_case("PORT"; "upper case")]
    #[test_case("port"; "lower case")]
    #[test_case("MY_APP_PORT_2"; "underscores and digits")]
    #[test_case("my.app port"; "dots and spaces")]
    #[test_case("ZAŻÓŁĆ"; "non-ASCII")]
    fn accepts_names_an_operating_system_can_hold(name: &str) -> Result<(), EnvError> {
        assert_eq!(EnvVar::try_new(name)?.name(), name);
        assert_eq!(EnvVar::new(name).name(), name);
        Ok(())
    }

    #[test_case(""; "empty")]
    #[test_case("="; "only equals sign")]
    #[test_case("PORT=8080"; "equals sign")]
    #[test_case("PO\0RT"; "NUL character")]
    fn rejects_names_an_operating_system_cannot_hold(name: &str) {
        let error = EnvVar::try_new(name).unwrap_err();

        assert!(matches!(error, EnvError::InvalidName { .. }), "{error}");
        assert_eq!(error.name(), name);
    }

    #[test]
    #[should_panic(expected = "must not be empty or contain `=` or NUL")]
    fn new_panics_on_an_invalid_name() {
        let _ = EnvVar::new("PORT=8080");
    }

    #[test]
    fn value_that_is_set_is_parsed() -> Result<(), EnvError> {
        assert_eq!(PORT.interpret::<u16>(set("8080"))?, Some(8080));
        assert_eq!(PORT.interpret::<bool>(set("true"))?, Some(true));
        assert_eq!(
            PORT.interpret::<String>(set(" as is "))?.as_deref(),
            Some(" as is ")
        );
        Ok(())
    }

    #[test]
    fn empty_value_counts_as_set() -> Result<(), EnvError> {
        assert_eq!(PORT.interpret::<String>(set(""))?, Some(String::new()));
        assert!(PORT.interpret::<u16>(set("")).is_err());
        Ok(())
    }

    #[test]
    fn variable_that_is_not_set_gives_nothing() -> Result<(), EnvError> {
        assert_eq!(PORT.interpret::<u16>(Err(VarError::NotPresent))?, None);
        Ok(())
    }

    #[test]
    fn value_that_does_not_parse_is_reported_with_the_reason() {
        let error = PORT.interpret::<u16>(set("http")).unwrap_err();

        assert_eq!(
            error.to_string(),
            r#"environment variable `PORT` has invalid value "http": invalid digit found in string"#
        );
    }

    #[test]
    fn value_that_is_not_unicode_is_reported() {
        let lookup = Err(VarError::NotUnicode(OsString::from("8080")));

        let error = PORT.interpret::<u16>(lookup).unwrap_err();

        assert!(matches!(error, EnvError::NotUnicode { .. }), "{error}");
        assert_eq!(error.name(), "PORT");
    }

    #[test]
    fn reads_the_environment_of_the_process() -> Result<(), EnvError> {
        // Cargo sets this variable for every test it runs.
        let package: String = EnvVar::new("CARGO_PKG_NAME").get()?;

        assert_eq!(package, "nimh_env");
        Ok(())
    }

    #[test]
    fn required_variable_that_is_not_set_is_an_error() {
        let error = UNSET.get::<String>().unwrap_err();

        assert_eq!(
            error.to_string(),
            "environment variable `NIMH_ENV_TESTS_UNSET` is not set"
        );
    }

    #[test]
    fn default_replaces_a_variable_that_is_not_set() -> Result<(), EnvError> {
        assert_eq!(UNSET.get_or(4)?, 4);
        assert_eq!(UNSET.get_opt::<u16>()?, None);
        Ok(())
    }

    #[test]
    fn default_does_not_hide_an_invalid_value() {
        let error = EnvVar::new("CARGO_PKG_NAME").get_or(4).unwrap_err();

        assert!(matches!(error, EnvError::InvalidValue { .. }), "{error}");
    }

    #[test]
    fn is_displayed_as_its_name() {
        assert_eq!(PORT.to_string(), "PORT");
    }
}
