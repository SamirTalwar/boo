use std::time::Instant;

use anyhow::anyhow;
use proptest::prelude::*;
use proptest::strategy::ValueTree;
use proptest::test_runner::TestRunner;

use boo::*;
use boo_core::identifier::*;
use boo_parser::generators;

fn main() -> anyhow::Result<()> {
    let any_expr = generators::gen(
        generators::ExprGenConfig {
            gen_identifier: Identifier::gen_ascii(1..=16).boxed().into(),
            ..Default::default()
        }
        .into(),
    );
    let mut runner = TestRunner::default();
    let tree = any_expr
        .new_tree(&mut runner)
        .map_err(|err| anyhow!("Generation failed: {}", err))?;

    let expr = tree.current();
    println!("Expression:\n{}\n", expr);

    let start_time = Instant::now();
    let result = evaluate(expr).expect("Could not interpret the expression.");
    let end_time = Instant::now();
    println!("Result:\n{}", result);

    println!("\nEvaluation took {:?}.", end_time - start_time);

    Ok(())
}
