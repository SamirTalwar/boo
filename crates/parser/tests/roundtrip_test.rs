use proptest::prelude::*;

use boo_core::ast::*;
use boo_lexer::*;
use boo_parser::generators;
use boo_parser::*;
use boo_test_helpers::proptest::*;

#[test]
fn test_rendering_and_parsing_an_expression() {
    check(&generators::arbitrary(), |input| {
        let rendered = format!("{}", input);
        let lexed = lex(&rendered)?;
        let parsed = parse(&lexed)?;
        prop_assert!(
            eq_ignoring_span(&input, &parsed),
            "{} and {} were not equal\nInput:  {:#?}\nParsed: {:#?}",
            &input,
            &parsed,
            &input,
            &parsed,
        );
        Ok(())
    })
}

fn eq_ignoring_span(left: &Expr, right: &Expr) -> bool {
    match (&left.0.value, &right.0.value) {
        (Expression::Primitive(left_value), Expression::Primitive(right_value)) => {
            left_value == right_value
        }
        (Expression::Identifier(left_name), Expression::Identifier(right_name)) => {
            left_name == right_name
        }
        (
            Expression::Assign(Assign {
                name: left_name,
                value: left_value,
                inner: left_inner,
            }),
            Expression::Assign(Assign {
                name: right_name,
                value: right_value,
                inner: right_inner,
            }),
        ) => {
            left_name == right_name
                && eq_ignoring_span(left_value, right_value)
                && eq_ignoring_span(left_inner, right_inner)
        }
        (
            Expression::Function(Function {
                parameter: left_parameter,
                body: left_body,
            }),
            Expression::Function(Function {
                parameter: right_parameter,
                body: right_body,
            }),
        ) => left_parameter == right_parameter && eq_ignoring_span(left_body, right_body),
        (
            Expression::Apply(Apply {
                function: left_function,
                argument: left_argument,
            }),
            Expression::Apply(Apply {
                function: right_function,
                argument: right_argument,
            }),
        ) => {
            eq_ignoring_span(left_function, right_function)
                && eq_ignoring_span(left_argument, right_argument)
        }
        (
            Expression::Infix(Infix {
                operation: left_operation,
                left: left_left,
                right: left_right,
                ..
            }),
            Expression::Infix(Infix {
                operation: right_operation,
                left: right_left,
                right: right_right,
                ..
            }),
        ) => {
            left_operation == right_operation
                && eq_ignoring_span(left_left, right_left)
                && eq_ignoring_span(left_right, right_right)
        }
        _ => false,
    }
}
