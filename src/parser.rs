use crate::ast::*;
use crate::error::*;
use crate::lexer::{Positioned, SourceSpan, Token};
use crate::primitive::*;

peg::parser! {
    grammar parser<'a>() for [&'a Token<'a>] {
        pub rule root() -> Expr<'a, ()> = e:expr() { e }

        pub rule expr() -> Expr<'a, ()> = precedence! {
            (quiet! { [Token::Let] } / expected!("let"))
            name:(quiet! { [Token::Identifier(name)] } / expected!("an identifier"))
            (quiet! { [Token::Assign] } / expected!("="))
            value:expr()
            (quiet! { [Token::In] } / expected!("in"))
            inner:expr() {
                let n = match name {
                    Token::Identifier(name) => *name,
                    _ => unreachable!(),
                };
                Expr::Let {
                    annotation: (),
                    name: n,
                    value: value.into(),
                    inner: inner.into(),
                }
            }
            --
            left:(@) (quiet! { [Token::Operator("+")] } / expected!("'+'")) right:@ {
                infix(left, Operation::Add, right)
            }
            left:(@) (quiet! { [Token::Operator("-")] } / expected!("'-'")) right:@ {
                infix(left, Operation::Subtract, right)
            }
            --
            left:(@) (quiet! { [Token::Operator("*")] } / expected!("'*'")) right:@ {
                infix(left, Operation::Multiply, right)
            }
            --
            p:primitive() { p }
            i:identifier() { i }
            --
            (quiet! { [Token::StartGroup] } / expected!("'('"))
            e:expr()
            (quiet! { [Token::EndGroup] } / expected!(")'")) {
                e
            }
        }

        rule primitive() -> Expr<'a, ()> =
            quiet! { [Token::Integer(n)] {
                Expr::Primitive {
                    annotation: (),
                    value: Primitive::Int(*n),
                }
            } } / expected!("an integer")

        rule identifier() -> Expr<'a, ()> =
            quiet! { [Token::Identifier(name)] {
                Expr::Identifier {
                    annotation: (),
                    name,
                }
            } } / expected!("an identifier")
    }
}

