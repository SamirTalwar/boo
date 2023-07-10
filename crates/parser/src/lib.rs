//! Transforms an input string into an evaluatable program.

pub mod lexer;
pub mod parser;
pub mod rewriter;

use boo_core::error::Result;

pub use lexer::lex;
pub use parser::parse_tokens;
pub use rewriter::rewrite;

pub fn parse(input: &str) -> Result<boo_core::expr::Expr> {
    let tokens = lex(input)?;
    let tree = parse_tokens(&tokens)?;
    Ok(rewrite(tree))
}
