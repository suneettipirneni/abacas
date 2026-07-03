use abacas::context::{Context, Symbol};
use abacas::expr::Expr;

const ADD: fn(Vec<Expr>, ctx: &Context) -> Expr = |exprs, ctx| Expr::Add(exprs).simplify(ctx).unwrap();
const MUL: fn(Vec<Expr>, ctx: &Context) -> Expr = |exprs, ctx| Expr::Mul(exprs).simplify(ctx).unwrap();

const COS: fn(Expr) -> Expr = |arg| Expr::Fun(Symbol::new("cos".into()).unwrap(), vec![arg]);
const NUM: fn(i8) -> Expr = |num| Expr::Num(num.into());

const X: fn(&str) -> Expr = |poly| Expr::Poly(Symbol::new("x".into()).unwrap(), poly.parse().unwrap());
const Y: fn(&str) -> Expr = |poly| Expr::Poly(Symbol::new("y".into()).unwrap(), poly.parse().unwrap());

#[test]
fn add() {
	let ctx = &Context::new();

	let expr = ADD(vec![], ctx);
	assert_eq!(expr.to_string(), "0");

	let expr = ADD(vec![NUM(2), X("x"), X("x + 2"), Y("x"), Y("-x + 2")], ctx);
	assert_eq!(expr.to_string(), "2x + 6");

	let expr = ADD(vec![NUM(2), X("x"), X("x + 2"), Y("x"), Y("-2x + 1")], ctx);
	assert_eq!(expr.to_string(), "5 + 2x - y");

	let expr = ADD(vec![NUM(2), COS(NUM(0)), NUM(-3), COS(NUM(0))], ctx);
	assert_eq!(expr.to_string(), "cos(0) * 2 - 1");
}

#[test]
fn mul() {
	let ctx = &Context::new();

	let expr = MUL(vec![], ctx);
	assert_eq!(expr.to_string(), "1");

	let expr = MUL(vec![NUM(2), X("x"), X("x + 2"), Y("x"), Y("2x^-1")], ctx);
	assert_eq!(expr.to_string(), "4x^2 + 8x");

	let expr = MUL(vec![NUM(2), X("x"), X("x + 2"), Y("x"), Y("x^-1 + 2")], ctx);
	assert_eq!(expr.to_string(), "4 * (x^2 + 2x) * (y + 0.5)");

	let expr = MUL(vec![NUM(2), COS(NUM(0)), NUM(-3), COS(NUM(0))], ctx);
	assert_eq!(expr.to_string(), "-6 * cos(0)^2");
}
