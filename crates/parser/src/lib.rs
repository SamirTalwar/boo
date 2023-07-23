//! Transforms an input string into an evaluatable program.

pub mod lexer;
pub mod parser;

use boo_core::error::Result;
use boo_language::Expr;

pub fn parse(input: &str) -> Result<Expr> {
    let tokens = lexer::lex(input)?;
    parser::parse_tokens(&tokens)
}
