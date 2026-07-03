//! The expression structure and its related items.

use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

use itertools::Itertools;
use rug::ops::Pow;

use crate::context::{Context, Symbol};
use crate::error::{Error, Result};
use crate::number::Number;
use crate::polynomial::Polynomial;

/// Represents a general expression.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Expr {
	/// Represents the sum of multiple expressions.
	Add(Vec<Self>),
	/// Represents a function call.
	Fun(Symbol, Vec<Self>),
	/// Represents the product of multiple expressions.
	Mul(Vec<Self>),
	/// Represents a constant number.
	Num(Number),
	/// Represents a polynomial.
	Poly(Symbol, Polynomial),
	/// Represents the power of two expressions.
	Pow(Box<Self>, Box<Self>),
}

// Constants
impl Expr {
	/// The number negative one (`-1`).
	pub fn neg_one() -> Self {
		Self::Num(Number::neg_one())
	}

	/// The number one (`1`).
	pub fn one() -> Self {
		Self::Num(Number::one())
	}

	/// The number zero (`0`).
	pub fn zero() -> Self {
		Self::Num(Number::zero())
	}
}

// Guards
impl Expr {
	/// Whether this is a constant number.
	pub const fn is_num(&self) -> bool {
		matches!(self, Self::Num(_))
	}

	/// Whether this is a constant number that also satisfies a predicate.
	pub fn is_num_and(&self, predicate: impl FnOnce(&Number) -> bool) -> bool {
		matches!(self, Self::Num(num) if predicate(num))
	}

	/// Whether this is a polynomial.
	pub const fn is_poly(&self) -> bool {
		matches!(self, Self::Poly(_, _))
	}

	/// Whether this is a polynomial that also satisfies a predicate.
	pub fn is_poly_and(&self, predicate: impl FnOnce(&Symbol, &Polynomial) -> bool) -> bool {
		matches!(self, Self::Poly(sym, poly) if predicate(sym, poly))
	}
}

// Operations
impl Expr {
	/// Evaluates this expression as an approximate value.
	pub fn evaluate(self, ctx: &Context) -> Result<f64> {
		match self {
			Self::Add(exprs) => exprs.into_iter().map(|expr| expr.evaluate(ctx)).sum(),
			Self::Fun(name, args) => Self::evaluate_fun(name, args, ctx),
			Self::Mul(exprs) => exprs.into_iter().map(|expr| expr.evaluate(ctx)).product(),
			Self::Num(num) => Ok(num.to_f64()),
			Self::Poly(sym, poly) => Self::evaluate_poly(sym, poly, ctx),
			Self::Pow(base, exp) => Self::evaluate_pow(*base, *exp, ctx),
		}
	}

	/// Evaluates a [`Self::Fun`] expression.
	fn evaluate_fun(name: Symbol, args: Vec<Expr>, ctx: &Context) -> Result<f64> {
		// Get the evaluator or return an error
		let Some(evaluator) = ctx.evaluators.get(&name) else {
			return Err(Error::UndeclaredValue(name));
		};

		// Evaluate the inner arguments
		let args = args.into_iter().map(|arg| arg.evaluate(ctx)).try_collect()?;

		// Execute the evaluator
		evaluator(args)
	}

	/// Evaluates a [`Self::Poly`] expression.
	fn evaluate_poly(sym: Symbol, poly: Polynomial, ctx: &Context) -> Result<f64> {
		// Get the constant or return an error
		let Some(constant) = ctx.constants.get(&sym) else {
			return Err(Error::UndeclaredValue(sym));
		};

		// Return the summed monomials
		Ok(poly.evaluate(*constant))
	}

	/// Evaluates a [`Self::Pow`] expression.
	fn evaluate_pow(base: Expr, exp: Expr, ctx: &Context) -> Result<f64> {
		// First evaluate the base and exponent separately
		let base = base.evaluate(ctx)?;
		let exp = exp.evaluate(ctx)?;

		// If base is zero and exponent is negative, return zero division error
		if base == 0.0 && exp < 0.0 {
			return Err(Error::DivisionByZero);
		}

		// Return the power of both evaluations
		Ok(base.pow(exp))
	}

	/// Returns the inner value if this expression is [`Self::Num`], otherwise returns [`None`].
	pub fn into_num(self) -> Option<Number> {
		match self {
			Self::Num(num) => Some(num),
			_ => None,
		}
	}

	/// Returns the inner value if this expression is [`Self::Poly`], otherwise returns [`None`].
	pub fn into_poly(self) -> Option<(Symbol, Polynomial)> {
		match self {
			Self::Poly(sym, poly) => Some((sym, poly)),
			_ => None,
		}
	}

