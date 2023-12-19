use proptest::prelude::*;

use boo_language::*;
use boo_test_helpers::proptest::*;

#[test]
fn test_rendering_and_parsing_an_expression() {
    check(&boo_generator::arbitrary(), |input| {
        let rendered = format!("{}", input);
        let parsed = boo_parser::parse(&rendered)?;
        let despanned = remove_spans(parsed);
        prop_assert_eq!(input, despanned, "\nrendered = {}\n", rendered);
        Ok(())
    })
}

pub fn remove_spans(expr: Expr) -> Expr {
    Expr::new(
        0.into(), // Replacement span to ensure they don't interfere with testing.
        match *expr.expression {
            Expression::Primitive(x) => Expression::Primitive(x),
            Expression::Identifier(x) => Expression::Identifier(x),
            Expression::Function(Function { parameters, body }) => Expression::Function(Function {
                parameters,
                body: remove_spans(body),
            }),
            Expression::Apply(Apply { function, argument }) => Expression::Apply(Apply {
                function: remove_spans(function),
                argument: remove_spans(argument),
            }),
            Expression::Assign(Assign { name, value, inner }) => Expression::Assign(Assign {
                name,
                value: remove_spans(value),
                inner: remove_spans(inner),
            }),
            Expression::Match(Match { value, patterns }) => Expression::Match(Match {
                value: remove_spans(value),
                patterns: patterns
                    .into_iter()
                    .map(|PatternMatch { pattern, result }| PatternMatch {
                        pattern,
                        result: remove_spans(result),
                    })
                    .collect(),
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
