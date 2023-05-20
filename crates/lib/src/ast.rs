macro_rules! expr {
    ($wrapper:tt) => {
        $crate::ast::expr! {
            wrapper = $wrapper;
            outer_type = Expr;
            outer_type_id = Expr;
            inner_type = Expression;
            inner_type_id = Expression;
        }
    };

    {
      wrapper = $wrapper:ty ;
      outer_type = $outer_type:ty ;
      outer_type_id = $outer_type_id:ident ;
      inner_type = $inner_type:ty ;
      inner_type_id = $inner_type_id:ident ;
    } => {
        pub type $outer_type_id = boo_fill_hole::fill_hole!($wrapper, $inner_type);

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum $inner_type_id {
            Primitive {
                value: $crate::primitive::Primitive,
            },
            Identifier {
                name: $crate::identifier::Identifier,
            },
            Let {
                name: $crate::identifier::Identifier,
                value: $outer_type,
                inner: $outer_type,
            },
            Infix {
                operation: $crate::operation::Operation,
                left: $outer_type,
                right: $outer_type,
            },
        }

        impl std::fmt::Display for $inner_type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Expression::Primitive { value } => value.fmt(f),
                    Expression::Identifier { name } => name.fmt(f),
                    Expression::Let { name, value, inner } => {
                        write!(f, "let {} = ({}) in ({})", name, value, inner)
                    }
                    Expression::Infix {
                        operation,
                        left,
                        right,
                    } => write!(f, "({}) {} ({})", left, operation, right),
                }
            }
        }
    };
}

pub(crate) use expr;
