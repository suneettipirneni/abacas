//! The polynomial structure and its related algorithms.

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign};
use std::slice::Iter;
use std::{fmt, mem, str};

use rug::ops::{NegAssign, Pow};

use crate::error::Error;
use crate::expr::Expr;
use crate::monomial::Monomial;
use crate::number::Number;

/// A polynomial with its monomials sorted by `degree` in descending order.
///
/// # Examples
///
/// Creating a [`Polynomial`]:
///
/// ```
/// use abacas::monomial::Monomial;
/// use abacas::polynomial::Polynomial;
///
/// let poly = Polynomial::new([Monomial::new(4, 2), Monomial::new(5, 3)]);
/// assert_eq!(poly.to_string(), "5x^3 + 4x^2");
///
/// let poly: Polynomial = "4x^2 + 5x^3".parse().unwrap();
/// assert_eq!(poly.to_string(), "5x^3 + 4x^2");
/// ```
///
/// Using arithmetic operations:
///
/// ```
/// use abacas::polynomial::Polynomial;
///
/// let a: Polynomial = "4x^4 + 3x^3 + 1".parse().unwrap();
/// let b: Polynomial = "2x^2 - 5".parse().unwrap();
///
/// let add = a.clone() + b.clone();
/// assert_eq!(add.to_string(), "4x^4 + 3x^3 + 2x^2 - 4");
///
/// let sub = a.clone() - b.clone() * 2;
/// assert_eq!(sub.to_string(), "4x^4 + 3x^3 - 4x^2 + 11");
///
/// let mul = a.clone() * &b;
/// assert_eq!(mul.to_string(), "8x^6 + 6x^5 - 20x^4 - 15x^3 + 2x^2 - 5");
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Polynomial(Vec<Monomial>);

impl Polynomial {
	/// The zero polynomial.
	pub const ZERO: Self = Self(Vec::new());

	/// Internal method to clean up a polynomial after operating on it.
	fn clean(&mut self) {
		self.0.retain(|mono| !mono.coeff.is_zero());
	}

	/// Returns the degree of the polynomial, or [`None`] for the zero polynomial.
	///
	/// # Examples
	///
	/// ```
	/// use abacas::polynomial::Polynomial;
	///
	/// let poly: Polynomial = "4x^999 + 2x^3 + 1".parse().unwrap();
	/// assert_eq!(poly.degree(), Some(&999.into()));
	/// ```
	pub fn degree(&self) -> Option<&Number> {
		self.0.first().map(|mono| &mono.degree)
	}

	/// Calculates division and remainder at the same time, returning [`None`] if the divisor is zero.
	///
	/// # Examples
	///
	/// ```
	/// use abacas::polynomial::Polynomial;
	///
	/// let dividend: Polynomial = "6x^5 + 5x^2 - 7".parse().unwrap();
	/// let divisor: Polynomial = "2x^2 - 1".parse().unwrap();
	///
	/// let (quotient, remainder) = dividend.clone().div_rem(&divisor).unwrap();
	///
	/// assert_eq!(quotient.to_string(), "3x^3 + 1.5x + 2.5");
	/// assert_eq!(remainder.to_string(), "1.5x - 4.5");
	/// assert_eq!(quotient * &divisor + remainder, dividend);
	/// ```
	pub fn div_rem(mut self, divisor: &Self) -> Option<(Self, Self)> {
		self.div_rem_mut(divisor).map(|remainder| (self, remainder))
	}

	/// Calculates division in-place and returns the remainder, or [`None`] if the divisor is zero.
	///
	/// # Examples
	///
	/// ```
	/// use abacas::polynomial::Polynomial;
	///
	/// let mut dividend: Polynomial = "6x^5 + 5x^2 - 7".parse().unwrap();
	/// let divisor: Polynomial = "2x^2 - 1".parse().unwrap();
	///
	/// let remainder = dividend.div_rem_mut(&divisor).unwrap();
	///
	/// assert_eq!(dividend.to_string(), "3x^3 + 1.5x + 2.5");
	/// assert_eq!(remainder.to_string(), "1.5x - 4.5");
	/// ```
	pub fn div_rem_mut(&mut self, divisor: &Self) -> Option<Self> {
		let (normalizer, terms) = divisor.0.split_first()?;

		let Some(mut degree) = self.degree().cloned() else {
			return Some(Self::ZERO);
		};

		while degree >= normalizer.degree {
			let monomial = self.get_or_insert(&degree);
			let coeff = monomial.coeff.clone() / &normalizer.coeff;

			monomial.coeff = coeff.clone();

			for term in terms {
				let degree = degree.clone() + &term.degree - &normalizer.degree;
				let monomial = self.get_or_insert(&degree);

				monomial.coeff -= &(coeff.clone() * &term.coeff);
			}

			degree -= 1;
		}

		self.clean();

		let index = self
			.search(&normalizer.degree)
			.map_or_else(|index| index, |index| index + 1);

		let remainder = Self::new(self.0.split_off(index));

		for monomial in &mut self.0 {
			monomial.degree -= &normalizer.degree;
		}

		Some(remainder)
	}

