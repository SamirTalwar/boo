macro_rules! expr {
    ($wrapper:tt) => {
        use $crate::identifier::Identifier;
        use $crate::operation::Operation;
        use $crate::primitive::Primitive;

        pub type Expr = boo_fill_hole::fill_hole!($wrapper, Expression);

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum Expression {
            Primitive {
                value: Primitive,
            },
            Identifier {
                name: Identifier,
            },
            Assign {
                name: Identifier,
                value: Expr,
                inner: Expr,
            },
            Infix {
                operation: Operation,
                left: Expr,
                right: Expr,
            },
        }

        impl std::fmt::Display for Expression {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Expression::Primitive { value } => value.fmt(f),
                    Expression::Identifier { name } => name.fmt(f),
                    Expression::Assign { name, value, inner } => {
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
