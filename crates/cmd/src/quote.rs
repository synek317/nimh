use std::borrow::Cow;
use std::ffi::OsStr;

/// Characters, besides letters and digits, that no common shell treats specially.
const PLAIN_PUNCTUATION: &str = "-_./:=@%+,";

/// Writes one argument the way it would be typed into a shell.
///
/// Most arguments are plain words and are returned as they are. Anything a shell would
/// split or interpret is wrapped in single quotes, inside which only the single quote
/// itself needs care: it is written as `'\''`.
///
/// The result is meant for people, in errors and logs. Bytes that are not valid Unicode
/// are replaced, so it must not be used to build a command that is actually executed.
pub(crate) fn quote(argument: &OsStr) -> Cow<'_, str> {
    let text = argument.to_string_lossy();
    if is_plain_word(&text) {
        text
    } else {
        Cow::Owned(format!("'{}'", text.replace('\'', r"'\''")))
    }
}

fn is_plain_word(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|character| character.is_alphanumeric() || PLAIN_PUNCTUATION.contains(character))
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("status"; "word")]
    #[test_case("--release"; "long option")]
    #[test_case("--jobs=4"; "option with a value")]
    #[test_case("src/main.rs"; "path")]
    #[test_case("user@example.com:repo.git"; "remote")]
    #[test_case("zażółć"; "non-ASCII letters")]
    fn plain_words_are_left_as_they_are(argument: &str) {
        assert_eq!(quote(OsStr::new(argument)), argument);
    }

    #[test_case("", "''"; "empty")]
    #[test_case("Fix the build", "'Fix the build'"; "spaces")]
    #[test_case("*.rs", "'*.rs'"; "glob")]
    #[test_case("$HOME", "'$HOME'"; "variable")]
    #[test_case("a;b", "'a;b'"; "command separator")]
    #[test_case("say \"hi\"", "'say \"hi\"'"; "double quotes")]
    #[test_case("it's", r"'it'\''s'"; "single quote")]
    #[test_case("line\nbreak", "'line\nbreak'"; "line break")]
    fn everything_else_is_wrapped_in_single_quotes(argument: &str, expected: &str) {
        assert_eq!(quote(OsStr::new(argument)), expected);
    }
}
