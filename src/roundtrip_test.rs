#![cfg(test)]

use crate::ast::Expr;
use crate::parser::parse;

#[test]
fn test_rendering_and_parsing_an_expression() {
    arbtest::builder().run(|u| {
        let input = u.arbitrary::<Expr<()>>()?;
        let rendered = format!("{}", input);
        let parsed = match parse(&rendered) {
            Ok(parsed) => parsed,
            Err(err) => panic!("Could not parse: {}", err),
        };
        assert_eq!(parsed, input);
        Ok(())
    })
}
