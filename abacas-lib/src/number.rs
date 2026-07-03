//! The number structure and its related operations.

use std::cmp::Ordering;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign};
use std::{fmt, str};

use rug::Rational;
use rug::ops::{DivRounding, DivRoundingAssign, NegAssign, Pow, PowAssign, RemRounding, RemRoundingAssign};

use crate::error::Error;

/// Represents a specific number. Currently uses [`Rational`] under the hood, however this should not be relied upon.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Number(Rational);

// Constants
impl Number {
	/// The number negative one (`-1`).
	pub fn neg_one() -> Self {
		Self(Rational::NEG_ONE.clone())
	}

	/// The number one (`1`).
	pub fn one() -> Self {
		Self(Rational::ONE.clone())
	}

	/// The number zero (`0`).
	pub fn zero() -> Self {
		Self(Rational::new())
	}
}

// Guards
impl Number {
	/// Whether this number is an integer.
	pub const fn is_integer(&self) -> bool {
		self.0.is_integer()
	}

	/// Whether this is the number negative one (`-1`).
	pub fn is_neg_one(&self) -> bool {
		self.0 == *Rational::NEG_ONE
	}

	/// Whether this number is less than zero.
	pub const fn is_negative(&self) -> bool {
		self.0.is_negative()
	}

	/// Whether this is the number one (`1`).
	pub fn is_one(&self) -> bool {
		self.0 == *Rational::ONE
	}

	/// Whether this number is greater than zero.
	pub const fn is_positive(&self) -> bool {
		self.0.is_positive()
	}

	/// Whether this is the number zero (`0`).
	pub const fn is_zero(&self) -> bool {
		self.0.is_zero()
	}
}

// Operations
impl Number {
	/// Gets the denominator of this number.
	pub fn denom(self) -> Self {
		Self(self.0.into_numer_denom().1.into())
	}

	/// Gets the greatest common divisor.
	pub fn gcd(mut self, rhs: &Self) -> Self {
		self.gcd_mut(rhs);
		self
	}

	/// Gets the greatest common divisor and assigns it in-place.
	pub fn gcd_mut(&mut self, rhs: &Self) {
		self.0.mutate_numer_denom(|numer, denom| {
			numer.gcd_mut(rhs.0.numer());
			denom.lcm_mut(rhs.0.denom());
		});
	}

	/// Gets the least common multiple.
	pub fn lcm(mut self, rhs: &Self) -> Self {
		self.lcm_mut(rhs);
		self
	}

	/// Gets the least common multiple and assigns it in-place.
	pub fn lcm_mut(&mut self, rhs: &Self) {
		self.0.mutate_numer_denom(|numer, denom| {
			numer.lcm_mut(rhs.0.numer());
			denom.gcd_mut(rhs.0.denom());
		});
	}

	/// Gets the numerator of this number.
	pub fn numer(self) -> Self {
		Self(self.0.into_numer_denom().0.into())
	}

	/// Gets the numerator and denominator of this number as a tuple.
	pub fn ratio(self) -> (Self, Self) {
		let (numer, denom) = self.0.into_numer_denom();
		(Self(numer.into()), Self(denom.into()))
	}

	/// Converts this number into an [`f32`].
	pub fn to_f32(&self) -> f32 {
		self.0.to_f32()
	}

	/// Converts this number into an [`f64`].
	pub fn to_f64(&self) -> f64 {
		self.0.to_f64()
	}

	/// Internal method to write this number with specific configuration.
	pub(crate) fn write(&self, f: &mut fmt::Formatter<'_>, abs: bool) -> fmt::Result {
		if abs {
			write!(f, "{}", self.0.to_f64().abs())
		} else {
			write!(f, "{}", self.0.to_f64())
		}
	}
}

impl<T> Add<T> for Number
where
	Self: AddAssign<T>,
{
	type Output = Self;

	fn add(mut self, rhs: T) -> Self::Output {
		self += rhs;
		self
	}
}

impl AddAssign<&Self> for Number {
	fn add_assign(&mut self, rhs: &Self) {
		self.0 += &rhs.0;
	}
}

impl<T> Div<T> for Number
where
	Self: DivAssign<T>,
{
	type Output = Self;

	fn div(mut self, rhs: T) -> Self::Output {
		self /= rhs;
		self
	}
}

impl DivAssign<&Self> for Number {
	fn div_assign(&mut self, rhs: &Self) {
		self.0 /= &rhs.0;
	}
}

