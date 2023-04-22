use crate::ast::*;
use crate::error::*;

peg::parser! {
    grammar parser() for str {
        pub rule root() -> Expr<()> = _ e:expr() _ { e }

        pub rule expr() -> Expr<()> = precedence! {
            left:(@) _ "+" _ right:@ {
                infix(left, Operation::Add, right)
            }
            left:(@) _ "-" _ right:@ {
                infix(left, Operation::Subtract, right)
            }
            --
            left:(@) _ "*" _ right:@ {
                infix(left, Operation::Multiply, right)
            }
            --
            p:primitive() { p }
            "(" _ e:expr() _ ")" { e }
        }

        rule primitive() -> Expr<()> =
            n:number() {
                Expr::Primitive {
                    annotation: (),
                    value: Primitive::Int(n),
                }
            }

        rule number() -> Int =
            n:$("-"? digit() (digit() / "_")*) { ?
                str::replace(n, "_", "").parse::<Int>().or(Err("number"))
            }

        rule digit() -> char = ['0'..='9']

        rule _ -> () = quiet!{ (" " / "\t")* { } }
    }
}

pub fn parse(input: String) -> Result<Expr<()>, BooError> {
    parser::root(&input).map_err(|inner| {
        let span = miette::SourceSpan::new(
            miette::SourceOffset::from_location(&input, inner.location.line, inner.location.column),
            0.into(),
        );
        BooError::ParseError { input, span, inner }
    })
}

fn infix(left: Expr<()>, operation: Operation, right: Expr<()>) -> Expr<()> {
    Expr::Infix {
        annotation: (),
        operation,
        left: Box::new(left),
        right: Box::new(right),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_an_integer() {
        arbtest::builder().run(|u| {
            let value = u.arbitrary::<Int>()?;
            let string = value.to_string();
            let expr = parse(string);
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
    fn test_parsing_an_integer_with_underscores() -> Result<(), BooError> {
        let expr = parse("123_456_789".to_string())?;
        assert_eq!(
            expr,
            Expr::Primitive {
                annotation: (),
                value: Primitive::Int(123_456_789),
            }
        );
        Ok(())
    }

    #[test]
    fn test_parsing_addition() {
        test_parsing_an_operation('+', Operation::Add)
    }

    #[test]
    fn test_parsing_subtraction() {
        test_parsing_an_operation('-', Operation::Subtract)
    }

    #[test]
    fn test_parsing_multiplication() {
        test_parsing_an_operation('*', Operation::Multiply)
    }

    fn test_parsing_an_operation(text: char, operation: Operation) {
        arbtest::builder().run(|u| {
            let left = u.arbitrary::<Int>()?;
            let right = u.arbitrary::<Int>()?;
            let string = format!("{} {} {}", left, text, right);
            let expr = parse(string);
            assert_eq!(
                expr,
                Ok(Expr::Infix {
                    annotation: (),
                    operation,
                    left: Box::new(Expr::Primitive {
                        annotation: (),
                        value: Primitive::Int(left),
                    }),
                    right: Box::new(Expr::Primitive {
                        annotation: (),
                        value: Primitive::Int(right),
                    }),
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
            let string = format!("{} + {} * {}", a, b, c);
            let expr = parse(string);
            assert_eq!(
                expr,
                Ok(Expr::Infix {
                    annotation: (),
                    operation: Operation::Add,
                    left: Box::new(Expr::Primitive {
                        annotation: (),
                        value: Primitive::Int(a),
                    }),
                    right: Box::new(Expr::Infix {
                        annotation: (),
                        operation: Operation::Multiply,
                        left: Box::new(Expr::Primitive {
                            annotation: (),
                            value: Primitive::Int(b),
                        }),
                        right: Box::new(Expr::Primitive {
                            annotation: (),
                            value: Primitive::Int(c),
                        }),
                    }),
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
            let string = format!("{} * {} - {}", a, b, c);
            let expr = parse(string);
            assert_eq!(
                expr,
                Ok(Expr::Infix {
                    annotation: (),
                    operation: Operation::Subtract,
                    left: Box::new(Expr::Infix {
                        annotation: (),
                        operation: Operation::Multiply,
                        left: Box::new(Expr::Primitive {
                            annotation: (),
                            value: Primitive::Int(a),
                        }),
                        right: Box::new(Expr::Primitive {
                            annotation: (),
                            value: Primitive::Int(b),
                        }),
                    }),
                    right: Box::new(Expr::Primitive {
                        annotation: (),
                        value: Primitive::Int(c),
                    }),
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
            let string = format!("{} * ({} + {})", a, b, c);
            let expr = parse(string);
            assert_eq!(
                expr,
                Ok(Expr::Infix {
                    annotation: (),
                    operation: Operation::Multiply,
                    left: Box::new(Expr::Primitive {
                        annotation: (),
                        value: Primitive::Int(a),
                    }),
                    right: Box::new(Expr::Infix {
                        annotation: (),
                        operation: Operation::Add,
                        left: Box::new(Expr::Primitive {
                            annotation: (),
                            value: Primitive::Int(b),
                        }),
                        right: Box::new(Expr::Primitive {
                            annotation: (),
                            value: Primitive::Int(c),
                        }),
                    }),
                })
            );
            Ok(())
        })
    }

    #[test]
    fn test_whitespace() {
        arbtest::builder().run(|u| {
            let a_count = u.int_in_range(0..=10)?;
            let b_count = u.int_in_range(0..=10)?;
            let c_count = u.int_in_range(0..=10)?;
            let d_count = u.int_in_range(0..=10)?;
            let e_count = u.int_in_range(0..=10)?;
            let f_count = u.int_in_range(1..=10)?; // between '-' and '3'
            let g_count = u.int_in_range(0..=10)?;
            let h_count = u.int_in_range(0..=10)?;
            let a = (0..a_count).map(|_| ' ').collect::<String>();
            let b = (0..b_count).map(|_| ' ').collect::<String>();
            let c = (0..c_count).map(|_| ' ').collect::<String>();
            let d = (0..d_count).map(|_| ' ').collect::<String>();
            let e = (0..e_count).map(|_| ' ').collect::<String>();
            let f = (0..f_count).map(|_| ' ').collect::<String>();
            let g = (0..g_count).map(|_| ' ').collect::<String>();
            let h = (0..h_count).map(|_| ' ').collect::<String>();

            let string = format!("{}1{}+{}({}2{}-{}3{}){}", a, b, c, d, e, f, g, h);
            let expr = parse(string);

            assert_eq!(
                expr,
                Ok(Expr::Infix {
                    annotation: (),
                    operation: Operation::Add,
                    left: Box::new(Expr::Primitive {
                        annotation: (),
                        value: Primitive::Int(1),
                    }),
                    right: Box::new(Expr::Infix {
                        annotation: (),
                        operation: Operation::Subtract,
                        left: Box::new(Expr::Primitive {
                            annotation: (),
                            value: Primitive::Int(2),
                        }),
                        right: Box::new(Expr::Primitive {
                            annotation: (),
                            value: Primitive::Int(3),
                        }),
                    }),
                })
            );
            Ok(())
        })
    }
}
