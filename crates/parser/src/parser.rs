//! Parses tokens into an AST.

use boo_core::error::*;
use boo_core::identifier::*;
use boo_core::primitive::*;
use boo_core::span::*;
use boo_core::types::*;
use boo_language::*;

use crate::lexer::*;

peg::parser! {
    grammar parser<'a>() for [&'a AnnotatedToken<'a, Span>] {
        pub rule root() -> Expr = e:expr() { e }

        pub rule expr() -> Expr = precedence! {
            let_:(quiet! { [AnnotatedToken { annotation: _, token: Token::Let }] } / expected!("let"))
            name:(quiet! { [AnnotatedToken { annotation: _, token: Token::Identifier(name) }] { name } } / expected!("an identifier"))
            (quiet! { [AnnotatedToken { annotation: _, token: Token::Assign }] } / expected!("="))
            value:expr()
            (quiet! { [AnnotatedToken { annotation: _, token: Token::In }] } / expected!("in"))
            inner:@ {
                Expr::new(
                    let_.annotation | inner.span,
                    Expression::Assign(Assign {
                        name: name.clone(),
                        value,
                        inner,
                    }),
                )
            }
            --
            expression:@ (quiet! { [AnnotatedToken { annotation: _, token: Token::Annotate }] } / expected!("':'")) typ:typ() {
                Expr::new(expression.span, Expression::Typed(Typed {
                    expression,
                    typ,
                }))
            }
            --
            fn_:(quiet! { [AnnotatedToken { annotation: _, token: Token::Fn }] } / expected!("fn"))
            parameters:(quiet! { [AnnotatedToken { annotation: _, token: Token::Identifier(name) }] { name } } / expected!("an identifier"))+
            (quiet! { [AnnotatedToken { annotation: _, token: Token::Arrow }] } / expected!("->"))
            body:@ {
                let span = fn_.annotation | body.span;
                Expr::new(span, Expression::Function(Function {
                    parameters: parameters.into_iter().cloned().collect(),
                    body,
                }))
            }
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
            e:(primitive_expr() / identifier_expr() / group()) { e }

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

        rule identifier_expr() -> Expr =
            identifier:identifier() {
                Expr::new(identifier.0, Expression::Identifier(identifier.1))
            }

        rule identifier() -> (Span, Identifier) =
            quiet! { [AnnotatedToken { annotation, token: Token::Identifier(name) }] {
                (*annotation, name.clone())
            } } / expected!("an identifier")

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

        rule typ() -> Monotype = precedence! {
            typ:typ_name() { typ }
            --
            parameter:@
            (quiet! { [AnnotatedToken { annotation: _, token: Token::Arrow }] } / expected!("->"))
            body:(@) {
                Type::Function { parameter, body }.into()
            }
            --
            (quiet! { [AnnotatedToken { annotation: _, token: Token::StartGroup }] } / expected!("'('"))
            typ:typ()
            (quiet! { [AnnotatedToken { annotation: _, token: Token::EndGroup }] } / expected!(")'")) {
                typ
            }
        }

        rule typ_name() -> Monotype =
            i:identifier() { ?
                 match i.1 {
                    Identifier::Name(name) if name.as_ref() == "Integer" => Ok(Type::Integer.into()),
                    _ => Err("unknown type"),
                }
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
