use std::iter::Peekable;

use abacas::context::{Context, Symbol};
use abacas::expr::Expr;
use abacas::monomial::Monomial;
use abacas::number::Number;
use rug::ops::Pow;

use crate::token::Token::{self, *};

pub struct Parser {}

impl Parser {
	pub fn parse_line(ctx: &mut Context, line: Vec<Token>) -> Expr {
		let mut it = line.into_iter().peekable();

		Self::expr_bp(ctx, 0, &mut it)
	}

	fn expr_bp<T>(ctx: &mut Context, min_bp: u8, tokens: &mut Peekable<T>) -> Expr
	where
		T: Iterator<Item = Token>,
	{
		let mut lhs = match tokens.next() {
			Some(Sub) => -Self::expr_bp(ctx, prefix_bp(Sub), tokens),
			Some(Number(num)) => Expr::Num(num),
			Some(Ident(name)) => {
				if tokens.peek().is_some_and(|x| *x == LParen) {
					let mut depth = 0;
					let mut params = vec![];
					let mut expression = vec![];

					loop {
						let token = tokens.next();

						if token.is_none() {
							break;
						}

						let token = token.unwrap();

						if token == RParen {
							if depth == 0 {
								break;
							}
							depth -= 1;
						}

						if token == LParen {
							depth += 1;
						}

						if token == Comma && depth == 0 {
							let mut it = expression.clone().into_iter().peekable();
							let data = Self::expr_bp(ctx, 0, &mut it);

							params.push(data);
							expression.clear();

							continue;
						}

						expression.push(token);
					}

					if !expression.is_empty() {
						let mut it = expression.clone().into_iter().peekable();
						let data = Self::expr_bp(ctx, 0, &mut it);

						params.push(data);
						expression.clear();
					}

					Expr::Fun(Symbol::new(name).expect("Error while parsing symbol"), params)
				} else {
					Expr::Poly(
						Symbol::new(name).expect("Error while parsing symbol"),
						Monomial::linear(Number::one()).into(),
					)
				}
			}
			Some(LParen) => {
				let lhs = Self::expr_bp(ctx, 0, tokens);
				assert_eq!(tokens.next(), Some(RParen));

				lhs
			}
			_ => panic!("Bad token"),
		};

		loop {
			match tokens.peek() {
				Some(t @ (Add | Sub | Mul | Div | Rem | Pow | Eq)) => {
					let (l_bp, r_bp) = infix_bp(t.clone());

					if l_bp < min_bp {
						break;
					}

					let op = tokens.next().unwrap();
					let rhs = Self::expr_bp(ctx, r_bp, tokens);

					match op {
						Eq => {
							if let Expr::Poly(ref name, ref poly) = lhs {
								if poly.degree().is_some_and(|x| x.is_one())
									&& poly.monomials().count() == 1
									&& poly.get(&Number::one()).unwrap().coeff.is_one()
								{
									ctx.variables.insert(name.clone(), rhs.clone());
									lhs = rhs;
								}
							} else {
								unimplemented!()
							}
						}
						Add => lhs = lhs + rhs,
						Sub => lhs = lhs - rhs,
						Mul => lhs = lhs * rhs,
						Div => lhs = lhs / rhs,
						Pow => lhs = lhs.pow(rhs),
						Rem => todo!(),

						_ => unreachable!(),
					}
				}
				Some(Number(_) | Ident(_) | LParen) => {
					let (l_bp, r_bp) = infix_bp(Token::Mul);
					if l_bp < min_bp {
						break;
					}

					let rhs = Self::expr_bp(ctx, r_bp, tokens);

					lhs = lhs * rhs;
				}
				Some(Comma) => unreachable!(),
				None | Some(RParen) => break,
			}
		}

		lhs
	}
}

pub fn infix_bp(op: Token) -> (u8, u8) {
	match op {
		Eq => (1, 2),
		Sub | Add => (3, 4),
		Mul | Div | Rem => (5, 6),
		Pow => (8, 7),

		_ => unreachable!(),
	}
}

pub fn prefix_bp(op: Token) -> u8 {
	match op {
		Sub => 7,

		_ => unreachable!(),
	}
}
