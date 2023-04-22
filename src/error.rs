use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Debug, Error, Diagnostic, PartialEq)]
pub enum BooError {
    #[error("Parse error: {inner}")]
    #[diagnostic(code(boo::parse_error))]
    ParseError {
        #[source_code]
        input: String,
        #[label = "error parsing at this location"]
        span: SourceSpan,
        inner: peg::error::ParseError<peg::str::LineCol>,
    },
}