	/// Evaluates this polynomial at the given value for `x`.
	pub fn evaluate(&self, value: f64) -> f64 {
		self.monomials()
			.map(|mono| mono.coeff.to_f64() * value.pow(mono.degree.to_f64()))
			.sum()
	}

	/// Extracts the common factor of all monomials.
	/// Returns [`None`] if the polynomial is zero or has coprime coefficients.
	///
	/// # Examples
	///
	/// ```
	/// use abacas::polynomial::Polynomial;
	///
	/// let poly: Polynomial = "16x^2 + 8x + 4".parse().unwrap();
	/// let (factor, rest) = poly.factor().unwrap();
	///
	/// assert_eq!(factor, 4);
	/// assert_eq!(rest.to_string(), "4x^2 + 2x + 1");
	/// ```
	pub fn factor(mut self) -> Option<(Number, Self)> {
		self.factor_mut().map(|factor| (factor, self))
	}

	/// Extracts the common factor of all monomials in-place.
	/// Returns [`None`] if the polynomial is zero or has coprime coefficients.
	///
	/// # Examples
	///
	/// ```
	/// use abacas::polynomial::Polynomial;
	///
	/// let mut poly: Polynomial = "16x^2 + 8x + 4".parse().unwrap();
	/// let factor = poly.factor_mut().unwrap();
	///
	/// assert_eq!(factor, 4);
	/// assert_eq!(poly.to_string(), "4x^2 + 2x + 1");
	/// ```
	pub fn factor_mut(&mut self) -> Option<Number> {
		let factor = self
			.monomials()
			.map(|mono| &mono.coeff)
			.fold(Number::zero(), Number::gcd);

		if factor <= 1 {
			return None;
		}

		*self /= &factor;

		Some(factor)
	}

	/// Returns the GCD of two polynomials in monic form.
	///
	/// # Examples
	///
	/// ```
	/// use abacas::polynomial::Polynomial;
	///
	/// let coeff = "x - 1".parse::<Polynomial>().unwrap();
	/// let a = coeff.clone() * &"x - 21".parse::<Polynomial>().unwrap();
	/// let b = coeff.clone() * &"4x - 9".parse::<Polynomial>().unwrap();
	///
	/// assert_eq!(a.gcd(b), coeff);
	/// ```
	pub fn gcd(mut self, mut other: Self) -> Self {
		while let Some(remainder) = self.div_rem_mut(&other) {
			self = other;
			other = remainder;
		}

		self.monic_mut();
		self
	}

	/// Returns the GCD of two polynomials in monic form, along with their Bézout coefficients.
	///
	/// # Examples
	///
	/// ```
	/// use abacas::polynomial::Polynomial;
	///
	/// let coeff = "x - 1".parse::<Polynomial>().unwrap();
	/// let a = coeff.clone() * &"x - 21".parse::<Polynomial>().unwrap();
	/// let b = coeff.clone() * &"4x - 9".parse::<Polynomial>().unwrap();
	///
	/// let (s, t, gcd) = a.clone().gcd_ext(b.clone());
	/// let bezout = s * &a + t * &b;
	///
	/// assert_eq!(bezout, gcd);
	/// assert_eq!(coeff, gcd);
	/// ```
	pub fn gcd_ext(self, other: Self) -> (Self, Self, Self) {
		let (mut old_r, mut r) = (self.clone(), other.clone());
		let (mut old_s, mut s) = (Self::from(1), Self::ZERO);

		while let Some(remainder) = old_r.div_rem_mut(&r) {
			let quotient = old_r;

			(old_r, r) = (r, remainder);
			(old_s, s) = (s.clone(), old_s - quotient * &s);
		}

		if let Some(factor) = old_r.monic_mut() {
			old_s /= &factor;
		}

		let old_t = if other.is_zero() {
			Self::ZERO
		} else {
			(old_r.clone() - self * &old_s) / &other
		};

		(old_s, old_t, old_r)
	}

