use std::fmt;

/// How many lines of stderr an error message shows.
const SHOWN_LINES: usize = 20;

/// The end of what a command printed to stderr, indented to sit below an error message.
///
/// A failing build can print thousands of lines, and an error message that long is no
/// longer a message. The reason for a failure is nearly always at the end, so the last
/// lines are shown and the rest is counted.
pub(crate) struct StderrExcerpt<'a>(pub(crate) &'a str);

impl fmt::Display for StderrExcerpt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let omitted = self.0.lines().count().saturating_sub(SHOWN_LINES);
        if omitted > 0 {
            write!(f, "\n  ... ({omitted} earlier lines omitted)")?;
        }
        self.0
            .lines()
            .skip(omitted)
            .try_for_each(|line| write!(f, "\n  {line}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn numbered_lines(count: usize) -> String {
        (1..=count)
            .map(|number| format!("line {number}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn nothing_is_shown_for_empty_stderr() {
        assert_eq!(StderrExcerpt("").to_string(), "");
    }

    #[test]
    fn every_line_is_indented() {
        let excerpt = StderrExcerpt("fatal: boom\n  hint: try again");

        assert_eq!(excerpt.to_string(), "\n  fatal: boom\n    hint: try again");
    }

    #[test]
    fn short_stderr_is_shown_in_full() {
        let stderr = numbered_lines(SHOWN_LINES);

        let excerpt = StderrExcerpt(&stderr).to_string();

        assert_eq!(excerpt.lines().count(), SHOWN_LINES + 1);
        assert!(excerpt.starts_with("\n  line 1\n"), "{excerpt}");
        assert!(!excerpt.contains("omitted"), "{excerpt}");
    }

    #[test]
    fn long_stderr_is_cut_down_to_its_end() {
        let stderr = numbered_lines(SHOWN_LINES + 5);

        let excerpt = StderrExcerpt(&stderr).to_string();

        assert!(
            excerpt.starts_with("\n  ... (5 earlier lines omitted)\n  line 6\n"),
            "{excerpt}"
        );
        assert!(excerpt.ends_with("\n  line 25"), "{excerpt}");
    }
}
