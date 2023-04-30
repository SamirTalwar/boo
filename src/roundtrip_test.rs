#![cfg(test)]

use crate::ast::Expr;
use crate::lexer::lex;
use crate::parser::parse;

#[test]
fn test_rendering_and_parsing_an_expression() {
    arbtest::builder().run(|u| {
        let input = u.arbitrary::<Expr<()>>()?;
        let rendered = format!("{}", input);
        let lexed = lex(&rendered).expect("Could not lex");
        let parsed = parse(&lexed).expect("Could not parse");
        assert_eq!(parsed, input);
        Ok(())
    })
}
