pub mod integer;

pub use integer::*;

#[derive(Debug, Clone, PartialEq, Eq, arbitrary::Arbitrary)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub enum Primitive {
    Integer(Integer),
}

impl std::fmt::Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Primitive::Integer(value) => write!(f, "{}", value),
        }
    }
}
