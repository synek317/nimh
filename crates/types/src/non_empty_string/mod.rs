use super::NonEmptyStr;
pub use empty_string_error::EmptyStringError;
use std::borrow::Borrow;
use std::fmt;
use std::str::FromStr;

mod conversions;
mod empty_string_error;
mod partial_eq;
#[cfg(any(test, feature = "serde"))]
mod serde_impl;

/// A string that is never empty and never starts or ends with whitespace.
///
/// The input is trimmed with [`str::trim`], which removes every character with the Unicode
/// `White_Space` property, among others: space, tab, `\n`, `\r`, no-break space (`U+00A0`)
/// and ideographic space (`U+3000`). Whitespace inside the string is kept as is. If nothing
/// is left after trimming, construction fails with [`EmptyStringError`].
///
/// Invisible characters that are not Unicode `White_Space`, such as zero-width space
/// (`U+200B`) or the byte order mark (`U+FEFF`), are **not** removed, so a string made only
/// of them is accepted.
///
/// Comparisons with `str` and `String` are exact: the other side is not trimmed.
///
/// With serde it serializes as a plain string. Deserialization trims and validates exactly
/// like [`NonEmptyString::new`], so a blank string is rejected.
///
/// Deliberately not implemented:
/// - `Default`, because there is no sensible non-empty default;
/// - `Deref<Target = str>` and `DerefMut`; call [`as_str`](Self::as_str) to use `str` methods;
/// - `From<&str>` and `From<String>`, because the conversion can fail; use
///   [`new`](Self::new), [`TryFrom`] or [`str::parse`] instead.
///
/// # Examples
///
/// ```
/// use nimh_types::NonEmptyString;
///
/// let name = NonEmptyString::new("  Ada  Lovelace \n")?;
/// assert_eq!(name, "Ada  Lovelace");
///
/// let parsed: NonEmptyString = "Ada".parse()?;
/// let converted = NonEmptyString::try_from(String::from("Ada"))?;
/// assert_eq!(parsed, converted);
///
/// assert!(NonEmptyString::new(" \t\n").is_err());
/// assert!(NonEmptyString::new("\u{200B}").is_ok());
/// # Ok::<(), nimh_types::EmptyStringError>(())
/// ```
// `Eq`, `Ord` and `Hash` are derived from the inner `String`, so they agree with `str`.
// `Borrow<str>` relies on that, e.g. for `HashMap` lookups by `&str`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NonEmptyString(String);

impl NonEmptyString {
    /// Creates a `NonEmptyString` from `value`, trimming whitespace from both ends.
    ///
    /// # Errors
    ///
    /// Returns [`EmptyStringError`], holding the original input, if `value` is empty or
    /// consists only of whitespace.
    ///
    /// # Examples
    ///
    /// ```
    /// use nimh_types::NonEmptyString;
    ///
    /// let name = NonEmptyString::new(String::from("  Ada "))?;
    /// assert_eq!(name, "Ada");
    ///
    /// let err = NonEmptyString::new("\t\n").unwrap_err();
    /// assert_eq!(err.input(), "\t\n");
    /// # Ok::<(), nimh_types::EmptyStringError>(())
    /// ```
    pub fn new(value: impl Into<String>) -> Result<Self, EmptyStringError> {
        let mut value = value.into();
        if value.trim().is_empty() {
            return Err(EmptyStringError::new(value));
        }

        value.truncate(value.trim_end().len());
        value.drain(..value.len() - value.trim_start().len());
        Ok(Self(value))
    }

