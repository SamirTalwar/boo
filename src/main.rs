pub mod ast;
pub mod error;
pub mod interpreter;
pub mod parser;

use crate::ast::*;
use crate::interpreter::interpret;

fn main() {
    let expr = Expr::Infix {
        annotation: (),
        operation: Operation::Multiply,
        left: Box::new(Expr::Infix {
            annotation: (),
            operation: Operation::Add,
            left: Box::new(Expr::Primitive {
                annotation: (),
                value: Primitive::Int(2),
            }),
            right: Box::new(Expr::Primitive {
                annotation: (),
                value: Primitive::Int(3),
            }),
        }),
        right: Box::new(Expr::Primitive {
            annotation: (),
            value: Primitive::Int(7),
        }),
    };
    let result = interpret(expr);

    match result {
        Expr::Primitive {
            annotation: _,
            value,
        } => println!("{}", value),
        result => panic!("Invalid result: {:?}", result),
    }
}
