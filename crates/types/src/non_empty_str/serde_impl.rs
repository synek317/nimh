use super::NonEmptyStr;
use serde::ser::{Serialize, Serializer};

impl Serialize for NonEmptyStr<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_as_a_plain_string() -> serde_json::Result<()> {
        let value = NonEmptyStr::new(" Ada ").expect("not blank");

        assert_eq!(serde_json::to_string(&value)?, r#""Ada""#);
        Ok(())
    }
}
