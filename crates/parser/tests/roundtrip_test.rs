use proptest::prelude::*;

use boo_core::ast::simple::Expr;
use boo_core::ast::*;
use boo_lexer::lex;
use boo_parser::parse;
use boo_test_helpers::proptest::*;

#[test]
fn test_rendering_and_parsing_an_expression() {
    check(&boo_generator::arbitrary::<Expr>(), |input| {
        let rendered = format!("{}", input);
        let lexed = lex(&rendered)?;
        let parsed = parse(&lexed)?.transform(&mut |_, expression| Expr::new(expression));
        prop_assert_eq!(input, parsed);
        Ok(())
    })
}
