use crate::ast;
use crate::span::Spanned;

use super::pool::*;

ast::expr!((PoolRef<Spanned<_>>));

pub type ExprPool = Pool<Spanned<Expression>>;

pub type ExprRef = PoolRef<Spanned<Expression>>;
