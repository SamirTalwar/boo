//! Parses tokens into an AST.

use boo_core::ast::*;
use boo_core::error::*;
use boo_core::operation::*;
use boo_core::primitive::*;
use boo_core::span::*;

use crate::ast::*;
use crate::lexer::*;

peg::parser! {
    grammar parser<'a>() for [&'a AnnotatedToken<'a, Span>] {
        pub rule root() -> Expr = e:expr() { e }

        pub rule expr() -> Expr = precedence! {
            fn_:(quiet! { [AnnotatedToken { annotation: _, token: Token::Fn }] } / expected!("fn"))
            parameter:(quiet! { [AnnotatedToken { annotation: _, token: Token::Identifier(name) }] } / expected!("an identifier"))
            (quiet! { [AnnotatedToken { annotation: _, token: Token::Arrow }] } / expected!("->"))
            body:expr() {
                let p = match &parameter.token {
                    Token::Identifier(parameter) => parameter,
                    _ => unreachable!(),
                };
                Expr::new(
                    fn_.annotation | body.annotation(),
                    Expression::Function(Function {
                        parameter: p.clone(),
                        body,
                    }),
                )
            }
            --
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
                Expr::new(
                    let_.annotation | inner.annotation(),
                    Expression::Assign(Assign {
                        name: n.clone(),
                        value,
                        inner,
                    }),
                )
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
            function:(@) argument:atomic_expr() {
                Expr::new(
                    function.annotation() | argument.annotation(),
                    Expression::Apply(Apply {
                        function,
                        argument,
                    }),
                )
            }
            --
            a:atomic_expr() { a }
        }

        rule atomic_expr() -> Expr =
            e:(primitive() / identifier() / group()) { e }

        rule group() -> Expr =
            (quiet! { [AnnotatedToken { annotation: _, token: Token::StartGroup }] } / expected!("'('"))
            e:expr()
            (quiet! { [AnnotatedToken { annotation: _, token: Token::EndGroup }] } / expected!(")'")) {
                e
            }

        rule primitive() -> Expr =
            quiet! { [AnnotatedToken { annotation, token: Token::Integer(n) }] {
                Expr::new(
                    *annotation,
                    Expression::Primitive(Primitive::Integer(n.clone())),
                )
            } } / expected!("an integer")

        rule identifier() -> Expr =
            quiet! { [AnnotatedToken { annotation, token: Token::Identifier(name) }] {
                Expr::new(
                    *annotation,
                    Expression::Identifier(name.clone()),
                )
            } } / expected!("an identifier")
    }
}

