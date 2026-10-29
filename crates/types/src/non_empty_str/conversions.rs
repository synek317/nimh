use super::NonEmptyStr;
use crate::{EmptyStringError, NonEmptyBoxStr, NonEmptyString};

impl<'a> TryFrom<&'a str> for NonEmptyStr<'a> {
    type Error = EmptyStringError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<'a> From<&'a NonEmptyString> for NonEmptyStr<'a> {
    fn from(value: &'a NonEmptyString) -> Self {
        Self(value.as_str())
    }
}

impl<'a> From<&'a NonEmptyBoxStr> for NonEmptyStr<'a> {
    fn from(value: &'a NonEmptyBoxStr) -> Self {
        Self(value.as_str())
    }
}

impl<'a> From<NonEmptyStr<'a>> for &'a str {
    fn from(value: NonEmptyStr<'a>) -> Self {
        value.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BoxStr;

    #[test]
    fn borrows_from_non_empty_string() -> Result<(), EmptyStringError> {
        let owned = NonEmptyString::new("Ada")?;

        assert_eq!(NonEmptyStr::from(&owned).as_str(), "Ada");
        Ok(())
    }

    #[test]
    fn borrows_from_non_empty_box_str() -> Result<(), EmptyStringError> {
        let owned = NonEmptyBoxStr::new(BoxStr::from("Ada"))?;

        assert_eq!(NonEmptyStr::from(&owned).as_str(), "Ada");
        Ok(())
    }
}
