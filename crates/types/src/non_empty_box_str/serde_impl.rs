use super::NonEmptyBoxStr;
use serde::de::{Deserialize, Deserializer, Error as _};
use serde::ser::{Serialize, Serializer};

impl Serialize for NonEmptyBoxStr {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for NonEmptyBoxStr {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_preserves_the_value() -> serde_json::Result<()> {
        let value = NonEmptyBoxStr::new(" Ada ").expect("not blank");

        let json = serde_json::to_string(&value)?;
        let deserialized = serde_json::from_str::<NonEmptyBoxStr>(&json)?;

        assert_eq!(deserialized, value);
        Ok(())
    }

    #[test]
    fn deserialization_rejects_blank_strings() {
        let error = serde_json::from_str::<NonEmptyBoxStr>(r#""  ""#).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("string is empty or contains only whitespace")
        );
    }
}
