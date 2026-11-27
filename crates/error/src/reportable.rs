use std::error::Error;

/// An error in any of the shapes a program can end with, all accepted by
/// [`report`](crate::report).
///
/// It is implemented for:
///
/// - everything that converts into `Box<dyn Error>`: types that implement [`Error`],
///   `Box<dyn Error>` itself, `anyhow::Error`, `String` and `&str`;
/// - `Box<dyn Error + Send + Sync>` and `Box<dyn Error + Send>`, which the standard library
///   offers no such conversion for.
///
/// `Shape` exists only to let the compiler tell these implementations apart. It is always
/// inferred and never has to be written.
pub trait Reportable<Shape> {
    /// Converts the error into the one shape that is enough to print it.
    fn into_boxed_error(self) -> Box<dyn Error>;
}

/// The shape of everything that converts into `Box<dyn Error>` on its own.
#[derive(Debug)]
pub enum Convertible {}

/// The shape of boxed errors that can also be sent to another thread.
#[derive(Debug)]
pub enum BoxedSend {}

/// The shape of boxed errors that can also be sent to and shared with another thread.
#[derive(Debug)]
pub enum BoxedSendSync {}

impl<E: Into<Box<dyn Error>>> Reportable<Convertible> for E {
    fn into_boxed_error(self) -> Box<dyn Error> {
        self.into()
    }
}

impl Reportable<BoxedSend> for Box<dyn Error + Send> {
    fn into_boxed_error(self) -> Box<dyn Error> {
        self
    }
}

impl Reportable<BoxedSendSync> for Box<dyn Error + Send + Sync> {
    fn into_boxed_error(self) -> Box<dyn Error> {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layered_error::LayeredError;

    /// Lets the compiler infer the shape, exactly as it has to in `report`.
    fn message<Shape>(error: impl Reportable<Shape>) -> String {
        error.into_boxed_error().to_string()
    }

    #[test]
    fn accepts_error_types() {
        assert_eq!(message(LayeredError::chain(&["concrete"])), "concrete");
    }

    #[test]
    fn accepts_strings() {
        assert_eq!(message("borrowed"), "borrowed");
        assert_eq!(message(String::from("owned")), "owned");
    }

    #[test]
    fn accepts_boxed_errors_with_any_thread_safety() {
        let plain: Box<dyn Error> = "plain".into();
        let send: Box<dyn Error + Send> = Box::new(LayeredError::chain(&["send"]));
        let send_sync: Box<dyn Error + Send + Sync> = "send and sync".into();

        assert_eq!(message(plain), "plain");
        assert_eq!(message(send), "send");
        assert_eq!(message(send_sync), "send and sync");
    }

    #[test]
    fn keeps_the_causes() {
        let error: Box<dyn Error + Send + Sync> = Box::new(LayeredError::chain(&["a", "b"]));

        let error = error.into_boxed_error();

        assert_eq!(
            error.source().map(ToString::to_string).as_deref(),
            Some("b")
        );
    }
}
