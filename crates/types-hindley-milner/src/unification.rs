use boo_core::types::{Monotype, Type};

use crate::subst::Subst;
use crate::types::Monomorphic;

pub fn unify(left: &Monotype, right: &Monotype) -> Option<Subst> {
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
            let parameter_subst = unify(left_parameter, right_parameter)?;
            let body_subst = unify(
                &left_body.substitute(&parameter_subst),
                &right_body.substitute(&parameter_subst),
            )?;
            let subst = parameter_subst.then(&body_subst);
            Some(subst)
        }
        (Type::Variable(left), Type::Variable(right)) if left == right => Some(Subst::empty()),
        (Type::Variable(_), Type::Variable(right)) => Some(Subst::of(right.clone(), left.clone())),
        (Type::Variable(var), _) => Some(Subst::of(var.clone(), right.clone())),
        (_, Type::Variable(var)) => Some(Subst::of(var.clone(), left.clone())),
        _ => None,
    }
}
