use nom::branch::alt;
use nom::character::complete::{char, multispace0, one_of};
use nom::combinator::{complete, map, map_res, opt, recognize};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::IResult;

use crate::ast::*;

type ParseResult<'a, T> = IResult<&'a str, T>;

#[derive(Debug, PartialEq)]
pub enum ParseError {
    Remaining(String),
    Nom(nom::Err<nom::error::Error<String>>),
}

pub fn parse(input: &str) -> Result<Expr<()>, ParseError> {
    match complete(parse_expr)(input) {
        Ok(("", expr)) => Ok(expr),
        Ok((remaining, _)) => Err(ParseError::Remaining(remaining.to_string())),
        Err(err) => Err(ParseError::Nom(err.map_input(|s| s.to_string()))),
    }
}

fn parse_expr(input: &str) -> ParseResult<'_, Expr<()>> {
    alt((parse_infix, parse_primitive))(input)
}

fn parse_primitive(input: &str) -> ParseResult<'_, Expr<()>> {
    map(
        map_res(
            recognize(preceded(
                opt(char('-')),
                many1(terminated(one_of("0123456789"), many0(char('_')))),
            )),
            |out: &str| str::replace(out, "_", "").parse::<Int>(),
        ),
        |value| Expr::Primitive {
            annotation: (),
            value: Primitive::Int(value),
        },
    )(input)
}

fn parse_infix(input: &str) -> ParseResult<'_, Expr<()>> {
    map(
        tuple((parse_primitive, parse_operation, parse_expr)),
        |(left, operation, right)| Expr::Infix {
            annotation: (),
            operation,
            left: Box::new(left),
            right: Box::new(right),
        },
    )(input)
}

fn parse_operation(input: &str) -> ParseResult<'_, Operation> {
    ws(alt((
        map(char('+'), |_| Operation::Add),
        map(char('-'), |_| Operation::Subtract),
        map(char('*'), |_| Operation::Multiply),
    )))(input)
}

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
fn ws<'a, F, T>(inner: F) -> impl FnMut(&'a str) -> ParseResult<T>
where
    F: FnMut(&'a str) -> ParseResult<T>,
{
    delimited(multispace0, inner, multispace0)
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