impl<T> DivRounding<T> for Number
where
	Self: DivRoundingAssign<T>,
{
	type Output = Self;

	fn div_ceil(mut self, rhs: T) -> Self::Output {
		self.div_ceil_assign(rhs);
		self
	}

	fn div_euc(mut self, rhs: T) -> Self::Output {
		self.div_euc_assign(rhs);
		self
	}

	fn div_floor(mut self, rhs: T) -> Self::Output {
		self.div_floor_assign(rhs);
		self
	}

	fn div_trunc(mut self, rhs: T) -> Self::Output {
		self.div_trunc_assign(rhs);
		self
	}
}

impl DivRoundingAssign<&Self> for Number {
	fn div_ceil_assign(&mut self, rhs: &Self) {
		self.0 /= &rhs.0;
		self.0.ceil_mut();
	}

	fn div_euc_assign(&mut self, rhs: &Self) {
		if rhs.is_positive() {
			self.div_floor_assign(rhs);
		} else {
			self.div_ceil_assign(rhs);
		}
	}

	fn div_floor_assign(&mut self, rhs: &Self) {
		self.0 /= &rhs.0;
		self.0.floor_mut();
	}

	fn div_trunc_assign(&mut self, rhs: &Self) {
		self.0 /= &rhs.0;
		self.0.trunc_mut();
	}
}

impl<T> Mul<T> for Number
where
	Self: MulAssign<T>,
{
	type Output = Self;

	fn mul(mut self, rhs: T) -> Self::Output {
		self *= rhs;
		self
	}
}

impl MulAssign<&Self> for Number {
	fn mul_assign(&mut self, rhs: &Self) {
		self.0 *= &rhs.0;
	}
}

impl Neg for Number {
	type Output = Self;

	fn neg(mut self) -> Self::Output {
		self.neg_assign();
		self
	}
}

impl NegAssign for Number {
	fn neg_assign(&mut self) {
		self.0.neg_assign();
	}
}

impl<T> Pow<T> for Number
where
	Self: PowAssign<T>,
{
	type Output = Self;

	fn pow(mut self, rhs: T) -> Self::Output {
		self.pow_assign(rhs);
		self
	}
}

impl PowAssign<&Self> for Number {
	fn pow_assign(&mut self, rhs: &Self) {
		if !rhs.is_integer() {
			panic!("exponent must be an integer");
		}

		let Some(exponent) = rhs.0.numer().as_abs().to_u32() else {
			panic!("exponent must be less than 2^32");
		};

		self.0.pow_assign(exponent);

		if rhs.is_negative() {
			self.0.recip_mut();
		}
	}
}

impl<T> Rem<T> for Number
where
	Self: RemAssign<T>,
{
	type Output = Self;

	fn rem(mut self, rhs: T) -> Self::Output {
		self %= rhs;
		self
	}
}

impl<T> RemAssign<T> for Number
where
	Self: RemRoundingAssign<T>,
{
	fn rem_assign(&mut self, rhs: T) {
		self.rem_trunc_assign(rhs);
	}
}

impl<T> RemRounding<T> for Number
where
	Self: RemRoundingAssign<T>,
{
	type Output = Self;

	fn rem_ceil(mut self, rhs: T) -> Self::Output {
		self.rem_ceil_assign(rhs);
		self
	}

	fn rem_euc(mut self, rhs: T) -> Self::Output {
		self.rem_euc_assign(rhs);
		self
	}

	fn rem_floor(mut self, rhs: T) -> Self::Output {
		self.rem_floor_assign(rhs);
		self
	}

	fn rem_trunc(mut self, rhs: T) -> Self::Output {
		self.rem_trunc_assign(rhs);
		self
	}
}

impl RemRoundingAssign<&Self> for Number {
	fn rem_ceil_assign(&mut self, rhs: &Self) {
		self.0 -= self.clone().div_ceil(rhs).0 * &rhs.0;
	}

	fn rem_euc_assign(&mut self, rhs: &Self) {
		self.0 -= self.clone().div_euc(rhs).0 * &rhs.0;
	}

	fn rem_floor_assign(&mut self, rhs: &Self) {
		self.0 -= self.clone().div_floor(rhs).0 * &rhs.0;
	}

	fn rem_trunc_assign(&mut self, rhs: &Self) {
		self.0 -= self.clone().div_trunc(rhs).0 * &rhs.0;
	}
}

impl<T> Sub<T> for Number
where
	Self: SubAssign<T>,
{
	type Output = Self;

	fn sub(mut self, rhs: T) -> Self::Output {
		self -= rhs;
		self
	}
}

