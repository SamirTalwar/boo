use std::rc::Rc;
use std::time::Instant;

use anyhow::anyhow;
use proptest::prelude::*;
use proptest::strategy::ValueTree;
use proptest::test_runner::TestRunner;

use boo::identifier::*;
use boo::*;

fn main() -> anyhow::Result<()> {
    let any_expr = parser::generators::gen(Rc::new(parser::generators::ExprGenConfig {
        gen_identifier: Rc::new(Identifier::gen_ascii(1..=16).boxed()),
        ..Default::default()
    }));
    let mut runner = TestRunner::default();
    let tree = any_expr
        .new_tree(&mut runner)
        .map_err(|err| anyhow!("Generation failed: {}", err))?;

    let expr = tree.current();
    println!("Expression:\n{}\n", expr);

    let start_time = Instant::now();
    let pool = pool_exprs(expr);
    let result = evaluate(&pool).expect("Could not interpret the expression.");
    let end_time = Instant::now();
    println!("Result:\n{}", result);

    println!("\nEvaluation took {:?}.", end_time - start_time);

    Ok(())
}
