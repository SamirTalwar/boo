use anyhow::anyhow;
use proptest::prelude::*;
use proptest::strategy::ValueTree;
use proptest::test_runner::TestRunner;

use boo::ast::*;
use boo::*;

fn main() -> anyhow::Result<()> {
    let any_expr = Expr::arbitrary();
    let mut runner = TestRunner::default();
    let tree = any_expr
        .new_tree(&mut runner)
        .map_err(|err| anyhow!("Generation failed: {}", err))?;

    let expr = tree.current();
    println!("Expression:\n{}\n", expr);

    let result = evaluate(expr.into()).expect("Could not interpret the expression.");
    println!("Result:\n{}", result);

    Ok(())
}
