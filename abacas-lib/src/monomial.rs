//! The monomial structure and its related algorithms.

use std::ops::{Add, Div, DivAssign, Mul, MulAssign, Neg, Sub};
use std::{fmt, str};

use rug::ops::{NegAssign, Pow, PowAssign};

use crate::error::Error;
use crate::number::Number;
use crate::polynomial::Polynomial;

/// A monomial `ax^b` consisting of coefficient `a` and degree `b`.
///
/// # Examples
///
/// Creating a [`Monomial`]:
///
/// ```
/// use abacas::monomial::Monomial;
///
/// let mono = Monomial::new(4, 10);
/// assert_eq!(mono.to_string(), "4x^10");
///
/// let mono: Monomial = "4x^10".parse().unwrap();
/// assert_eq!(mono.to_string(), "4x^10");
/// ```
///
/// Using arithmetic operations:
///
/// ```
/// use abacas::monomial::Monomial;
///
/// let add = Monomial::new(4, 10) + Monomial::new(1, 20);
/// assert_eq!(add.to_string(), "x^20 + 4x^10");
///
/// let mul = Monomial::new(4, 10) * &Monomial::linear(2);
/// assert_eq!(mul.to_string(), "8x^11");
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Monomial {
	/// The coefficient of the monomial
	pub coeff: Number,
	/// The degree of the monomial
	pub degree: Number,
}

impl Monomial {
	/// Creates a constant monomial. Panics if `coeff` is zero.
	///
	/// # Examples
	///
	/// ```
	/// use abacas::monomial::Monomial;
	///
	/// let mono = Monomial::constant(4);
	/// assert_eq!(mono.to_string(), "4");
	/// ```
	pub fn constant(coeff: impl Into<Number>) -> Self {
		Self::new(coeff, 0)
	}

	/// Creates a linear monomial. Panics if `coeff` is zero.
	///
	/// # Examples
	///
	/// ```
	/// use abacas::monomial::Monomial;
	///
	/// let mono = Monomial::linear(2);
	/// assert_eq!(mono.to_string(), "2x");
	/// ```
	pub fn linear(coeff: impl Into<Number>) -> Self {
		Self::new(coeff, 1)
	}

	/// Creates a new monomial. Panics if `coeff` is zero.
	///
	/// # Examples
	///
	/// ```
	/// use abacas::monomial::Monomial;
	///
	/// let mono = Monomial::new(4, 22);
	/// assert_eq!(mono.to_string(), "4x^22");
	/// ```
	pub fn new(coeff: impl Into<Number>, degree: impl Into<Number>) -> Self {
		let coeff = coeff.into();
		let degree = degree.into();

		if coeff.is_zero() {
			panic!("coefficient must not be zero");
		}

		Self { coeff, degree }
	}

	/// Internal method to write this monomial with specific configuration.
	pub(crate) fn write(&self, f: &mut fmt::Formatter<'_>, abs: bool, sym: &str) -> fmt::Result {
		if self.degree.is_zero() {
			return self.coeff.write(f, abs);
		}

		if self.coeff.is_neg_one() && !abs {
			write!(f, "-")?;
		} else if !self.coeff.is_neg_one() && !self.coeff.is_one() {
			self.coeff.write(f, abs)?;
		}

		if self.degree.is_one() {
			write!(f, "{sym}")
		} else {
			write!(f, "{sym}^{}", self.degree)
		}
	}
}

impl<T: Into<Number>> From<T> for Monomial {
	fn from(value: T) -> Self {
		Self::constant(value)
	}
}

impl<T> Add<T> for Monomial
where
	Polynomial: Add<T, Output = Polynomial>,
{
	type Output = Polynomial;

	fn add(self, rhs: T) -> Self::Output {
		Polynomial::from(self) + rhs
	}
}

impl<T> Div<T> for Monomial
where
	Self: DivAssign<T>,
{
	type Output = Self;

	fn div(mut self, rhs: T) -> Self::Output {
		self /= rhs;
		self
	}
}

impl<T> DivAssign<T> for Monomial
where
	Number: DivAssign<T>,
{
	fn div_assign(&mut self, rhs: T) {
		self.coeff /= rhs;
	}
}

impl DivAssign<&Self> for Monomial {
	fn div_assign(&mut self, rhs: &Self) {
		self.coeff /= &rhs.coeff;
		self.degree -= &rhs.degree;
	}
}

impl<T> Mul<T> for Monomial
where
	Self: MulAssign<T>,
{
	type Output = Self;

	fn mul(mut self, rhs: T) -> Self::Output {
		self *= rhs;
		self
	}
}

impl<T> MulAssign<T> for Monomial
where
	Number: MulAssign<T>,
{
	fn mul_assign(&mut self, rhs: T) {
		self.coeff *= rhs;
	}
}

impl MulAssign<&Self> for Monomial {
	fn mul_assign(&mut self, rhs: &Self) {
		self.coeff *= &rhs.coeff;
		self.degree += &rhs.degree;
	}
}

impl Neg for Monomial {
	type Output = Self;

	fn neg(mut self) -> Self::Output {
		self.neg_assign();
		self
	}
}

impl NegAssign for Monomial {
	fn neg_assign(&mut self) {
		self.coeff.neg_assign();
	}
}

impl<T> Pow<T> for Monomial
where
	Self: PowAssign<T>,
{
	type Output = Self;

	fn pow(mut self, rhs: T) -> Self::Output {
		self.pow_assign(rhs);
		self
	}
}

impl<T: Copy> PowAssign<T> for Monomial
where
	Number: MulAssign<T> + PowAssign<T>,
{
	fn pow_assign(&mut self, rhs: T) {
		self.coeff.pow_assign(rhs);
		self.degree *= rhs;
	}
}

impl<T> Sub<T> for Monomial
where
	Polynomial: Sub<T, Output = Polynomial>,
{
	type Output = Polynomial;

	fn sub(self, rhs: T) -> Self::Output {
		Polynomial::from(self) - rhs
	}
}

impl fmt::Display for Monomial {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.write(f, false, "x")
	}
}

impl str::FromStr for Monomial {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let (init, degree) = if let Some((init, tail)) = s.split_once("x^") {
			(init, tail.parse()?)
		} else if let Some(init) = s.strip_suffix('x') {
			(init, Number::one())
		} else {
			(s, Number::zero())
		};

		let coeff = match init {
			"" | "+" if !degree.is_zero() => Number::one(),
			"-" if !degree.is_zero() => Number::neg_one(),
			_ => init.parse()?,
		};

		if coeff.is_zero() {
			return Err(Error::InvalidNumber(coeff));
		}

		Ok(Self::new(coeff, degree))
	}
}
