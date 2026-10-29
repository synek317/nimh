//! Types for [NiMh](https://github.com/synek317/nimh), also available through `nimh::types`.
//!
//! NiMh: [crates.io](https://crates.io/crates/nimh) / [docs.rs](https://docs.rs/nimh) / [github](https://github.com/synek317/nimh).

pub use non_empty_box_str::NonEmptyBoxStr;
pub use non_empty_str::NonEmptyStr;
pub use non_empty_string::{EmptyStringError, NonEmptyString};

mod non_empty_box_str;
mod non_empty_str;
mod non_empty_string;

/// An owned, immutable string slice.
pub type BoxStr = Box<str>;
