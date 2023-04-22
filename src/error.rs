#[derive(Debug, PartialEq)]
pub enum BooError {
    ParseError(peg::error::ParseError<peg::str::LineCol>),
}
