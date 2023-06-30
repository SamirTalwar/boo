use std::sync::Arc;

use crate::error::Result;
use crate::identifier::Identifier;
use crate::primitive::Primitive;

pub trait NativeContext {
    fn lookup_value(&self, identifier: &Identifier) -> Result<Primitive>;
}

type Implementation = Arc<dyn Fn(&dyn NativeContext) -> Result<Primitive>>;

#[derive(Clone)]
pub struct Native {
    pub unique_name: Identifier,
    pub implementation: Implementation,
}

impl std::fmt::Debug for Native {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "native {:?}", self.unique_name)
    }
}

impl std::fmt::Display for Native {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.unique_name.fmt(f)
    }
}

impl PartialEq for Native {
    fn eq(&self, other: &Self) -> bool {
        self.unique_name == other.unique_name
    }
}

impl Eq for Native {}

impl std::hash::Hash for Native {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.unique_name.hash(state)
    }
}
