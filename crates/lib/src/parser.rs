pub mod ast;
pub mod builders;
pub mod generators;

use crate::error::*;
use crate::lexer::*;
use crate::operation::*;
use crate::parser::ast::*;
use crate::primitive::*;
use crate::span::*;

peg::parser! {
    grammar parser<'a>() for [&'a AnnotatedToken<'a, Span>] {
        pub rule root() -> Expr<Span> = e:expr() { e }

        pub rule expr() -> Expr<Span> = precedence! {
            let_:(quiet! { [AnnotatedToken { annotation: _, token: Token::Let }] } / expected!("let"))
            name:(quiet! { [AnnotatedToken { annotation: _, token: Token::Identifier(name) }] } / expected!("an identifier"))
            (quiet! { [AnnotatedToken { annotation: _, token: Token::Assign }] } / expected!("="))
            value:expr()
            (quiet! { [AnnotatedToken { annotation: _, token: Token::In }] } / expected!("in"))
            inner:expr() {
                let n = match &name.token {
                    Token::Identifier(name) => name,
                    _ => unreachable!(),
                };
                Annotated {
                    annotation: let_.annotation | inner.annotation,
                    value: Expression::Let {
                        name: n.clone(),
                        value,
                        inner,
                    }
                }.into()
            }
            --
            left:(@) (quiet! { [AnnotatedToken { annotation: _, token: Token::Operator("+") }] } / expected!("'+'")) right:@ {
                construct_infix(left, Operation::Add, right)
            }
            left:(@) (quiet! { [AnnotatedToken { annotation: _, token: Token::Operator("-") }] } / expected!("'-'")) right:@ {
                construct_infix(left, Operation::Subtract, right)
            }
            --
            left:(@) (quiet! { [AnnotatedToken { annotation: _, token: Token::Operator("*") }] } / expected!("'*'")) right:@ {
                construct_infix(left, Operation::Multiply, right)
            }
            --
            p:primitive() { p }
            i:identifier() { i }
            --
            (quiet! { [AnnotatedToken { annotation: _, token: Token::StartGroup }] } / expected!("'('"))
            e:expr()
            (quiet! { [AnnotatedToken { annotation: _, token: Token::EndGroup }] } / expected!(")'")) {
                e
            }
        }

        rule primitive() -> Expr<Span> =
            quiet! { [AnnotatedToken { annotation, token: Token::Integer(n) }] {
                Annotated {
                    annotation: *annotation,
                    value: Expression::Primitive {
                        value: Primitive::Integer(n.clone()),
                    }
                }.into()
            } } / expected!("an integer")

        rule identifier() -> Expr<Span> =
            quiet! { [AnnotatedToken { annotation, token: Token::Identifier(name) }] {
                Annotated {
                    annotation: *annotation,
                    value: Expression::Identifier {
                        name: name.clone(),
                    }
                }.into()
            } } / expected!("an identifier")
    }
}

pub fn parse(input: &[AnnotatedToken<Span>]) -> Result<Expr<Span>> {
    parser::root(&(input.iter().collect::<Vec<_>>())).map_err(|inner| {
        let span: Span = if inner.location < input.len() {
            input[inner.location].annotation
        } else {
            input
                .last()
                .map(|s| s.annotation.end.into())
                .unwrap_or(0.into())
        };
        let mut expected_tokens: Vec<&str> = inner.expected.tokens().collect();
        expected_tokens.sort();
        Error::ParseError {
            span,
            expected_tokens,
        }
    })
}

