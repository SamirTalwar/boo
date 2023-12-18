use std::time::Instant;

use anyhow::anyhow;
use proptest::prelude::*;
use proptest::strategy::ValueTree;
use proptest::test_runner::TestRunner;

use boo::evaluation::Evaluator;
use boo::identifier::*;
use boo::*;

fn main() -> anyhow::Result<()> {
    let any_expr = boo_generator::gen(
        boo_generator::ExprGenConfig {
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

    let core_expr = expr.to_core()?;

    let mut evaluator = OptimizedEvaluator::new();
    builtins::prepare(&mut evaluator)?;

    let start_time = Instant::now();
    let result = evaluator
        .evaluate(core_expr)
        .expect("Could not interpret the expression.");
    let end_time = Instant::now();
    println!("Result:\n{}", result);

    println!("\nEvaluation took {:?}.", end_time - start_time);

    Ok(())
}
