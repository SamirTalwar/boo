use logos::Logos;
pub use miette::{SourceOffset, SourceSpan};

use crate::error::Error;
use crate::primitive::Int;

#[derive(Debug, Clone, PartialEq, Logos)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Positioned<T> {
    pub span: SourceSpan,
    pub value: T,
}

pub fn lex(input: &str) -> Result<Vec<Positioned<Token>>, Error> {
    Token::lexer(input)
        .spanned()
        .map(move |(token, span)| {
            let source_span: SourceSpan = span.clone().into();
            token
                .map(|value| Positioned {
                    span: source_span,
                    value,
                })
                .map_err(|_| Error::UnexpectedToken {
                    span: source_span,
                    token: input[span].to_string(),
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
            Ok(vec![Positioned {
                span: (0..3).into(),
                value: Token::Integer(123),
            }])
        );
    }

    #[test]
    fn test_lexing_a_negative_integer() {
        let input = "-456";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            Ok(vec![Positioned {
                span: (0..4).into(),
                value: Token::Integer(-456),
            }])
        );
    }

    #[test]
    fn test_lexing_an_integer_with_underscores() {
        let input = "987_654_321";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            Ok(vec![Positioned {
                span: (0..11).into(),
                value: Token::Integer(987_654_321),
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
                Positioned {
                    span: (0..1).into(),
                    value: Token::Integer(1),
                },
                Positioned {
                    span: (2..3).into(),
                    value: Token::Operator("+"),
                },
                Positioned {
                    span: (4..5).into(),
                    value: Token::Integer(2),
                },
                Positioned {
                    span: (6..7).into(),
                    value: Token::Operator("-"),
                },
                Positioned {
                    span: (8..9).into(),
                    value: Token::Integer(3),
                },
                Positioned {
                    span: (10..11).into(),
                    value: Token::Operator("*"),
                },
                Positioned {
                    span: (12..13).into(),
                    value: Token::Integer(4),
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
                Positioned {
                    span: (0..1).into(),
                    value: Token::Integer(1),
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
                    value: Token::Integer(2),
                },
                Positioned {
                    span: (7..8).into(),
                    value: Token::Operator("+"),
                },
                Positioned {
                    span: (9..10).into(),
                    value: Token::Integer(3),
                },
                Positioned {
                    span: (10..11).into(),
                    value: Token::EndGroup,
                },
                Positioned {
                    span: (12..13).into(),
                    value: Token::Operator("-"),
                },
                Positioned {
                    span: (14..15).into(),
                    value: Token::Integer(4),
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
                Positioned {
                    span: (0..3).into(),
                    value: Token::Let,
                },
                Positioned {
                    span: (4..9).into(),
                    value: Token::Identifier("thing"),
                },
                Positioned {
                    span: (10..11).into(),
                    value: Token::Assign,
                },
                Positioned {
                    span: (12..13).into(),
                    value: Token::Integer(9),
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
                Positioned {
                    span: (0..3).into(),
                    value: Token::Identifier("foo"),
                },
                Positioned {
                    span: (4..5).into(),
                    value: Token::Operator("+"),
                },
                Positioned {
                    span: (6..9).into(),
                    value: Token::Identifier("bar"),
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
                Positioned {
                    span: (0..3).into(),
                    value: Token::Let,
                },
                Positioned {
                    span: (4..9).into(),
                    value: Token::Identifier("price"),
                },
                Positioned {
                    span: (10..11).into(),
                    value: Token::Assign,
                },
                Positioned {
                    span: (12..13).into(),
                    value: Token::Integer(3),
                },
                Positioned {
                    span: (14..16).into(),
                    value: Token::In,
                },
                Positioned {
                    span: (17..20).into(),
                    value: Token::Let,
                },
                Positioned {
                    span: (21..29).into(),
                    value: Token::Identifier("quantity"),
                },
                Positioned {
                    span: (30..31).into(),
                    value: Token::Assign,
                },
                Positioned {
                    span: (32..33).into(),
                    value: Token::Integer(5),
                },
                Positioned {
                    span: (34..36).into(),
                    value: Token::In,
                },
                Positioned {
                    span: (37..42).into(),
                    value: Token::Identifier("price"),
                },
                Positioned {
                    span: (43..44).into(),
                    value: Token::Operator("*"),
                },
                Positioned {
                    span: (45..53).into(),
                    value: Token::Identifier("quantity"),
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
