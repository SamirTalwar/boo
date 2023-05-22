macro_rules! expr {
    ($wrapper:tt) => {
        pub type Expr = boo_fill_hole::fill_hole!($wrapper, Expression);

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum Expression {
            Primitive {
                value: $crate::primitive::Primitive,
            },
            Identifier {
                name: $crate::identifier::Identifier,
            },
            Let {
                name: $crate::identifier::Identifier,
                value: Expr,
                inner: Expr,
            },
            Infix {
                operation: $crate::operation::Operation,
                left: Expr,
                right: Expr,
            },
        }

        impl std::fmt::Display for Expression {
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
