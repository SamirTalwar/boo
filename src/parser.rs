use crate::ast::*;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    Remaining(String),
    Peg(peg::error::ParseError<peg::str::LineCol>),
}

peg::parser! {
    grammar parser() for str {
        pub rule expr() -> Expr<()> =
            infix() / primitive()

        rule infix() -> Expr<()> =
            left:primitive() _ operation:operation() _ right:expr() {
                Expr::Infix {
                    annotation: (),
                    operation,
                    left: Box::new(left),
                    right: Box::new(right),
                }
            }

        rule operation() -> Operation =
            "+" { Operation::Add }
            / "-" { Operation::Subtract }
            / "*" { Operation::Multiply }

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

        rule _ -> () = " " / "\t"
    }
}

pub fn parse(input: &str) -> Result<Expr<()>, ParseError> {
    parser::expr(input).map_err(ParseError::Peg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_an_integer() {
        arbtest::builder().run(|u| {
            let value = u.arbitrary::<Int>()?;
            let string = value.to_string();
            let expr = parse(&string);
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
    fn test_parsing_an_integer_with_underscores() -> Result<(), ParseError> {
        let expr = parse("123_456_789")?;
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
            let expr = parse(&string);
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
    fn test_parsing_a_more_complex_operation() {
        arbtest::builder().run(|u| {
            let a = u.arbitrary::<Int>()?;
            let b = u.arbitrary::<Int>()?;
            let c = u.arbitrary::<Int>()?;
            let string = format!("{} + {} * {}", a, b, c);
            let expr = parse(&string);
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
}
