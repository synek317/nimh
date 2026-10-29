use super::{EmptyStringError, NonEmptyString};
use std::borrow::Borrow;
use std::fmt;

mod conversions;
mod partial_eq;
#[cfg(any(test, feature = "serde"))]
mod serde_impl;

/// A borrowed string slice that is never empty and never starts or ends with whitespace.
///
/// Construction trims whitespace from both ends and borrows the resulting subslice without
/// allocating. Whitespace inside the string is kept as is.
///
/// # Examples
///
/// ```
/// use nimh_types::NonEmptyStr;
///
/// let name = NonEmptyStr::new("  Ada  ")?;
/// assert_eq!(name.as_str(), "Ada");
/// # Ok::<(), nimh_types::EmptyStringError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonEmptyStr<'a>(&'a str);

impl<'a> NonEmptyStr<'a> {
    /// Creates a borrowed `NonEmptyStr`, trimming whitespace from both ends.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyStringError`] if `value` is empty or consists only of whitespace.
    pub fn new(value: &'a str) -> Result<Self, EmptyStringError> {
        match value.trim() {
            "" => Err(EmptyStringError::new(value.to_owned())),
            trimmed => Ok(Self(trimmed)),
        }
    }

    /// Returns the trimmed string slice.
    #[must_use]
    pub fn as_str(&self) -> &'a str {
        self.0
    }

    /// Copies the borrowed value into a `NonEmptyString`.
    #[must_use]
    pub fn to_owned(&self) -> NonEmptyString {
        NonEmptyString::from(*self)
    }
}

impl AsRef<str> for NonEmptyStr<'_> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl Borrow<str> for NonEmptyStr<'_> {
    fn borrow(&self) -> &str {
        self.0
    }
}

impl fmt::Display for NonEmptyStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("abc", "abc"; "already trimmed")]
    #[test_case("  abc  ", "abc"; "surrounding whitespace")]
    #[test_case(" a  b ", "a  b"; "inner whitespace")]
    #[test_case("\u{200B}", "\u{200B}"; "zero-width space")]
    fn borrows_the_trimmed_slice(input: &str, expected: &str) {
        let value = NonEmptyStr::new(input).expect("not blank");

        assert_eq!(value.as_str(), expected);
    }

    #[test_case("")]
    #[test_case(" \t\n")]
    #[test_case("\u{3000}")]
    fn rejects_blank_input(input: &str) {
        let error = NonEmptyStr::new(input).unwrap_err();

        assert_eq!(error.input(), input);
    }

    #[test]
    fn copies_into_non_empty_string() -> Result<(), EmptyStringError> {
        let borrowed = NonEmptyStr::new("Ada")?;

        assert_eq!(borrowed.to_owned().as_str(), "Ada");
        Ok(())
    }
}
