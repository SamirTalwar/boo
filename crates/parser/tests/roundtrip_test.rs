use proptest::prelude::*;

use boo_language::*;
use boo_parser::{lex, parse_tokens};
use boo_test_helpers::proptest::*;

#[test]
fn test_rendering_and_parsing_an_expression() {
    check(&boo_generator::arbitrary(), |input| {
        let rendered = format!("{}", input);
        let lexed = lex(&rendered)?;
        let parsed = parse_tokens(&lexed)?;
        let despanned = remove_spans(parsed);
        prop_assert_eq!(input, despanned);
        Ok(())
    })
}

pub fn remove_spans(expr: Expr) -> Expr {
    Expr::new(
        0.into(), // Replacement span to ensure they don't interfere with testing.
        match *expr.expression {
            Expression::Primitive(x) => Expression::Primitive(x),
            Expression::Identifier(x) => Expression::Identifier(x),
            Expression::Assign(Assign { name, value, inner }) => Expression::Assign(Assign {
                name,
                value: remove_spans(value),
                inner: remove_spans(inner),
            }),
            Expression::Function(Function { parameter, body }) => Expression::Function(Function {
                parameter,
                body: remove_spans(body),
            }),
            Expression::Apply(Apply { function, argument }) => Expression::Apply(Apply {
                function: remove_spans(function),
                argument: remove_spans(argument),
            }),
            Expression::Infix(Infix {
                operation,
                left,
                right,
            }) => Expression::Infix(Infix {
                operation,
                left: remove_spans(left),
                right: remove_spans(right),
            }),
        },
    )
}
