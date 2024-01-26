use boo::error::Result;
use boo::evaluation::Evaluator;
use boo::types::{Monotype, Type};
use boo::*;
use boo_naive_evaluator::NaiveEvaluator;
use boo_optimized_evaluator::OptimizedEvaluator;

#[test]
fn test_integer() -> Result<()> {
    check_program("integer", "123", Type::Integer.into(), "123")
}

#[test]
fn test_mathematical_operators() -> Result<()> {
    check_program(
        "mathematical_operators",
        "7 + 3 * 5 - 2",
        Type::Integer.into(),
        "20",
    )
}

#[test]
fn test_overriding_precedence() -> Result<()> {
    check_program(
        "overriding_precedence",
        "2 * (3 + 4)",
        Type::Integer.into(),
        "14",
    )
}

#[test]
fn test_function_application() -> Result<()> {
    check_program(
        "function_application",
        "(fn x -> x + x) 9",
        Type::Integer.into(),
        "18",
    )
}

#[test]
fn test_function_application_with_computation() -> Result<()> {
    check_program(
        "function_application_with_computation",
        "(fn x y -> x + y) (8 * 2) (3 * 4)",
        Type::Integer.into(),
        "28",
    )
}

#[test]
fn test_assignment() -> Result<()> {
    check_program(
        "assignment",
        "let seven = 7 in seven",
        Type::Integer.into(),
        "7",
    )
}

#[test]
fn test_assignment_and_use() -> Result<()> {
    check_program(
        "assignment_and_use",
        "let eight = 8 in eight * 3",
        Type::Integer.into(),
        "24",
    )
}

#[test]
fn test_named_function_application() -> Result<()> {
    check_program(
        "named_function_application",
        "let double = fn input -> input + input in double 6",
        Type::Integer.into(),
        "12",
    )
}

#[test]
fn test_named_function_application_nested() -> Result<()> {
    check_program(
        "named_function_application_nested",
        "let double = fn input -> input + input in double (double 4)",
        Type::Integer.into(),
        "16",
    )
}

#[test]
fn test_function_application_with_named_argument() -> Result<()> {
    check_program(
        "function_application_with_named_argument",
        "let value = 99 in (fn wibble -> wibble - 1) value",
        Type::Integer.into(),
        "98",
    )
}

#[test]
fn test_named_function_application_with_named_argument() -> Result<()> {
    check_program(
        "named_function_application_with_named_argument",
        "let negate = fn thing -> 0 - thing in let life = 42 in negate life",
        Type::Integer.into(),
        "-42",
    )
}

#[test]
fn test_closing_over_a_variable() -> Result<()> {
    check_program(
        "closing_over_a_variable",
        "let something = 12 in let add_something = fn target -> target + something in add_something 9",
        Type::Integer.into(),
        "21",
    )
}

#[test]
fn test_polymorphic_let() -> Result<()> {
    check_program(
        "polymorphic_let",
        "let id = fn x -> x in id id id (id 7)",
        Type::Integer.into(),
        "7",
    )
}

#[test]
fn test_pattern_matching_on_integers() -> Result<()> {
    check_program(
        "pattern_matching_on_integers",
        "match 2 { 1 -> 2; 2 -> 3; 3 -> 4; _ -> 0 }",
        Type::Integer.into(),
        "3",
    )
}

#[test]
fn test_pattern_matching_on_functions() -> Result<()> {
    check_program(
        "pattern_matching_on_functions",
        "(match 1 { 1 -> fn x -> 2; _ -> fn x -> x }) 3",
        Type::Integer.into(),
        "2",
    )
}

#[test]
fn test_expression_type_annotations() -> Result<()> {
    check_program(
        "expression_type_annotations",
        "let id_int = fn x -> (x: Integer) in id_int (1 + (2: Integer))",
        Type::Integer.into(),
        "3",
    )
}

fn check_program(
    name: &str,
    program: &str,
    expected_type: Monotype,
    expected_result_str: &str,
) -> Result<()> {
    let ast = parse(program)?.to_core()?;
    insta::with_settings!({ description => program }, {
        insta::assert_debug_snapshot!(name.to_string() + "__parse", ast);
    });

    let expected_result = match parse(expected_result_str)?.to_core()?.take() {
        ast::Expression::Primitive(primitive) => evaluation::Evaluated::Primitive(primitive),
        expression => panic!("Expected result that is not a primitive: {:?}", expression),
    };

    let actual_type = boo_types_hindley_milner::type_of(&ast)?;
    assert_eq!(actual_type, expected_type);

    let mut optimized_evaluator = OptimizedEvaluator::new();
    builtins::prepare(&mut optimized_evaluator)?;
    let mut naive_evaluator = NaiveEvaluator::new();
    builtins::prepare(&mut naive_evaluator)?;

    let efficient_result = optimized_evaluator.evaluate(ast.clone())?;
    assert_eq!(efficient_result, expected_result);

    let naive_result = naive_evaluator.evaluate(ast)?;
    assert_eq!(naive_result, expected_result);

    Ok(())
}
