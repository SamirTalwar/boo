use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, thiserror::Error, miette::Diagnostic)]
pub enum Error {
    #[error("Unexpected token: {token}")]
    #[diagnostic(code(boo::lex::unexpected_token))]
    UnexpectedToken {
        #[label = "unexpected token"]
        span: miette::SourceSpan,
        token: String,
    },

    #[error("Parse error: expected one of {expected_tokens:?}")]
    #[diagnostic(code(boo::parse::error))]
    ParseError {
        #[label = "error parsing at this location"]
        span: miette::SourceSpan,
        expected_tokens: HashSet<&'static str>,
    },
}
