use std::ops::{BitOr, Range};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }
}

impl BitOr for Span {
    type Output = Span;

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Spanned<Value> {
    pub span: Span,
    pub value: Value,
}

// ignores the actual span
impl<Value: std::fmt::Display> std::fmt::Display for Spanned<Value> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