    /// Returns the string as a `&str`.
    ///
    /// # Examples
    ///
    /// ```
    /// use nimh_types::NonEmptyString;
    ///
    /// let name = NonEmptyString::new(" Ada ")?;
    /// assert_eq!(name.as_str().len(), 3);
    /// # Ok::<(), nimh_types::EmptyStringError>(())
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns a borrowed `NonEmptyStr`.
    #[must_use]
    pub fn as_non_empty_str(&self) -> NonEmptyStr<'_> {
        NonEmptyStr::from(self)
    }

    /// Consumes the `NonEmptyString` and returns the inner, trimmed `String`.
    ///
    /// # Examples
    ///
    /// ```
    /// use nimh_types::NonEmptyString;
    ///
    /// let name = NonEmptyString::new(" Ada ")?;
    /// let name: String = name.into_string();
    /// assert_eq!(name, "Ada");
    /// # Ok::<(), nimh_types::EmptyStringError>(())
    /// ```
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl TryFrom<String> for NonEmptyString {
    type Error = EmptyStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for NonEmptyString {
    type Error = EmptyStringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl FromStr for NonEmptyString {
    type Err = EmptyStringError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl AsRef<str> for NonEmptyString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for NonEmptyString {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for NonEmptyString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl From<NonEmptyString> for String {
    fn from(value: NonEmptyString) -> Self {
        value.into_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeSet, HashMap};
    use test_case::test_case;

    #[test_case(""; "empty")]
    #[test_case(" "; "space")]
    #[test_case("\t\n\r"; "tab newline carriage return")]
    #[test_case("\u{00A0}"; "no-break space")]
    #[test_case("\u{3000}"; "ideographic space")]
    #[test_case(
        " \u{00A0}\t\u{3000}\u{2028}\u{0085}\n";
        "mixed Unicode whitespace"
    )]
    fn rejects_blank_input_and_keeps_it_in_the_error(input: &str) {
        let err = NonEmptyString::new(input).unwrap_err();

        assert_eq!(err.input(), input);
    }

    #[test_case("  abc  ", "abc"; "surrounding spaces")]
    #[test_case("  a  b  ", "a  b"; "inner whitespace is kept")]
    #[test_case("\u{3000}\t abc \u{00A0}\r\n", "abc"; "Unicode whitespace")]
    #[test_case("abc", "abc"; "already trimmed")]
    #[test_case("a \t\n b", "a \t\n b"; "already trimmed with inner whitespace")]
    #[test_case(" zażółć gęślą jaźń ", "zażółć gęślą jaźń"; "non-ASCII")]
    #[test_case("\u{200B}", "\u{200B}"; "zero-width space alone")]
    #[test_case(
        " \u{200B}abc\u{200B} ",
        "\u{200B}abc\u{200B}";
        "zero-width space is not trimmed"
    )]
    #[test_case("\u{FEFF}", "\u{FEFF}"; "byte order mark alone")]
    fn trims_unicode_whitespace_from_both_ends_only(
        input: &str,
        expected: &str,
    ) -> Result<(), EmptyStringError> {
        let value = NonEmptyString::new(input)?;

        assert_eq!(value.as_str(), expected);
        Ok(())
    }

    #[test_case("  abc "; "needs trimming")]
    #[test_case("abc"; "already trimmed")]
    #[test_case(""; "empty")]
    #[test_case(" \t\n"; "blank")]
    fn all_constructors_give_the_same_result(input: &str) {
        let expected = NonEmptyString::new(input);

        assert_eq!(input.parse::<NonEmptyString>(), expected, "FromStr");
        assert_eq!(NonEmptyString::try_from(input), expected, "TryFrom<&str>");
        assert_eq!(
            NonEmptyString::try_from(input.to_owned()),
            expected,
            "TryFrom<String>"
        );
        assert_eq!(
            NonEmptyString::new(input.to_owned()),
            expected,
            "new(String)"
        );
    }

    #[test]
    fn hash_map_can_be_searched_by_str() -> Result<(), EmptyStringError> {
        let ages = HashMap::from([(NonEmptyString::new("Ada")?, 36)]);

        assert_eq!(ages.get("Ada"), Some(&36));
        assert_eq!(ages.get("ada"), None);
        Ok(())
    }

    #[test]
    fn ordered_set_can_be_searched_by_str() -> Result<(), EmptyStringError> {
        let names = BTreeSet::from([NonEmptyString::new("Grace")?, NonEmptyString::new("Ada")?]);

        assert!(names.contains("Ada"));
        assert_eq!(names.first().map(NonEmptyString::as_str), Some("Ada"));
        Ok(())
    }

    #[test]
    fn exposes_the_trimmed_string() -> Result<(), EmptyStringError> {
        let value = NonEmptyString::new(" abc ")?;

        assert_eq!(value.as_str(), "abc");
        assert_eq!(AsRef::<str>::as_ref(&value), "abc");
        assert_eq!(Borrow::<str>::borrow(&value), "abc");
        assert_eq!(value.clone().into_string(), "abc");
        assert_eq!(String::from(value), "abc");
        Ok(())
    }

    #[test]
    fn display_writes_the_trimmed_string_and_honours_format_flags() -> Result<(), EmptyStringError>
    {
        let value = NonEmptyString::new(" abc ")?;

        assert_eq!(value.to_string(), "abc");
        assert_eq!(format!("[{value:>5}]"), "[  abc]");
        Ok(())
    }
}
