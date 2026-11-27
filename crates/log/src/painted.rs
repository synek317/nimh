use std::fmt;

use crate::style::Style;

/// Text that is displayed in a style, or plainly when it has none.
pub(crate) struct Painted<T> {
    text: T,
    style: Option<Style>,
}

impl<T> Painted<T> {
    pub(crate) fn styled(text: T, style: Style) -> Self {
        Self {
            text,
            style: Some(style),
        }
    }

    pub(crate) fn plain(text: T) -> Self {
        Self { text, style: None }
    }
}

impl<T: fmt::Display> fmt::Display for Painted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.style {
            Some(style) => write!(f, "{}{}{}", style.start(), self.text, Style::END),
            None => self.text.fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styled_text_is_wrapped_in_escape_sequences() {
        let painted = Painted::styled("text", Style::DIM);

        assert_eq!(painted.to_string(), "\u{1b}[2mtext\u{1b}[0m");
    }

    #[test]
    fn plain_text_is_displayed_as_it_is() {
        let painted = Painted::plain(42);

        assert_eq!(painted.to_string(), "42");
    }
}
