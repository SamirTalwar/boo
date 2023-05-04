#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
pub enum Primitive {
    Integer(Integer),
}

pub type Integer = i64;

impl std::fmt::Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Primitive::Integer(value) => write!(f, "{}", value),
        }
    }
}
