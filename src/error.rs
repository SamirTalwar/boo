#[derive(Debug, Clone, PartialEq, thiserror::Error, miette::Diagnostic)]
pub enum Error {
    #[error("Unexpected token: {token}")]
    #[diagnostic(code(boo::lex::unexpected_token))]
    UnexpectedToken {
        #[label = "unexpected token"]
        span: miette::SourceSpan,
        token: String,
    },

    #[error("Parse error: {inner}")]
    #[diagnostic(code(boo::parse::error))]
    ParseError {
        #[label = "error parsing at this location"]
        span: miette::SourceSpan,
        inner: peg::error::ParseError<peg::str::LineCol>,
    },
}