impl SubAssign<&Self> for Number {
	fn sub_assign(&mut self, rhs: &Self) {
		self.0 -= &rhs.0;
	}
}

impl fmt::Display for Number {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.write(f, false)
	}
}

impl str::FromStr for Number {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let full = match s.split_once('.') {
			Some((int, fract)) => format!("{}{}/1{}", int, fract, "0".repeat(fract.len())),
			None => s.into(),
		};

		full.parse().map(Self).map_err(|_| Error::InvalidString(full))
	}
}

macro_rules! impl_float {
	($($float:ty,)*) => {
		$(
			impl TryFrom<$float> for Number {
				type Error = $float;

				fn try_from(value: $float) -> Result<Self, Self::Error> {
					value.try_into().map(Self).map_err(|_| value)
				}
			}
		)*
	};
}

macro_rules! impl_int {
	($($int:ty,)*) => {
		$(
			impl From<$int> for Number {
				fn from(value: $int) -> Self {
					Self(value.into())
				}
			}

			impl AddAssign<$int> for Number {
				fn add_assign(&mut self, rhs: $int) {
					self.0 += rhs;
				}
			}

			impl DivAssign<$int> for Number {
				fn div_assign(&mut self, rhs: $int) {
					self.0 /= rhs;
				}
			}

			impl DivRoundingAssign<$int> for Number {
				fn div_ceil_assign(&mut self, rhs: $int) {
					self.0 /= rhs;
					self.0.ceil_mut();
				}

				fn div_euc_assign(&mut self, rhs: $int) {
					if rhs > 0 {
						self.div_floor_assign(rhs);
					} else {
						self.div_ceil_assign(rhs);
					}
				}

				fn div_floor_assign(&mut self, rhs: $int) {
					self.0 /= rhs;
					self.0.floor_mut();
				}

				fn div_trunc_assign(&mut self, rhs: $int) {
					self.0 /= rhs;
					self.0.trunc_mut();
				}
			}

			impl MulAssign<$int> for Number {
				fn mul_assign(&mut self, rhs: $int) {
					self.0 *= rhs;
				}
			}

			impl PartialEq<$int> for Number {
				fn eq(&self, other: &$int) -> bool {
					self.0 == *other
				}
			}

			impl PartialOrd<$int> for Number {
				fn partial_cmp(&self, other: &$int) -> Option<Ordering> {
					self.0.partial_cmp(other)
				}
			}


			impl PowAssign<$int> for Number {
				fn pow_assign(&mut self, rhs: $int) {
					// TODO: Find a good way to remove this allocation
					self.pow_assign(&Self::from(rhs));
				}
			}

			impl RemRoundingAssign<$int> for Number {
				fn rem_ceil_assign(&mut self, rhs: $int) {
					self.0 -= self.clone().div_ceil(rhs).0 * rhs;
				}

				fn rem_euc_assign(&mut self, rhs: $int) {
					self.0 -= self.clone().div_euc(rhs).0 * rhs;
				}

				fn rem_floor_assign(&mut self, rhs: $int) {
					self.0 -= self.clone().div_floor(rhs).0 * rhs;
				}

				fn rem_trunc_assign(&mut self, rhs: $int) {
					self.0 -= self.clone().div_trunc(rhs).0 * rhs;
				}
			}

			impl SubAssign<$int> for Number {
				fn sub_assign(&mut self, rhs: $int) {
					self.0 -= rhs;
				}
			}
		)*
	};
}

macro_rules! impl_rational {
	($($name:ident, $name_mut:ident, $doc:literal;)*) => {
		impl Number {
			$(
				#[doc = concat!("Gets the ", $doc, " of this number.")]
				pub fn $name(self) -> Self {
					Self(self.0.$name())
				}

				#[doc = concat!("Gets the ", $doc, " of this number and assigns it in-place.")]
				pub fn $name_mut(&mut self) {
					self.0.$name_mut();
				}
			)*
		}
	};
}

impl_float! {
	f32, f64,
}

impl_int! {
	i8, i16, i32, i64, i128, isize,
	u8, u16, u32, u64, u128, usize,
}

impl_rational! {
	abs, abs_mut, "absolute value";
	ceil, ceil_mut, "ceiled integer";
	floor, floor_mut, "floored integer";
	recip, recip_mut, "reciprocal value";
	round, round_mut, "rounded integer";
	signum, signum_mut, "sign";
	square, square_mut, "squared value";
	trunc, trunc_mut, "truncated integer";
}
