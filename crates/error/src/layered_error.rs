//! A test fixture: an error with any number of causes behind it.

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub(crate) struct LayeredError {
    message: &'static str,
    source: Option<Box<LayeredError>>,
}

impl LayeredError {
    /// Builds an error from its messages, starting with the outermost one and ending with
    /// the root cause.
    pub(crate) fn chain(messages: &[&'static str]) -> Self {
        let (message, causes) = messages
            .split_first()
            .expect("an error needs at least one message");
        Self {
            message,
            source: (!causes.is_empty()).then(|| Box::new(Self::chain(causes))),
        }
    }
}

impl fmt::Display for LayeredError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message)
    }
}

impl Error for LayeredError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref().map(|source| source as &dyn Error)
    }
}
