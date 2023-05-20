pub mod builders;
pub mod generators;

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
    };
}

expr! {
    wrapper = (Rc<Annotated<Annotation, _>>),
    parameters = Annotation,
}
