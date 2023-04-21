use nom::character::complete::{char, one_of};
use nom::combinator::{complete, map, map_res, opt, recognize};
use nom::multi::{many0, many1};
use nom::sequence::{preceded, terminated};
use nom::IResult;

use crate::ast::*;

type ParseResult<'a> = IResult<&'a str, Expr<()>>;

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

fn parse_expr(input: &str) -> ParseResult<'_> {
    parse_primitive(input)
}

fn parse_primitive(input: &str) -> ParseResult<'_> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpreting_an_integer() {
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
    fn test_interpreting_an_integer_with_underscores() -> Result<(), ParseError> {
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
}
