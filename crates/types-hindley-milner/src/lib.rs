use std::fmt::Display;
use std::iter;
use std::sync::Arc;

use lazy_static::lazy_static;

use boo_core::builtins;
use boo_core::error::{Error, Result};
use boo_core::expr::{self, Expr, Expression};
use boo_core::identifier::Identifier;
use boo_core::primitive::Primitive;
use boo_core::types::{Monotype, Polytype, Type, TypeVariable};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Env(im::HashMap<Arc<Identifier>, Polytype>);

impl Env {
    fn get(&self, key: &Identifier) -> Option<&Polytype> {
        self.0.get(key)
    }

    fn update(&self, key: Identifier, value: Polytype) -> Self {
        Self(self.0.update(Arc::new(key), value))
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

struct Subst(im::HashMap<TypeVariable, Monotype>);

impl Subst {
    fn new() -> Self {
        Self(im::HashMap::new())
    }

    fn get(&self, key: &TypeVariable) -> Option<&Monotype> {
        self.0.get(key)
    }

    fn then(self, other: Self) -> Self {
        let mut empty_fresh = FreshVariables::new();
        Self(self.0.clone().union_with(other.0, |_, later_type| {
            later_type.substitute(&self, &mut empty_fresh)
        }))
    }

    fn merge(self, other: Self) -> Result<Self> {
        {
            let disagreeing_variables = self
                .0
                .clone()
                .intersection_with(other.0.clone(), |a, b| (a, b))
                .into_iter()
                .filter(|(v, _)| {
                    let mut empty_fresh = FreshVariables::new();
                    let var = Type::Variable(v.clone());
                    var.substitute(&self, &mut empty_fresh)
                        != var.substitute(&other, &mut empty_fresh)
                })
                .collect::<std::collections::BTreeMap<_, _>>();
            if !disagreeing_variables.is_empty() {
                return Err(Error::TypeSubstitutionOverlapError {
                    variables: disagreeing_variables,
                });
            }
        }
        Ok(Self(self.0.union(other.0)))
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

trait Types {
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

lazy_static! {
    static ref INTEGER_TYPE: Monotype = Type::Integer.into();
}

pub fn w(expr: &Expr) -> Result<Monotype> {
    let (_, typ) = W::type_of(expr)?;
    Ok(typ)
}

struct W {}

impl W {
    fn type_of(expr: &Expr) -> Result<(Subst, Monotype)> {
        let base_context = builtins::types()
            .map(|(name, typ)| (name.clone(), typ))
            .collect::<Env>();
        let mut fresh = FreshVariables::new();
        Self::infer(base_context, &mut fresh, expr)
    }

    fn infer(env: Env, fresh: &mut FreshVariables, expr: &Expr) -> Result<(Subst, Monotype)> {
        match expr.expression.as_ref() {
            Expression::Primitive(Primitive::Integer(_)) => {
                Ok((Subst::new(), INTEGER_TYPE.clone()))
            }
            Expression::Native(native) => env
                .get(&native.unique_name)
                .ok_or_else(|| Error::UnknownVariable {
                    span: expr.span,
                    name: native.unique_name.to_string(),
                })
                .map(|typ| (Subst::new(), typ.substitute(&Subst::new(), fresh).mono)),
            Expression::Identifier(identifier) => env
                .get(identifier)
                .ok_or_else(|| Error::UnknownVariable {
                    span: expr.span,
                    name: identifier.to_string(),
                })
                .map(|typ| (Subst::new(), typ.substitute(&Subst::new(), fresh).mono)),
            Expression::Function(expr::Function { parameter, body }) => {
                let parameter_type = Type::Variable(fresh.next());
                let (subst, body_type) = Self::infer(
                    env.update(
                        parameter.clone(),
                        Polytype::unquantified(parameter_type.clone().into()),
                    ),
                    fresh,
                    body,
                )?;
                let result = Type::Function {
                    parameter: parameter_type.into(),
                    body: body_type,
                }
                .substitute(&subst, fresh)
                .into();
                Ok((subst, result))
            }
            Expression::Apply(expr::Apply { function, argument }) => {
                let (function_subst, function_type) = Self::infer(env.clone(), fresh, function)?;
                let (argument_subst, argument_type) =
                    Self::infer(env.substitute(&function_subst, fresh), fresh, argument)?;
                let body_type: Monotype = Type::Variable(fresh.next()).into();
                let expected_function_type: Monotype = Type::Function {
                    parameter: argument_type.clone(),
                    body: body_type.clone(),
                }
                .into();
                let body_subst = Self::unify(
                    &function_type.substitute(&argument_subst, fresh),
                    &expected_function_type,
                )
                .ok_or(Error::TypeUnificationError {
                    left_span: function.span,
                    left_type: function_type,
                    right_span: argument.span,
                    right_type: argument_type,
                })?;
                let result = body_type.substitute(&body_subst, fresh);
                let subst = function_subst.then(argument_subst).then(body_subst);
                Ok((subst, result))
            }
            Expression::Assign(expr::Assign { name, value, inner }) => {
                let (value_subst, value_type) = Self::infer(env.clone(), fresh, value)?;
                let (inner_subst, inner_type) = Self::infer(
                    env.substitute(&value_subst, fresh).update(
                        name.clone(),
                        Polytype {
                            quantifiers: value_type
                                .free()
                                .relative_complement(env.free())
                                .into_iter()
                                .collect(),
                            mono: value_type,
                        },
                    ),
                    fresh,
                    inner,
                )?;
                let subst = value_subst.then(inner_subst);
                Ok((subst, inner_type))
            }
            Expression::Match(expr::Match { value, patterns }) => {
                let _ = Self::infer(env.clone(), fresh, value)?;
                let result_placeholder = Type::Variable(fresh.next()).into();
                let mut subst = Subst::new();
                for expr::PatternMatch { pattern: _, result } in patterns {
                    let (result_subst, result_type) = Self::infer(env.clone(), fresh, result)?;
                    let unified =
                        Self::unify(&result_type, &result_placeholder).ok_or_else(|| {
                            Error::TypeUnificationError {
                                left_span: expr.span,
                                left_type: result_placeholder.clone(),
                                right_span: result.span,
                                right_type: result_type.clone(),
                            }
                        })?;
                    subst = subst.then(result_subst).merge(unified)?;
                }
                let result = result_placeholder.substitute(&subst, fresh);
                Ok((subst, result))
            }
        }
    }

    fn unify(left: &Monotype, right: &Monotype) -> Option<Subst> {
        match (left.as_ref(), right.as_ref()) {
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
                let mut empty_fresh = FreshVariables::new();
                let parameter_subst = Self::unify(left_parameter, right_parameter)?;
                let body_subst = Self::unify(
                    &left_body.substitute(&parameter_subst, &mut empty_fresh),
                    &right_body.substitute(&parameter_subst, &mut empty_fresh),
                )?;
                let subst = parameter_subst.then(body_subst);
                Some(subst)
            }
            (Type::Variable(left), Type::Variable(right)) if left == right => Some(Subst::new()),
            (Type::Variable(_), Type::Variable(right)) => {
                Some(Subst::from_iter([(right.clone(), left.clone())]))
            }
            (Type::Variable(var), _) => Some(Subst::from_iter([(var.clone(), right.clone())])),
            (_, Type::Variable(var)) => Some(Subst::from_iter([(var.clone(), left.clone())])),
            (Type::Integer, Type::Integer) => Some(Subst::new()),
            _ => None,
        }
    }
}

struct FreshVariables {
    values: Box<dyn Iterator<Item = TypeVariable>>,
}

impl FreshVariables {
    pub fn new() -> Self {
        Self {
            values: Box::new(
                iter::successors(Some(0), |x| Some(x + 1))
                    .map(|x| TypeVariable::new(format!("_{x}"))),
            ),
        }
    }

    pub fn next(&mut self) -> TypeVariable {
        self.values.next().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use boo_test_helpers::proptest::check;

    use super::*;

    #[test]
    #[ignore = "doesn't fully work yet"]
    fn test_arbitrary_expressions() {
        let generator = boo_generator::gen(
            boo_generator::ExprGenConfig {
                gen_identifier: Identifier::gen_ascii(1..=16).boxed().into(),
                ..Default::default()
            }
            .into(),
        );
        check(&generator, |input| {
            let rendered = format!("{}", input);
            let expr = input.clone().to_core()?;

            let actual_type = w(&expr)?;

            prop_assert_eq!(
                actual_type,
                Type::Integer.into(),
                "\nrendered = {}\n",
                rendered
            );
            Ok(())
        })
    }
}
