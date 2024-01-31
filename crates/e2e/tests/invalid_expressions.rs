use boo::error::{Error, Result};
use boo::evaluation::Evaluator;
use boo::*;
use boo_evaluation_reduction::ReducingEvaluator;
use boo_optimized_evaluator::PoolingEvaluator;

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

    let type_check_result = boo_types_hindley_milner::type_of(&ast);
    assert_eq!(type_check_result, Err(expected_error.clone()));

    {
        let mut reducing_evaluator = ReducingEvaluator::new();
        builtins::prepare(&mut reducing_evaluator)?;
        let actual_result = reducing_evaluator.evaluate(ast.clone());
        assert_eq!(actual_result, Err(expected_error.clone()));
    }

    {
        let mut optimized_evaluator = PoolingEvaluator::new_recursive();
        builtins::prepare(&mut optimized_evaluator)?;
        let actual_result = optimized_evaluator.evaluate(ast);
        assert_eq!(actual_result, Err(expected_error));
    }

    Ok(())
}