	/// Simplifies this expression on a best-effort basis.
	pub fn simplify(self, ctx: &Context) -> Result<Self> {
		match self {
			Self::Add(exprs) => Self::simplify_add(exprs, ctx),
			Self::Fun(name, args) => Self::simplify_fun(name, args, ctx),
			Self::Mul(exprs) => Self::simplify_mul(exprs, ctx),
			Self::Num(_) => Ok(self),
			Self::Poly(sym, poly) => Self::simplify_poly(sym, poly, ctx),
			Self::Pow(base, exp) => Self::simplify_pow(base, exp, ctx),
		}
	}

	/// Simplifies a [`Self::Add`] expression.
	fn simplify_add(mut exprs: Vec<Self>, ctx: &Context) -> Result<Self> {
		// Simplify all elements individually and flatten inner sums
		exprs = exprs
			.into_iter()
			.map(|expr| match expr.simplify(ctx)? {
				Self::Add(exprs) => Ok(exprs),
				expr => Ok(vec![expr]),
			})
			.flatten_ok()
			.try_collect()?;

		// Add all polynomials into one per symbol
		let mut polys = exprs
			.extract_if(.., |expr| expr.is_poly())
			.map(|expr| expr.into_poly().unwrap())
			.into_grouping_map()
			.reduce(|lhs, _, rhs| lhs + rhs);

		// Add all numbers into one, extracting constant parts from the polynomials
		let mut num = exprs
			.extract_if(.., |expr| expr.is_num())
			.map(|expr| expr.into_num().unwrap())
			.chain(polys.values_mut().map(Polynomial::split_constant_mut))
			.reduce(|lhs, rhs| lhs + &rhs)
			.filter(|num| !num.is_zero());

		// Remove potential newly created zero polynomials
		polys.retain(|_, poly| !poly.is_zero());

		// If only one polynomial is left, add the constant back into it
		if let Ok(poly) = polys.values_mut().exactly_one() {
			num.take().into_iter().for_each(|num| *poly += num);
		}

		// For every other expression, count how often it appears
		let counts = exprs.into_iter().counts();

		// Convert into iterator of products and chain extracted number and polynomials
		let iter = counts
			.into_iter()
			.map(|(expr, count)| (expr * Self::Num(count.into())).simplify(ctx))
			.chain(num.into_iter().map(|num| Ok(Self::Num(num))))
			.chain(polys.into_iter().map(|(sym, poly)| Ok(Self::Poly(sym, poly))));

		// If at most one element is left, return it separately
		let mut result: Vec<_> = match iter.at_most_one() {
			Ok(expr) => return Ok(expr.transpose()?.unwrap_or_else(Self::zero)),
			Err(iter) => iter.try_collect()?,
		};

		// Sort the resulting vec
		result.sort_by(Self::cmp);

		// Return the result as a new sum
		Ok(Self::Add(result))
	}

	/// Simplifies a [`Self::Fun`] expression.
	fn simplify_fun(name: Symbol, mut args: Vec<Self>, ctx: &Context) -> Result<Self> {
		// Simplify the inner arguments
		args = args.into_iter().map(|arg| Self::simplify(arg, ctx)).try_collect()?;

		// Handle user-defined functions with higher priority
		if let Some(function) = ctx.functions.get(&name) {
			// Check that the argument count matches
			if function.params.len() != args.len() {
				return Err(Error::ArgumentCount(function.name.clone()));
			}

			// Create a new temporary context with the arguments
			let mut temp = ctx.clone();

			for (name, arg) in function.params.iter().zip(args) {
				temp.variables.insert(name.clone(), arg);
			}

			// Simplify the body with the added arguments
			return function.body.clone().simplify(&temp);
		}

		// If no user-defined function was found, look up a simplifier
		if let Some(simplifier) = ctx.simplifiers.get(&name) {
			return simplifier(args);
		}

		// Return the result as a new function call
		Ok(Self::Fun(name, args))
	}

