use std::rc::Rc;

use crate::ast;
use crate::span::Spanned;

ast::expr!((Rc<Spanned<_>>));
