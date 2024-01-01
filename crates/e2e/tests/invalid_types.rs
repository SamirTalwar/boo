use boo::error::{Error, Result};
use boo::types::{Type, TypeVariable};
use boo::*;

#[test]
fn test_rejects_incorrect_types() -> Result<()> {
    expect_error(
        "rejects_incorrect_types",
        "1 + (fn x -> 3)",
        Error::TypeUnificationError {
            left_span: Some((0..14).into()),
            left_type: Type::Function {
                parameter: Type::Integer.into(),
                body: Type::Integer.into(),
            }
            .into(),
            right_span: Some((5..14).into()),
            right_type: Type::Function {
                parameter: Type::Variable(TypeVariable::new_from_str("_3")).into(),
                body: Type::Integer.into(),
            }
            .into(),
        },
    )
}

#[test]
fn test_match_expressions_must_be_of_the_same_type() -> Result<()> {
    expect_error(
        "match_expressions_must_be_of_the_same_type",
        "match 0 { 1 -> 2; _ -> fn x -> x }",
        Error::TypeUnificationError {
            left_span: Some((15..16).into()),
            left_type: Type::Integer.into(),
            right_span: Some((23..32).into()),
            right_type: Type::Function {
                parameter: Type::Variable(TypeVariable::new_from_str("_1")).into(),
                body: Type::Variable(TypeVariable::new_from_str("_1")).into(),
            }
            .into(),
        },
    )
}

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

    let result = boo_types_hindley_milner::w(&ast);
    assert_eq!(result, Err(expected_error));

    Ok(())
}
