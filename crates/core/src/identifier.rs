//! Identifiers, used for variable and parameter names.

use std::collections::HashSet;
use std::sync::Arc;

use lazy_static::lazy_static;
use proptest::strategy::Strategy;
use regex::Regex;

/// An identifier is a valid name for a variable.
///
/// Valid identifiers start with a letter or underscore, and can then be
/// followed by 0 or more letters, numbers, or underscores. At least one
/// non-underscore character is required.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Identifier {
    Name(Arc<String>),
    Operator(Arc<String>),
    AvoidingCapture {
        original: Box<Identifier>,
        suffix: u32,
    },
}

/// Errors that can happen when dealing with identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
pub enum IdentifierError {
    /// Returned when attempting to construct a new [`Identifier`] with an
    /// invalid name.
    #[error("invalid identifier")]
    InvalidIdentifier,
}

lazy_static! {
    static ref VALID_IDENTIFIER_NAME_REGEX: Regex =
        Regex::new(&(
            r"^".to_string()
            + &VALID_IDENTIFIER_NAME_INITIAL_CHARACTER_REGEX
            + &VALID_IDENTIFIER_NAME_CHARACTER_REGEX + "*"
            + "$"
    )).unwrap();
    static ref VALID_IDENTIFIER_NAME_INITIAL_CHARACTER_REGEX: &'static str =
        r"[_\p{Letter}]";
    static ref VALID_IDENTIFIER_NAME_CHARACTER_REGEX: &'static str =
        r"[_\p{Letter}\p{Number}]";

    static ref VALID_OPERATORS: HashSet<&'static str> = ["+", "-", "*"].into();

    // ensure that the set of keywords matches the keywords defined in lexer.rs
    static ref KEYWORDS: HashSet<&'static str> = ["fn", "in", "let", "match"].into();
}

impl Identifier {
    /// Constructs a new identifier from a valid name. If the name is invalid,
    /// returns [`IdentifierError::InvalidIdentifier`].
    pub fn name_from_string(name: String) -> Result<Self, IdentifierError> {
        if Self::is_valid_name(&name) {
            Ok(Self::Name(Arc::new(name)))
        } else {
            Err(IdentifierError::InvalidIdentifier)
        }
    }

    /// Constructs a new identifier from a valid name. If the name is invalid,
    /// returns [`IdentifierError::InvalidIdentifier`].
    pub fn name_from_str(name: &str) -> Result<Self, IdentifierError> {
        if Self::is_valid_name(name) {
            Ok(Self::Name(Arc::new(name.to_string())))
        } else {
            Err(IdentifierError::InvalidIdentifier)
        }
    }

    /// Constructs a new identifier from a valid name. If the name is invalid,
    /// returns [`IdentifierError::InvalidIdentifier`].
    pub fn operator_from_str(operator: &str) -> Result<Self, IdentifierError> {
        if Self::is_valid_operator(operator) {
            Ok(Self::Operator(Arc::new(operator.to_string())))
        } else {
            Err(IdentifierError::InvalidIdentifier)
        }
    }

    pub fn name(&self) -> String {
        match self {
            Identifier::Name(name) => name.to_string(),
            Identifier::Operator(operator) => format!("({operator})"),
            Identifier::AvoidingCapture {
                original,
                suffix: _,
            } => original.name(),
        }
    }

    fn is_valid_name(name: &str) -> bool {
        !KEYWORDS.contains(name)
            && !name.chars().all(|c| c == '_')
            && VALID_IDENTIFIER_NAME_REGEX.is_match(name)
    }

    fn is_valid_operator(operator: &str) -> bool {
        VALID_OPERATORS.contains(operator)
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Identifier::Name(name) => name.fmt(f),
            Identifier::Operator(operator) => operator.fmt(f),
            Identifier::AvoidingCapture { original, .. } => original.fmt(f),
        }
    }
}

impl Identifier {
    /// A proptest strategy for constructing an arbitrary identifier.
    pub fn arbitrary() -> impl Strategy<Value = Identifier> {
        Self::arbitrary_of_length(1..=16)
    }

    /// A proptest strategy for constructing an arbitrary identifier within
    /// specific length bounds.
    pub fn arbitrary_of_length(
        length: std::ops::RangeInclusive<usize>,
    ) -> impl Strategy<Value = Identifier> {
        assert!(
            *length.start() > 0,
            "Cannot generate an arbitrary identifier of length 0."
        );
        proptest::string::string_regex(&format!(
            "{}{}{{{},{}}}",
            *VALID_IDENTIFIER_NAME_INITIAL_CHARACTER_REGEX,
            *VALID_IDENTIFIER_NAME_CHARACTER_REGEX,
            length.start() - 1,
            length.end() - 1,
        ))
        .unwrap()
        .prop_filter("invalid name", |name| Identifier::is_valid_name(name))
        .prop_map(|x| Identifier::name_from_string(x).unwrap())
    }

    /// A proptest strategy for constructing an arbitrary identifier within
    /// specific length bounds, limited to lowercase ASCII characters.
    pub fn gen_ascii(length: std::ops::RangeInclusive<usize>) -> impl Strategy<Value = Identifier> {
        assert!(
            *length.start() > 0,
            "Cannot generate an arbitrary identifier of length 0."
        );
        proptest::string::string_regex(&format!(
            "[a-z_][a-z0-9_]{{{},{}}}",
            length.start() - 1,
            length.end() - 1,
        ))
        .unwrap()
        .prop_filter("invalid name", |name| Identifier::is_valid_name(name))
        .prop_map(|x| Identifier::name_from_string(x).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alphabetic_names_are_allowed() {
        assert_eq!(
            Identifier::name_from_str("name"),
            Ok(Identifier::Name(Arc::new("name".to_string())))
        );
    }

    #[test]
    fn test_numbers_are_allowed() {
        assert_eq!(
            Identifier::name_from_str("name123"),
            Ok(Identifier::Name(Arc::new("name123".to_string())))
        );
    }

    #[test]
    fn test_underscores_are_allowed() {
        assert_eq!(
            Identifier::name_from_str("x_y_z"),
            Ok(Identifier::Name(Arc::new("x_y_z".to_string())))
        );
    }

    #[test]
    fn test_names_cannot_consist_entirely_of_underscores() {
        assert_eq!(
            Identifier::name_from_str("____"),
            Err(IdentifierError::InvalidIdentifier)
        );
    }

    #[test]
    fn test_empty_identifiers_are_rejected() {
        assert_eq!(
            Identifier::name_from_str(""),
            Err(IdentifierError::InvalidIdentifier)
        );
    }

    #[test]
    fn test_symbols_at_the_start_are_rejected() {
        assert_eq!(
            Identifier::name_from_str("!abc"),
            Err(IdentifierError::InvalidIdentifier)
        );
    }

    #[test]
    fn test_symbols_in_the_middle_are_rejected() {
        assert_eq!(
            Identifier::name_from_str("foo<bar"),
            Err(IdentifierError::InvalidIdentifier)
        );
    }

    #[test]
    fn test_hyphens_are_rejected() {
        assert_eq!(
            Identifier::name_from_str("x-y-z"),
            Err(IdentifierError::InvalidIdentifier)
        );
    }
}