	/// Simplifies a [`Self::Mul`] expression.
	fn simplify_mul(mut exprs: Vec<Self>, ctx: &Context) -> Result<Self> {
		// Simplify all elements individually and flatten inner products
		exprs = exprs
			.into_iter()
			.map(|expr| match expr.simplify(ctx)? {
				Self::Mul(exprs) => Ok(exprs),
				expr => Ok(vec![expr]),
			})
			.flatten_ok()
			.try_collect()?;

		// Multiply all polynomials into one per symbol
		let mut polys = exprs
			.extract_if(.., |expr| expr.is_poly())
			.map(|expr| expr.into_poly().unwrap())
			.into_grouping_map()
			.reduce(|lhs, _, rhs| lhs * &rhs);

		// Multiply all numbers into one, extracting monic factors from the polynomials
		let mut num = exprs
			.extract_if(.., |expr| expr.is_num())
			.map(|expr| expr.into_num().unwrap())
			.chain(polys.values_mut().filter_map(Polynomial::monic_mut))
			.reduce(|lhs, rhs| lhs * &rhs)
			.filter(|num| !num.is_one());

		// If the number is zero, the product will be zero
		if num.as_ref().is_some_and(Number::is_zero) {
			return Ok(Self::zero());
		}

		// Remove potential newly created one polynomials
		polys.retain(|_, poly| !poly.is_one());

		// If only one polynomial is left, multiply the factor back into it
		if let Ok(poly) = polys.values_mut().exactly_one() {
			num.take().into_iter().for_each(|num| *poly *= &num);
		}

		// For every other expression, count how often it appears
		let counts = exprs.into_iter().counts();

		// Convert into iterator of powers and chain extracted number and polynomials
		let iter = counts
			.into_iter()
			.map(|(expr, count)| expr.pow(Self::Num(count.into())).simplify(ctx))
			.chain(num.into_iter().map(|num| Ok(Self::Num(num))))
			.chain(polys.into_iter().map(|(sym, poly)| Ok(Self::Poly(sym, poly))));

		// If at most one element is left, return it separately
		let mut result: Vec<_> = match iter.at_most_one() {
			Ok(expr) => return Ok(expr.transpose()?.unwrap_or_else(Self::one)),
			Err(iter) => iter.try_collect()?,
		};

		// Sort the resulting vec
		result.sort_by(Self::cmp);

		// Return the result as a new product
		Ok(Self::Mul(result))
	}

	/// Simplifies a [`Self::Poly`] expression.
	fn simplify_poly(sym: Symbol, poly: Polynomial, ctx: &Context) -> Result<Self> {
		// If the polynomial is constant, return it as a number
		if poly.is_constant() {
			return Ok(Self::Num(poly.split_constant().0));
		}

		// If the symbol is a declared variable, insert it into the polynomial
		if let Some(variable) = ctx.variables.get(&sym) {
			// Return the simplified sum
			return poly.into_expr(variable).simplify(ctx);
		}

		// Return the result as a new polynomial
		Ok(Self::Poly(sym, poly))
	}

	/// Simplifies a [`Self::Pow`] expression.
	fn simplify_pow(mut base: Box<Self>, mut exp: Box<Self>, ctx: &Context) -> Result<Self> {
		// First simplify the base and exponent separately
		*base = base.simplify(ctx)?;
		*exp = exp.simplify(ctx)?;

		// If base is zero and exponent is negative, return zero division error
		if base.is_num_and(Number::is_zero) && exp.is_num_and(Number::is_negative) {
			return Err(Error::DivisionByZero);
		}

		// If base is one or exponent is zero, return one
		if base.is_num_and(Number::is_one) || exp.is_num_and(Number::is_zero) {
			return Ok(Self::one());
		}

		// If base is zero or exponent is one, return the base
		if base.is_num_and(Number::is_zero) || exp.is_num_and(Number::is_one) {
			return Ok(*base);
		}

		// Return the result as a new power
		Ok(Self::Pow(base, exp))
	}

	/// Compares this expression with another for a consistent ordering.
	fn cmp(&self, other: &Self) -> Ordering {
		match (self, other) {
			// If both are sums, compare the vecs
			(Self::Add(lhs), Self::Add(rhs)) => Self::cmp_vecs(lhs, rhs),

			// If both are function calls, compare name first, then arguments
			(Self::Fun(lhs_name, lhs_args), Self::Fun(rhs_name, rhs_args)) => {
				lhs_name.cmp(rhs_name).then_with(|| Self::cmp_vecs(lhs_args, rhs_args))
			}

			// If both are products, compare the vecs
			(Self::Mul(lhs), Self::Mul(rhs)) => Self::cmp_vecs(lhs, rhs),

			// If both are numbers, compare them directly
			(Self::Num(lhs), Self::Num(rhs)) => lhs.cmp(rhs),

			// If both are polynomials, compare symbol first, then monomials
			(Self::Poly(lhs_sym, lhs_poly), Self::Poly(rhs_sym, rhs_poly)) => {
				lhs_sym.cmp(rhs_sym).then_with(|| Self::cmp_polys(lhs_poly, rhs_poly))
			}

			// If both are powers, compare base first, then exponent
			(Self::Pow(lhs_base, lhs_exp), Self::Pow(rhs_base, rhs_exp)) => {
				lhs_base.cmp(rhs_base).then_with(|| lhs_exp.cmp(rhs_exp))
			}

			// Otherwise, compare the discriminants
			(Self::Add(_), _) => Ordering::Less,
			(_, Self::Add(_)) => Ordering::Greater,
			(Self::Fun(_, _), _) => Ordering::Less,
			(_, Self::Fun(_, _)) => Ordering::Greater,
			(Self::Mul(_), _) => Ordering::Less,
			(_, Self::Mul(_)) => Ordering::Greater,
			(Self::Num(_), _) => Ordering::Less,
			(_, Self::Num(_)) => Ordering::Greater,
			(Self::Poly(_, _), _) => Ordering::Less,
			(_, Self::Poly(_, _)) => Ordering::Greater,
		}
	}

