use std::collections::HashSet;

use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Identifier<'a> {
    name: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentifierError {
    InvalidIdentifier,
}

lazy_static! {
    static ref VALID_IDENTIFIER_REGEX: Regex =
        Regex::new(r"^[_\p{Letter}][_\p{Number}\p{Letter}]*$").unwrap();
    // ensure that the set of keywords matches the keywords defined in lexer.rs
    static ref KEYWORDS: HashSet<&'static str> = ["in", "let"].into();
}

impl<'a> Identifier<'a> {
    pub fn new(name: &'a str) -> Result<Self, IdentifierError> {
        if Self::is_valid(name) {
            Ok(Identifier { name })
        } else {
            Err(IdentifierError::InvalidIdentifier)
        }
    }

    fn is_valid(name: &str) -> bool {
        !KEYWORDS.contains(name) && VALID_IDENTIFIER_REGEX.is_match(name)
    }
}

impl<'a> std::fmt::Display for Identifier<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

impl<'a> arbitrary::Arbitrary<'a> for Identifier<'a> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        lazy_static! {
            static ref FALLBACK: Identifier<'static> = Identifier::new("fallback").unwrap();
        }
        let generated = u.arbitrary::<&'a str>()?;
        if generated.is_empty() {
            return Ok(*FALLBACK);
        }
        let indices = generated
            .char_indices()
            .map(|(i, _)| i)
            .collect::<Vec<usize>>();
        // drop characters until it works out
        for start in 0..indices.len() - 1 {
            for end in (start + 1..indices.len()).rev() {
                let attempt = &generated[indices[start]..indices[end]];
                if Self::is_valid(attempt) {
                    return Identifier::new(attempt).map_err(|_| arbitrary::Error::IncorrectFormat);
                }
            }
        }
        Ok(*FALLBACK)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alphabetic_names_are_allowed() {
        assert_eq!(Identifier::new("name"), Ok(Identifier { name: "name" }));
    }

    #[test]
    fn test_numbers_are_allowed() {
        assert_eq!(
            Identifier::new("name123"),
            Ok(Identifier { name: "name123" })
        );
    }

    #[test]
    fn test_underscores_are_allowed() {
        assert_eq!(Identifier::new("x_y_z"), Ok(Identifier { name: "x_y_z" }));
    }

    #[test]
    fn test_empty_identifiers_are_rejected() {
        assert_eq!(Identifier::new(""), Err(IdentifierError::InvalidIdentifier));
    }

    #[test]
    fn test_symbols_at_the_start_are_rejected() {
        assert_eq!(
            Identifier::new("!abc"),
            Err(IdentifierError::InvalidIdentifier)
        );
    }

    #[test]
    fn test_symbols_in_the_middle_are_rejected() {
        assert_eq!(
            Identifier::new("foo<bar"),
            Err(IdentifierError::InvalidIdentifier)
        );
    }

    #[test]
    fn test_hyphens_are_rejected() {
        assert_eq!(
            Identifier::new("x-y-z"),
            Err(IdentifierError::InvalidIdentifier)
        );
    }
}
