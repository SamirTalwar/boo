#[derive(Debug, thiserror::Error, miette::Diagnostic, PartialEq)]
pub enum Error {
    #[error("Parse error: {inner}")]
    #[diagnostic(code(boo::parse_error))]
    ParseError {
        #[label = "error parsing at this location"]
        span: miette::SourceSpan,
        inner: peg::error::ParseError<peg::str::LineCol>,
    },
}
