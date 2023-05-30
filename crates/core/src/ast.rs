#[macro_export]
macro_rules! expr {
    ($wrapper:tt) => {
        use $crate::identifier::Identifier;
        use $crate::operation::Operation;
        use $crate::primitive::Primitive;

        pub type Expr = boo_fill_hole::fill_hole!($wrapper, Expression);

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum Expression {
            Primitive(Primitive),
            Identifier(Identifier),
            Assign(Assign),
            Function(Function),
            Apply(Apply),
            Infix(Infix),
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct Assign {
            pub name: Identifier,
            pub value: Expr,
            pub inner: Expr,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct Function {
            pub parameter: Identifier,
            pub body: Expr,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct Apply {
            pub function: Expr,
            pub argument: Expr,
        }

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct Infix {
            pub operation: Operation,
            pub left: Expr,
            pub right: Expr,
        }

        impl std::fmt::Display for Expression {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    Expression::Primitive(x) => x.fmt(f),
                    Expression::Identifier(x) => x.fmt(f),
                    Expression::Assign(x) => x.fmt(f),
                    Expression::Function(x) => x.fmt(f),
                    Expression::Apply(x) => x.fmt(f),
                    Expression::Infix(x) => x.fmt(f),
                }
            }
        }

        impl std::fmt::Display for Assign {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "let {} = ({}) in ({})",
                    self.name, self.value, self.inner
                )
            }
        }

        impl std::fmt::Display for Function {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "fn {} -> ({})", self.parameter, self.body)
            }
        }

        impl std::fmt::Display for Apply {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "({}) ({})", self.function, self.argument)
            }
        }

        impl std::fmt::Display for Infix {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "({}) {} ({})", self.left, self.operation, self.right)
            }
        }
    };
}
