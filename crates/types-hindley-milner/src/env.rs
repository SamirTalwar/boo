use std::fmt::Display;
use std::sync::Arc;

use boo_core::identifier::Identifier;
use boo_core::types::{Polytype, TypeVariable};

use crate::fresh::FreshVariables;
use crate::subst::Subst;
use crate::types::Types;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Env(im::HashMap<Arc<Identifier>, Polytype>);

impl Env {
    pub fn get(&self, key: &Identifier) -> Option<&Polytype> {
        self.0.get(key)
    }

    pub fn update(&self, key: Identifier, value: Polytype) -> Self {
        Self(self.0.update(Arc::new(key), value))
    }
}

impl Types for Env {
    fn free(&self) -> im::HashSet<TypeVariable> {
        self.0.values().flat_map(|t| t.free().into_iter()).collect()
    }

    fn substitute(&self, substitutions: &Subst, fresh: &mut FreshVariables) -> Self {
        self.0
            .iter()
            .map(|(key, value)| (Arc::clone(key), value.substitute(substitutions, fresh)))
            .collect()
    }
}

impl FromIterator<(Identifier, Polytype)> for Env {
    fn from_iter<T: IntoIterator<Item = (Identifier, Polytype)>>(iter: T) -> Self {
        Self::from_iter(iter.into_iter().map(|(key, value)| (Arc::new(key), value)))
    }
}

impl FromIterator<(Arc<Identifier>, Polytype)> for Env {
    fn from_iter<T: IntoIterator<Item = (Arc<Identifier>, Polytype)>>(iter: T) -> Self {
        Self(im::HashMap::from_iter(iter))
    }
}

impl Display for Env {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut items = self.0.iter();
        if let Some((first_id, first_type)) = items.next() {
            write!(f, "Γ ⊢ {}: {}", first_id.name(), first_type)?;
            for (next_id, next_type) in items {
                write!(f, ", {}: {}", next_id.name(), next_type)?;
            }
            Ok(())
        } else {
            write!(f, "Γ ⊢ ∅")
        }
    }
}
