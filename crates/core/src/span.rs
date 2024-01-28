//! Represents a span of text in the original source.

use std::ops::{BitOr, Range};

/// A range, representing a span of text in the original source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    /// Converts the span to a [`Range`].
    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }
}

impl BitOr for Span {
    type Output = Span;

    /// Combines two spans to provide a new span encompassing both of the
    /// original ranges.
    fn bitor(self, rhs: Span) -> Self::Output {
        Self::Output {
            start: self.start.min(rhs.start),
            end: self.end.max(rhs.end),
        }
    }
}

impl From<usize> for Span {
    fn from(value: usize) -> Self {
        Self {
            start: value,
            end: value,
        }
    }
}

impl From<Range<usize>> for Span {
    fn from(value: Range<usize>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl From<Span> for miette::SourceSpan {
    fn from(val: Span) -> Self {
        val.range().into()
    }
}

/// A value, optionally associated with a span.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Spanned<Value> {
    pub span: Option<Span>,
    pub value: Value,
}

impl<Value> Spanned<Value> {
    pub fn as_ref(&self) -> Spanned<&Value> {
        Spanned {
            span: self.span,
            value: &self.value,
        }
    }
}
