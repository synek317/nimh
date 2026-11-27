use std::error::Error;

use crate::ErrorChain;

/// Adds [`display_chain`](Self::display_chain) to every error.
///
/// It is implemented for all types that implement [`Error`], and it also works on boxed
/// errors such as `Box<dyn Error>`.
pub trait DisplayChain: Error {
    /// Shows this error together with everything that caused it.
    ///
    /// `{}` puts the chain on one line and `{:#}` gives every cause its own line. See
    /// [`ErrorChain`] for the details.
    ///
    /// # Examples
    ///
    /// ```
    /// use nimh_error::DisplayChain;
    /// use std::error::Error;
    ///
    /// let error: Box<dyn Error> = "http".parse::<u16>().unwrap_err().into();
    ///
    /// assert_eq!(
    ///     format!("cannot start: {}", error.display_chain()),
    ///     "cannot start: invalid digit found in string",
    /// );
    /// ```
    #[must_use]
    fn display_chain(&self) -> ErrorChain<'_, Self> {
        ErrorChain::new(self)
    }
}

impl<E: Error + ?Sized> DisplayChain for E {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layered_error::LayeredError;

    #[test]
    fn works_on_concrete_errors() {
        let error = LayeredError::chain(&["a", "b"]);

        assert_eq!(error.display_chain().to_string(), "a: b");
    }

    #[test]
    fn works_on_boxed_errors() {
        let error: Box<dyn Error + Send + Sync> = Box::new(LayeredError::chain(&["a", "b"]));

        assert_eq!(error.display_chain().to_string(), "a: b");
    }

    #[test]
    fn works_on_error_references() {
        let error = LayeredError::chain(&["a", "b"]);
        let error: &dyn Error = &error;

        assert_eq!(format!("{:#}", error.display_chain()), "a\n  caused by: b");
    }
}
