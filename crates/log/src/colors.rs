use std::env;
use std::ffi::OsStr;
use std::fmt;
use std::io::{self, IsTerminal};

use crate::painted::Painted;
use crate::style::Style;

/// Whether log lines are colored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Colors {
    On,
    Off,
}

impl Colors {
    /// Decides whether colors would help whoever reads stderr.
    pub(crate) fn detect() -> Self {
        Self::choose(
            io::stderr().is_terminal(),
            env::var_os("NO_COLOR").as_deref(),
        )
    }

    /// Colors are escape sequences, which a terminal turns into colors and a file keeps as
    /// noise. And whoever sets `NO_COLOR` to anything but an empty value has asked for none,
    /// as described at <https://no-color.org>.
    fn choose(stderr_is_terminal: bool, no_color: Option<&OsStr>) -> Self {
        let colors_are_unwanted = no_color.is_some_and(|value| !value.is_empty());
        if stderr_is_terminal && !colors_are_unwanted {
            Self::On
        } else {
            Self::Off
        }
    }

    /// Gives `text` the given style, or leaves it plain when colors are off.
    pub(crate) fn paint<T: fmt::Display>(self, style: Style, text: T) -> Painted<T> {
        match self {
            Self::On => Painted::styled(text, style),
            Self::Off => Painted::plain(text),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(true, None, Colors::On; "terminal")]
    #[test_case(true, Some(""), Colors::On; "terminal with empty NO_COLOR")]
    #[test_case(true, Some("1"), Colors::Off; "terminal with NO_COLOR")]
    #[test_case(false, None, Colors::Off; "file or pipe")]
    #[test_case(false, Some("1"), Colors::Off; "file or pipe with NO_COLOR")]
    fn colors_are_on_only_where_they_help(
        stderr_is_terminal: bool,
        no_color: Option<&str>,
        expected: Colors,
    ) {
        let colors = Colors::choose(stderr_is_terminal, no_color.map(OsStr::new));

        assert_eq!(colors, expected);
    }

    #[test]
    fn paint_applies_the_style_only_when_colors_are_on() {
        assert_eq!(
            Colors::On.paint(Style::DIM, "text").to_string(),
            "\u{1b}[2mtext\u{1b}[0m"
        );
        assert_eq!(Colors::Off.paint(Style::DIM, "text").to_string(), "text");
    }
}
