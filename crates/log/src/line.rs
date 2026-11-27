use std::fmt;

use log::{Level, Record};

use crate::colors::Colors;
use crate::style::Style;
use crate::timestamp::Timestamp;

/// One log record as it appears on stderr: when, how important, who and what.
///
/// ```text
/// 2026-11-11T12:41:03.123Z  INFO my_app::server: listening on port 8080
/// ```
///
/// The level is the only part in a strong color, because it is what the eye looks for. The
/// time and the module are dimmed, which leaves the message as the most readable part.
pub(crate) struct Line<'a> {
    pub(crate) timestamp: Timestamp,
    pub(crate) record: &'a Record<'a>,
    pub(crate) colors: Colors,
}

impl fmt::Display for Line<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level = self.record.level();
        write!(
            f,
            "{} {} {}: {}",
            self.colors.paint(Style::DIM, self.timestamp),
            self.colors.paint(Style::of(level), label(level)),
            self.colors.paint(Style::DIM, self.record.target()),
            self.record.args(),
        )
    }
}

/// The name of a level, padded so that messages line up below each other.
fn label(level: Level) -> &'static str {
    match level {
        Level::Error => "ERROR",
        Level::Warn => " WARN",
        Level::Info => " INFO",
        Level::Debug => "DEBUG",
        Level::Trace => "TRACE",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use test_case::test_case;

    /// 2026-11-11T12:41:03.123Z
    const TIMESTAMP: Timestamp =
        Timestamp::from_duration_since_epoch(Duration::from_millis(1_794_400_863_123));

    fn line(level: Level, colors: Colors) -> String {
        // The record borrows its arguments, so it has to be used in the statement that
        // creates it.
        Line {
            timestamp: TIMESTAMP,
            record: &Record::builder()
                .level(level)
                .target("my_app::server")
                .args(format_args!("listening on port {}", 8080))
                .build(),
            colors,
        }
        .to_string()
    }

    #[test_case(Level::Error, "ERROR"; "error")]
    #[test_case(Level::Warn, " WARN"; "warn")]
    #[test_case(Level::Info, " INFO"; "info")]
    #[test_case(Level::Debug, "DEBUG"; "debug")]
    #[test_case(Level::Trace, "TRACE"; "trace")]
    fn plain_line_shows_time_level_module_and_message(level: Level, label: &str) {
        assert_eq!(
            line(level, Colors::Off),
            format!("2026-11-11T12:41:03.123Z {label} my_app::server: listening on port 8080")
        );
    }

    #[test]
    fn colored_line_dims_everything_but_the_level_and_the_message() {
        assert_eq!(
            line(Level::Warn, Colors::On),
            "\u{1b}[2m2026-11-11T12:41:03.123Z\u{1b}[0m \
             \u{1b}[33m WARN\u{1b}[0m \
             \u{1b}[2mmy_app::server\u{1b}[0m: \
             listening on port 8080"
        );
    }
}