/// Parses a slice of [`Token`] values, annotated with a [`Span`], into an
/// expression.
///
/// Returns an error if an unexpected token is found.
pub fn parse_tokens(input: &[AnnotatedToken<Span>]) -> Result<Expr> {
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

fn construct_infix(left: Expr, operation: Operation, right: Expr) -> Expr {
    Expr::new(
        left.annotation() | right.annotation(),
        Expression::Infix(Infix {
            operation,
            left,
            right,
        }),
    )
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use boo_core::identifier::*;
    use boo_test_helpers::proptest::*;

    use super::builders::*;
    use super::*;

    #[test]
    fn test_parsing_an_integer() {
        // value
        check(&Integer::arbitrary(), |value| {
            let expected = primitive_integer(0..10, value.clone());
            let tokens = vec![AnnotatedToken {
                annotation: (0..10).into(),
                token: Token::Integer(value),
            }];

            let actual = parse_tokens(&tokens);

            prop_assert_eq!(actual, Ok(expected));
            Ok(())
        })
    }

    #[test]
    fn test_variables() {
        // let name = variable in name * constant
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

                let actual = parse_tokens(&tokens);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_parsing_a_function() {
        // fn argument -> argument
        check(&Identifier::arbitrary(), |argument| {
            let expected = function(0..9, argument.clone(), identifier(8..9, argument.clone()));
            let tokens = vec![
                AnnotatedToken {
                    annotation: (0..2).into(),
                    token: Token::Fn,
                },
                AnnotatedToken {
                    annotation: (3..4).into(),
                    token: Token::Identifier(argument.clone()),
                },
                AnnotatedToken {
                    annotation: (5..7).into(),
                    token: Token::Arrow,
                },
                AnnotatedToken {
                    annotation: (8..9).into(),
                    token: Token::Identifier(argument),
                },
            ];

            let actual = parse_tokens(&tokens);

            prop_assert_eq!(actual, Ok(expected));
            Ok(())
        })
    }

    #[test]
    fn test_parsing_a_more_complicated_function() {
        // fn argument -> argument + argument
        check(&Identifier::arbitrary(), |argument| {
            let expected = function(
                0..13,
                argument.clone(),
                infix(
                    8..13,
                    Operation::Add,
                    identifier(8..9, argument.clone()),
                    identifier(12..13, argument.clone()),
                ),
            );
            let tokens = vec![
                AnnotatedToken {
                    annotation: (0..2).into(),
                    token: Token::Fn,
                },
                AnnotatedToken {
                    annotation: (3..4).into(),
                    token: Token::Identifier(argument.clone()),
                },
                AnnotatedToken {
                    annotation: (5..7).into(),
                    token: Token::Arrow,
                },
                AnnotatedToken {
                    annotation: (8..9).into(),
                    token: Token::Identifier(argument.clone()),
                },
                AnnotatedToken {
                    annotation: (10..11).into(),
                    token: Token::Operator("+"),
                },
                AnnotatedToken {
                    annotation: (12..13).into(),
                    token: Token::Identifier(argument),
                },
            ];

            let actual = parse_tokens(&tokens);

            prop_assert_eq!(actual, Ok(expected));
            Ok(())
        })
    }

    #[test]
    fn test_parsing_function_application() {
        // (fn argument -> argument + argument) input
        check(
            &(Identifier::arbitrary(), Integer::arbitrary()),
            |(argument, input)| {
                let expected = apply(
                    1..19,
                    function(
                        1..14,
                        argument.clone(),
                        infix(
                            9..14,
                            Operation::Add,
                            identifier(9..10, argument.clone()),
                            identifier(13..14, argument.clone()),
                        ),
                    ),
                    primitive_integer(16..19, input.clone()),
                );
                let tokens = vec![
                    AnnotatedToken {
                        annotation: (0..1).into(),
                        token: Token::StartGroup,
                    },
                    AnnotatedToken {
                        annotation: (1..3).into(),
                        token: Token::Fn,
                    },
                    AnnotatedToken {
                        annotation: (4..5).into(),
                        token: Token::Identifier(argument.clone()),
                    },
                    AnnotatedToken {
                        annotation: (6..8).into(),
                        token: Token::Arrow,
                    },
                    AnnotatedToken {
                        annotation: (9..10).into(),
                        token: Token::Identifier(argument.clone()),
                    },
                    AnnotatedToken {
                        annotation: (11..12).into(),
                        token: Token::Operator("+"),
                    },
                    AnnotatedToken {
                        annotation: (13..14).into(),
                        token: Token::Identifier(argument),
                    },
                    AnnotatedToken {
                        annotation: (14..15).into(),
                        token: Token::EndGroup,
                    },
                    AnnotatedToken {
                        annotation: (16..19).into(),
                        token: Token::Integer(input),
                    },
                ];

                let actual = parse_tokens(&tokens);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_parsing_named_function_application() {
        // function_name input
        check(
            &(Identifier::arbitrary(), Integer::arbitrary()),
            |(function_name, input)| {
                let expected = apply(
                    0..5,
                    identifier(0..1, function_name.clone()),
                    primitive_integer(2..5, input.clone()),
                );
                let tokens = vec![
                    AnnotatedToken {
                        annotation: (0..1).into(),
                        token: Token::Identifier(function_name),
                    },
                    AnnotatedToken {
                        annotation: (2..5).into(),
                        token: Token::Integer(input),
                    },
                ];

                let actual = parse_tokens(&tokens);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_parsing_named_function_application_with_multiple_arguments() {
        // function_name a b c
        check(
            &(
                Identifier::arbitrary(),
                Integer::arbitrary(),
                Identifier::arbitrary(),
                Integer::arbitrary(),
            ),
            |(function_name, a, b, c)| {
                let expected = apply(
                    0..7,
                    apply(
                        0..5,
                        apply(
                            0..3,
                            identifier(0..1, function_name.clone()),
                            primitive_integer(2..3, a.clone()),
                        ),
                        identifier(4..5, b.clone()),
                    ),
                    primitive_integer(6..7, c.clone()),
                );
                let tokens = vec![
                    AnnotatedToken {
                        annotation: (0..1).into(),
                        token: Token::Identifier(function_name),
                    },
                    AnnotatedToken {
                        annotation: (2..3).into(),
                        token: Token::Integer(a),
                    },
                    AnnotatedToken {
                        annotation: (4..5).into(),
                        token: Token::Identifier(b),
                    },
                    AnnotatedToken {
                        annotation: (6..7).into(),
                        token: Token::Integer(c),
                    },
                ];

                let actual = parse_tokens(&tokens);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_parsing_assigned_function_application() {
        // let function_name = fn argument -> (argument + argument) in function_name input
        check(
            &(
                Identifier::arbitrary(),
                Identifier::arbitrary(),
                Integer::arbitrary(),
            ),
            |(function_name, argument, input)| {
                let expected = assign(
                    0..32,
                    function_name.clone(),
                    function(
                        8..22,
                        argument.clone(),
                        infix(
                            17..22,
                            Operation::Add,
                            identifier(17..18, argument.clone()),
                            identifier(21..22, argument.clone()),
                        ),
                    ),
                    apply(
                        27..32,
                        identifier(27..28, function_name.clone()),
                        primitive_integer(29..32, input.clone()),
                    ),
                );
                let tokens = vec![
                    AnnotatedToken {
                        annotation: (0..3).into(),
                        token: Token::Let,
                    },
                    AnnotatedToken {
                        annotation: (4..5).into(),
                        token: Token::Identifier(function_name.clone()),
                    },
                    AnnotatedToken {
                        annotation: (6..7).into(),
                        token: Token::Assign,
                    },
                    AnnotatedToken {
                        annotation: (8..10).into(),
                        token: Token::Fn,
                    },
                    AnnotatedToken {
                        annotation: (11..12).into(),
                        token: Token::Identifier(argument.clone()),
                    },
                    AnnotatedToken {
                        annotation: (13..15).into(),
                        token: Token::Arrow,
                    },
                    AnnotatedToken {
                        annotation: (16..17).into(),
                        token: Token::StartGroup,
                    },
                    AnnotatedToken {
                        annotation: (17..18).into(),
                        token: Token::Identifier(argument.clone()),
                    },
                    AnnotatedToken {
                        annotation: (19..20).into(),
                        token: Token::Operator("+"),
                    },
                    AnnotatedToken {
                        annotation: (21..22).into(),
                        token: Token::Identifier(argument),
                    },
                    AnnotatedToken {
                        annotation: (22..23).into(),
                        token: Token::EndGroup,
                    },
                    AnnotatedToken {
                        annotation: (24..26).into(),
                        token: Token::In,
                    },
                    AnnotatedToken {
                        annotation: (27..28).into(),
                        token: Token::Identifier(function_name),
                    },
                    AnnotatedToken {
                        annotation: (29..32).into(),
                        token: Token::Integer(input),
                    },
                ];

                let actual = parse_tokens(&tokens);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
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
        // left `operation` right
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

                let actual = parse_tokens(&tokens);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_parsing_an_operation_on_function_calls() {
        // f left + g right
        check(
            &(
                Identifier::arbitrary(),
                Integer::arbitrary(),
                Identifier::arbitrary(),
                Integer::arbitrary(),
            ),
            |(f, left, g, right)| {
                let expected = infix(
                    0..9,
                    Operation::Add,
                    apply(
                        0..3,
                        identifier(0..1, f.clone()),
                        primitive_integer(2..3, left.clone()),
                    ),
                    apply(
                        6..9,
                        identifier(6..7, g.clone()),
                        primitive_integer(8..9, right.clone()),
                    ),
                );
                let tokens = vec![
                    AnnotatedToken {
                        annotation: (0..1).into(),
                        token: Token::Identifier(f),
                    },
                    AnnotatedToken {
                        annotation: (2..3).into(),
                        token: Token::Integer(left),
                    },
                    AnnotatedToken {
                        annotation: (4..5).into(),
                        token: Token::Operator("+"),
                    },
                    AnnotatedToken {
                        annotation: (6..7).into(),
                        token: Token::Identifier(g),
                    },
                    AnnotatedToken {
                        annotation: (8..9).into(),
                        token: Token::Integer(right),
                    },
                ];

                let actual = parse_tokens(&tokens);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_parsing_two_operations_with_higher_precedence_to_the_right() {
        // a + b * c
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

                let actual = parse_tokens(&tokens);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_parsing_two_operations_with_higher_precedence_to_the_left() {
        // a * b - c
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

                let actual = parse_tokens(&tokens);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_parentheses() {
        // a * (b + c)
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

                let actual = parse_tokens(&tokens);

                prop_assert_eq!(actual, Ok(expected));
                Ok(())
            },
        )
    }

    #[test]
    fn test_fails_to_parse_gracefully() {
        // + value
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
            let actual = parse_tokens(&tokens);

            prop_assert_eq!(
                actual,
                Err(Error::ParseError {
                    span: (0..1).into(),
                    expected_tokens: ["'('", "an identifier", "an integer", "fn", "let"].into(),
                })
            );
            Ok(())
        })
    }

    #[test]
    fn test_fails_to_parse_at_the_end() {
        // value +
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
            let actual = parse_tokens(&tokens);

            prop_assert_eq!(
                actual,
                Err(Error::ParseError {
                    span: (3..3).into(),
                    expected_tokens: ["'('", "an identifier", "an integer", "fn", "let"].into(),
                })
            );
            Ok(())
        })
    }
}
