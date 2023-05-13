use crate::ast::*;
use crate::error::*;
use crate::lexer::*;
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
                Expr::Let {
                    annotation: let_.annotation | *inner.annotation(),
                    name: n.clone(),
                    value: value.into(),
                    inner: inner.into(),
                }
            }
            --
            left:(@) (quiet! { [AnnotatedToken { annotation: _, token: Token::Operator("+") }] } / expected!("'+'")) right:@ {
                infix(left, Operation::Add, right)
            }
            left:(@) (quiet! { [AnnotatedToken { annotation: _, token: Token::Operator("-") }] } / expected!("'-'")) right:@ {
                infix(left, Operation::Subtract, right)
            }
            --
            left:(@) (quiet! { [AnnotatedToken { annotation: _, token: Token::Operator("*") }] } / expected!("'*'")) right:@ {
                infix(left, Operation::Multiply, right)
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
                Expr::Primitive {
                    annotation: *annotation,
                    value: Primitive::Integer(n.clone()),
                }
            } } / expected!("an integer")

        rule identifier() -> Expr<Span> =
            quiet! { [AnnotatedToken { annotation, token: Token::Identifier(name) }] {
                Expr::Identifier {
                    annotation: *annotation,
                    name: name.clone(),
                }
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

fn infix(left: Expr<Span>, operation: Operation, right: Expr<Span>) -> Expr<Span> {
    Expr::Infix {
        annotation: *left.annotation() | *right.annotation(),
        operation,
        left: left.into(),
        right: right.into(),
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use proptest::test_runner::TestRunner;

    use crate::identifier::*;

    use super::*;

    #[test]
    fn test_parsing_an_integer() {
        TestRunner::default()
            .run(&Integer::arbitrary(), |value| {
                let tokens = vec![AnnotatedToken {
                    annotation: (0..10).into(),
                    token: Token::Integer(value.clone()),
                }];
                let expr = parse(&tokens);

                prop_assert_eq!(
                    expr,
                    Ok(Expr::Primitive {
                        annotation: (0..10).into(),
                        value: Primitive::Integer(value),
                    })
                );
                Ok(())
            })
            .unwrap()
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
        TestRunner::default()
            .run(
                &(Integer::arbitrary(), Integer::arbitrary()),
                |(left, right)| {
                    let tokens = vec![
                        AnnotatedToken {
                            annotation: (0..1).into(),
                            token: Token::Integer(left.clone()),
                        },
                        AnnotatedToken {
                            annotation: (2..3).into(),
                            token: Token::Operator(text),
                        },
                        AnnotatedToken {
                            annotation: (4..5).into(),
                            token: Token::Integer(right.clone()),
                        },
                    ];
                    let expr = parse(&tokens);

                    prop_assert_eq!(
                        expr,
                        Ok(Expr::Infix {
                            annotation: (0..5).into(),
                            operation,
                            left: Expr::Primitive {
                                annotation: (0..1).into(),
                                value: Primitive::Integer(left),
                            }
                            .into(),
                            right: Expr::Primitive {
                                annotation: (4..5).into(),
                                value: Primitive::Integer(right),
                            }
                            .into(),
                        })
                    );
                    Ok(())
                },
            )
            .unwrap()
    }

    #[test]
    fn test_parsing_two_operations_with_higher_precedence_to_the_right() {
        TestRunner::default()
            .run(
                &(
                    Integer::arbitrary(),
                    Integer::arbitrary(),
                    Integer::arbitrary(),
                ),
                |(a, b, c)| {
                    let tokens = vec![
                        AnnotatedToken {
                            annotation: (0..1).into(),
                            token: Token::Integer(a.clone()),
                        },
                        AnnotatedToken {
                            annotation: (2..3).into(),
                            token: Token::Operator("+"),
                        },
                        AnnotatedToken {
                            annotation: (4..5).into(),
                            token: Token::Integer(b.clone()),
                        },
                        AnnotatedToken {
                            annotation: (6..7).into(),
                            token: Token::Operator("*"),
                        },
                        AnnotatedToken {
                            annotation: (8..9).into(),
                            token: Token::Integer(c.clone()),
                        },
                    ];
                    let expr = parse(&tokens);

                    prop_assert_eq!(
                        expr,
                        Ok(Expr::Infix {
                            annotation: (0..9).into(),
                            operation: Operation::Add,
                            left: Expr::Primitive {
                                annotation: (0..1).into(),
                                value: Primitive::Integer(a),
                            }
                            .into(),
                            right: Expr::Infix {
                                annotation: (4..9).into(),
                                operation: Operation::Multiply,
                                left: Expr::Primitive {
                                    annotation: (4..5).into(),
                                    value: Primitive::Integer(b),
                                }
                                .into(),
                                right: Expr::Primitive {
                                    annotation: (8..9).into(),
                                    value: Primitive::Integer(c),
                                }
                                .into(),
                            }
                            .into(),
                        })
                    );
                    Ok(())
                },
            )
            .unwrap()
    }

    #[test]
    fn test_parsing_two_operations_with_higher_precedence_to_the_left() {
        TestRunner::default()
            .run(
                &(
                    Integer::arbitrary(),
                    Integer::arbitrary(),
                    Integer::arbitrary(),
                ),
                |(a, b, c)| {
                    let tokens = vec![
                        AnnotatedToken {
                            annotation: (0..1).into(),
                            token: Token::Integer(a.clone()),
                        },
                        AnnotatedToken {
                            annotation: (2..3).into(),
                            token: Token::Operator("*"),
                        },
                        AnnotatedToken {
                            annotation: (4..5).into(),
                            token: Token::Integer(b.clone()),
                        },
                        AnnotatedToken {
                            annotation: (6..7).into(),
                            token: Token::Operator("-"),
                        },
                        AnnotatedToken {
                            annotation: (8..9).into(),
                            token: Token::Integer(c.clone()),
                        },
                    ];
                    let expr = parse(&tokens);

                    prop_assert_eq!(
                        expr,
                        Ok(Expr::Infix {
                            annotation: (0..9).into(),
                            operation: Operation::Subtract,
                            left: Expr::Infix {
                                annotation: (0..5).into(),
                                operation: Operation::Multiply,
                                left: Expr::Primitive {
                                    annotation: (0..1).into(),
                                    value: Primitive::Integer(a),
                                }
                                .into(),
                                right: Expr::Primitive {
                                    annotation: (4..5).into(),
                                    value: Primitive::Integer(b),
                                }
                                .into(),
                            }
                            .into(),
                            right: Expr::Primitive {
                                annotation: (8..9).into(),
                                value: Primitive::Integer(c),
                            }
                            .into(),
                        })
                    );
                    Ok(())
                },
            )
            .unwrap()
    }

    #[test]
    fn test_variables() {
        TestRunner::default()
            .run(
                &(
                    Identifier::arbitrary_of_max_length(16),
                    Integer::arbitrary(),
                    Integer::arbitrary(),
                ),
                |(name, variable, constant)| {
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
                            token: Token::Integer(variable.clone()),
                        },
                        AnnotatedToken {
                            annotation: (8..9).into(),
                            token: Token::In,
                        },
                        AnnotatedToken {
                            annotation: (10..11).into(),
                            token: Token::Identifier(name.clone()),
                        },
                        AnnotatedToken {
                            annotation: (12..13).into(),
                            token: Token::Operator("*"),
                        },
                        AnnotatedToken {
                            annotation: (14..15).into(),
                            token: Token::Integer(constant.clone()),
                        },
                    ];
                    let expr = parse(&tokens);

                    prop_assert_eq!(
                        expr,
                        Ok(Expr::Let {
                            annotation: (0..15).into(),
                            name: name.clone(),
                            value: Expr::Primitive {
                                annotation: (6..7).into(),
                                value: Primitive::Integer(variable),
                            }
                            .into(),
                            inner: Expr::Infix {
                                annotation: (10..15).into(),
                                operation: Operation::Multiply,
                                left: Expr::Identifier {
                                    annotation: (10..11).into(),
                                    name,
                                }
                                .into(),
                                right: Expr::Primitive {
                                    annotation: (14..15).into(),
                                    value: Primitive::Integer(constant),
                                }
                                .into(),
                            }
                            .into(),
                        })
                    );
                    Ok(())
                },
            )
            .unwrap()
    }

    #[test]
    fn test_parentheses() {
        TestRunner::default()
            .run(
                &(
                    Integer::arbitrary(),
                    Integer::arbitrary(),
                    Integer::arbitrary(),
                ),
                |(a, b, c)| {
                    let tokens = vec![
                        AnnotatedToken {
                            annotation: (0..1).into(),
                            token: Token::Integer(a.clone()),
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
                            token: Token::Integer(b.clone()),
                        },
                        AnnotatedToken {
                            annotation: (7..8).into(),
                            token: Token::Operator("+"),
                        },
                        AnnotatedToken {
                            annotation: (9..10).into(),
                            token: Token::Integer(c.clone()),
                        },
                        AnnotatedToken {
                            annotation: (10..11).into(),
                            token: Token::EndGroup,
                        },
                    ];
                    let expr = parse(&tokens);

                    prop_assert_eq!(
                        expr,
                        Ok(Expr::Infix {
                            annotation: (0..10).into(),
                            operation: Operation::Multiply,
                            left: Expr::Primitive {
                                annotation: (0..1).into(),
                                value: Primitive::Integer(a),
                            }
                            .into(),
                            right: Expr::Infix {
                                annotation: (5..10).into(),
                                operation: Operation::Add,
                                left: Expr::Primitive {
                                    annotation: (5..6).into(),
                                    value: Primitive::Integer(b),
                                }
                                .into(),
                                right: Expr::Primitive {
                                    annotation: (9..10).into(),
                                    value: Primitive::Integer(c),
                                }
                                .into(),
                            }
                            .into(),
                        })
                    );
                    Ok(())
                },
            )
            .unwrap()
    }

    #[test]
    fn test_fails_to_parse_gracefully() {
        TestRunner::default()
            .run(&Integer::arbitrary(), |value| {
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
                let expr = parse(&tokens);

                prop_assert_eq!(
                    expr,
                    Err(Error::ParseError {
                        span: (0..1).into(),
                        expected_tokens: ["'('", "an identifier", "an integer", "let"].into(),
                    })
                );
                Ok(())
            })
            .unwrap()
    }

    #[test]
    fn test_fails_to_parse_at_the_end() {
        TestRunner::default()
            .run(&Integer::arbitrary(), |value| {
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
                let expr = parse(&tokens);

                prop_assert_eq!(
                    expr,
                    Err(Error::ParseError {
                        span: (3..3).into(),
                        expected_tokens: ["'('", "an identifier", "an integer", "let"].into(),
                    })
                );
                Ok(())
            })
            .unwrap()
    }
}
