use boo_core::types::{Monotype, Type, TypeVariable};

use crate::subst::Subst;
use crate::types::{FreeVariables, Monomorphic};

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
        (Type::Variable(l), Type::Variable(r)) if l == r => Some(Subst::empty()),
        (Type::Variable(var), _) => var_bind(var, right),
        (_, Type::Variable(var)) => var_bind(var, left),
        _ => None,
    }
}

fn var_bind(var: &TypeVariable, typ: &Monotype) -> Option<Subst> {
    if typ.free().contains(var) {
        None
    } else {
        Some(Subst::of(var.clone(), typ.clone()))
    }
}
