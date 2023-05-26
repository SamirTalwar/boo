macro_rules! expr {
    ($wrapper:tt) => {
        use $crate::identifier::Identifier;
        use $crate::operation::Operation;
        use $crate::primitive::Primitive;

        pub type Expr = boo_fill_hole::fill_hole!($wrapper, Expression);

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum Expression {
            Primitive(Primitive),
            Identifier(Identifier),
            Assign(Assign),
            Function(Function),
            Apply(Apply),
            Infix(Infix),
        }

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct Assign {
            pub name: Identifier,
            pub value: Expr,
            pub inner: Expr,
        }

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct Function {
            pub parameter: Identifier,
            pub body: Expr,
        }

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct Apply {
            pub function: Expr,
            pub argument: Expr,
        }

        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct Infix {
            pub operation: Operation,
            pub left: Expr,
            pub right: Expr,
        }

        impl std::fmt::Display for Expression {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Expression::Primitive(value) => value.fmt(f),
                    Expression::Identifier(name) => name.fmt(f),
                    Expression::Assign(Assign { name, value, inner }) => {
                        write!(f, "let {} = ({}) in ({})", name, value, inner)
                    }
                    Expression::Function(Function { parameter, body }) => {
                        write!(f, "fn {} -> {}", parameter, body)
                    }
                    Expression::Apply(Apply { function, argument }) => {
                        write!(f, "({}) ({})", function, argument)
                    }
                    Expression::Infix(Infix {
                        operation,
                        left,
                        right,
                    }) => write!(f, "({}) {} ({})", left, operation, right),
                }
            }
        }
    };
}

pub(crate) use expr;