	/// Returns the monomial with the given degree, or [`None`] if the degree is not present.
	///
	/// # Examples
	///
	/// ```
	/// use abacas::monomial::Monomial;
	/// use abacas::polynomial::Polynomial;
	///
	/// let poly: Polynomial = "4x^9 + 2x^3 + x^2 + 100".parse().unwrap();
	/// assert_eq!(poly.get(&9.into()), Some(&Monomial::new(4, 9)));
	/// ```
	pub fn get(&self, degree: &Number) -> Option<&Monomial> {
		self.search(degree).ok().and_then(|index| self.0.get(index))
	}

	/// Internal method to get a monomial or insert it if it does not exist.
	fn get_or_insert(&mut self, degree: &Number) -> &mut Monomial {
		let index = self
			.search(degree)
			.inspect_err(|&index| {
				let coeff = 0.into();
				let degree = degree.clone();

				self.0.insert(index, Monomial { coeff, degree });
			})
			.unwrap_or_else(|index| index);

		&mut self.0[index]
	}

	/// Creates a new expression from this polynomial, replacing `x` with the given expression.
	pub fn into_expr(self, expr: &Expr) -> Expr {
		let exprs = self
			.0
			.into_iter()
			.map(|mono| Expr::Num(mono.coeff) * expr.clone().pow(Expr::Num(mono.degree)))
			.collect();

		Expr::Add(exprs)
	}

	/// Returns whether this polynomial can be represented as a constant [`Number`].
	///
	/// # Examples
	///
	/// ```
	/// use abacas::monomial::Monomial;
	/// use abacas::polynomial::Polynomial;
	///
	/// assert!(Polynomial::ZERO.is_constant());
	/// assert!(Polynomial::from(5).is_constant());
	///
	/// assert!(!Polynomial::from(Monomial::linear(3)).is_constant());
	/// assert!(!Polynomial::from(Monomial::linear(3) + 5).is_constant());
	/// ```
	pub const fn is_constant(&self) -> bool {
		self.is_zero() || matches!(self.0.as_slice(), [mono] if mono.degree.is_zero())
	}

	/// Returns whether this polynomial is the number negative one (`-1`).
	///
	/// # Examples
	///
	/// ```
	/// use abacas::polynomial::Polynomial;
	///
	/// assert!(Polynomial::from(-1).is_neg_one());
	/// assert!(!Polynomial::from(1).is_neg_one());
	/// ```
	pub fn is_neg_one(&self) -> bool {
		matches!(self.0.as_slice(), [mono] if mono.coeff.is_neg_one() && mono.degree.is_zero())
	}

	/// Returns whether this polynomial is the number one (`1`).
	///
	/// # Examples
	///
	/// ```
	/// use abacas::polynomial::Polynomial;
	///
	/// assert!(Polynomial::from(1).is_one());
	/// assert!(!Polynomial::from(-1).is_one());
	/// ```
	pub fn is_one(&self) -> bool {
		matches!(self.0.as_slice(), [mono] if mono.coeff.is_one() && mono.degree.is_zero())
	}

	/// Returns whether this is the zero polynomial.
	///
	/// # Examples
	///
	/// ```
	/// use abacas::polynomial::Polynomial;
	///
	/// assert!(Polynomial::ZERO.is_zero());
	/// assert!(!Polynomial::from(1).is_zero());
	/// ```
	pub const fn is_zero(&self) -> bool {
		self.0.is_empty()
	}

	/// Returns the leading coefficient of the polynomial, or [`None`] for the zero polynomial.
	///
	/// # Examples
	///
	/// ```
	/// use abacas::polynomial::Polynomial;
	///
	/// let poly: Polynomial = "4x^999 + 2x^3 + 1".parse().unwrap();
	/// assert_eq!(poly.leading(), Some(&4.into()));
	/// ```
	pub fn leading(&self) -> Option<&Number> {
		self.0.first().map(|mono| &mono.coeff)
	}

	/// Creates a monic polynomial by dividing all monomials by the leading coefficient.
	/// Returns [`None`] if the polynomial is zero or already monic.
	///
	/// # Examples
	///
	/// ```
	/// use abacas::polynomial::Polynomial;
	///
	/// let poly: Polynomial = "16x^9 + 4x^3 + 32".parse().unwrap();
	/// let (factor, monic) = poly.monic().unwrap();
	///
	/// assert_eq!(factor, 16);
	/// assert_eq!(monic.to_string(), "x^9 + 0.25x^3 + 2");
	/// ```
	pub fn monic(mut self) -> Option<(Number, Self)> {
		self.monic_mut().map(|factor| (factor, self))
	}

