#[derive(Debug, PartialEq)]
pub enum BooError {
    ParseError(peg::error::ParseError<peg::str::LineCol>),
}

impl std::fmt::Display for BooError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(error) => write!(f, "Parse error: {}", error),
        }
    }
}
