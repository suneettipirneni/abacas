//! The error type and all related types.

use std::{error, fmt, result};

use crate::context::Symbol;
use crate::number::Number;

/// The error type used across the library.
#[derive(Debug)]
pub enum Error {
	/// The function received an invalid argument count.
	ArgumentCount(Symbol),
	/// The expression tried to divide by zero.
	DivisionByZero,
	/// The parser encountered an invalid number.
	InvalidNumber(Number),
	/// The parser encountered an invalid string.
	InvalidString(String),
	/// The provided value is not declared in the current context.
	UndeclaredValue(Symbol),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::ArgumentCount(name) => write!(f, "invalid argument count: {name}"),
			Self::DivisionByZero => write!(f, "division by zero"),
			Self::InvalidNumber(number) => write!(f, "invalid number: {number}"),
			Self::InvalidString(string) => write!(f, "invalid string: {string}"),
			Self::UndeclaredValue(name) => write!(f, "undeclared value: {name}"),
		}
	}
}

impl error::Error for Error {}

/// The standard result type, but with the error set to [`Error`].
pub type Result<T> = result::Result<T, Error>;
