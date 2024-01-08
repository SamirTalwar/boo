mod algorithm_w;
mod env;
mod fresh;
mod subst;
mod types;

use boo_core::error::Result;
use boo_core::expr::Expr;
use boo_core::types::Monotype;

pub fn type_of(expr: &Expr) -> Result<Monotype> {
    algorithm_w::type_of(expr)
}
