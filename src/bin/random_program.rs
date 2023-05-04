use rand::RngCore;

use boo::ast::*;
use boo::interpreter::*;

fn main() {
    let unstructured_size = rand::random::<u16>();
    let mut unstructured_bytes = vec![0; unstructured_size.into()];
    rand::thread_rng().fill_bytes(&mut unstructured_bytes);
    let mut unstructured = arbitrary::Unstructured::new(&unstructured_bytes);
    let expr = unstructured.arbitrary::<Expr<()>>().unwrap();
    println!("Expression:\n{}\n", expr);
    let result = interpret(expr.into()).expect("Could not interpret the expression.");
    println!("Result:\n{}", result);
}
