use crate::span::Span;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, thiserror::Error, miette::Diagnostic)]
pub enum Error {
    #[error("Unexpected token: {token}")]
    #[diagnostic(code(boo::lex::unexpected_token))]
    UnexpectedToken {
        #[label("unexpected token")]
        span: Span,
        token: String,
    },

    #[error("Parse error: expected one of {expected_tokens:?}")]
    #[diagnostic(code(boo::parse::error))]
    ParseError {
        #[label("{}", expected_one_of(expected_tokens))]
        span: Span,
        expected_tokens: Vec<&'static str>,
    },

    #[error("Unknown variable: {name:?}")]
    #[diagnostic(code(boo::interpret::unknown_variable))]
    UnknownVariable {
        #[label("unknown variable")]
        span: Span,
        name: String,
    },

    #[error("Could not apply the function")]
    #[diagnostic(code(boo::interpret::unknown_variable))]
    InvalidFunctionApplication {
        #[label("invalid function")]
        span: Span,
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
