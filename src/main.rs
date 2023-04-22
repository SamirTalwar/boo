pub mod ast;
pub mod error;
pub mod interpreter;
pub mod parser;

use crate::ast::*;
use crate::error::BooError;
use crate::interpreter::interpret;
use crate::parser::parse;

fn main() -> Result<(), BooError> {
    let mut args = std::env::args();
    args.next();
    let Some(input) = args.next() else {
        panic!("No input.")
    };
    let expr = parse(&input)?;
    let result = interpret(expr);

    match result {
        Expr::Primitive {
            annotation: _,
            value,
        } => {
            println!("{}", value);
            Ok(())
        }
        result => panic!("Invalid result: {:?}", result),
    }
}
