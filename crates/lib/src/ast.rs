pub mod builders;

use std::rc::Rc;

use im::HashSet;
use proptest::{strategy::BoxedStrategy, strategy::Strategy};

use crate::identifier::Identifier;
use crate::operation::Operation;
use crate::primitive::Primitive;
use crate::span::Span;

use boo_fill_hole::fill_hole;

macro_rules! expr {
    {
      wrapper = $wrapper:tt ,
      parameters = $($parameters:ident) , * ,
    } => {
        expr! {
            wrapper = $wrapper;
            outer_type = Expr<$($parameters) , *>;
            outer_type_id = Expr;
            outer_type_parameters = $($parameters) , *;
            inner_type = Expression<$($parameters) , *>;
            inner_type_id = Expression;
            inner_type_parameters = $($parameters) , *;
        }
    };

    {
      wrapper = $wrapper:ty ;
      outer_type = $outer_type:ty ;
      outer_type_id = $outer_type_id:ident ;
      outer_type_parameters = $($outer_type_parameters:ident) , * ;
      inner_type = $inner_type:ty ;
      inner_type_id = $inner_type_id:ident ;
      inner_type_parameters = $($inner_type_parameters:ident) , * ;
    } => {
        pub type $outer_type_id < $($outer_type_parameters) , * > = fill_hole!($wrapper, $inner_type);

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct Annotated<Annotation, Value> {
            pub annotation: Annotation,
            pub value: Value,
        }

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum $inner_type_id < $($inner_type_parameters) , * > {
            Primitive {
                value: Primitive,
            },
            Identifier {
                name: Identifier,
            },
            Let {
                name: Identifier,
                value: $outer_type,
                inner: $outer_type,
            },
            Infix {
                operation: Operation,
                left: $outer_type,
                right: $outer_type,
            },
        }

        #[derive(Debug)]
        pub struct ExprGenConfig {
            pub depth: std::ops::Range<usize>,
            pub gen_identifier: Rc<BoxedStrategy<Identifier>>,
        }

        impl Default for ExprGenConfig {
            fn default() -> Self {
                Self {
                    depth: 0..8,
                    gen_identifier: Rc::new(Identifier::arbitrary().boxed()),
                }
            }
        }

        pub mod generators {
            // note that the spans generated are nonsense

            use super::*;

            pub fn arbitrary() -> impl Strategy<Value = Expr<Span>> {
                gen(Rc::new(Default::default()))
            }

            pub fn gen(config: Rc<ExprGenConfig>) -> impl Strategy<Value = Expr<Span>> {
                let start_depth = config.depth.clone();
                gen_nested(config, start_depth, HashSet::new())
            }

            fn gen_nested(
                config: Rc<ExprGenConfig>,
                depth: std::ops::Range<usize>,
                bound_identifiers: HashSet<Identifier>,
            ) -> impl Strategy<Value = Expr<Span>> {
                let mut choices: Vec<BoxedStrategy<Expr<Span>>> = Vec::new();

                if depth.start == 0 {
                    choices.push(
                        Primitive::arbitrary()
                            .prop_map(|value| {
                                Annotated {
                                    annotation: 0.into(),
                                    value: Expression::Primitive { value },
                                }
                                .into()
                            })
                            .boxed(),
                    );

                    if !bound_identifiers.is_empty() {
                        let bound = bound_identifiers.clone();
                        choices.push(
                            proptest::arbitrary::any::<proptest::sample::Index>()
                                .prop_map(move |index| {
                                    Annotated {
                                        annotation: 0.into(),
                                        value: Expression::Identifier {
                                            name: bound
                                                .iter()
                                                .nth(index.index(bound.len()))
                                                .unwrap()
                                                .clone(),
                                        },
                                    }
                                    .into()
                                })
                                .boxed(),
                        );
                    }
                }

                if depth.end > 0 {
                    let next_start = if depth.start == 0 { 0 } else { depth.start - 1 };
                    let next_end = depth.end - 1;

                    choices.push({
                        let conf = config.clone();
                        let bound = bound_identifiers.clone();
                        proptest::arbitrary::any::<Operation>()
                            .prop_flat_map(move |operation| {
                                (
                                    gen_nested(conf.clone(), next_start..next_end, bound.clone()),
                                    gen_nested(conf.clone(), next_start..next_end, bound.clone()),
                                )
                                    .prop_map(move |(left, right)| {
                                        {
                                            Annotated {
                                                annotation: 0.into(),
                                                value: Expression::Infix {
                                                    operation,
                                                    left,
                                                    right,
                                                },
                                            }
                                        }
                                        .into()
                                    })
                            })
                            .boxed()
                    });

                    choices.push({
                        let conf = config;
                        let bound = bound_identifiers;
                        conf.gen_identifier
                            .clone()
                            .prop_flat_map(move |name| {
                                let gen_value =
                                    gen_nested(conf.clone(), next_start..next_end, bound.clone());
                                let gen_inner = gen_nested(
                                    conf.clone(),
                                    next_start..next_end,
                                    bound.update(name.clone()),
                                );
                                (gen_value, gen_inner).prop_map(move |(value, inner)| {
                                    Annotated {
                                        annotation: 0.into(),
                                        value: Expression::Let {
                                            name: name.clone(),
                                            value,
                                            inner,
                                        },
                                    }
                                    .into()
                                })
                            })
                            .boxed()
                    });
                }

                proptest::strategy::Union::new(choices)
            }
        }
    };
}

expr! {
    wrapper = (Rc<Annotated<Annotation, _>>),
    parameters = Annotation,
}
