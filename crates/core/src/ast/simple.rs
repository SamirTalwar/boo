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