pub fn parse<'a>(input: &'a [Positioned<Token>]) -> Result<Expr<'a, ()>> {
    parser::root(
        &(input
            .iter()
            .map(|positioned| &positioned.value)
            .collect::<Vec<_>>()),
    )
    .map_err(|inner| {
        let span: SourceSpan = if inner.location < input.len() {
            input[inner.location].span
        } else {
            input
                .last()
                .map(|s| (s.span.offset() + s.span.len()).into())
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

fn infix<'a>(left: Expr<'a, ()>, operation: Operation, right: Expr<'a, ()>) -> Expr<'a, ()> {
    Expr::Infix {
        annotation: (),
        operation,
        left: left.into(),
        right: right.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_an_integer() {
        arbtest::builder().run(|u| {
            let value = u.arbitrary::<Int>()?;
            let tokens = vec![Positioned {
                span: (0..10).into(),
                value: Token::Integer(value),
            }];
            let expr = parse(&tokens);

            assert_eq!(
                expr,
                Ok(Expr::Primitive {
                    annotation: (),
                    value: Primitive::Int(value),
                })
            );
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
        arbtest::builder().run(|u| {
            let left = u.arbitrary::<Int>()?;
            let right = u.arbitrary::<Int>()?;
            let tokens = vec![
                Positioned {
                    span: (0..1).into(),
                    value: Token::Integer(left),
                },
                Positioned {
                    span: (2..3).into(),
                    value: Token::Operator(text),
                },
                Positioned {
                    span: (4..5).into(),
                    value: Token::Integer(right),
                },
            ];
            let expr = parse(&tokens);

            assert_eq!(
                expr,
                Ok(Expr::Infix {
                    annotation: (),
                    operation,
                    left: Expr::Primitive {
                        annotation: (),
                        value: Primitive::Int(left),
                    }
                    .into(),
                    right: Expr::Primitive {
                        annotation: (),
                        value: Primitive::Int(right),
                    }
                    .into(),
                })
            );
            Ok(())
        })
    }

    #[test]
    fn test_parsing_two_operations_with_higher_precedence_to_the_right() {
        arbtest::builder().run(|u| {
            let a = u.arbitrary::<Int>()?;
            let b = u.arbitrary::<Int>()?;
            let c = u.arbitrary::<Int>()?;
            let tokens = vec![
                Positioned {
                    span: (0..1).into(),
                    value: Token::Integer(a),
                },
                Positioned {
                    span: (2..3).into(),
                    value: Token::Operator("+"),
                },
                Positioned {
                    span: (4..5).into(),
                    value: Token::Integer(b),
                },
                Positioned {
                    span: (6..7).into(),
                    value: Token::Operator("*"),
                },
                Positioned {
                    span: (8..9).into(),
                    value: Token::Integer(c),
                },
            ];
            let expr = parse(&tokens);

            assert_eq!(
                expr,
                Ok(Expr::Infix {
                    annotation: (),
                    operation: Operation::Add,
                    left: Expr::Primitive {
                        annotation: (),
                        value: Primitive::Int(a),
                    }
                    .into(),
                    right: Expr::Infix {
                        annotation: (),
                        operation: Operation::Multiply,
                        left: Expr::Primitive {
                            annotation: (),
                            value: Primitive::Int(b),
                        }
                        .into(),
                        right: Expr::Primitive {
                            annotation: (),
                            value: Primitive::Int(c),
                        }
                        .into(),
                    }
                    .into(),
                })
            );
            Ok(())
        })
    }

    #[test]
    fn test_parsing_two_operations_with_higher_precedence_to_the_left() {
        arbtest::builder().run(|u| {
            let a = u.arbitrary::<Int>()?;
            let b = u.arbitrary::<Int>()?;
            let c = u.arbitrary::<Int>()?;
            let tokens = vec![
                Positioned {
                    span: (0..1).into(),
                    value: Token::Integer(a),
                },
                Positioned {
                    span: (2..3).into(),
                    value: Token::Operator("*"),
                },
                Positioned {
                    span: (4..5).into(),
                    value: Token::Integer(b),
                },
                Positioned {
                    span: (6..7).into(),
                    value: Token::Operator("-"),
                },
                Positioned {
                    span: (8..9).into(),
                    value: Token::Integer(c),
                },
            ];
            let expr = parse(&tokens);

            assert_eq!(
                expr,
                Ok(Expr::Infix {
                    annotation: (),
                    operation: Operation::Subtract,
                    left: Expr::Infix {
                        annotation: (),
                        operation: Operation::Multiply,
                        left: Expr::Primitive {
                            annotation: (),
                            value: Primitive::Int(a),
                        }
                        .into(),
                        right: Expr::Primitive {
                            annotation: (),
                            value: Primitive::Int(b),
                        }
                        .into(),
                    }
                    .into(),
                    right: Expr::Primitive {
                        annotation: (),
                        value: Primitive::Int(c),
                    }
                    .into(),
                })
            );
            Ok(())
        })
    }

    #[test]
    fn test_variables() {
        arbtest::builder().run(|u| {
            let variable = u.arbitrary::<Int>()?;
            let constant = u.arbitrary::<Int>()?;
            let tokens = vec![
                Positioned {
                    span: (0..1).into(),
                    value: Token::Let,
                },
                Positioned {
                    span: (2..3).into(),
                    value: Token::Identifier("number"),
                },
                Positioned {
                    span: (4..5).into(),
                    value: Token::Assign,
                },
                Positioned {
                    span: (6..7).into(),
                    value: Token::Integer(variable),
                },
                Positioned {
                    span: (8..9).into(),
                    value: Token::In,
                },
                Positioned {
                    span: (10..11).into(),
                    value: Token::Identifier("number"),
                },
                Positioned {
                    span: (12..13).into(),
                    value: Token::Operator("*"),
                },
                Positioned {
                    span: (14..15).into(),
                    value: Token::Integer(constant),
                },
            ];
            let expr = parse(&tokens);

            assert_eq!(
                expr,
                Ok(Expr::Let {
                    annotation: (),
                    name: "number",
                    value: Expr::Primitive {
                        annotation: (),
                        value: Primitive::Int(variable),
                    }
                    .into(),
                    inner: Expr::Infix {
                        annotation: (),
                        operation: Operation::Multiply,
                        left: Expr::Identifier {
                            annotation: (),
                            name: "number",
                        }
                        .into(),
                        right: Expr::Primitive {
                            annotation: (),
                            value: Primitive::Int(constant),
                        }
                        .into(),
                    }
                    .into(),
                })
            );
            Ok(())
        })
    }

    #[test]
    fn test_parentheses() {
        arbtest::builder().run(|u| {
            let a = u.arbitrary::<Int>()?;
            let b = u.arbitrary::<Int>()?;
            let c = u.arbitrary::<Int>()?;
            let tokens = vec![
                Positioned {
                    span: (0..1).into(),
                    value: Token::Integer(a),
                },
                Positioned {
                    span: (2..3).into(),
                    value: Token::Operator("*"),
                },
                Positioned {
                    span: (4..5).into(),
                    value: Token::StartGroup,
                },
                Positioned {
                    span: (5..6).into(),
                    value: Token::Integer(b),
                },
                Positioned {
                    span: (7..8).into(),
                    value: Token::Operator("+"),
                },
                Positioned {
                    span: (9..10).into(),
                    value: Token::Integer(c),
                },
                Positioned {
                    span: (10..11).into(),
                    value: Token::EndGroup,
                },
            ];
            let expr = parse(&tokens);

            assert_eq!(
                expr,
                Ok(Expr::Infix {
                    annotation: (),
                    operation: Operation::Multiply,
                    left: Expr::Primitive {
                        annotation: (),
                        value: Primitive::Int(a),
                    }
                    .into(),
                    right: Expr::Infix {
                        annotation: (),
                        operation: Operation::Add,
                        left: Expr::Primitive {
                            annotation: (),
                            value: Primitive::Int(b),
                        }
                        .into(),
                        right: Expr::Primitive {
                            annotation: (),
                            value: Primitive::Int(c),
                        }
                        .into(),
                    }
                    .into(),
                })
            );
            Ok(())
        })
    }

    #[test]
    fn test_fails_to_parse_gracefully() {
        arbtest::builder().run(|u| {
            let value = u.arbitrary::<Int>()?;
            let tokens = vec![
                Positioned {
                    span: (0..1).into(),
                    value: Token::Operator("+"),
                },
                Positioned {
                    span: (2..3).into(),
                    value: Token::Integer(value),
                },
            ];
            let expr = parse(&tokens);

            assert_eq!(
                expr,
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
        arbtest::builder().run(|u| {
            let value = u.arbitrary::<Int>()?;
            let tokens = vec![
                Positioned {
                    span: (0..1).into(),
                    value: Token::Integer(value),
                },
                Positioned {
                    span: (2..3).into(),
                    value: Token::Operator("+"),
                },
            ];
            let expr = parse(&tokens);

            assert_eq!(
                expr,
                Err(Error::ParseError {
                    span: (3..3).into(),
                    expected_tokens: ["'('", "an identifier", "an integer", "let"].into(),
                })
            );
            Ok(())
        })
    }
}
