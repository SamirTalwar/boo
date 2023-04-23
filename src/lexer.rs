use logos::Logos;

use crate::error::Error;
use crate::primitive::Int;

#[derive(Debug, PartialEq, Logos)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    #[regex(r"-?[0-9](_?[0-9])*", |token|
        str::replace(token.slice(), "_", "").parse::<Int>().ok()
    )]
    Integer(Int),
}

pub fn lex(input: &str) -> impl Iterator<Item = Result<Token, Error>> + '_ {
    Token::lexer(input).spanned().map(move |(token, span)| {
        token.map_err(|_| Error::UnexpectedToken {
            span: span.clone().into(),
            token: input[span].to_string(),
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexing_nothing() {
        let input = "";
        let tokens = lex(input).collect::<Vec<_>>();

        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn test_lexing_an_integer() {
        let input = "123";
        let tokens = lex(input).collect::<Vec<_>>();

        assert_eq!(tokens, vec![Ok(Token::Integer(123))]);
    }

    #[test]
    fn test_lexing_a_negative_integer() {
        let input = "-456";
        let tokens = lex(input).collect::<Vec<_>>();

        assert_eq!(tokens, vec![Ok(Token::Integer(-456))]);
    }

    #[test]
    fn test_lexing_an_integer_with_underscores() {
        let input = "987_654_321";
        let tokens = lex(input).collect::<Vec<_>>();

        assert_eq!(tokens, vec![Ok(Token::Integer(987_654_321))]);
    }

    #[test]
    fn test_lexing_rejects_an_integer_starting_with_an_underscore() {
        let input = "_2";
        let mut tokens = lex(input);

        assert_eq!(
            tokens.next(),
            Some(Err(Error::UnexpectedToken {
                span: miette::SourceSpan::new(
                    miette::SourceOffset::from_location(input, 1, 1),
                    1.into(),
                ),
                token: "_".to_string(),
            }))
        );
    }

    #[test]
    fn test_lexing_rejects_anything_else() {
        let input = "1 / 2";
        let mut tokens = lex(input);

        assert_eq!(tokens.next(), Some(Ok(Token::Integer(1))));

        assert_eq!(
            tokens.next(),
            Some(Err(Error::UnexpectedToken {
                span: miette::SourceSpan::new(
                    miette::SourceOffset::from_location(input, 1, 3),
                    1.into(),
                ),
                token: "/".to_string(),
            }))
        );
    }
}
