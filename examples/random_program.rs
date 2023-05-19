use std::rc::Rc;

use anyhow::anyhow;
use proptest::prelude::*;
use proptest::strategy::ValueTree;
use proptest::test_runner::TestRunner;

use boo::ast::*;
use boo::identifier::*;
use boo::*;

fn main() -> anyhow::Result<()> {
    let any_expr = ast::generators::gen(Rc::new(ExprGenConfig {
        gen_identifier: Rc::new(Identifier::gen_ascii(1..=16).boxed()),
        ..Default::default()
    }));
    let mut runner = TestRunner::default();
    let tree = any_expr
        .new_tree(&mut runner)
        .map_err(|err| anyhow!("Generation failed: {}", err))?;

    let expr = tree.current();
    println!("Expression:\n{}\n", expr);

    let result = evaluate(expr).expect("Could not interpret the expression.");
    println!("Result:\n{}", result);

    Ok(())
}