	/// Compares two polynomials for a consistent ordering.
	fn cmp_polys(lhs: &Polynomial, rhs: &Polynomial) -> Ordering {
		lhs.monomials()
			.zip(rhs.monomials())
			.map(|(lhs, rhs)| lhs.coeff.cmp(&rhs.coeff).then_with(|| lhs.degree.cmp(&rhs.degree)))
			.find(|ord| ord.is_ne())
			.unwrap_or_else(|| lhs.monomials().len().cmp(&rhs.monomials().len()))
	}

	/// Compares two expression vecs for a consistent ordering.
	fn cmp_vecs(lhs: &[Self], rhs: &[Self]) -> Ordering {
		lhs.iter()
			.zip(rhs)
			.map(|(lhs, rhs)| lhs.cmp(rhs))
			.find(|ord| ord.is_ne())
			.unwrap_or_else(|| lhs.len().cmp(&rhs.len()))
	}

	/// Formats this expression with parentheses if necessary.
	fn with_parens(&self) -> impl fmt::Display {
		struct WithParens<'a>(&'a Expr);

		impl fmt::Display for WithParens<'_> {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				match self.0 {
					// If the expression has more than one term, use parentheses
					Expr::Add(exprs) if exprs.len() > 1 => write!(f, "({})", self.0),
					Expr::Mul(exprs) if exprs.len() > 1 => write!(f, "({})", self.0),
					Expr::Poly(_, poly) if poly.monomials().len() > 1 => write!(f, "({})", self.0),

					// Otherwise, write the expression normally
					_ => write!(f, "{}", self.0),
				}
			}
		}

		WithParens(self)
	}

	/// Writes a [`Self::Add`] expression, choosing between plus and minus dynamically.
	fn write_add(f: &mut fmt::Formatter, exprs: &[Self]) -> fmt::Result {
		// Format the first expression normally
		if let Some(first) = exprs.first() {
			write!(f, "{}", first)?;
		}

		for expr in exprs.iter().skip(1) {
			match expr {
				// If the number is negative, extract the minus
				Self::Num(num) if num.is_negative() => {
					write!(f, " - ")?;
					num.write(f, true)?;
				}

				// If the polyomial has a negative leading coefficient, extract the minus
				Self::Poly(sym, poly) if poly.leading().is_some_and(Number::is_negative) => {
					write!(f, " - ")?;
					poly.write(f, true, sym.name())?;
				}

				// Otherwise, write the expression normally
				_ => write!(f, " + {}", expr)?,
			}
		}

		Ok(())
	}
}

impl Add<Self> for Expr {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self::Add(vec![self, rhs])
	}
}

impl Div<Self> for Expr {
	type Output = Self;

	#[expect(clippy::suspicious_arithmetic_impl)]
	fn div(self, rhs: Self) -> Self::Output {
		self * rhs.pow(Self::neg_one())
	}
}

impl Mul<Self> for Expr {
	type Output = Self;

	fn mul(self, rhs: Self) -> Self::Output {
		Self::Mul(vec![self, rhs])
	}
}

impl Neg for Expr {
	type Output = Self;

	fn neg(self) -> Self::Output {
		self * Self::neg_one()
	}
}

impl Pow<Self> for Expr {
	type Output = Self;

	fn pow(self, rhs: Self) -> Self::Output {
		Self::Pow(self.into(), rhs.into())
	}
}

impl Sub<Self> for Expr {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		self + rhs * Self::neg_one()
	}
}

impl fmt::Display for Expr {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Add(exprs) => Self::write_add(f, exprs),
			Self::Fun(name, args) => write!(f, "{name}({})", args.iter().format(", ")),
			Self::Mul(exprs) => write!(f, "{}", exprs.iter().map(Self::with_parens).format(" * ")),
			Self::Num(num) => write!(f, "{num}"),
			Self::Poly(sym, poly) => poly.write(f, false, sym.name()),
			Self::Pow(base, exp) => write!(f, "{}^{}", base.with_parens(), exp.with_parens()),
		}
	}
}
