use super::NonEmptyString;
use serde::de::{Deserialize, Deserializer, Error as _};
use serde::ser::{Serialize, Serializer};

impl Serialize for NonEmptyString {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for NonEmptyString {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use test_case::test_case;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Person {
        name: NonEmptyString,
    }

    /// The same shape as `Person`, but accepts any string, to produce invalid input.
    #[derive(Serialize)]
    struct RawPerson<'a> {
        name: &'a str,
    }

    fn person_json(name: &str) -> String {
        serde_json::to_string(&RawPerson { name }).expect("a string field always serializes")
    }

    #[test]
    fn serializes_as_a_plain_string() -> serde_json::Result<()> {
        let value = NonEmptyString::new(" abc ").expect("not blank");

        assert_eq!(serde_json::to_string(&value)?, r#""abc""#);
        Ok(())
    }

    #[test_case("abc"; "ASCII")]
    #[test_case("a \t\n b"; "inner whitespace")]
    #[test_case("zażółć gęślą jaźń"; "non-ASCII")]
    #[test_case(r#"say "hi" \o/"#; "quotes and backslashes")]
    #[test_case("\u{200B}"; "zero-width space")]
    fn round_trip_gives_the_same_value(name: &str) -> serde_json::Result<()> {
        let person = Person {
            name: NonEmptyString::new(name).expect("not blank"),
        };

        let json = serde_json::to_string(&person)?;
        let back: Person = serde_json::from_str(&json)?;

        assert_eq!(back, person);
        Ok(())
    }

    #[test]
    fn deserialization_trims() -> serde_json::Result<()> {
        let person: Person = serde_json::from_str(&person_json("\u{3000} Ada \n"))?;

        assert_eq!(person.name, "Ada");
        Ok(())
    }

    #[test_case(""; "empty")]
    #[test_case("   "; "spaces")]
    #[test_case("\t\n\r"; "control whitespace")]
    #[test_case("\u{00A0}"; "no-break space")]
    #[test_case("\u{3000}"; "ideographic space")]
    fn deserialization_rejects_blank_struct_field(name: &str) {
        let err = serde_json::from_str::<Person>(&person_json(name)).unwrap_err();

        let message = err.to_string();
        assert!(
            message.contains("string is empty or contains only whitespace"),
            "{message}"
        );
        assert!(message.contains("line 1 column"), "{message}");
    }

    #[test]
    fn deserialization_rejects_non_strings() {
        let err = serde_json::from_str::<Person>(r#"{"name": 42}"#).unwrap_err();

        assert!(err.to_string().contains("invalid type: integer"), "{err}");
    }
}
