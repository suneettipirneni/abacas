//! The context structure and its related items.

pub mod evaluators;
pub mod simplifiers;

use std::borrow::Cow;
use std::collections::HashMap;
use std::f64::consts;
use std::fmt;

use crate::error::Result;
use crate::expr::Expr;

/// The type of an evaluator function.
pub type Evaluator = fn(args: Vec<f64>) -> Result<f64>;

/// The type of a simplifier function.
pub type Simplifier = fn(args: Vec<Expr>) -> Result<Expr>;

/// The global data of a session, including functions and variables.
#[derive(Clone, Debug, Default)]
pub struct Context {
	/// The constants declared in this context.
	pub constants: HashMap<Symbol, f64>,
	/// The evaluators declared in this context.
	pub evaluators: HashMap<Symbol, Evaluator>,
	/// The functions declared in this context.
	pub functions: HashMap<Symbol, Function>,
	/// The simplifiers declared in this context.
	pub simplifiers: HashMap<Symbol, Simplifier>,
	/// The variables stored in this context.
	pub variables: HashMap<Symbol, Expr>,
}

impl Context {
	/// Creates a new, empty context. You usually want [`Self::new`] instead.
	pub fn empty() -> Self {
		Self::default()
	}

	/// Creates a new context with predefined constants, evaluators and simplifiers.
	pub fn new() -> Self {
		let mut ctx = Self::empty();

		ctx.constants.insert(Symbol::E, consts::E);
		ctx.constants.insert(Symbol::PI, consts::PI);

		ctx.evaluators.insert(Symbol::ABS, evaluators::abs);
		ctx.evaluators.insert(Symbol::CEIL, evaluators::ceil);
		ctx.evaluators.insert(Symbol::FLOOR, evaluators::floor);
		ctx.evaluators.insert(Symbol::MAX, evaluators::max);
		ctx.evaluators.insert(Symbol::MIN, evaluators::min);
		ctx.evaluators.insert(Symbol::ROUND, evaluators::round);
		ctx.evaluators.insert(Symbol::TRUNC, evaluators::trunc);

		ctx.simplifiers.insert(Symbol::ABS, simplifiers::abs);
		ctx.simplifiers.insert(Symbol::CEIL, simplifiers::ceil);
		ctx.simplifiers.insert(Symbol::FLOOR, simplifiers::floor);
		ctx.simplifiers.insert(Symbol::MAX, simplifiers::max);
		ctx.simplifiers.insert(Symbol::MIN, simplifiers::min);
		ctx.simplifiers.insert(Symbol::ROUND, simplifiers::round);
		ctx.simplifiers.insert(Symbol::TRUNC, simplifiers::trunc);

		ctx
	}
}

/// Represents a user-defined function that can be inserted during simplification.
#[derive(Clone, Debug)]
pub struct Function {
	/// The function body that gets inserted when calling the function.
	pub body: Expr,
	/// The name of the function.
	pub name: Symbol,
	/// The parameters required by this function.
	pub params: Vec<Symbol>,
}

impl Function {
	/// Creates a new function.
	pub fn new(body: Expr, name: Symbol, params: Vec<Symbol>) -> Self {
		Self { body, name, params }
	}
}

/// Represents a symbol that can be used as an identifier for functions and variables.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Symbol(Cow<'static, str>);

impl Symbol {
	/// The symbol `abs`.
	pub const ABS: Self = Self(Cow::Borrowed("abs"));

	/// The symbol `ceil`.
	pub const CEIL: Self = Self(Cow::Borrowed("ceil"));

	/// The symbol `e`.
	pub const E: Self = Self(Cow::Borrowed("e"));

	/// The symbol `floor`.
	pub const FLOOR: Self = Self(Cow::Borrowed("floor"));

	/// The symbol `max`.
	pub const MAX: Self = Self(Cow::Borrowed("max"));

	/// The symbol `min`.
	pub const MIN: Self = Self(Cow::Borrowed("min"));

	/// The symbol `pi`.
	pub const PI: Self = Self(Cow::Borrowed("pi"));

	/// The symbol `round`.
	pub const ROUND: Self = Self(Cow::Borrowed("round"));

	/// The symbol `trunc`.
	pub const TRUNC: Self = Self(Cow::Borrowed("trunc"));

	/// Gets the name of this symbol.
	pub fn name(&self) -> &str {
		&self.0
	}

	/// Creates a new symbol with the given name. Symbols must not be empty and contain no whitespace.
	pub fn new(name: String) -> Option<Self> {
		if name.is_empty() || name.chars().any(char::is_whitespace) {
			None
		} else {
			Some(Self(name.into()))
		}
	}
}

impl fmt::Display for Symbol {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}
