use super::NonEmptyStr;

impl PartialEq<str> for NonEmptyStr<'_> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for NonEmptyStr<'_> {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for NonEmptyStr<'_> {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<NonEmptyStr<'_>> for str {
    fn eq(&self, other: &NonEmptyStr<'_>) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<NonEmptyStr<'_>> for &str {
    fn eq(&self, other: &NonEmptyStr<'_>) -> bool {
        *self == other.as_str()
    }
}

impl PartialEq<NonEmptyStr<'_>> for String {
    fn eq(&self, other: &NonEmptyStr<'_>) -> bool {
        self.as_str() == other.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EmptyStringError;
    use test_case::test_case;

    #[test_case("abc", "abc", true; "equal")]
    #[test_case("abc", "ABC", false; "case-sensitive")]
    #[test_case("abc", " abc ", false; "other side is not trimmed")]
    fn compares_with_string_types_in_both_directions(
        value: &str,
        other: &str,
        expected: bool,
    ) -> Result<(), EmptyStringError> {
        let value = NonEmptyStr::new(value)?;
        let other_string = other.to_owned();

        assert_eq!(value == *other, expected, "NonEmptyStr == str");
        assert_eq!(*other == value, expected, "str == NonEmptyStr");
        assert_eq!(value == other, expected, "NonEmptyStr == &str");
        assert_eq!(other == value, expected, "&str == NonEmptyStr");
        assert_eq!(value == other_string, expected, "NonEmptyStr == String");
        assert_eq!(other_string == value, expected, "String == NonEmptyStr");
        Ok(())
    }
}
