//! Typed environment variables for [NiMh](https://github.com/synek317/nimh), also available
//! through `nimh::env`.
//!
//! `NiMh`: [crates.io](https://crates.io/crates/nimh) / [docs.rs](https://docs.rs/nimh) / [github](https://github.com/synek317/nimh).
//!
//! Configuration read from the environment arrives as text, and three things can go wrong on
//! its way to a usable value: the variable is missing, it is not Unicode, or it does not
//! parse. `std::env::var` reports the first two without naming the variable and leaves the
//! third to the caller. Here one call does all of it, and the error says which variable is
//! wrong, what it holds and why that is not acceptable:
//!
//! ```text
//! environment variable `PORT` has invalid value "http": invalid digit found in string
//! ```
//!
//! # Reading a variable
//!
//! A value can be of any type that implements `FromStr`. There are three ways to ask for
//! it, one for each meaning a missing variable can have:
//!
//! ```no_run
//! use std::net::IpAddr;
//!
//! // Required: the application cannot work without it.
//! let port: u16 = nimh_env::var("PORT")?;
//!
//! // Optional, with a value to use instead.
//! let workers: usize = nimh_env::var_or("WORKERS", 4)?;
//!
//! // Optional, and its absence means something.
//! let bind_address: Option<IpAddr> = nimh_env::var_opt("BIND_ADDRESS")?;
//! # Ok::<(), nimh_env::EnvError>(())
//! ```
//!
//! A variable set to something unusable is always an error, also when a default exists. A
//! typo in `WORKERS=fuor` should stop the application, not start it quietly with 4 workers.
//!
//! The value is parsed exactly as it was set. It is not trimmed, and an empty value
//! (`PORT=`) counts as set.
//!
//! # Naming a variable once
//!
//! `EnvVar` is a variable name that has been checked. As a `const` it is checked by the
//! compiler, lives in one place and can be read wherever it is needed:
//!
//! ```
//! use nimh_env::EnvVar;
//!
//! const WORKERS: EnvVar = EnvVar::new("NIMH_DOCS_WORKERS");
//!
//! let workers: usize = WORKERS.get_or(4)?;
//! assert_eq!(workers, 4);
//! # Ok::<(), nimh_env::EnvError>(())
//! ```
//!
//! # Logs
//!
//! Falling back to a default is announced at the debug level of the `log` facade, with the
//! name of the variable only. Values never reach the logs, because they are often secrets.

use std::error::Error;
use std::str::FromStr;

pub use env_error::EnvError;
pub use env_var::EnvVar;

mod env_error;
mod env_var;

/// Reads a required environment variable and parses it.
///
/// # Errors
///
/// Returns [`EnvError`] when `name` cannot be the name of a variable, or when the variable
/// is not set, is not valid Unicode or does not parse as `T`.
///
/// # Examples
///
/// ```
/// let error = nimh_env::var::<u16>("NIMH_DOCS_UNSET").unwrap_err();
///
/// assert_eq!(
///     error.to_string(),
///     "environment variable `NIMH_DOCS_UNSET` is not set",
/// );
/// ```
pub fn var<T>(name: &str) -> Result<T, EnvError>
where
    T: FromStr,
    T::Err: Into<Box<dyn Error + Send + Sync>>,
{
    EnvVar::try_new(name)?.get()
}

/// Reads an environment variable and parses it, or returns `default` when it is not set.
///
/// # Errors
///
/// Returns [`EnvError`] when `name` cannot be the name of a variable, or when the variable
/// is set but is not valid Unicode or does not parse as `T`.
///
/// # Examples
///
/// ```
/// let workers: usize = nimh_env::var_or("NIMH_DOCS_UNSET", 4)?;
///
/// assert_eq!(workers, 4);
/// # Ok::<(), nimh_env::EnvError>(())
/// ```
pub fn var_or<T>(name: &str, default: T) -> Result<T, EnvError>
where
    T: FromStr,
    T::Err: Into<Box<dyn Error + Send + Sync>>,
{
    EnvVar::try_new(name)?.get_or(default)
}

/// Reads an environment variable and parses it, or returns `None` when it is not set.
///
/// # Errors
///
/// Returns [`EnvError`] when `name` cannot be the name of a variable, or when the variable
/// is set but is not valid Unicode or does not parse as `T`.
///
/// # Examples
///
/// ```
/// let workers: Option<usize> = nimh_env::var_opt("NIMH_DOCS_UNSET")?;
///
/// assert_eq!(workers, None);
/// # Ok::<(), nimh_env::EnvError>(())
/// ```
pub fn var_opt<T>(name: &str) -> Result<Option<T>, EnvError>
where
    T: FromStr,
    T::Err: Into<Box<dyn Error + Send + Sync>>,
{
    EnvVar::try_new(name)?.get_opt()
}

#[cfg(test)]
mod tests {
    use super::*;

    const UNSET: &str = "NIMH_ENV_TESTS_UNSET";

    #[test]
    fn required_variable_that_is_not_set_is_an_error() {
        let error = var::<String>(UNSET).unwrap_err();

        assert!(matches!(error, EnvError::NotSet { .. }), "{error}");
    }

    #[test]
    fn optional_variable_that_is_not_set_gives_the_default_or_nothing() -> Result<(), EnvError> {
        assert_eq!(var_or(UNSET, 4)?, 4);
        assert_eq!(var_opt::<u16>(UNSET)?, None);
        Ok(())
    }

    #[test]
    fn invalid_name_is_reported_by_every_function() {
        let errors = [
            var::<String>("A=B").unwrap_err(),
            var_or("A=B", String::new()).unwrap_err(),
            var_opt::<String>("A=B").unwrap_err(),
        ];

        for error in errors {
            assert!(matches!(error, EnvError::InvalidName { .. }), "{error}");
        }
    }
}
