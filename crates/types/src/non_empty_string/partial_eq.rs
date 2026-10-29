use super::NonEmptyString;

impl PartialEq<str> for NonEmptyString {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for NonEmptyString {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for NonEmptyString {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<NonEmptyString> for str {
    fn eq(&self, other: &NonEmptyString) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<NonEmptyString> for &str {
    fn eq(&self, other: &NonEmptyString) -> bool {
        *self == other.as_str()
    }
}

impl PartialEq<NonEmptyString> for String {
    fn eq(&self, other: &NonEmptyString) -> bool {
        self.as_str() == other.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EmptyStringError;
    use test_case::test_case;

    #[test_case("abc", "abc", true; "equal")]
    #[test_case("abc", "abd", false; "different")]
    #[test_case("abc", "ABC", false; "case-sensitive")]
    #[test_case("abc", " abc ", false; "other side is not trimmed")]
    #[test_case("zażółć", "zażółć", true; "non-ASCII")]
    fn compares_with_string_types_in_both_directions(
        value: &str,
        other: &str,
        expected: bool,
    ) -> Result<(), EmptyStringError> {
        let value = NonEmptyString::new(value)?;
        let other_string = other.to_owned();

        assert_eq!(value == *other, expected, "NonEmptyString == str");
        assert_eq!(*other == value, expected, "str == NonEmptyString");
        assert_eq!(value == other, expected, "NonEmptyString == &str");
        assert_eq!(other == value, expected, "&str == NonEmptyString");
        assert_eq!(value == other_string, expected, "NonEmptyString == String");
        assert_eq!(other_string == value, expected, "String == NonEmptyString");
        Ok(())
    }
}
