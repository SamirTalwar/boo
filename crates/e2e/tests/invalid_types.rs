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

fn expect_error(name: &str, program: &str, expected_error: Error) -> Result<()> {
    let ast = parse(program)?.to_core()?;
    insta::with_settings!({ description => program }, {
        insta::assert_debug_snapshot!(name.to_string() + "__parse", ast);
    });

    let result = boo_types_hindley_milner::type_of(&ast);
    assert_eq!(result, Err(expected_error));

    Ok(())
}