fn construct_infix(left: Expr<Span>, operation: Operation, right: Expr<Span>) -> Expr<Span> {
    Annotated {
        annotation: left.annotation | right.annotation,
        value: Expression::Infix {
            operation,
            left,
            right,
        },
    }
    .into()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::identifier::*;
    use crate::proptest_helpers::*;

    use super::builders::*;
    use super::*;

    #[test]
    fn test_parsing_an_integer() {
        check(&Integer::arbitrary(), |value| {
            let expected = primitive_integer(0..10, value.clone());
            let tokens = vec![AnnotatedToken {
                annotation: (0..10).into(),
                token: Token::Integer(value),
            }];

            let actual = parse(&tokens);

            prop_assert_eq!(actual, Ok(expected));
            Ok(())
        })
    }

    #[test]
    fn test_parsing_addition() {
        test_parsing_an_operation("+", Operation::Add)
    }

    #[test]
    fn test_parsing_subtraction() {
        test_parsing_an_operation("-", Operation::Subtract)
    }

    #[test]
    fn test_parsing_multiplication() {
        test_parsing_an_operation("*", Operation::Multiply)
    }

    fn test_parsing_an_operation(text: &str, operation: Operation) {
        check(
            &(Integer::arbitrary(), Integer::arbitrary()),
            |(left, right)| {
                let expected = infix(
                    0..5,
                    operation,
                    primitive_integer(0..1, left.clone()),
                    primitive_integer(4..5, right.clone()),
                );
                let tokens = vec![
                    AnnotatedToken {
                        annotation: (0..1).into(),
                        token: Token::Integer(left),
                    },
                    AnnotatedToken {
                        annotation: (2..3).into(),
                        token: Token::Operator(text),
                    },
                    AnnotatedToken {
                        annotation: (4..5).into(),
                        token: Token::Integer(right),
                    },
                ];

                let actual = parse(&tokens);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_parsing_two_operations_with_higher_precedence_to_the_right() {
        check(
            &(
                Integer::arbitrary(),
                Integer::arbitrary(),
                Integer::arbitrary(),
            ),
            |(a, b, c)| {
                let expected = infix(
                    0..9,
                    Operation::Add,
                    primitive_integer(0..1, a.clone()),
                    infix(
                        4..9,
                        Operation::Multiply,
                        primitive_integer(4..5, b.clone()),
                        primitive_integer(8..9, c.clone()),
                    ),
                );
                let tokens = vec![
                    AnnotatedToken {
                        annotation: (0..1).into(),
                        token: Token::Integer(a),
                    },
                    AnnotatedToken {
                        annotation: (2..3).into(),
                        token: Token::Operator("+"),
                    },
                    AnnotatedToken {
                        annotation: (4..5).into(),
                        token: Token::Integer(b),
                    },
                    AnnotatedToken {
                        annotation: (6..7).into(),
                        token: Token::Operator("*"),
                    },
                    AnnotatedToken {
                        annotation: (8..9).into(),
                        token: Token::Integer(c),
                    },
                ];

                let actual = parse(&tokens);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_parsing_two_operations_with_higher_precedence_to_the_left() {
        check(
            &(
                Integer::arbitrary(),
                Integer::arbitrary(),
                Integer::arbitrary(),
            ),
            |(a, b, c)| {
                let expected = infix(
                    0..9,
                    Operation::Subtract,
                    infix(
                        0..5,
                        Operation::Multiply,
                        primitive_integer(0..1, a.clone()),
                        primitive_integer(4..5, b.clone()),
                    ),
                    primitive_integer(8..9, c.clone()),
                );
                let tokens = vec![
                    AnnotatedToken {
                        annotation: (0..1).into(),
                        token: Token::Integer(a),
                    },
                    AnnotatedToken {
                        annotation: (2..3).into(),
                        token: Token::Operator("*"),
                    },
                    AnnotatedToken {
                        annotation: (4..5).into(),
                        token: Token::Integer(b),
                    },
                    AnnotatedToken {
                        annotation: (6..7).into(),
                        token: Token::Operator("-"),
                    },
                    AnnotatedToken {
                        annotation: (8..9).into(),
                        token: Token::Integer(c),
                    },
                ];

                let actual = parse(&tokens);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_variables() {
        check(
            &(
                Identifier::arbitrary(),
                Integer::arbitrary(),
                Integer::arbitrary(),
            ),
            |(name, variable, constant)| {
                let expected = assign(
                    0..15,
                    name.clone(),
                    primitive_integer(6..7, variable.clone()),
                    infix(
                        10..15,
                        Operation::Multiply,
                        identifier(10..11, name.clone()),
                        primitive_integer(14..15, constant.clone()),
                    ),
                );
                let tokens = vec![
                    AnnotatedToken {
                        annotation: (0..1).into(),
                        token: Token::Let,
                    },
                    AnnotatedToken {
                        annotation: (2..3).into(),
                        token: Token::Identifier(name.clone()),
                    },
                    AnnotatedToken {
                        annotation: (4..5).into(),
                        token: Token::Assign,
                    },
                    AnnotatedToken {
                        annotation: (6..7).into(),
                        token: Token::Integer(variable),
                    },
                    AnnotatedToken {
                        annotation: (8..9).into(),
                        token: Token::In,
                    },
                    AnnotatedToken {
                        annotation: (10..11).into(),
                        token: Token::Identifier(name),
                    },
                    AnnotatedToken {
                        annotation: (12..13).into(),
                        token: Token::Operator("*"),
                    },
                    AnnotatedToken {
                        annotation: (14..15).into(),
                        token: Token::Integer(constant),
                    },
                ];

                let actual = parse(&tokens);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_parentheses() {
        check(
            &(
                Integer::arbitrary(),
                Integer::arbitrary(),
                Integer::arbitrary(),
            ),
            |(a, b, c)| {
                let expected = infix(
                    0..10,
                    Operation::Multiply,
                    primitive_integer(0..1, a.clone()),
                    infix(
                        5..10,
                        Operation::Add,
                        primitive_integer(5..6, b.clone()),
                        primitive_integer(9..10, c.clone()),
                    ),
                );
                let tokens = vec![
                    AnnotatedToken {
                        annotation: (0..1).into(),
                        token: Token::Integer(a),
                    },
                    AnnotatedToken {
                        annotation: (2..3).into(),
                        token: Token::Operator("*"),
                    },
                    AnnotatedToken {
                        annotation: (4..5).into(),
                        token: Token::StartGroup,
                    },
                    AnnotatedToken {
                        annotation: (5..6).into(),
                        token: Token::Integer(b),
                    },
                    AnnotatedToken {
                        annotation: (7..8).into(),
                        token: Token::Operator("+"),
                    },
                    AnnotatedToken {
                        annotation: (9..10).into(),
                        token: Token::Integer(c),
                    },
                    AnnotatedToken {
                        annotation: (10..11).into(),
                        token: Token::EndGroup,
                    },
                ];

                let actual = parse(&tokens);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_fails_to_parse_gracefully() {
        check(&Integer::arbitrary(), |value| {
            let tokens = vec![
                AnnotatedToken {
                    annotation: (0..1).into(),
                    token: Token::Operator("+"),
                },
                AnnotatedToken {
                    annotation: (2..3).into(),
                    token: Token::Integer(value),
                },
            ];
            let actual = parse(&tokens);

            prop_assert_eq!(
                actual,
                Err(Error::ParseError {
                    span: (0..1).into(),
                    expected_tokens: ["'('", "an identifier", "an integer", "let"].into(),
                })
            );
            Ok(())
        })
    }

    #[test]
    fn test_fails_to_parse_at_the_end() {
        check(&Integer::arbitrary(), |value| {
            let tokens = vec![
                AnnotatedToken {
                    annotation: (0..1).into(),
                    token: Token::Integer(value),
                },
                AnnotatedToken {
                    annotation: (2..3).into(),
                    token: Token::Operator("+"),
                },
            ];
            let actual = parse(&tokens);

            prop_assert_eq!(
                actual,
                Err(Error::ParseError {
                    span: (3..3).into(),
                    expected_tokens: ["'('", "an identifier", "an integer", "let"].into(),
                })
            );
            Ok(())
        })
    }
}
