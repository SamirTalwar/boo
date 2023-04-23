use logos::Logos;
pub use miette::{SourceOffset, SourceSpan};

use crate::error::Error;
use crate::primitive::Int;

#[derive(Debug, Clone, Copy, PartialEq, Logos)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    #[regex(r"-?[0-9](_?[0-9])*", |token|
        str::replace(token.slice(), "_", "").parse::<Int>().ok()
    )]
    Integer(Int),
    #[regex(r"\+|\-|\*", |token| token.slice().chars().next())]
    Operator(char),
    #[token(r"(")]
    StartGroup,
    #[token(r")")]
    EndGroup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Positioned<T> {
    pub span: SourceSpan,
    pub value: T,
}

pub fn lex(input: &str) -> Vec<Result<Positioned<Token>, Error>> {
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

        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn test_lexing_an_integer() {
        let input = "123";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            vec![Ok(Positioned {
                span: (0..3).into(),
                value: Token::Integer(123),
            })]
        );
    }

    #[test]
    fn test_lexing_a_negative_integer() {
        let input = "-456";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            vec![Ok(Positioned {
                span: (0..4).into(),
                value: Token::Integer(-456),
            })]
        );
    }

    #[test]
    fn test_lexing_an_integer_with_underscores() {
        let input = "987_654_321";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            vec![Ok(Positioned {
                span: (0..11).into(),
                value: Token::Integer(987_654_321),
            })]
        );
    }

    #[test]
    fn test_lexing_rejects_an_integer_starting_with_an_underscore() {
        let input = "_2";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            vec![
                Err(Error::UnexpectedToken {
                    span: (0..1).into(),
                    token: "_".to_string(),
                }),
                Ok(Positioned {
                    span: (1..2).into(),
                    value: Token::Integer(2),
                }),
            ]
        );
    }

    #[test]
    fn test_lexing_operators() {
        let input = "1 + 2 - 3 * 4";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            vec![
                Ok(Positioned {
                    span: (0..1).into(),
                    value: Token::Integer(1),
                }),
                Ok(Positioned {
                    span: (2..3).into(),
                    value: Token::Operator('+'),
                }),
                Ok(Positioned {
                    span: (4..5).into(),
                    value: Token::Integer(2),
                }),
                Ok(Positioned {
                    span: (6..7).into(),
                    value: Token::Operator('-'),
                }),
                Ok(Positioned {
                    span: (8..9).into(),
                    value: Token::Integer(3),
                }),
                Ok(Positioned {
                    span: (10..11).into(),
                    value: Token::Operator('*'),
                }),
                Ok(Positioned {
                    span: (12..13).into(),
                    value: Token::Integer(4),
                }),
            ]
        );
    }

    #[test]
    fn test_lexing_parentheses() {
        let input = "1 * (2 + 3) - 4";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            vec![
                Ok(Positioned {
                    span: (0..1).into(),
                    value: Token::Integer(1),
                }),
                Ok(Positioned {
                    span: (2..3).into(),
                    value: Token::Operator('*'),
                }),
                Ok(Positioned {
                    span: (4..5).into(),
                    value: Token::StartGroup,
                }),
                Ok(Positioned {
                    span: (5..6).into(),
                    value: Token::Integer(2),
                }),
                Ok(Positioned {
                    span: (7..8).into(),
                    value: Token::Operator('+'),
                }),
                Ok(Positioned {
                    span: (9..10).into(),
                    value: Token::Integer(3),
                }),
                Ok(Positioned {
                    span: (10..11).into(),
                    value: Token::EndGroup,
                }),
                Ok(Positioned {
                    span: (12..13).into(),
                    value: Token::Operator('-'),
                }),
                Ok(Positioned {
                    span: (14..15).into(),
                    value: Token::Integer(4),
                }),
            ]
        );
    }

    #[test]
    fn test_lexing_rejects_anything_else() {
        let input = "1 / 2";
        let tokens = lex(input);

        assert_eq!(
            tokens,
            vec![
                Ok(Positioned {
                    span: (0..1).into(),
                    value: Token::Integer(1),
                }),
                Err(Error::UnexpectedToken {
                    span: (2..3).into(),
                    token: "/".to_string(),
                }),
                Ok(Positioned {
                    span: (4..5).into(),
                    value: Token::Integer(2),
                }),
            ]
        );
    }
}
