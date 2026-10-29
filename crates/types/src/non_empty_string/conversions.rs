use super::NonEmptyString;
use crate::{BoxStr, EmptyStringError, NonEmptyBoxStr, NonEmptyStr};

impl TryFrom<BoxStr> for NonEmptyString {
    type Error = EmptyStringError;

    fn try_from(value: BoxStr) -> Result<Self, Self::Error> {
        Self::new(String::from(value))
    }
}

impl From<NonEmptyBoxStr> for NonEmptyString {
    fn from(value: NonEmptyBoxStr) -> Self {
        Self(String::from(value.into_boxed_str()))
    }
}

impl From<&NonEmptyBoxStr> for NonEmptyString {
    fn from(value: &NonEmptyBoxStr) -> Self {
        Self(value.as_str().to_owned())
    }
}

impl From<NonEmptyStr<'_>> for NonEmptyString {
    fn from(value: NonEmptyStr<'_>) -> Self {
        Self(value.as_str().to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_from_non_empty_box_str_without_validation() -> Result<(), EmptyStringError> {
        let source = NonEmptyBoxStr::new("Ada")?;

        assert_eq!(NonEmptyString::from(source).as_str(), "Ada");
        Ok(())
    }

    #[test]
    fn converts_from_non_empty_str_without_validation() -> Result<(), EmptyStringError> {
        let source = NonEmptyStr::new("Ada")?;

        assert_eq!(NonEmptyString::from(source).as_str(), "Ada");
        Ok(())
    }
}
