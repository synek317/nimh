//! Readable error reports for [NiMh](https://github.com/synek317/nimh), also available through
//! `nimh::error`.
//!
//! `NiMh`: [crates.io](https://crates.io/crates/nimh) / [docs.rs](https://docs.rs/nimh) / [github](https://github.com/synek317/nimh).
//!
//! An error rarely tells the whole story on its own. *"No such file or directory"* is how the
//! story ends; how it began (which file, and what the application wanted with it) is kept in
//! the errors that wrap it, one `source()` after another. Printing an error with `{}` shows
//! only the outermost one, and returning it from `main` shows its `Debug` form. This crate
//! prints the whole story instead.
//!
//! # An error with its causes
//!
//! `DisplayChain` adds `display_chain()` to every error. `{}` puts the chain on one line,
//! which suits logs. `{:#}` gives every cause its own line, which suits a terminal.
//!
//! ```
//! use nimh_error::DisplayChain;
//! # use std::error::Error;
//! # use std::fmt;
//! # use std::num::ParseIntError;
//! #
//! # #[derive(Debug)]
//! # struct ConfigError(ParseIntError);
//! #
//! # impl fmt::Display for ConfigError {
//! #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//! #         f.write_str("cannot load the configuration")
//! #     }
//! # }
//! #
//! # impl Error for ConfigError {
//! #     fn source(&self) -> Option<&(dyn Error + 'static)> {
//! #         Some(&self.0)
//! #     }
//! # }
//! #
//! # fn load_config() -> Result<u16, ConfigError> {
//! #     "http".parse().map_err(ConfigError)
//! # }
//!
//! // `load_config` fails with a `ConfigError` whose source is a `ParseIntError`.
//! let error = load_config().unwrap_err();
//!
//! assert_eq!(
//!     error.display_chain().to_string(),
//!     "cannot load the configuration: invalid digit found in string",
//! );
//! assert_eq!(
//!     format!("{:#}", error.display_chain()),
//!     "cannot load the configuration\n  caused by: invalid digit found in string",
//! );
//! ```
//!
//! # Reporting from `main`
//!
//! `report` turns the result of the whole program into an exit code and, on failure, prints
//! the error with its causes to stderr.
//!
//! ```no_run
//! use std::error::Error;
//! use std::process::ExitCode;
//!
//! fn main() -> ExitCode {
//!     nimh_error::report(run())
//! }
//!
//! fn run() -> Result<(), Box<dyn Error>> {
//!     let port: u16 = "http".parse()?;
//!     println!("listening on port {port}");
//!     Ok(())
//! }
//! ```
//!
//! ```text
//! error: invalid digit found in string
//! ```

pub use display_chain::DisplayChain;
pub use error_chain::ErrorChain;
pub use report::report;
pub use reportable::{BoxedSend, BoxedSendSync, Convertible, Reportable};

mod display_chain;
mod error_chain;
#[cfg(test)]
mod layered_error;
mod report;
mod reportable;
