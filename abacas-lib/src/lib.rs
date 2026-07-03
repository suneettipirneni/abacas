#![doc = include_str!("../../README.md")]
#![warn(missing_docs)]

pub mod context;
pub mod error;
pub mod expr;
pub mod monomial;
pub mod number;
pub mod polynomial;

/// The library version currently in use.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
