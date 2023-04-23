#[derive(Debug, Clone, PartialEq, thiserror::Error, miette::Diagnostic)]
pub enum Error {
    #[error("Unexpected token: {token}")]
    #[diagnostic(code(boo::lex::unexpected_token))]
    UnexpectedToken {
        #[label("unexpected token")]
        span: miette::SourceSpan,
        token: String,
    },

    #[error("Parse error: expected one of {expected_tokens:?}")]
    #[diagnostic(code(boo::parse::error))]
    ParseError {
        #[label("{}", expected_one_of(expected_tokens))]
        span: miette::SourceSpan,
        expected_tokens: Vec<&'static str>,
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
