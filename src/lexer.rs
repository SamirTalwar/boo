use logos::Logos;

use crate::error::{Error, Result};
use crate::primitive::Int;
use crate::span::Span;

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
    #[token(r"=")]
    Assign,
    #[regex(r"-?[0-9](_?[0-9])*", |token|
        str::replace(token.slice(), "_", "").parse::<Int>().ok()
    )]
    Integer(Int),
    #[regex(r"\+|\-|\*")]
    Operator(&'a str),
    #[regex(r"[a-z]+")]
    Identifier(&'a str),
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
                token: Token::Integer(123),
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
                token: Token::Integer(-456),
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
                token: Token::Integer(987_654_321),
            }])
        );
    }

    #[test]
    fn test_lexing_rejects_an_integer_starting_with_an_underscore() {
        let input = "_2";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            Err(Error::UnexpectedToken {
                span: (0..1).into(),
                token: "_".to_string(),
            })
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
                    token: Token::Integer(1),
                },
                AnnotatedToken {
                    annotation: (2..3).into(),
                    token: Token::Operator("+"),
                },
                AnnotatedToken {
                    annotation: (4..5).into(),
                    token: Token::Integer(2),
                },
                AnnotatedToken {
                    annotation: (6..7).into(),
                    token: Token::Operator("-"),
                },
                AnnotatedToken {
                    annotation: (8..9).into(),
                    token: Token::Integer(3),
                },
                AnnotatedToken {
                    annotation: (10..11).into(),
                    token: Token::Operator("*"),
                },
                AnnotatedToken {
                    annotation: (12..13).into(),
                    token: Token::Integer(4),
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
                    token: Token::Integer(1),
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
                    token: Token::Integer(2),
                },
                AnnotatedToken {
                    annotation: (7..8).into(),
                    token: Token::Operator("+"),
                },
                AnnotatedToken {
                    annotation: (9..10).into(),
                    token: Token::Integer(3),
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
                    token: Token::Integer(4),
                },
            ])
        );
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
                    token: Token::Identifier("thing"),
                },
                AnnotatedToken {
                    annotation: (10..11).into(),
                    token: Token::Assign,
                },
                AnnotatedToken {
                    annotation: (12..13).into(),
                    token: Token::Integer(9),
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
                    token: Token::Identifier("foo"),
                },
                AnnotatedToken {
                    annotation: (4..5).into(),
                    token: Token::Operator("+"),
                },
                AnnotatedToken {
                    annotation: (6..9).into(),
                    token: Token::Identifier("bar"),
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
                    token: Token::Identifier("price"),
                },
                AnnotatedToken {
                    annotation: (10..11).into(),
                    token: Token::Assign,
                },
                AnnotatedToken {
                    annotation: (12..13).into(),
                    token: Token::Integer(3),
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
                    token: Token::Identifier("quantity"),
                },
                AnnotatedToken {
                    annotation: (30..31).into(),
                    token: Token::Assign,
                },
                AnnotatedToken {
                    annotation: (32..33).into(),
                    token: Token::Integer(5),
                },
                AnnotatedToken {
                    annotation: (34..36).into(),
                    token: Token::In,
                },
                AnnotatedToken {
                    annotation: (37..42).into(),
                    token: Token::Identifier("price"),
                },
                AnnotatedToken {
                    annotation: (43..44).into(),
                    token: Token::Operator("*"),
                },
                AnnotatedToken {
                    annotation: (45..53).into(),
                    token: Token::Identifier("quantity"),
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
