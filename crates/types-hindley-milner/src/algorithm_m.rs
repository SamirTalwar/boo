#![cfg(test)] // not finished yet; see the broken tests below

use boo_core::builtins;
use boo_core::error::{Error, Result};
use boo_core::expr::{self, Expr, Expression};
use boo_core::primitive::Primitive;
use boo_core::types::{Monotype, Polytype, Type};

use crate::env::Env;
use crate::fresh::FreshVariables;
use crate::subst::Subst;
use crate::types::{FreeVariables, Monomorphic, Polymorphic};
use crate::unification::unify;

pub fn type_of(expr: &Expr) -> Result<Monotype> {
    let base_context = builtins::types()
        .map(|(name, typ)| (name.clone(), typ))
        .collect::<Env>();
    let mut fresh = FreshVariables::new();
    let target = Monotype::from(Type::Variable(fresh.next()));
    let subst = infer(base_context, &mut fresh, expr, target.clone())?;
    let mut result = target;
    loop {
        let next = result.substitute(&subst);
        if result == next {
            break;
        }
        result = next;
    }
    Ok(result)
}

fn infer(
    env: Env,
    fresh: &mut FreshVariables,
    expr: &Expr,
    target_type: Monotype,
) -> Result<Subst> {
    match expr.expression.as_ref() {
        Expression::Primitive(Primitive::Integer(_)) => unify(&target_type, &Type::Integer.into())
            .ok_or_else(|| Error::TypeMismatch {
                span: expr.span,
                expected_type: target_type,
                actual_type: Type::Integer.into(),
            }),
        Expression::Native(_) => unreachable!("Native expression without a type."),
        Expression::Identifier(identifier) => env
            .get(identifier)
            .ok_or_else(|| Error::UnknownVariable {
                span: expr.span,
                name: identifier.to_string(),
            })
            .and_then(|typ| {
                let source_type = typ.substitute(&Subst::empty(), fresh).mono;
                unify(&target_type, &source_type).ok_or(Error::TypeMismatch {
                    span: expr.span,
                    expected_type: target_type,
                    actual_type: source_type,
                })
            }),
        Expression::Function(expr::Function { parameter, body }) => {
            let parameter_type = Monotype::from(Type::Variable(fresh.next()));
            let body_type = Monotype::from(Type::Variable(fresh.next()));
            let source_type = Monotype::from(Type::Function {
                parameter: parameter_type.clone(),
                body: body_type.clone(),
            });
            let function_subst = unify(&target_type, &source_type).ok_or(Error::TypeMismatch {
                span: expr.span,
                expected_type: target_type,
                actual_type: source_type,
            })?;
            let substituted_body_type = body_type.substitute(&function_subst);
            let body_env = env.substitute(&function_subst, fresh).update(
                parameter.clone(),
                Polytype::unquantified(parameter_type.substitute(&function_subst)),
            );
            let body_subst = infer(body_env, fresh, body, substituted_body_type)?;
            Ok(function_subst.then(&body_subst))
        }
        Expression::Apply(expr::Apply { function, argument }) => {
            let parameter_type = Monotype::from(Type::Variable(fresh.next()));
            let function_type = Monotype::from(Type::Function {
                parameter: parameter_type.clone(),
                body: target_type.clone(),
            });
            let function_subst = infer(env.clone(), fresh, function, function_type.clone())?;
            let argument_type = parameter_type.substitute(&function_subst);
            let argument_env = env.substitute(&function_subst, fresh);
            let argument_subst = infer(argument_env, fresh, argument, argument_type.clone())?;
            function_subst
                .merge(&argument_subst)
                .ok_or_else(|| Error::TypeMismatch {
                    span: argument.span,
                    expected_type: target_type.substitute(&function_subst),
                    actual_type: argument_type.substitute(&function_subst.then(&argument_subst)),
                })
        }
        Expression::Assign(expr::Assign { name, value, inner }) => {
            let value_type = Monotype::from(Type::Variable(fresh.next()));
            let value_subst = infer(env.clone(), fresh, value, value_type.clone())?;
            let substituted_value_type = value_type.substitute(&value_subst);
            let inner_type = target_type.substitute(&value_subst);
            let inner_env = env.substitute(&value_subst, fresh).update(
                name.clone(),
                Polytype {
                    quantifiers: substituted_value_type
                        .free()
                        .relative_complement(env.free())
                        .into_iter()
                        .collect(),
                    mono: substituted_value_type.substitute(&value_subst),
                },
            );
            let inner_subst = infer(inner_env, fresh, inner, inner_type)?;
            Ok(value_subst.then(&inner_subst))
        }
        Expression::Match(expr::Match { value, patterns }) => {
            let value_type = Monotype::from(Type::Variable(fresh.next()));
            let _ = infer(env.clone(), fresh, value, value_type)?;
            patterns.iter().try_fold(
                Subst::empty(),
                |subst, expr::PatternMatch { pattern: _, result }| {
                    let result_subst = infer(env.clone(), fresh, result, target_type.clone())?;
                    subst
                        .merge(&result_subst)
                        .ok_or_else(|| Error::TypeMismatch {
                            span: expr.span,
                            expected_type: target_type.substitute(&subst),
                            actual_type: target_type.substitute(&result_subst),
                        })
                },
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use boo_core::identifier::Identifier;
    use boo_core::types::TypeVariable;
    use boo_parser::parse;
    use boo_test_helpers::proptest::check;

    use super::*;

    #[ignore]
    #[test]
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
            eprintln!("rendered: {rendered}");
            let expr = input.clone().to_core()?;

            let actual_type = type_of(&expr)?;

            prop_assert_eq!(actual_type, Type::Integer.into());
            Ok(())
        })
    }

    #[test]
    fn test_rejects_incorrect_types() -> Result<()> {
        let program = "1 + (fn x -> 3)";
        let ast = parse(program)?.to_core()?;

        let result = type_of(&ast);

        assert_eq!(
            result,
            Err(Error::TypeMismatch {
                span: Some((5..14).into()),
                expected_type: Type::Integer.into(),
                actual_type: Type::Function {
                    parameter: Type::Variable(TypeVariable::new_from_str("_5")).into(),
                    body: Type::Variable(TypeVariable::new_from_str("_6")).into(), // TODO: should be `Type::Integer`
                }
                .into(),
            }),
        );
        Ok(())
    }

    #[test]
    fn test_parameters_are_monomorphic() -> Result<()> {
        let program = "fn x -> x x";
        let ast = parse(program)?.to_core()?;

        let result = type_of(&ast);

        assert_eq!(
            result,
            Err(Error::TypeMismatch {
                span: Some((10..11).into()),
                expected_type: Type::Variable(TypeVariable::new_from_str("_4")).into(),
                actual_type: Type::Function {
                    parameter: Type::Variable(TypeVariable::new_from_str("_4")).into(),
                    body: Type::Variable(TypeVariable::new_from_str("_2")).into(),
                }
                .into()
            }),
        );
        Ok(())
    }

    #[test]
    fn test_match_expressions_must_be_of_the_same_type() -> Result<()> {
        let program = "match 0 { 1 -> 2; _ -> fn x -> x }";
        let ast = parse(program)?.to_core()?;

        let result = type_of(&ast);

        assert_eq!(
            result,
            Err(Error::TypeMismatch {
                span: Some((0..34).into()), // TODO: should be `(23..32)`
                expected_type: Type::Integer.into(),
                actual_type: Type::Function {
                    parameter: Type::Variable(TypeVariable::new_from_str("_2")).into(),
                    body: Type::Variable(TypeVariable::new_from_str("_3")).into(), // TOOD: should be `"_2"`
                }
                .into(),
            }),
        );
        Ok(())
    }
}
