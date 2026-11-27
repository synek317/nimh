//! Readable logs in one line of setup for [NiMh](https://github.com/synek317/nimh), also
//! available through `nimh::log`.
//!
//! `NiMh`: [crates.io](https://crates.io/crates/nimh) / [docs.rs](https://docs.rs/nimh) / [github](https://github.com/synek317/nimh).
//!
//! `NiMh` crates, like much of the Rust ecosystem, describe what they are doing through the
//! [`log`](https://docs.rs/log) facade. Those messages go nowhere until the application
//! installs a logger. This crate is that logger, for applications that want readable lines
//! on stderr and nothing to configure:
//!
//! ```no_run
//! use nimh_log::{debug, info};
//!
//! nimh_log::init();
//!
//! info!("listening on port {}", 8080);
//! debug!("this line appears when LOG_LEVEL=debug");
//! ```
//!
//! ```text
//! 2026-11-11T12:41:03.123Z  INFO my_app: listening on port 8080
//! ```
//!
//! A line starts with the time in UTC, so that logs from different machines can be put side
//! by side. Then come the level, the module that wrote the message and the message itself.
//!
//! # Choosing what is logged
//!
//! `init` reads the `LOG_LEVEL` environment variable: `off`, `error`, `warn`, `info`,
//! `debug` or `trace`, in any letter case. Without it, messages at the `info` level and
//! above are logged. `init_with_level` leaves the choice to the application instead.
//!
//! # Colors
//!
//! Levels are colored when stderr is a terminal, and never when it is a file or a pipe.
//! Setting the [`NO_COLOR`](https://no-color.org) environment variable turns colors off
//! everywhere.
//!
//! # Logging macros
//!
//! The macros and level types of the `log` crate are re-exported, so an application needs
//! no dependency on `log` of its own.

use nimh_env::EnvVar;

pub use log::{Level, LevelFilter, debug, error, info, log, log_enabled, trace, warn};

use colors::Colors;
use logger::Logger;

mod civil_date;
mod colors;
mod line;
mod logger;
mod painted;
mod style;
mod timestamp;

/// The environment variable [`init`] reads the level from.
const LOG_LEVEL: EnvVar = EnvVar::new("LOG_LEVEL");

/// The level [`init`] uses when `LOG_LEVEL` does not say otherwise.
const DEFAULT_LEVEL: LevelFilter = LevelFilter::Info;

/// Starts logging to stderr at the level named by the `LOG_LEVEL` environment variable, or
/// at the `info` level when it is not set.
///
/// A `LOG_LEVEL` that names no level does not stop the application: logging starts at the
/// `info` level, and the first line logged is a warning about the variable.
///
/// Only the first call, of this function or of [`init_with_level`], has an effect.
///
/// # Examples
///
/// ```no_run
/// nimh_log::init();
///
/// nimh_log::info!("ready");
/// ```
pub fn init() {
    match LOG_LEVEL.get_opt() {
        Ok(level) => init_with_level(level.unwrap_or(DEFAULT_LEVEL)),
        Err(error) => {
            init_with_level(DEFAULT_LEVEL);
            warn!("{error}; logging at the {DEFAULT_LEVEL} level instead");
        }
    }
}

/// Starts logging to stderr at `level`, whatever the environment says.
///
/// Only the first call, of this function or of [`init`], has an effect. The same is true
/// when the application has already installed another logger for the `log` facade.
///
/// # Examples
///
/// ```no_run
/// use nimh_log::LevelFilter;
///
/// nimh_log::init_with_level(LevelFilter::Debug);
///
/// nimh_log::debug!("ready");
/// ```
pub fn init_with_level(level: LevelFilter) {
    let logger = Logger::new(Colors::detect());
    if log::set_boxed_logger(Box::new(logger)).is_ok() {
        log::set_max_level(level);
    }
}
