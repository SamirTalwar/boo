//! Transforms an input string into an evaluatable program.

pub mod ast;
pub mod lexer;
pub mod parser;

pub use boo_core::error::Result;

pub use ast::Expr;
pub use lexer::lex;
pub use parser::parse_tokens;

pub fn parse(input: &str) -> Result<Expr> {
    let tokens = lex(input)?;
    parse_tokens(&tokens)
}