	/// Creates a monic polynomial in-place by dividing all monomials by the leading coefficient.
	/// Returns [`None`] if the polynomial is zero or already monic.
	///
	/// # Examples
	///
	/// ```
	/// use abacas::polynomial::Polynomial;
	///
	/// let mut poly: Polynomial = "16x^9 + 4x^3 + 32".parse().unwrap();
	/// let factor = poly.monic_mut().unwrap();
	///
	/// assert_eq!(factor, 16);
	/// assert_eq!(poly.to_string(), "x^9 + 0.25x^3 + 2");
	/// ```
	pub fn monic_mut(&mut self) -> Option<Number> {
		let factor = self.leading()?.clone();

		if factor.is_one() {
			return None;
		}

		*self /= &factor;

		Some(factor)
	}

	/// Returns an iterator over the contained monomials.
	///
	/// # Examples
	///
	/// ```
	/// use abacas::polynomial::Polynomial;
	///
	/// let poly: Polynomial = "3x^2 - 2x + x^-1".parse().unwrap();
	/// assert_eq!(poly.monomials().len(), 3);
	/// ```
	pub fn monomials(&self) -> Iter<'_, Monomial> {
		self.0.iter()
	}

	/// Creates a new polynomial from the given monomials.
	///
	/// # Examples
	///
	/// ```
	/// use abacas::monomial::Monomial;
	/// use abacas::polynomial::Polynomial;
	///
	/// let poly = Polynomial::new([Monomial::new(4, 2), Monomial::new(9, 9)]);
	/// assert_eq!(poly.to_string(), "9x^9 + 4x^2");
	/// ```
	pub fn new(monomials: impl IntoIterator<Item = Monomial>) -> Self {
		monomials.into_iter().fold(Self::ZERO, Self::add)
	}

	/// Internal method to search for the index of the given degree.
	fn search(&self, degree: &Number) -> Result<usize, usize> {
		self.0.binary_search_by(|mono| degree.cmp(&mono.degree))
	}

	/// Splits the constant part from the polynomial and returns it.
	///
	/// # Examples
	///
	/// ```
	/// use abacas::polynomial::Polynomial;
	///
	/// let poly: Polynomial = "16x^2 + 8x + 4".parse().unwrap();
	/// let (constant, rest) = poly.split_constant();
	///
	/// assert_eq!(constant, 4);
	/// assert_eq!(rest.to_string(), "16x^2 + 8x");
	/// ```
	pub fn split_constant(mut self) -> (Number, Self) {
		(self.split_constant_mut(), self)
	}

	/// Splits the constant part from the polynomial in-place and returns it.
	///
	/// # Examples
	///
	/// ```
	/// use abacas::polynomial::Polynomial;
	///
	/// let mut poly: Polynomial = "16x^2 + 8x + 4".parse().unwrap();
	/// let constant = poly.split_constant_mut();
	///
	/// assert_eq!(constant, 4);
	/// assert_eq!(poly.to_string(), "16x^2 + 8x");
	/// ```
	pub fn split_constant_mut(&mut self) -> Number {
		self.search(&Number::zero())
			.map(|index| self.0.remove(index).coeff)
			.unwrap_or_default()
	}

	/// Internal method to write this polynomial with specific configuration.
	pub(crate) fn write(&self, f: &mut fmt::Formatter<'_>, abs: bool, sym: &str) -> fmt::Result {
		match self.0.first() {
			Some(first) => first.write(f, abs, sym)?,
			None => write!(f, "0")?,
		}

		for monomial in self.monomials().skip(1) {
			if monomial.coeff.is_negative() {
				write!(f, " - ")?;
			} else {
				write!(f, " + ")?;
			}

			monomial.write(f, true, sym)?;
		}

		Ok(())
	}
}

impl<T: Into<Number>> From<T> for Polynomial {
	fn from(value: T) -> Self {
		let value = value.into();

		if value.is_zero() {
			Self::ZERO
		} else {
			Self::new([value.into()])
		}
	}
}

impl From<Monomial> for Polynomial {
	fn from(value: Monomial) -> Self {
		Self::new([value])
	}
}

impl<T> Add<T> for Polynomial
where
	Self: AddAssign<T>,
{
	type Output = Self;

	fn add(mut self, rhs: T) -> Self::Output {
		self += rhs;
		self
	}
}

