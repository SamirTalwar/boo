#![cfg(test)]

use crate::ast::Expr;
use crate::lexer::lex;
use crate::parser::parse;

#[test]
fn test_rendering_and_parsing_an_expression() {
    arbtest::builder().budget_ms(250).run(|u| {
        let input = u.arbitrary::<Expr<()>>()?;
        let rendered = format!("{}", input);
        let lexed = lex(&rendered).unwrap_or_else(|err| {
            panic!(
                "Could not lex: {:?}",
                miette::Report::from(err).with_source_code(rendered.clone())
            )
        });
        let parsed = parse(&lexed).unwrap_or_else(|err| {
            panic!(
                "Could not parse: {:?}",
                miette::Report::from(err).with_source_code(rendered.clone())
            )
        });
        assert!(
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
    match (left, right) {
        (
            Expr::Primitive {
                value: left_value, ..
            },
            Expr::Primitive {
                value: right_value, ..
            },
        ) => left_value == right_value,
        (
            Expr::Identifier {
                name: left_name, ..
            },
            Expr::Identifier {
                name: right_name, ..
            },
        ) => left_name == right_name,
        (
            Expr::Let {
                name: left_name,
                value: left_value,
                inner: left_inner,
                ..
            },
            Expr::Let {
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
            Expr::Infix {
                operation: left_operation,
                left: left_left,
                right: left_right,
                ..
            },
            Expr::Infix {
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
