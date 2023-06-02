use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expr(Box<Expression<Expr>>);

impl Expr {
    pub fn new(expression: Expression<Expr>) -> Self {
        Self(Box::new(expression))
    }
}

impl ExpressionWrapper for Expr {
    type Annotation = ();

    fn map<Next>(self, f: &mut impl FnMut(Self::Annotation, Expression<Next>) -> Next) -> Next {
        let mapped = self.0.map(f);
        f((), mapped)
    }
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub mod builders {
    use crate::primitive::*;

    use super::*;

    pub fn primitive(value: Primitive) -> Expr {
        Expr::new(Expression::Primitive(value))
    }

    pub fn primitive_integer(value: Integer) -> Expr {
        primitive(Primitive::Integer(value))
    }

    pub fn identifier(name: Identifier) -> Expr {
        Expr::new(Expression::Identifier(name))
    }

    pub fn identifier_string(name: String) -> Expr {
        identifier(Identifier::new(name).unwrap())
    }

    pub fn assign(name: Identifier, value: Expr, inner: Expr) -> Expr {
        Expr::new(Expression::Assign(Assign { name, value, inner }))
    }

    pub fn assign_string(name: String, value: Expr, inner: Expr) -> Expr {
        assign(Identifier::new(name).unwrap(), value, inner)
    }

    pub fn function(parameter: Identifier, body: Expr) -> Expr {
        Expr::new(Expression::Function(Function { parameter, body }))
    }

    pub fn apply(function: Expr, argument: Expr) -> Expr {
        Expr::new(Expression::Apply(Apply { function, argument }))
    }

    pub fn infix(operation: Operation, left: Expr, right: Expr) -> Expr {
        Expr::new(Expression::Infix(Infix {
            operation,
            left,
            right,
        }))
    }
}
