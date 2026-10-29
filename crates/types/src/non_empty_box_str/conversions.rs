use super::NonEmptyBoxStr;
use crate::{BoxStr, EmptyStringError, NonEmptyStr, NonEmptyString};
use std::str::FromStr;

impl TryFrom<BoxStr> for NonEmptyBoxStr {
    type Error = EmptyStringError;

    fn try_from(value: BoxStr) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for NonEmptyBoxStr {
    type Error = EmptyStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value.into_boxed_str())
    }
}

impl TryFrom<&str> for NonEmptyBoxStr {
    type Error = EmptyStringError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl FromStr for NonEmptyBoxStr {
    type Err = EmptyStringError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl From<NonEmptyString> for NonEmptyBoxStr {
    fn from(value: NonEmptyString) -> Self {
        Self(value.into_string().into_boxed_str())
    }
}

impl From<&NonEmptyString> for NonEmptyBoxStr {
    fn from(value: &NonEmptyString) -> Self {
        Self(BoxStr::from(value.as_str()))
    }
}

impl From<NonEmptyStr<'_>> for NonEmptyBoxStr {
    fn from(value: NonEmptyStr<'_>) -> Self {
        Self(BoxStr::from(value.as_str()))
    }
}

impl From<NonEmptyBoxStr> for BoxStr {
    fn from(value: NonEmptyBoxStr) -> Self {
        value.into_boxed_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_from_non_empty_string_without_validation() -> Result<(), EmptyStringError> {
        let source = NonEmptyString::new("Ada")?;

        assert_eq!(NonEmptyBoxStr::from(source).as_str(), "Ada");
        Ok(())
    }

    #[test]
    fn converts_from_non_empty_str_without_validation() -> Result<(), EmptyStringError> {
        let source = NonEmptyStr::new("Ada")?;

        assert_eq!(NonEmptyBoxStr::from(source).as_str(), "Ada");
        Ok(())
    }
}
