//! Turning the bytes a command printed into text.

/// Decodes what a command printed to stdout and drops the line break at its end.
///
/// Nearly every program ends its output with a line break, and nearly every caller would
/// have to remove it. Everything else is kept exactly, including leading whitespace.
///
/// Returns `None` when the output is not valid UTF-8. Replacing the invalid bytes would hand
/// the caller a value that silently differs from what the command produced.
pub(crate) fn text(stdout: Vec<u8>) -> Option<String> {
    let mut text = String::from_utf8(stdout).ok()?;
    text.truncate(text.trim_end_matches(['\n', '\r']).len());
    Some(text)
}

/// Decodes what a command printed to stderr.
///
/// This text only ever ends up in errors and logs, where being readable matters more than
/// being exact: bytes that are not valid UTF-8 are replaced and surrounding whitespace is
/// dropped.
pub(crate) fn diagnostics(stderr: &[u8]) -> String {
    String::from_utf8_lossy(stderr).trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("", ""; "nothing")]
    #[test_case("hello", "hello"; "no line break")]
    #[test_case("hello\n", "hello"; "line break")]
    #[test_case("hello\r\n", "hello"; "Windows line break")]
    #[test_case("hello\n\n\n", "hello"; "several line breaks")]
    #[test_case("  one\ntwo  \n", "  one\ntwo  "; "other whitespace is kept")]
    #[test_case("zażółć\n", "zażółć"; "non-ASCII")]
    fn text_drops_only_the_line_break_at_the_end(stdout: &str, expected: &str) {
        assert_eq!(text(stdout.as_bytes().to_vec()).as_deref(), Some(expected));
    }

    #[test]
    fn text_rejects_invalid_utf_8() {
        assert_eq!(text(vec![b'o', b'k', 0xFF]), None);
    }

    #[test_case(b"", ""; "nothing")]
    #[test_case(b"\n  fatal: boom\n\n", "fatal: boom"; "surrounding whitespace is dropped")]
    #[test_case(b"one\n  two\n", "one\n  two"; "inner layout is kept")]
    #[test_case(b"bad \xFF byte", "bad \u{FFFD} byte"; "invalid UTF-8 is replaced")]
    fn diagnostics_are_made_readable(stderr: &[u8], expected: &str) {
        assert_eq!(diagnostics(stderr), expected);
    }
}
