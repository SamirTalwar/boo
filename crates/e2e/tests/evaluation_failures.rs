use boo::error::{Error, Result};
use boo::evaluation::Evaluator;
use boo::*;
use boo_naive_evaluator::NaiveEvaluator;
use boo_optimized_evaluator::OptimizedEvaluator;

#[test]
fn test_unknown_variable() -> Result<()> {
    expect_error(
        "unknown_variable",
        "123 + xyz",
        Error::UnknownVariable {
            span: Some((6..9).into()),
            name: "xyz".to_string(),
        },
    )
}

#[test]
fn test_does_not_close_over_variables_out_of_scope() -> Result<()> {
    expect_error(
        "does_not_close_over_variables_out_of_scope",
        "let fun = (let one = 1 in fn param -> one + param + external) in let external = 2 in fun 3",
        Error::UnknownVariable {
            span: Some((52..60).into()),
            name: "external".to_string(),
        },
    )
}

fn expect_error(name: &str, program: &str, expected_error: Error) -> Result<()> {
    let ast = parse(program)?.to_core()?;
    insta::with_settings!({ description => program }, {
        insta::assert_debug_snapshot!(name.to_string() + "__parse", ast);
    });

    let mut optimized_evaluator = OptimizedEvaluator::new();
    builtins::prepare(&mut optimized_evaluator)?;
    let mut naive_evaluator = NaiveEvaluator::new();
    builtins::prepare(&mut naive_evaluator)?;

    let efficient_result = optimized_evaluator.evaluate(ast.clone());
    assert_eq!(efficient_result, Err(expected_error.clone()));

    let naive_result = naive_evaluator.evaluate(ast);
    assert_eq!(naive_result, Err(expected_error));

    Ok(())
}
