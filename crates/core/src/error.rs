//! The set of possible interpretation errors.

use crate::span::Span;
use crate::types;

/// An alias for [`Result`][std::result::Result] with the error type fixed to
/// [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// The set of possible interpretation errors.
///
/// This can be used with [`thiserror`] and [`miette`].
#[derive(Debug, Clone, PartialEq, thiserror::Error, miette::Diagnostic)]
pub enum Error {
    #[error("Unexpected token: {token}")]
    #[diagnostic(code(boo::lexer::unexpected_token))]
    UnexpectedToken {
        #[label("unexpected token")]
        span: Span,
        token: String,
    },

    #[error("Parse error: expected one of {expected_tokens:?}")]
    #[diagnostic(code(boo::parser::error))]
    ParseError {
        #[label("{}", expected_one_of(expected_tokens))]
        span: Span,
        expected_tokens: Vec<&'static str>,
    },

    #[error("Match expression without a base case")]
    #[diagnostic(code(boo::verifier::match_without_base_case))]
    MatchWithoutBaseCase {
        #[label("match expression requires a base case")]
        span: Option<Span>,
    },

    #[error("Could not unify types")]
    #[diagnostic(code(boo::type_checker::type_error))]
    TypeUnificationError {
        #[label("{left_type}")]
        left_span: Option<Span>,
        left_type: types::Monotype,
        #[label("{right_type}")]
        right_span: Option<Span>,
        right_type: types::Monotype,
    },

    #[error("Could not apply the function")]
    #[diagnostic(code(boo::evaluator::invalid_function_application))]
    InvalidFunctionApplication {
        #[label("invalid function")]
        span: Option<Span>,
    },

    #[error("Invalid primitive")]
    #[diagnostic(code(boo::evaluator::type_error))]
    InvalidPrimitive {
        #[label("invalid primitive")]
        span: Option<Span>,
    },

    #[error("Unknown variable: {name:?}")]
    #[diagnostic(code(boo::evaluator::unknown_variable))]
    UnknownVariable {
        #[label("unknown variable")]
        span: Option<Span>,
        name: String,
    },
}

fn expected_one_of(strings: &[&str]) -> String {
    match strings {
        [] => "<nothing>".to_string(),
        [a] => format!("expected {}", a),
        [a, b] => format!("expected {} or {}", a, b),
        _ => format!(
            "expected one of {}, or {}",
            strings[0..strings.len() - 1].join(", "),
            strings[strings.len() - 1]
        ),
    }
}
