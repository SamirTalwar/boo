use boo_core::expr;
use boo_core::span::Spanned;

use super::pool::*;

expr!((PoolRef<Spanned<_>>));

impl Copy for Expr {}

pub type ExprPool = Pool<Spanned<Expression>>;
