//! The standard set of evaluators.

use crate::context::Symbol;
use crate::error::{Error, Result};

/// Returns the absolute value.
pub fn abs(args: Vec<f64>) -> Result<f64> {
	if args.len() != 1 {
		return Err(Error::ArgumentCount(Symbol::ABS));
	}

	Ok(args.into_iter().next().unwrap().abs())
}

/// Rounds its argument towards positive infinity.
pub fn ceil(args: Vec<f64>) -> Result<f64> {
	if args.len() != 1 {
		return Err(Error::ArgumentCount(Symbol::CEIL));
	}

	Ok(args.into_iter().next().unwrap().ceil())
}

/// Rounds its argument towards negative infinity.
pub fn floor(args: Vec<f64>) -> Result<f64> {
	if args.len() != 1 {
		return Err(Error::ArgumentCount(Symbol::FLOOR));
	}

	Ok(args.into_iter().next().unwrap().floor())
}

/// Returns the greatest argument.
pub fn max(args: Vec<f64>) -> Result<f64> {
	if args.is_empty() {
		return Err(Error::ArgumentCount(Symbol::MAX));
	}

	Ok(args.into_iter().max_by(f64::total_cmp).unwrap())
}

/// Returns the smallest argument.
pub fn min(args: Vec<f64>) -> Result<f64> {
	if args.is_empty() {
		return Err(Error::ArgumentCount(Symbol::MIN));
	}

	Ok(args.into_iter().min_by(f64::total_cmp).unwrap())
}

/// Rounds its argument towards the nearest integer.
pub fn round(args: Vec<f64>) -> Result<f64> {
	if args.len() != 1 {
		return Err(Error::ArgumentCount(Symbol::ROUND));
	}

	Ok(args.into_iter().next().unwrap().round())
}

/// Rounds its argument towards zero.
pub fn trunc(args: Vec<f64>) -> Result<f64> {
	if args.len() != 1 {
		return Err(Error::ArgumentCount(Symbol::TRUNC));
	}

	Ok(args.into_iter().next().unwrap().trunc())
}
