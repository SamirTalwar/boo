//! Parses tokens into an AST.

use boo_core::error::*;
use boo_core::primitive::*;
use boo_core::span::*;
use boo_language::*;

use crate::lexer::*;

peg::parser! {
    grammar parser<'a>() for [&'a AnnotatedToken<'a, Span>] {
        pub rule root() -> Expr = e:expr() { e }

        pub rule expr() -> Expr = precedence! {
            x:function() { x }
            --
            x:assign() { x }
            --
            x:match_() { x }
            --
            left:(@) (quiet! { [AnnotatedToken { annotation: _, token: Token::Operator("+") }] } / expected!("'+'")) right:@ {
                construct_infix(left, Operation::Add, right)
            }
            left:(@) (quiet! { [AnnotatedToken { annotation: _, token: Token::Operator("-") }] } / expected!("'-'")) right:@ {
                construct_infix(left, Operation::Subtract, right)
            }
            --
            left:(@) (quiet! { [AnnotatedToken { annotation: _, token: Token::Operator("*") }] } / expected!("'*'")) right:@ {
                construct_infix(left, Operation::Multiply, right)
            }
            --
            function:(@) argument:atomic_expr() {
                Expr::new(
                    function.span | argument.span,
                    Expression::Apply(Apply {
                        function,
                        argument,
                    }),
                )
            }
            --
            x:atomic_expr() { x }
        }

        rule atomic_expr() -> Expr =
            e:(primitive_expr() / identifier() / group()) { e }

        rule group() -> Expr =
            (quiet! { [AnnotatedToken { annotation: _, token: Token::StartGroup }] } / expected!("'('"))
            e:expr()
            (quiet! { [AnnotatedToken { annotation: _, token: Token::EndGroup }] } / expected!(")'")) {
                e
            }

        rule primitive_expr() -> Expr =
            primitive:primitive() {
                Expr::new(primitive.0, Expression::Primitive(primitive.1))
            }

        rule primitive() -> (Span, Primitive) =
            quiet! { [AnnotatedToken { annotation, token: Token::Integer(n) }] {
                (*annotation, Primitive::Integer(n.clone()))
            } } / expected!("an integer")

        rule identifier() -> Expr =
            quiet! { [AnnotatedToken { annotation, token: Token::Identifier(name) }] {
                Expr::new(
                    *annotation,
                    Expression::Identifier(name.clone()),
                )
            } } / expected!("an identifier")

        rule function() -> Expr =
            fn_:(quiet! { [AnnotatedToken { annotation: _, token: Token::Fn }] } / expected!("fn"))
            parameters_:(quiet! { [AnnotatedToken { annotation: _, token: Token::Identifier(name) }] } / expected!("an identifier"))+
            (quiet! { [AnnotatedToken { annotation: _, token: Token::Arrow }] } / expected!("->"))
            body:expr() {
                let span = fn_.annotation | body.span;
                let parameters = parameters_.into_iter().map(|parameter|
                    match &parameter.token {
                        Token::Identifier(identifier) => identifier.clone(),
                        _ => unreachable!(),
                    }
                ).collect();
                Expr::new(fn_.annotation | body.span, Expression::Function(Function {
                    parameters,
                    body,
                }))
            }

        rule assign() -> Expr =
            let_:(quiet! { [AnnotatedToken { annotation: _, token: Token::Let }] } / expected!("let"))
            name:(quiet! { [AnnotatedToken { annotation: _, token: Token::Identifier(name) }] } / expected!("an identifier"))
            (quiet! { [AnnotatedToken { annotation: _, token: Token::Assign }] } / expected!("="))
            value:expr()
            (quiet! { [AnnotatedToken { annotation: _, token: Token::In }] } / expected!("in"))
            inner:expr() {
                let n = match &name.token {
                    Token::Identifier(name) => name,
                    _ => unreachable!(),
                };
                Expr::new(
                    let_.annotation | inner.span,
                    Expression::Assign(Assign {
                        name: n.clone(),
                        value,
                        inner,
                    }),
                )
            }

        rule match_() -> Expr =
            match_:(quiet! { [AnnotatedToken { annotation: _, token: Token::Match }] } / expected!("match"))
            value:expr()
            block_start:(quiet! { [AnnotatedToken { annotation: _, token: Token::BlockStart }] } / expected!("{"))
            patterns:(pattern_match() ++ quiet! { [AnnotatedToken { annotation: _, token: Token::Separator }] } / expected!(";"))
            block_end:(quiet! { [AnnotatedToken { annotation: _, token: Token::BlockEnd }] } / expected!("}")) {
                Expr::new(
                    match_.annotation | block_end.annotation,
                    Expression::Match(Match {
                        value,
                        patterns,
                    }),
                )
            }

        rule pattern_match() -> PatternMatch =
            pattern:(pattern_primitive() / pattern_anything())
            (quiet! { [AnnotatedToken { annotation: _, token: Token::Arrow }] } / expected!("->"))
            result:expr() {
                PatternMatch {
                    pattern,
                    result,
                }
            }

        rule pattern_primitive() -> Pattern =
            primitive:primitive() {
                Pattern::Primitive(primitive.1)
            }

        rule pattern_anything() -> Pattern =
            (quiet! { [AnnotatedToken { annotation: _, token: Token::Anything }] } / expected!("_")) {
                Pattern::Anything
            }
    }
}

/// Parses a slice of [`Token`] values, annotated with a [`Span`], into an
/// expression.
///
/// Returns an error if an unexpected token is found.
pub fn parse_tokens(input: &[AnnotatedToken<Span>]) -> Result<Expr> {
    parser::root(&(input.iter().collect::<Vec<_>>())).map_err(|inner| {
        let span: Span = if inner.location < input.len() {
            input[inner.location].annotation
        } else {
            input
                .last()
                .map(|s| s.annotation.end.into())
                .unwrap_or(0.into())
        };
        let mut expected_tokens: Vec<&str> = inner.expected.tokens().collect();
        expected_tokens.sort();
        Error::ParseError {
            span,
            expected_tokens,
        }
    })
}

fn construct_infix(left: Expr, operation: Operation, right: Expr) -> Expr {
    Expr::new(
        left.span | right.span,
        Expression::Infix(Infix {
            operation,
            left,
            right,
        }),
    )
}
