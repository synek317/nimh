use super::{BoxStr, EmptyStringError, NonEmptyStr};
use std::borrow::Borrow;
use std::fmt;

mod conversions;
mod partial_eq;
#[cfg(any(test, feature = "serde"))]
mod serde_impl;

/// An owned boxed string that is never empty and never starts or ends with whitespace.
///
/// It has no spare capacity and is useful for immutable strings kept in memory for a long time.
/// Construction trims whitespace from both ends.
///
/// # Examples
///
/// ```
/// use nimh_types::NonEmptyBoxStr;
///
/// let name = NonEmptyBoxStr::new("  Ada  ")?;
/// assert_eq!(name.as_str(), "Ada");
/// # Ok::<(), nimh_types::EmptyStringError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonEmptyBoxStr(BoxStr);

impl NonEmptyBoxStr {
    /// Creates a `NonEmptyBoxStr`, trimming whitespace from both ends.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyStringError`] if `value` is empty or consists only of whitespace.
    pub fn new(value: impl Into<BoxStr>) -> Result<Self, EmptyStringError> {
        let value = value.into();
        match value.trim() {
            "" => Err(EmptyStringError::new(String::from(value))),
            trimmed if trimmed.len() == value.len() => Ok(Self(value)),
            trimmed => Ok(Self(BoxStr::from(trimmed))),
        }
    }

    /// Returns the string as a `&str`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns a borrowed `NonEmptyStr`.
    #[must_use]
    pub fn as_non_empty_str(&self) -> NonEmptyStr<'_> {
        NonEmptyStr::from(self)
    }

    /// Consumes the value and returns its boxed string.
    #[must_use]
    pub fn into_boxed_str(self) -> BoxStr {
        self.0
    }
}

impl AsRef<str> for NonEmptyBoxStr {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for NonEmptyBoxStr {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for NonEmptyBoxStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("abc", "abc"; "already trimmed")]
    #[test_case("  abc  ", "abc"; "surrounding whitespace")]
    #[test_case(" a  b ", "a  b"; "inner whitespace")]
    fn owns_the_trimmed_string(input: &str, expected: &str) {
        let value = NonEmptyBoxStr::new(input).expect("not blank");

        assert_eq!(value.as_str(), expected);
    }

    #[test_case("")]
    #[test_case(" \t\n")]
    #[test_case("\u{3000}")]
    fn rejects_blank_input(input: &str) {
        let error = NonEmptyBoxStr::new(input).unwrap_err();

        assert_eq!(error.input(), input);
    }
}
