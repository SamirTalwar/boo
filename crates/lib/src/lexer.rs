use std::str::FromStr;

use logos::Logos;

use crate::error::*;
use crate::identifier::*;
use crate::primitive::*;
use crate::span::*;

#[derive(Debug, Clone, PartialEq, Eq, Logos)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token<'a> {
    #[token(r"(")]
    StartGroup,
    #[token(r")")]
    EndGroup,
    #[token(r"let")]
    Let,
    #[token(r"in")]
    In,
    #[token(r"fn")]
    Fn,
    #[token(r"=")]
    Assign,
    #[token(r"->")]
    Arrow,
    #[regex(r"-?[0-9](_?[0-9])*", |token|
        str::replace(token.slice(), "_", "").parse::<Integer>().ok()
    )]
    Integer(Integer),
    #[regex(r"\+|\-|\*")]
    Operator(&'a str),
    // note that the following regex is duplicated from identifier.rs
    #[regex(r"[_\p{Letter}][_\p{Number}\p{Letter}]*", |token|
        Identifier::from_str(token.slice()).map_err(|_| ())
    )]
    Identifier(Identifier),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotatedToken<'a, Annotation> {
    pub annotation: Annotation,
    pub token: Token<'a>,
}

pub fn lex(input: &str) -> Result<Vec<AnnotatedToken<Span>>> {
    Token::lexer(input)
        .spanned()
        .map(move |(token, span)| {
            let span: Span = span.into();
            token
                .map(|value| AnnotatedToken {
                    annotation: span,
                    token: value,
                })
                .map_err(|_| Error::UnexpectedToken {
                    span,
                    token: input[span.range()].to_string(),
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use boo_test_helpers::proptest::*;

    use super::*;

    #[test]
    fn test_lexing_nothing() {
        let input = "";
        let tokens = lex(input);

        assert_eq!(tokens, Ok(vec![]));
    }

    #[test]
    fn test_lexing_an_integer() {
        let input = "123";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            Ok(vec![AnnotatedToken {
                annotation: (0..3).into(),
                token: Token::Integer(123.into()),
            }])
        );
    }

    #[test]
    fn test_lexing_a_negative_integer() {
        let input = "-456";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            Ok(vec![AnnotatedToken {
                annotation: (0..4).into(),
                token: Token::Integer((-456).into()),
            }])
        );
    }

    #[test]
    fn test_lexing_an_integer_with_underscores() {
        let input = "987_654_321";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            Ok(vec![AnnotatedToken {
                annotation: (0..11).into(),
                token: Token::Integer(987_654_321.into()),
            }])
        );
    }

    #[test]
    fn test_lexing_operators() {
        let input = "1 + 2 - 3 * 4";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            Ok(vec![
                AnnotatedToken {
                    annotation: (0..1).into(),
                    token: Token::Integer(1.into()),
                },
                AnnotatedToken {
                    annotation: (2..3).into(),
                    token: Token::Operator("+"),
                },
                AnnotatedToken {
                    annotation: (4..5).into(),
                    token: Token::Integer(2.into()),
                },
                AnnotatedToken {
                    annotation: (6..7).into(),
                    token: Token::Operator("-"),
                },
                AnnotatedToken {
                    annotation: (8..9).into(),
                    token: Token::Integer(3.into()),
                },
                AnnotatedToken {
                    annotation: (10..11).into(),
                    token: Token::Operator("*"),
                },
                AnnotatedToken {
                    annotation: (12..13).into(),
                    token: Token::Integer(4.into()),
                },
            ])
        );
    }

    #[test]
    fn test_lexing_parentheses() {
        let input = "1 * (2 + 3) - 4";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            Ok(vec![
                AnnotatedToken {
                    annotation: (0..1).into(),
                    token: Token::Integer(1.into()),
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
                    token: Token::Integer(2.into()),
                },
                AnnotatedToken {
                    annotation: (7..8).into(),
                    token: Token::Operator("+"),
                },
                AnnotatedToken {
                    annotation: (9..10).into(),
                    token: Token::Integer(3.into()),
                },
                AnnotatedToken {
                    annotation: (10..11).into(),
                    token: Token::EndGroup,
                },
                AnnotatedToken {
                    annotation: (12..13).into(),
                    token: Token::Operator("-"),
                },
                AnnotatedToken {
                    annotation: (14..15).into(),
                    token: Token::Integer(4.into()),
                },
            ])
        );
    }

    #[test]
    fn test_lexing_identifier() {
        check(&Identifier::arbitrary(), |identifier| {
            let input = format!("{}", identifier);
            let tokens = lex(&input);

            prop_assert_eq!(
                tokens,
                Ok(vec![AnnotatedToken {
                    annotation: (0..input.len()).into(),
                    token: Token::Identifier(identifier),
                }])
            );
            Ok(())
        })
    }

    #[test]
    fn test_lexing_variable_assignment() {
        let input = "let thing = 9";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            Ok(vec![
                AnnotatedToken {
                    annotation: (0..3).into(),
                    token: Token::Let,
                },
                AnnotatedToken {
                    annotation: (4..9).into(),
                    token: Token::Identifier(Identifier::from_str("thing").unwrap()),
                },
                AnnotatedToken {
                    annotation: (10..11).into(),
                    token: Token::Assign,
                },
                AnnotatedToken {
                    annotation: (12..13).into(),
                    token: Token::Integer(9.into()),
                },
            ])
        );
    }

    #[test]
    fn test_lexing_variable_use() {
        let input = "foo + bar";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            Ok(vec![
                AnnotatedToken {
                    annotation: (0..3).into(),
                    token: Token::Identifier(Identifier::from_str("foo").unwrap()),
                },
                AnnotatedToken {
                    annotation: (4..5).into(),
                    token: Token::Operator("+"),
                },
                AnnotatedToken {
                    annotation: (6..9).into(),
                    token: Token::Identifier(Identifier::from_str("bar").unwrap()),
                },
            ])
        );
    }

    #[test]
    fn test_lexing_variable_assignment_and_use() {
        let input = "let price = 3 in let quantity = 5 in price * quantity";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            Ok(vec![
                AnnotatedToken {
                    annotation: (0..3).into(),
                    token: Token::Let,
                },
                AnnotatedToken {
                    annotation: (4..9).into(),
                    token: Token::Identifier(Identifier::from_str("price").unwrap()),
                },
                AnnotatedToken {
                    annotation: (10..11).into(),
                    token: Token::Assign,
                },
                AnnotatedToken {
                    annotation: (12..13).into(),
                    token: Token::Integer(3.into()),
                },
                AnnotatedToken {
                    annotation: (14..16).into(),
                    token: Token::In,
                },
                AnnotatedToken {
                    annotation: (17..20).into(),
                    token: Token::Let,
                },
                AnnotatedToken {
                    annotation: (21..29).into(),
                    token: Token::Identifier(Identifier::from_str("quantity").unwrap()),
                },
                AnnotatedToken {
                    annotation: (30..31).into(),
                    token: Token::Assign,
                },
                AnnotatedToken {
                    annotation: (32..33).into(),
                    token: Token::Integer(5.into()),
                },
                AnnotatedToken {
                    annotation: (34..36).into(),
                    token: Token::In,
                },
                AnnotatedToken {
                    annotation: (37..42).into(),
                    token: Token::Identifier(Identifier::from_str("price").unwrap()),
                },
                AnnotatedToken {
                    annotation: (43..44).into(),
                    token: Token::Operator("*"),
                },
                AnnotatedToken {
                    annotation: (45..53).into(),
                    token: Token::Identifier(Identifier::from_str("quantity").unwrap()),
                },
            ])
        );
    }

    #[test]
    fn test_lexing_a_function() {
        let input = "fn x -> x + 1";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            Ok(vec![
                AnnotatedToken {
                    annotation: (0..2).into(),
                    token: Token::Fn,
                },
                AnnotatedToken {
                    annotation: (3..4).into(),
                    token: Token::Identifier(Identifier::from_str("x").unwrap()),
                },
                AnnotatedToken {
                    annotation: (5..7).into(),
                    token: Token::Arrow,
                },
                AnnotatedToken {
                    annotation: (8..9).into(),
                    token: Token::Identifier(Identifier::from_str("x").unwrap()),
                },
                AnnotatedToken {
                    annotation: (10..11).into(),
                    token: Token::Operator("+"),
                },
                AnnotatedToken {
                    annotation: (12..13).into(),
                    token: Token::Integer(1.into()),
                },
            ])
        );
    }

    #[test]
    fn test_lexing_rejects_anything_else() {
        let input = "1 / 2";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            Err(Error::UnexpectedToken {
                span: (2..3).into(),
                token: "/".to_string(),
            })
        );
    }
}