impl<T: Into<Number>> AddAssign<T> for Polynomial {
	fn add_assign(&mut self, rhs: T) {
		let rhs = rhs.into();

		if rhs.is_zero() {
			return;
		}

		match self.search(&Number::zero()) {
			Ok(index) => self.0[index].coeff += &rhs,
			Err(index) => self.0.insert(index, rhs.into()),
		}

		self.clean();
	}
}

impl AddAssign<Monomial> for Polynomial {
	fn add_assign(&mut self, rhs: Monomial) {
		match self.search(&rhs.degree) {
			Ok(index) => self.0[index].coeff += &rhs.coeff,
			Err(index) => self.0.insert(index, rhs),
		}

		self.clean();
	}
}

impl AddAssign<Self> for Polynomial {
	fn add_assign(&mut self, rhs: Self) {
		for monomial in rhs.0 {
			*self += monomial;
		}
	}
}

impl<T> Div<T> for Polynomial
where
	Self: DivAssign<T>,
{
	type Output = Self;

	fn div(mut self, rhs: T) -> Self::Output {
		self /= rhs;
		self
	}
}

impl<T: Copy> DivAssign<T> for Polynomial
where
	Monomial: DivAssign<T>,
{
	fn div_assign(&mut self, rhs: T) {
		for monomial in &mut self.0 {
			*monomial /= rhs;
		}
	}
}

impl DivAssign<&Self> for Polynomial {
	fn div_assign(&mut self, rhs: &Self) {
		self.div_rem_mut(rhs).expect("division by zero");
	}
}

impl<T> Mul<T> for Polynomial
where
	Self: MulAssign<T>,
{
	type Output = Self;

	fn mul(mut self, rhs: T) -> Self::Output {
		self *= rhs;
		self
	}
}

impl<T: Copy> MulAssign<T> for Polynomial
where
	Monomial: MulAssign<T>,
{
	fn mul_assign(&mut self, rhs: T) {
		for monomial in &mut self.0 {
			*monomial *= rhs;
		}
	}
}

impl MulAssign<&Self> for Polynomial {
	fn mul_assign(&mut self, rhs: &Self) {
		let old = mem::take(self);

		for monomial in &rhs.0 {
			*self += old.clone() * monomial;
		}
	}
}

impl Neg for Polynomial {
	type Output = Self;

	fn neg(mut self) -> Self::Output {
		self.neg_assign();
		self
	}
}

impl NegAssign for Polynomial {
	fn neg_assign(&mut self) {
		for monomial in &mut self.0 {
			monomial.neg_assign();
		}
	}
}

impl<T> Rem<T> for Polynomial
where
	Self: RemAssign<T>,
{
	type Output = Self;

	fn rem(mut self, rhs: T) -> Self::Output {
		self %= rhs;
		self
	}
}

impl RemAssign<&Self> for Polynomial {
	fn rem_assign(&mut self, rhs: &Self) {
		*self = self.div_rem_mut(rhs).expect("division by zero");
	}
}

impl<T> Sub<T> for Polynomial
where
	Self: SubAssign<T>,
{
	type Output = Self;

	fn sub(mut self, rhs: T) -> Self::Output {
		self -= rhs;
		self
	}
}

impl<T: Into<Number>> SubAssign<T> for Polynomial {
	fn sub_assign(&mut self, rhs: T) {
		let rhs = rhs.into();

		if rhs.is_zero() {
			return;
		}

		match self.search(&Number::zero()) {
			Ok(index) => self.0[index].coeff -= &rhs,
			Err(index) => self.0.insert(index, (-rhs).into()),
		}

		self.clean();
	}
}

impl SubAssign<Monomial> for Polynomial {
	fn sub_assign(&mut self, rhs: Monomial) {
		match self.search(&rhs.degree) {
			Ok(index) => self.0[index].coeff -= &rhs.coeff,
			Err(index) => self.0.insert(index, -rhs),
		}

		self.clean();
	}
}

impl SubAssign<Self> for Polynomial {
	fn sub_assign(&mut self, rhs: Self) {
		for monomial in rhs.0 {
			*self -= monomial;
		}
	}
}

impl fmt::Display for Polynomial {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.write(f, false, "x")
	}
}

impl str::FromStr for Polynomial {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut result = Self::ZERO;

		for full in s.split(" + ") {
			for (index, part) in full.split(" - ").enumerate() {
				let monomial: Monomial = match part.parse() {
					Ok(monomial) => monomial,
					Err(Error::InvalidNumber(number)) if number.is_zero() => continue,
					Err(error) => return Err(error),
				};

				if index == 0 {
					result += monomial;
				} else {
					result -= monomial;
				}
			}
		}

		Ok(result)
	}
}
