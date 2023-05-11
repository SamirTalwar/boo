use rand::RngCore;

use boo::ast::*;
use boo::*;

fn main() {
    let mut unstructured_bytes = vec![0; 0x10000];
    rand::thread_rng().fill_bytes(&mut unstructured_bytes);
    let mut unstructured = arbitrary::Unstructured::new(&unstructured_bytes);

    let expr = unstructured.arbitrary::<Expr<()>>().unwrap();
    println!("Expression:\n{}\n", expr);

    let result = evaluate(expr.into()).expect("Could not interpret the expression.");
    println!("Result:\n{}", result);
}
