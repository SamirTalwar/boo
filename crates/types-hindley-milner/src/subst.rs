use std::fmt::Display;

use boo_core::types::{Monotype, Type, TypeVariable};

use crate::types::Monomorphic;

#[derive(Debug, Clone)]
pub struct Subst(im::HashMap<TypeVariable, Monotype>);

impl Subst {
    pub fn empty() -> Self {
        Self(im::HashMap::new())
    }

    pub fn of(key: TypeVariable, value: Monotype) -> Self {
        Self(im::HashMap::from_iter([(key, value)]))
    }

    pub fn get(&self, key: &TypeVariable) -> Option<&Monotype> {
        self.0.get(key)
    }

    pub fn then(&self, other: &Self) -> Self {
        Self(
            self.0
                .clone()
                .union_with(other.0.clone(), |_, later_type| later_type.substitute(self)),
        )
    }

    pub fn merge(&self, other: &Self) -> Option<Self> {
        let new_substitutions = self
            .0
            .clone()
            .intersection_with(other.0.clone(), |a, b| (a, b))
            .into_iter()
            .map(|(v, _)| {
                let var = Type::Variable(v.clone());
                match_types(&var.substitute(self).into(), &var.substitute(other).into())
            })
            .collect::<Option<Vec<Subst>>>()?;
        let existing_substitutions = Self(self.0.clone().union(other.0.clone()));
        let all_substitutions = new_substitutions
            .into_iter()
            .fold(existing_substitutions, |x, y| x.then(&y));
        Some(all_substitutions)
    }
}

impl Display for Subst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut items = self.0.iter();
        if let Some((first_var, first_type)) = items.next() {
            write!(f, "{} ↦ {}", first_var, first_type)?;
            for (next_var, next_type) in items {
                write!(f, ", {} ↦ {}", next_var, next_type)?;
            }
            Ok(())
        } else {
            write!(f, "∅")
        }
    }
}

impl FromIterator<(TypeVariable, Monotype)> for Subst {
    fn from_iter<T: IntoIterator<Item = (TypeVariable, Monotype)>>(iter: T) -> Self {
        Self(im::HashMap::from_iter(iter))
    }
}

fn match_types(left: &Monotype, right: &Monotype) -> Option<Subst> {
    match (left.as_ref(), right.as_ref()) {
        (Type::Integer, Type::Integer) => Some(Subst::empty()),
        (
            Type::Function {
                parameter: left_parameter,
                body: left_body,
            },
            Type::Function {
                parameter: right_parameter,
                body: right_body,
            },
        ) => {
            let parameter_subst = match_types(left_parameter, right_parameter)?;
            let body_subst = match_types(left_body, right_body)?;
            parameter_subst.merge(&body_subst)
        }
        (left, Type::Variable(right)) => Some(Subst::of(right.clone(), left.clone().into())),
        (Type::Variable(left), right) => Some(Subst::of(left.clone(), right.clone().into())),
        _ => None,
    }
}
