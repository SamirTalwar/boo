//! The lexer for the Boo language.

use logos::Logos;

use boo_core::error::*;
use boo_core::identifier::*;
use boo_core::primitive::*;
use boo_core::span::*;

/// The set of tokens generated by the lexer.
#[derive(Debug, Clone, PartialEq, Eq, Logos)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token<'a> {
    #[token(r"(")]
    StartGroup,
    #[token(r")")]
    EndGroup,
    #[token(r"{")]
    BlockStart,
    #[token(r"}")]
    BlockEnd,
    #[token(r";")]
    Separator,
    #[token(r"_")]
    Anything,
    #[token(r"let")]
    Let,
    #[token(r"in")]
    In,
    #[token(r"fn")]
    Fn,
    #[token(r"match")]
    Match,
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
        Identifier::name_from_str(token.slice()).map_err(|_| ())
    )]
    Identifier(Identifier),
}

/// A wrapper around a token that provides a specific annotation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotatedToken<'a, Annotation> {
    pub annotation: Annotation,
    pub token: Token<'a>,
}

/// Lexes the input and produces a vector of tokens, annotated with their spans,
/// or an error.
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
