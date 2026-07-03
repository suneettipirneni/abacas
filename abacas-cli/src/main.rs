use abacas::VERSION;
use abacas::context::Context;
use argh::FromArgs;
use dark_light::{Mode, detect};
use logos::Logos;

mod parser;
mod token;

use std::borrow::Cow::{self, Borrowed, Owned};
use std::fmt::Write;
use std::process::exit;

use rustyline::error::ReadlineError;
use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::validate::MatchingBracketValidator;
use rustyline::{Completer, Config, Editor, Helper, Hinter, Validator};
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

use crate::parser::Parser;
use crate::token::Token;

#[derive(FromArgs)]
/// Configuation options for abacas. Pass no arguments for REPL
struct CasConfig {
	#[argh(option, short = 'e')]
	/// mathematical expression to run through the CAS.
	expr: Option<String>,

	#[argh(switch)]
	/// prevent the CAS from folding the parsed expr
	raw: bool,
}

fn main() {
	let cfg: CasConfig = argh::from_env();

	if cfg.expr.is_none() {
		repl(cfg);
		return;
	}

	let exp = cfg.expr.unwrap();
	let tokens = Token::lexer(&exp).collect::<Result<Vec<Token>, ()>>().unwrap();

	let mut ctx = Context::new();
	let mut ast = Parser::parse_line(&mut ctx, tokens);

	if !cfg.raw {
		ast = ast.simplify(&ctx).expect("Error while simplifying");
	}

	println!("{ast}");

	if !ast.is_num()
		&& let Ok(float) = ast.evaluate(&ctx)
	{
		println!("Approximation: {float}")
	}
}

#[derive(Helper, Completer, Hinter, Validator)]
struct HighlightHelper {
	#[rustyline(Validator)]
	validator: MatchingBracketValidator,
	colored_prompt: String,
}

impl Highlighter for HighlightHelper {
	fn highlight_prompt<'b, 's: 'b, 'p: 'b>(&'s self, prompt: &'p str, default: bool) -> Cow<'b, str> {
		if default {
			Borrowed(&self.colored_prompt)
		} else {
			Borrowed(prompt)
		}
	}

	fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
		Owned("\x1b[1m".to_owned() + hint + "\x1b[m")
	}

	fn highlight<'l>(&self, line: &'l str, _: usize) -> Cow<'l, str> {
		let ps = SyntaxSet::load_defaults_newlines();
		let ts = ThemeSet::load_defaults();

		let light_theme = ts.themes["base16-ocean.light"].clone();
		let dark_theme = ts.themes["base16-mocha.dark"].clone();

		let theme = match detect().unwrap_or(Mode::Unspecified) {
			Mode::Dark => dark_theme,
			_ => light_theme,
		};

		let syntax = ps.find_syntax_by_extension("rs").unwrap();
		let mut highlighter = HighlightLines::new(syntax, &theme);

		let highlighted =
			highlighter
				.highlight_line(line, &ps)
				.unwrap()
				.into_iter()
				.fold(String::new(), |mut acc, (style, text)| {
					let _ = write!(
						acc,
						"\x1b[38;2;{};{};{}m{}\x1b[0m",
						style.foreground.r, style.foreground.g, style.foreground.b, text
					);

					acc
				});

		Cow::Owned(highlighted)
	}

	fn highlight_char(&self, _: &str, _: usize, _: CmdKind) -> bool {
		true
	}
}

fn repl(cfg: CasConfig) {
	println!("Welcome to abacas v{}\nTo exit, press CTRL+C or CTRL+D", VERSION);

	let config = Config::builder().build();

	let h = HighlightHelper {
		colored_prompt: "".to_owned(),
		validator: MatchingBracketValidator::new(),
	};
	let mut rl = Editor::with_config(config).unwrap();
	rl.set_helper(Some(h));

	let mut ctx = Context::new();

	loop {
		"\x1b[1m\x1b[32m[In]:\x1b[0m ".clone_into(&mut rl.helper_mut().expect("No helper").colored_prompt);

		let readline = rl.readline("\x1b[1m\x1b[32m[In]:\x1b[0m ");

		match readline {
			Ok(line) => {
				if line.trim() == "exit" {
					break;
				}

				println!("\x1b[1m\x1b[31m[Out]:\x1b[0m ");

				let tokens = Token::lexer(&line).collect::<Result<Vec<Token>, ()>>().unwrap();

				let mut ast = Parser::parse_line(&mut ctx, tokens);

				if !cfg.raw {
					ast = ast.simplify(&ctx).unwrap();
				}

				println!("{ast}");

				if !ast.is_num()
					&& let Ok(float) = ast.evaluate(&ctx)
				{
					println!("Approximation: {float}")
				}
			}
			Err(ReadlineError::Interrupted) => {
				println!("CTRL-C");
				break;
			}
			Err(ReadlineError::Eof) => {
				println!("CTRL-D");
				break;
			}
			Err(err) => {
				println!("Error: {err:?}");
				break;
			}
		}
	}
	exit(1);
}
