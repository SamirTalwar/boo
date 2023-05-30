use boo_core::expr;
use boo_core::span::Spanned;

use super::pool::*;

expr!((PoolRef<Spanned<_>>));

pub type ExprPool = Pool<Spanned<Expression>>;

pub type ExprRef = PoolRef<Spanned<Expression>>;
