use std::fmt;

use log::Level;

/// A way to render text in a terminal, kept as the parameters of an ANSI escape sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Style(&'static str);

impl Style {
    /// For the parts of a line that should stay out of the way.
    pub(crate) const DIM: Self = Self("2");

    /// The escape sequence that switches every style off again.
    pub(crate) const END: &'static str = "\u{1b}[0m";

    /// The color of a level: the more urgent the level, the louder the color.
    pub(crate) fn of(level: Level) -> Self {
        match level {
            Level::Error => Self("1;31"),
            Level::Warn => Self("33"),
            Level::Info => Self("32"),
            Level::Debug => Self("34"),
            Level::Trace => Self("35"),
        }
    }

    /// The escape sequence that switches this style on.
    pub(crate) fn start(self) -> impl fmt::Display {
        StartOf(self)
    }
}

struct StartOf(Style);

impl fmt::Display for StartOf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\u{1b}[{}m", self.0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_is_an_escape_sequence_with_the_parameters_of_the_style() {
        assert_eq!(Style::DIM.start().to_string(), "\u{1b}[2m");
        assert_eq!(Style::of(Level::Error).start().to_string(), "\u{1b}[1;31m");
    }

    #[test]
    fn every_level_has_its_own_color() {
        let styles = Level::iter().map(Style::of).collect::<Vec<_>>();

        for (index, style) in styles.iter().enumerate() {
            assert!(!styles[index + 1..].contains(style), "{style:?}");
        }
    }
}
