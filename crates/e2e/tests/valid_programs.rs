use boo::error::Result;
use boo::*;
use boo_naive_evaluator::naively_evaluate;

#[test]
fn test_integer() -> Result<()> {
    check_program("integer", "123", "123")
}

#[test]
fn test_mathematical_operators() -> Result<()> {
    check_program("mathematical_operators", "7 + 3 * 5 - 2", "20")
}

#[test]
fn test_overriding_precedence() -> Result<()> {
    check_program("overriding_precedence", "2 * (3 + 4)", "14")
}

#[test]
fn test_function_application() -> Result<()> {
    check_program("function_application", "(fn x -> x + x) 9", "18")
}

#[test]
fn test_function_application_with_computation() -> Result<()> {
    check_program(
        "function_application_with_computation",
        "(fn x -> fn y -> x + y) (8 * 2) (3 * 4)",
        "28",
    )
}

#[test]
fn test_assignment() -> Result<()> {
    check_program("assignment", "let seven = 7 in seven", "7")
}

#[test]
fn test_assignment_and_use() -> Result<()> {
    check_program("assignment_and_use", "let eight = 8 in eight * 3", "24")
}

#[test]
fn test_named_function_application() -> Result<()> {
    check_program(
        "named_function_application",
        "let double = fn input -> input + input in double 6",
        "12",
    )
}

#[test]
fn test_named_function_application_nested() -> Result<()> {
    check_program(
        "named_function_application_nested",
        "let double = fn input -> input + input in double (double 4)",
        "16",
    )
}

#[test]
fn test_function_application_with_named_argument() -> Result<()> {
    check_program(
        "function_application_with_named_argument",
        "let value = 99 in (fn wibble -> wibble - 1) value",
        "98",
    )
}

#[test]
fn test_named_function_application_with_named_argument() -> Result<()> {
    check_program(
        "named_function_application_with_named_argument",
        "let negate = fn thing -> 0 - thing in let life = 42 in negate life",
        "-42",
    )
}

#[test]
fn test_closing_over_a_variable() -> Result<()> {
    check_program(
        "closing_over_a_variable",
        "let something = 12 in let add_something = fn target -> target + something in add_something 9",
        "21",
    )
}

fn check_program(name: &str, program: &str, expected_result_str: &str) -> Result<()> {
    let ast = parse(program)?;
    insta::with_settings!({ description => program }, {
        insta::assert_debug_snapshot!(name.to_string() + "__parse", ast);
    });

    let expected_result = match *parse(expected_result_str)?.expression {
        ast::Expression::Primitive(p) => p,
        expression => panic!("Expected result that is not a primitive: {:?}", expression),
    };

    let expr = boo::builtins::prepare(ast);
    let efficient_result = evaluate(expr.clone())?;
    assert_eq!(
        efficient_result,
        evaluator::Evaluated::Primitive(expected_result.clone())
    );

    let naive_result = naively_evaluate(expr)?;
    assert_eq!(
        *naive_result.expression,
        ast::Expression::Primitive(expected_result)
    );

    Ok(())
}
