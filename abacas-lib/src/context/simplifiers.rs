//! The standard set of simplifiers.

use crate::context::Symbol;
use crate::error::{Error, Result};
use crate::expr::Expr;

/// Returns the absolute value.
pub fn abs(args: Vec<Expr>) -> Result<Expr> {
	if args.len() != 1 {
		return Err(Error::ArgumentCount(Symbol::ABS));
	}

	if args.first().unwrap().is_num() {
		Ok(Expr::Num(args.into_iter().next().unwrap().into_num().unwrap().abs()))
	} else {
		Ok(Expr::Fun(Symbol::ABS, args))
	}
}

/// Rounds its argument towards positive infinity.
pub fn ceil(args: Vec<Expr>) -> Result<Expr> {
	if args.len() != 1 {
		return Err(Error::ArgumentCount(Symbol::CEIL));
	}

	if args.first().unwrap().is_num() {
		Ok(Expr::Num(args.into_iter().next().unwrap().into_num().unwrap().ceil()))
	} else {
		Ok(Expr::Fun(Symbol::CEIL, args))
	}
}

/// Rounds its argument towards negative infinity.
pub fn floor(args: Vec<Expr>) -> Result<Expr> {
	if args.len() != 1 {
		return Err(Error::ArgumentCount(Symbol::FLOOR));
	}

	if args.first().unwrap().is_num() {
		Ok(Expr::Num(args.into_iter().next().unwrap().into_num().unwrap().floor()))
	} else {
		Ok(Expr::Fun(Symbol::FLOOR, args))
	}
}

/// Returns the greatest argument.
pub fn max(mut args: Vec<Expr>) -> Result<Expr> {
	if args.is_empty() {
		return Err(Error::ArgumentCount(Symbol::MAX));
	}

	let maximum = args
		.extract_if(.., |arg| arg.is_num())
		.map(|arg| arg.into_num().unwrap())
		.max();

	if let Some(num) = maximum {
		args.push(Expr::Num(num));
	}

	if args.len() == 1 {
		Ok(args.into_iter().next().unwrap())
	} else {
		Ok(Expr::Fun(Symbol::MAX, args))
	}
}

/// Returns the smallest argument.
pub fn min(mut args: Vec<Expr>) -> Result<Expr> {
	if args.is_empty() {
		return Err(Error::ArgumentCount(Symbol::MIN));
	}

	let minimum = args
		.extract_if(.., |arg| arg.is_num())
		.map(|arg| arg.into_num().unwrap())
		.min();

	if let Some(num) = minimum {
		args.push(Expr::Num(num));
	}

	if args.len() == 1 {
		Ok(args.into_iter().next().unwrap())
	} else {
		Ok(Expr::Fun(Symbol::MIN, args))
	}
}

/// Rounds its argument towards the nearest integer.
pub fn round(args: Vec<Expr>) -> Result<Expr> {
	if args.len() != 1 {
		return Err(Error::ArgumentCount(Symbol::ROUND));
	}

	if args.first().unwrap().is_num() {
		Ok(Expr::Num(args.into_iter().next().unwrap().into_num().unwrap().round()))
	} else {
		Ok(Expr::Fun(Symbol::ROUND, args))
	}
}

/// Rounds its argument towards zero.
pub fn trunc(args: Vec<Expr>) -> Result<Expr> {
	if args.len() != 1 {
		return Err(Error::ArgumentCount(Symbol::TRUNC));
	}

	if args.first().unwrap().is_num() {
		Ok(Expr::Num(args.into_iter().next().unwrap().into_num().unwrap().trunc()))
	} else {
		Ok(Expr::Fun(Symbol::TRUNC, args))
	}
}
