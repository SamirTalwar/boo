pub mod integer;

#[cfg(test)]
use proptest::strategy::Strategy;

pub use integer::*;

#[derive(Debug, Clone, PartialEq, Eq, arbitrary::Arbitrary)]
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

#[cfg(test)]
impl Primitive {
    pub fn arbitrary() -> impl Strategy<Value = Primitive> {
        Integer::arbitrary().prop_map(Primitive::Integer)
    }
}
