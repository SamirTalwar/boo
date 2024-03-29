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
    let (_, typ) = infer(base_context, &mut fresh, expr)?;
    Ok(typ)
}

fn infer(env: Env, fresh: &mut FreshVariables, expr: &Expr) -> Result<(Subst, Monotype)> {
    match expr.expression() {
        Expression::Primitive(Primitive::Integer(_)) => Ok((Subst::empty(), Type::Integer.into())),
        Expression::Native(_) => unreachable!("Native expression without a type."),
        Expression::Identifier(identifier) => env
            .get(identifier)
            .ok_or_else(|| Error::UnknownVariable {
                span: expr.span(),
                name: identifier.to_string(),
            })
            .map(|typ| (Subst::empty(), typ.substitute(&Subst::empty(), fresh).mono)),
        Expression::Function(expr::Function { parameter, body }) => {
            let parameter_type = Type::Variable(fresh.next());
            let (subst, body_type) = infer(
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
            .substitute(&subst)
            .into();
            Ok((subst, result))
        }
        Expression::Apply(expr::Apply { function, argument }) => {
            let (function_subst, function_type) = infer(env.clone(), fresh, function)?;
            let (argument_subst, argument_type) =
                infer(env.substitute(&function_subst, fresh), fresh, argument)?;
            let body_type: Monotype = Type::Variable(fresh.next()).into();
            let expected_function_type: Monotype = Type::Function {
                parameter: argument_type.clone(),
                body: body_type.clone(),
            }
            .into();
            let body_subst = unify(
                &function_type.substitute(&argument_subst),
                &expected_function_type,
            )
            .ok_or(Error::TypeUnificationError {
                left_span: function.span(),
                left_type: function_type,
                right_span: argument.span(),
                right_type: argument_type,
            })?;
            let result = body_type.substitute(&body_subst);
            let subst = function_subst.then(&argument_subst).then(&body_subst);
            Ok((subst, result))
        }
        Expression::Assign(expr::Assign { name, value, inner }) => {
            let (value_subst, value_type) = infer(env.clone(), fresh, value)?;
            let (inner_subst, inner_type) = infer(
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
            let subst = value_subst.then(&inner_subst);
            Ok((subst, inner_type))
        }
        Expression::Match(expr::Match { value, patterns }) => {
            let _ = infer(env.clone(), fresh, value)?;
            let result_placeholder = Type::Variable(fresh.next()).into();
            let mut pattern_iter = patterns.iter();
            let expr::PatternMatch {
                pattern: _,
                result: first_result,
            } = pattern_iter
                .next()
                .ok_or(Error::MatchWithoutBaseCase { span: expr.span() })?;
            let (first_result_subst, first_result_type) = infer(env.clone(), fresh, first_result)?;
            let first_unified =
                unify(&first_result_type, &result_placeholder).ok_or_else(|| {
                    Error::TypeUnificationError {
                        left_span: expr.span(),
                        left_type: result_placeholder.clone(),
                        right_span: first_result.span(),
                        right_type: first_result_type.clone(),
                    }
                })?;
            let mut subst = first_result_subst.then(&first_unified);
            for expr::PatternMatch { pattern: _, result } in pattern_iter {
                let (result_subst, result_type) = infer(env.clone(), fresh, result)?;
                let unified = unify(&result_type, &result_placeholder).ok_or_else(|| {
                    Error::TypeUnificationError {
                        left_span: expr.span(),
                        left_type: result_placeholder.clone(),
                        right_span: result.span(),
                        right_type: result_type.clone(),
                    }
                })?;
                subst = subst.merge(&result_subst.then(&unified)).ok_or_else(|| {
                    Error::TypeUnificationError {
                        left_span: first_result.span(),
                        left_type: first_result_type.clone(),
                        right_span: result.span(),
                        right_type: result_type,
                    }
                })?;
            }
            let result = result_placeholder.substitute(&subst);
            Ok((subst, result))
        }
        Expression::Typed(expr::Typed { expression, typ }) => {
            let (expression_subst, expression_type) = infer(env.clone(), fresh, expression)?;
            let subst = unify(&expression_type, typ)
                .and_then(|typ_subst| expression_subst.merge(&typ_subst))
                .ok_or_else(|| Error::TypeUnificationError {
                    left_span: expression.span(),
                    left_type: expression_type.clone(),
                    right_span: None,
                    right_type: typ.clone(),
                })?;
            let result_type = expression_type.substitute(&subst);
            Ok((subst, result_type))
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
            Err(Error::TypeUnificationError {
                left_span: Some((0..14).into()),
                left_type: Type::Function {
                    parameter: Type::Integer.into(),
                    body: Type::Integer.into(),
                }
                .into(),
                right_span: Some((5..14).into()),
                right_type: Type::Function {
                    parameter: Type::Variable(TypeVariable::new_from_str("_3")).into(),
                    body: Type::Integer.into(),
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
            Err(Error::TypeUnificationError {
                left_span: Some((8..9).into()),
                left_type: Type::Variable(TypeVariable::new_from_str("_0")).into(),
                right_span: Some((10..11).into()),
                right_type: Type::Variable(TypeVariable::new_from_str("_0")).into(),
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
            Err(Error::TypeUnificationError {
                left_span: Some((15..16).into()),
                left_type: Type::Integer.into(),
                right_span: Some((23..32).into()),
                right_type: Type::Function {
                    parameter: Type::Variable(TypeVariable::new_from_str("_1")).into(),
                    body: Type::Variable(TypeVariable::new_from_str("_1")).into(),
                }
                .into(),
            }),
        );
        Ok(())
    }

    #[test]
    fn test_type_annotations_are_respected() -> Result<()> {
        let program = "(fn x -> x + 1): Integer";
        let ast = parse(program)?.to_core()?;

        let result = type_of(&ast);

        assert_eq!(
            result,
            Err(Error::TypeUnificationError {
                left_span: Some((1..14).into()),
                left_type: Type::Function {
                    parameter: Type::Integer.into(),
                    body: Type::Integer.into()
                }
                .into(),
                right_span: None, // TODO: should be `Some((17..24).into())`
                right_type: Type::Integer.into(),
            }),
        );
        Ok(())
    }
}
