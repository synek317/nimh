use super::NonEmptyBoxStr;

impl PartialEq<str> for NonEmptyBoxStr {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for NonEmptyBoxStr {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for NonEmptyBoxStr {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<NonEmptyBoxStr> for str {
    fn eq(&self, other: &NonEmptyBoxStr) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<NonEmptyBoxStr> for &str {
    fn eq(&self, other: &NonEmptyBoxStr) -> bool {
        *self == other.as_str()
    }
}

impl PartialEq<NonEmptyBoxStr> for String {
    fn eq(&self, other: &NonEmptyBoxStr) -> bool {
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
        let value = NonEmptyBoxStr::new(value)?;
        let other_string = other.to_owned();

        assert_eq!(value == *other, expected, "NonEmptyBoxStr == str");
        assert_eq!(*other == value, expected, "str == NonEmptyBoxStr");
        assert_eq!(value == other, expected, "NonEmptyBoxStr == &str");
        assert_eq!(other == value, expected, "&str == NonEmptyBoxStr");
        assert_eq!(value == other_string, expected, "NonEmptyBoxStr == String");
        assert_eq!(other_string == value, expected, "String == NonEmptyBoxStr");
        Ok(())
    }
}
