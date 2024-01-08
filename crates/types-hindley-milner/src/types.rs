use boo_core::types::{Monotype, Polytype, Type, TypeVariable};

use crate::fresh::FreshVariables;
use crate::subst::Subst;

pub trait Types {
    fn free(&self) -> im::HashSet<TypeVariable>;

    fn substitute(&self, substitutions: &Subst, fresh: &mut FreshVariables) -> Self;
}

impl Types for Type<Monotype> {
    fn free(&self) -> im::HashSet<TypeVariable> {
        match self {
            Type::Integer => im::HashSet::new(),
            Type::Function { parameter, body } => parameter.free().union(body.free()),
            Type::Variable(variable) => im::hashset![variable.clone()],
        }
    }

    fn substitute(&self, substitutions: &Subst, fresh: &mut FreshVariables) -> Self {
        match self {
            Type::Integer => Type::Integer,
            Type::Function { parameter, body } => Type::Function {
                parameter: parameter.substitute(substitutions, fresh),
                body: body.substitute(substitutions, fresh),
            },
            Type::Variable(variable) => match substitutions.get(variable) {
                None => Type::Variable(variable.clone()),
                Some(t) => (*t.0).clone(),
            },
        }
    }
}

impl Types for Monotype {
    fn free(&self) -> im::HashSet<TypeVariable> {
        self.0.free()
    }

    fn substitute(&self, substitutions: &Subst, fresh: &mut FreshVariables) -> Self {
        self.0.substitute(substitutions, fresh).into()
    }
}

impl Types for Polytype {
    fn free(&self) -> im::HashSet<TypeVariable> {
        let quantifiers = self.quantifiers.iter().cloned().collect();
        self.mono.free().relative_complement(quantifiers)
    }

    fn substitute(&self, substitutions: &Subst, fresh: &mut FreshVariables) -> Self {
        let replacements = self
            .quantifiers
            .iter()
            .map(|q| (q.clone(), fresh.next()))
            .collect::<Vec<_>>();
        let new_quantifiers = replacements
            .iter()
            .map(|(_, after)| after.clone())
            .collect();
        let replacements_subst = replacements
            .into_iter()
            .map(|(before, after)| (before, Type::Variable(after).into()))
            .collect();
        Self {
            quantifiers: new_quantifiers,
            mono: self
                .mono
                .substitute(&replacements_subst, fresh)
                .substitute(substitutions, fresh),
        }
    }
}
