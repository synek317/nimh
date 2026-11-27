//! Opinionated, reusable helpers for Rust applications. Batteries included. `NiMh` helps you
//! build applications faster and keep their code simple, with consistent APIs that are easy to
//! use correctly.
//!
//! Memory and performance still matter, but avoiding an allocation or saving a few CPU
//! cycles shouldn't make application code harder to write or understand.
//!
//! # A small application
//!
//! ```no_run
//! use nimh::log::info;
//! use nimh::prelude::*;
//! use std::error::Error;
//! use std::process::ExitCode;
//!
//! const RELEASES: EnvVar = EnvVar::new("RELEASES_DIR");
//!
//! fn main() -> ExitCode {
//!     nimh::log::init();
//!     nimh::error::report(deploy())
//! }
//!
//! fn deploy() -> Result<(), Box<dyn Error>> {
//!     let releases: String = RELEASES.get_or(String::from("/srv/releases"))?;
//!     let commit = Cmd::new("git").args(["rev-parse", "--short", "HEAD"]).read()?;
//!     let release = format!("{releases}/{commit}");
//!
//!     info!("deploying {commit} to {release}");
//!     Cmd::new("cargo").args(["build", "--release"]).run()?;
//!     nimh::fs::cp_r("target/release", &release)?;
//!     nimh::fs::ln_sfn(&release, format!("{releases}/current"))?;
//!     Ok(())
//! }
//! ```
//!
//! # Crates
//!
//! | Crate | Module | What it is for | Links |
//! | --- | --- | --- | --- |
//! | `nimh` | | everything below in one dependency | [crates.io](https://crates.io/crates/nimh) / [docs.rs](https://docs.rs/nimh) / [github](https://github.com/synek317/nimh) |
//! | `nimh_cmd` | `nimh::cmd` | running external commands | [crates.io](https://crates.io/crates/nimh_cmd) / [docs.rs](https://docs.rs/nimh_cmd) / [github](https://github.com/synek317/nimh/tree/HEAD/crates/cmd) |
//! | `nimh_env` | `nimh::env` | typed environment variables | [crates.io](https://crates.io/crates/nimh_env) / [docs.rs](https://docs.rs/nimh_env) / [github](https://github.com/synek317/nimh/tree/HEAD/crates/env) |
//! | `nimh_error` | `nimh::error` | errors shown with their causes | [crates.io](https://crates.io/crates/nimh_error) / [docs.rs](https://docs.rs/nimh_error) / [github](https://github.com/synek317/nimh/tree/HEAD/crates/error) |
//! | `nimh_fs` | `nimh::fs` | file system helpers modelled after Unix commands | [crates.io](https://crates.io/crates/nimh_fs) / [docs.rs](https://docs.rs/nimh_fs) / [github](https://github.com/synek317/nimh/tree/HEAD/crates/fs) |
//! | `nimh_log` | `nimh::log` | readable logs in one line of setup | [crates.io](https://crates.io/crates/nimh_log) / [docs.rs](https://docs.rs/nimh_log) / [github](https://github.com/synek317/nimh/tree/HEAD/crates/log) |
//! | `nimh_types` | `nimh::types` | types with validated invariants | [crates.io](https://crates.io/crates/nimh_types) / [docs.rs](https://docs.rs/nimh_types) / [github](https://github.com/synek317/nimh/tree/HEAD/crates/types) |

pub use nimh_cmd as cmd;
pub use nimh_env as env;
pub use nimh_error as error;
pub use nimh_fs as fs;
pub use nimh_log as log;
pub use nimh_types as types;

pub mod prelude;
