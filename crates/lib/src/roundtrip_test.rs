#![cfg(test)]

use proptest::prelude::*;

use crate::ast::*;
use crate::proptest_helpers::check;
use crate::*;

#[test]
fn test_rendering_and_parsing_an_expression() {
    check(&ast::generators::arbitrary(), |input| {
        let rendered = format!("{}", input);
        let lexed = lex(&rendered)?;
        let parsed = parse(&lexed)?;
        prop_assert!(
            eq_ignoring_annotations(&parsed, &input),
            "{} and {} were not equal\nLexed: {:?}",
            &parsed,
            &input,
            &lexed,
        );
        Ok(())
    })
}

fn eq_ignoring_annotations<LeftAnnotation, RightAnnotation>(
    left: &Expr<LeftAnnotation>,
    right: &Expr<RightAnnotation>,
) -> bool {
    match (&left.value, &right.value) {
        (
            Expression::Primitive {
                value: left_value, ..
            },
            Expression::Primitive {
                value: right_value, ..
            },
        ) => left_value == right_value,
        (
            Expression::Identifier {
                name: left_name, ..
            },
            Expression::Identifier {
                name: right_name, ..
            },
        ) => left_name == right_name,
        (
            Expression::Let {
                name: left_name,
                value: left_value,
                inner: left_inner,
                ..
            },
            Expression::Let {
                name: right_name,
                value: right_value,
                inner: right_inner,
                ..
            },
        ) => {
            left_name == right_name
                && eq_ignoring_annotations(left_value, right_value)
                && eq_ignoring_annotations(left_inner, right_inner)
        }
        (
            Expression::Infix {
                operation: left_operation,
                left: left_left,
                right: left_right,
                ..
            },
            Expression::Infix {
                operation: right_operation,
                left: right_left,
                right: right_right,
                ..
            },
        ) => {
            left_operation == right_operation
                && eq_ignoring_annotations(left_left, right_left)
                && eq_ignoring_annotations(left_right, right_right)
        }
        _ => false,
    }
}
