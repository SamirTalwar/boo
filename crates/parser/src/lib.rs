//! Transforms an input string into an evaluatable program.

pub mod lexer;
pub mod parser;

use boo_core::error::Result;
use boo_language::Expr;

pub fn parse(input: &str) -> Result<Expr> {
    let tokens = lexer::lex(input)?;
    parser::parse_tokens(&tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_nothing() {
        let input = "";
        let parsed = parse(input);

        insta::assert_debug_snapshot!(parsed, @r###"
        Err(
            ParseError {
                span: Span {
                    start: 0,
                    end: 0,
                },
                expected_tokens: [
                    "'('",
                    "an identifier",
                    "an integer",
                    "fn",
                    "let",
                    "match",
                ],
            },
        )
        "###);
    }

    #[test]
    fn test_parsing_an_integer() {
        let input = "123";
        let parsed = parse(input);

        insta::assert_debug_snapshot!(parsed, @r###"
        Ok(
            Expr {
                span: Span {
                    start: 0,
                    end: 3,
                },
                expression: Primitive(
                    Integer(
                        Small(
                            123,
                        ),
                    ),
                ),
            },
        )
        "###);
    }

    #[test]
    fn test_parsing_a_negative_integer() {
        let input = "-456";
        let parsed = parse(input);

        insta::assert_debug_snapshot!(parsed, @r###"
        Ok(
            Expr {
                span: Span {
                    start: 0,
                    end: 4,
                },
                expression: Primitive(
                    Integer(
                        Small(
                            -456,
                        ),
                    ),
                ),
            },
        )
        "###);
    }

    #[test]
    fn test_parsing_an_integer_with_underscores() {
        let input = "987_654_321";
        let parsed = parse(input);

        insta::assert_debug_snapshot!(parsed, @r###"
        Ok(
            Expr {
                span: Span {
                    start: 0,
                    end: 11,
                },
                expression: Primitive(
                    Integer(
                        Small(
                            987654321,
                        ),
                    ),
                ),
            },
        )
        "###);
    }

    #[test]
    fn test_parsing_operators() {
        let input = "1 + 2 - 3 * 4";
        let parsed = parse(input);

        insta::assert_debug_snapshot!(parsed, @r###"
        Ok(
            Expr {
                span: Span {
                    start: 0,
                    end: 13,
                },
                expression: Infix(
                    Infix {
                        operation: Subtract,
                        left: Expr {
                            span: Span {
                                start: 0,
                                end: 5,
                            },
                            expression: Infix(
                                Infix {
                                    operation: Add,
                                    left: Expr {
                                        span: Span {
                                            start: 0,
                                            end: 1,
                                        },
                                        expression: Primitive(
                                            Integer(
                                                Small(
                                                    1,
                                                ),
                                            ),
                                        ),
                                    },
                                    right: Expr {
                                        span: Span {
                                            start: 4,
                                            end: 5,
                                        },
                                        expression: Primitive(
                                            Integer(
                                                Small(
                                                    2,
                                                ),
                                            ),
                                        ),
                                    },
                                },
                            ),
                        },
                        right: Expr {
                            span: Span {
                                start: 8,
                                end: 13,
                            },
                            expression: Infix(
                                Infix {
                                    operation: Multiply,
                                    left: Expr {
                                        span: Span {
                                            start: 8,
                                            end: 9,
                                        },
                                        expression: Primitive(
                                            Integer(
                                                Small(
                                                    3,
                                                ),
                                            ),
                                        ),
                                    },
                                    right: Expr {
                                        span: Span {
                                            start: 12,
                                            end: 13,
                                        },
                                        expression: Primitive(
                                            Integer(
                                                Small(
                                                    4,
                                                ),
                                            ),
                                        ),
                                    },
                                },
                            ),
                        },
                    },
                ),
            },
        )
        "###);
    }

    #[test]
    fn test_parsing_parentheses() {
        let input = "1 * (2 + 3) - 4";
        let parsed = parse(input);

        insta::assert_debug_snapshot!(parsed, @r###"
        Ok(
            Expr {
                span: Span {
                    start: 0,
                    end: 15,
                },
                expression: Infix(
                    Infix {
                        operation: Subtract,
                        left: Expr {
                            span: Span {
                                start: 0,
                                end: 10,
                            },
                            expression: Infix(
                                Infix {
                                    operation: Multiply,
                                    left: Expr {
                                        span: Span {
                                            start: 0,
                                            end: 1,
                                        },
                                        expression: Primitive(
                                            Integer(
                                                Small(
                                                    1,
                                                ),
                                            ),
                                        ),
                                    },
                                    right: Expr {
                                        span: Span {
                                            start: 5,
                                            end: 10,
                                        },
                                        expression: Infix(
                                            Infix {
                                                operation: Add,
                                                left: Expr {
                                                    span: Span {
                                                        start: 5,
                                                        end: 6,
                                                    },
                                                    expression: Primitive(
                                                        Integer(
                                                            Small(
                                                                2,
                                                            ),
                                                        ),
                                                    ),
                                                },
                                                right: Expr {
                                                    span: Span {
                                                        start: 9,
                                                        end: 10,
                                                    },
                                                    expression: Primitive(
                                                        Integer(
                                                            Small(
                                                                3,
                                                            ),
                                                        ),
                                                    ),
                                                },
                                            },
                                        ),
                                    },
                                },
                            ),
                        },
                        right: Expr {
                            span: Span {
                                start: 14,
                                end: 15,
                            },
                            expression: Primitive(
                                Integer(
                                    Small(
                                        4,
                                    ),
                                ),
                            ),
                        },
                    },
                ),
            },
        )
        "###);
    }

    #[test]
    fn test_parsing_variable_assignment() {
        let input = "let thing = 9";
        let parsed = parse(input);

        insta::assert_debug_snapshot!(parsed, @r###"
        Err(
            ParseError {
                span: Span {
                    start: 13,
                    end: 13,
                },
                expected_tokens: [
                    "'('",
                    "'*'",
                    "'+'",
                    "'-'",
                    "an identifier",
                    "an integer",
                    "in",
                ],
            },
        )
        "###);
    }

    #[test]
    fn test_parsing_variable_use() {
        let input = "foo + bar";
        let parsed = parse(input);

        insta::assert_debug_snapshot!(parsed, @r###"
        Ok(
            Expr {
                span: Span {
                    start: 0,
                    end: 9,
                },
                expression: Infix(
                    Infix {
                        operation: Add,
                        left: Expr {
                            span: Span {
                                start: 0,
                                end: 3,
                            },
                            expression: Identifier(
                                Name(
                                    "foo",
                                ),
                            ),
                        },
                        right: Expr {
                            span: Span {
                                start: 6,
                                end: 9,
                            },
                            expression: Identifier(
                                Name(
                                    "bar",
                                ),
                            ),
                        },
                    },
                ),
            },
        )
        "###);
    }

    #[test]
    fn test_parsing_variable_assignment_and_use() {
        let input = "let price = 3 in let quantity = 5 in price * quantity";
        let parsed = parse(input);

        insta::assert_debug_snapshot!(parsed, @r###"
        Ok(
            Expr {
                span: Span {
                    start: 0,
                    end: 53,
                },
                expression: Assign(
                    Assign {
                        name: Name(
                            "price",
                        ),
                        value: Expr {
                            span: Span {
                                start: 12,
                                end: 13,
                            },
                            expression: Primitive(
                                Integer(
                                    Small(
                                        3,
                                    ),
                                ),
                            ),
                        },
                        inner: Expr {
                            span: Span {
                                start: 17,
                                end: 53,
                            },
                            expression: Assign(
                                Assign {
                                    name: Name(
                                        "quantity",
                                    ),
                                    value: Expr {
                                        span: Span {
                                            start: 32,
                                            end: 33,
                                        },
                                        expression: Primitive(
                                            Integer(
                                                Small(
                                                    5,
                                                ),
                                            ),
                                        ),
                                    },
                                    inner: Expr {
                                        span: Span {
                                            start: 37,
                                            end: 53,
                                        },
                                        expression: Infix(
                                            Infix {
                                                operation: Multiply,
                                                left: Expr {
                                                    span: Span {
                                                        start: 37,
                                                        end: 42,
                                                    },
                                                    expression: Identifier(
                                                        Name(
                                                            "price",
                                                        ),
                                                    ),
                                                },
                                                right: Expr {
                                                    span: Span {
                                                        start: 45,
                                                        end: 53,
                                                    },
                                                    expression: Identifier(
                                                        Name(
                                                            "quantity",
                                                        ),
                                                    ),
                                                },
                                            },
                                        ),
                                    },
                                },
                            ),
                        },
                    },
                ),
            },
        )
        "###);
    }

    #[test]
    fn test_parsing_a_function() {
        let input = "fn x -> x + 1";
        let parsed = parse(input);

        insta::assert_debug_snapshot!(parsed, @r###"
        Ok(
            Expr {
                span: Span {
                    start: 0,
                    end: 13,
                },
                expression: Function(
                    Function {
                        parameters: [
                            Name(
                                "x",
                            ),
                        ],
                        body: Expr {
                            span: Span {
                                start: 8,
                                end: 13,
                            },
                            expression: Infix(
                                Infix {
                                    operation: Add,
                                    left: Expr {
                                        span: Span {
                                            start: 8,
                                            end: 9,
                                        },
                                        expression: Identifier(
                                            Name(
                                                "x",
                                            ),
                                        ),
                                    },
                                    right: Expr {
                                        span: Span {
                                            start: 12,
                                            end: 13,
                                        },
                                        expression: Primitive(
                                            Integer(
                                                Small(
                                                    1,
                                                ),
                                            ),
                                        ),
                                    },
                                },
                            ),
                        },
                    },
                ),
            },
        )
        "###);
    }

    #[test]
    fn test_parsing_a_function_with_multiple_arguments() {
        let input = "fn x y -> x * y";
        let parsed = parse(input);

        insta::assert_debug_snapshot!(parsed, @r###"
        Ok(
            Expr {
                span: Span {
                    start: 0,
                    end: 15,
                },
                expression: Function(
                    Function {
                        parameters: [
                            Name(
                                "x",
                            ),
                            Name(
                                "y",
                            ),
                        ],
                        body: Expr {
                            span: Span {
                                start: 10,
                                end: 15,
                            },
                            expression: Infix(
                                Infix {
                                    operation: Multiply,
                                    left: Expr {
                                        span: Span {
                                            start: 10,
                                            end: 11,
                                        },
                                        expression: Identifier(
                                            Name(
                                                "x",
                                            ),
                                        ),
                                    },
                                    right: Expr {
                                        span: Span {
                                            start: 14,
                                            end: 15,
                                        },
                                        expression: Identifier(
                                            Name(
                                                "y",
                                            ),
                                        ),
                                    },
                                },
                            ),
                        },
                    },
                ),
            },
        )
        "###);
    }

    #[test]
    fn test_parsing_function_application() {
        let input = "func one two three";
        let parsed = parse(input);

        insta::assert_debug_snapshot!(parsed, @r###"
        Ok(
            Expr {
                span: Span {
                    start: 0,
                    end: 18,
                },
                expression: Apply(
                    Apply {
                        function: Expr {
                            span: Span {
                                start: 0,
                                end: 12,
                            },
                            expression: Apply(
                                Apply {
                                    function: Expr {
                                        span: Span {
                                            start: 0,
                                            end: 8,
                                        },
                                        expression: Apply(
                                            Apply {
                                                function: Expr {
                                                    span: Span {
                                                        start: 0,
                                                        end: 4,
                                                    },
                                                    expression: Identifier(
                                                        Name(
                                                            "func",
                                                        ),
                                                    ),
                                                },
                                                argument: Expr {
                                                    span: Span {
                                                        start: 5,
                                                        end: 8,
                                                    },
                                                    expression: Identifier(
                                                        Name(
                                                            "one",
                                                        ),
                                                    ),
                                                },
                                            },
                                        ),
                                    },
                                    argument: Expr {
                                        span: Span {
                                            start: 9,
                                            end: 12,
                                        },
                                        expression: Identifier(
                                            Name(
                                                "two",
                                            ),
                                        ),
                                    },
                                },
                            ),
                        },
                        argument: Expr {
                            span: Span {
                                start: 13,
                                end: 18,
                            },
                            expression: Identifier(
                                Name(
                                    "three",
                                ),
                            ),
                        },
                    },
                ),
            },
        )
        "###);
    }

    #[test]
    fn test_parsing_inline_function_application() {
        let input = "(fn argument -> argument + argument) input";
        let parsed = parse(input);

        insta::assert_debug_snapshot!(parsed, @r###"
        Ok(
            Expr {
                span: Span {
                    start: 1,
                    end: 42,
                },
                expression: Apply(
                    Apply {
                        function: Expr {
                            span: Span {
                                start: 1,
                                end: 35,
                            },
                            expression: Function(
                                Function {
                                    parameters: [
                                        Name(
                                            "argument",
                                        ),
                                    ],
                                    body: Expr {
                                        span: Span {
                                            start: 16,
                                            end: 35,
                                        },
                                        expression: Infix(
                                            Infix {
                                                operation: Add,
                                                left: Expr {
                                                    span: Span {
                                                        start: 16,
                                                        end: 24,
                                                    },
                                                    expression: Identifier(
                                                        Name(
                                                            "argument",
                                                        ),
                                                    ),
                                                },
                                                right: Expr {
                                                    span: Span {
                                                        start: 27,
                                                        end: 35,
                                                    },
                                                    expression: Identifier(
                                                        Name(
                                                            "argument",
                                                        ),
                                                    ),
                                                },
                                            },
                                        ),
                                    },
                                },
                            ),
                        },
                        argument: Expr {
                            span: Span {
                                start: 37,
                                end: 42,
                            },
                            expression: Identifier(
                                Name(
                                    "input",
                                ),
                            ),
                        },
                    },
                ),
            },
        )
        "###);
    }

    #[test]
    fn test_parsing_assigned_function_application() {
        let input =
            "let important_function = fn thing -> (thing + thing) in important_function input";
        let parsed = parse(input);

        insta::assert_debug_snapshot!(parsed, @r###"
        Ok(
            Expr {
                span: Span {
                    start: 0,
                    end: 80,
                },
                expression: Assign(
                    Assign {
                        name: Name(
                            "important_function",
                        ),
                        value: Expr {
                            span: Span {
                                start: 25,
                                end: 51,
                            },
                            expression: Function(
                                Function {
                                    parameters: [
                                        Name(
                                            "thing",
                                        ),
                                    ],
                                    body: Expr {
                                        span: Span {
                                            start: 38,
                                            end: 51,
                                        },
                                        expression: Infix(
                                            Infix {
                                                operation: Add,
                                                left: Expr {
                                                    span: Span {
                                                        start: 38,
                                                        end: 43,
                                                    },
                                                    expression: Identifier(
                                                        Name(
                                                            "thing",
                                                        ),
                                                    ),
                                                },
                                                right: Expr {
                                                    span: Span {
                                                        start: 46,
                                                        end: 51,
                                                    },
                                                    expression: Identifier(
                                                        Name(
                                                            "thing",
                                                        ),
                                                    ),
                                                },
                                            },
                                        ),
                                    },
                                },
                            ),
                        },
                        inner: Expr {
                            span: Span {
                                start: 56,
                                end: 80,
                            },
                            expression: Apply(
                                Apply {
                                    function: Expr {
                                        span: Span {
                                            start: 56,
                                            end: 74,
                                        },
                                        expression: Identifier(
                                            Name(
                                                "important_function",
                                            ),
                                        ),
                                    },
                                    argument: Expr {
                                        span: Span {
                                            start: 75,
                                            end: 80,
                                        },
                                        expression: Identifier(
                                            Name(
                                                "input",
                                            ),
                                        ),
                                    },
                                },
                            ),
                        },
                    },
                ),
            },
        )
        "###);
    }

    #[test]
    fn test_parsing_function_application_within_infix_operations() {
        let input = "f left + g right";
        let parsed = parse(input);

        insta::assert_debug_snapshot!(parsed, @r###"
        Ok(
            Expr {
                span: Span {
                    start: 0,
                    end: 16,
                },
                expression: Infix(
                    Infix {
                        operation: Add,
                        left: Expr {
                            span: Span {
                                start: 0,
                                end: 6,
                            },
                            expression: Apply(
                                Apply {
                                    function: Expr {
                                        span: Span {
                                            start: 0,
                                            end: 1,
                                        },
                                        expression: Identifier(
                                            Name(
                                                "f",
                                            ),
                                        ),
                                    },
                                    argument: Expr {
                                        span: Span {
                                            start: 2,
                                            end: 6,
                                        },
                                        expression: Identifier(
                                            Name(
                                                "left",
                                            ),
                                        ),
                                    },
                                },
                            ),
                        },
                        right: Expr {
                            span: Span {
                                start: 9,
                                end: 16,
                            },
                            expression: Apply(
                                Apply {
                                    function: Expr {
                                        span: Span {
                                            start: 9,
                                            end: 10,
                                        },
                                        expression: Identifier(
                                            Name(
                                                "g",
                                            ),
                                        ),
                                    },
                                    argument: Expr {
                                        span: Span {
                                            start: 11,
                                            end: 16,
                                        },
                                        expression: Identifier(
                                            Name(
                                                "right",
                                            ),
                                        ),
                                    },
                                },
                            ),
                        },
                    },
                ),
            },
        )
        "###);
    }

    #[test]
    fn test_parsing_a_match_expression() {
        let input = "match 2 { 1 -> 2; 2 -> 3; 3 -> 4; _ -> 0 }";
        let parsed = parse(input);

        insta::assert_debug_snapshot!(parsed, @r###"
        Ok(
            Expr {
                span: Span {
                    start: 0,
                    end: 42,
                },
                expression: Match(
                    Match {
                        value: Expr {
                            span: Span {
                                start: 6,
                                end: 7,
                            },
                            expression: Primitive(
                                Integer(
                                    Small(
                                        2,
                                    ),
                                ),
                            ),
                        },
                        patterns: [
                            PatternMatch {
                                pattern: Primitive(
                                    Integer(
                                        Small(
                                            1,
                                        ),
                                    ),
                                ),
                                result: Expr {
                                    span: Span {
                                        start: 15,
                                        end: 16,
                                    },
                                    expression: Primitive(
                                        Integer(
                                            Small(
                                                2,
                                            ),
                                        ),
                                    ),
                                },
                            },
                            PatternMatch {
                                pattern: Primitive(
                                    Integer(
                                        Small(
                                            2,
                                        ),
                                    ),
                                ),
                                result: Expr {
                                    span: Span {
                                        start: 23,
                                        end: 24,
                                    },
                                    expression: Primitive(
                                        Integer(
                                            Small(
                                                3,
                                            ),
                                        ),
                                    ),
                                },
                            },
                            PatternMatch {
                                pattern: Primitive(
                                    Integer(
                                        Small(
                                            3,
                                        ),
                                    ),
                                ),
                                result: Expr {
                                    span: Span {
                                        start: 31,
                                        end: 32,
                                    },
                                    expression: Primitive(
                                        Integer(
                                            Small(
                                                4,
                                            ),
                                        ),
                                    ),
                                },
                            },
                            PatternMatch {
                                pattern: Anything,
                                result: Expr {
                                    span: Span {
                                        start: 39,
                                        end: 40,
                                    },
                                    expression: Primitive(
                                        Integer(
                                            Small(
                                                0,
                                            ),
                                        ),
                                    ),
                                },
                            },
                        ],
                    },
                ),
            },
        )
        "###);
    }

    #[test]
    fn test_parsing_rejects_anything_else() {
        let input = "1 / 2";
        let parsed = parse(input);

        insta::assert_debug_snapshot!(parsed, @r###"
        Err(
            UnexpectedToken {
                span: Span {
                    start: 2,
                    end: 3,
                },
                token: "/",
            },
        )
        "###);
    }

    #[test]
    fn test_parsing_rejects_an_unfinished_expression() {
        let input = "3 +";
        let parsed = parse(input);

        insta::assert_debug_snapshot!(parsed, @r###"
        Err(
            ParseError {
                span: Span {
                    start: 3,
                    end: 3,
                },
                expected_tokens: [
                    "'('",
                    "an identifier",
                    "an integer",
                    "fn",
                    "let",
                    "match",
                ],
            },
        )
        "###);
    }
}
