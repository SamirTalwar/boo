use std::collections::HashSet;
use std::str::FromStr;

use lazy_static::lazy_static;
use proptest::strategy::Strategy;
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentifierError {
    InvalidIdentifier,
}

lazy_static! {
    static ref VALID_IDENTIFIER_REGEX: Regex =
        Regex::new(&(
            r"^".to_string()
            + &VALID_IDENTIFIER_INITIAL_CHARACTER_REGEX
            + &VALID_IDENTIFIER_CHARACTER_REGEX + "*"
            + "$"
    )).unwrap();
    static ref VALID_IDENTIFIER_INITIAL_CHARACTER_REGEX: &'static str =
        r"[_\p{Letter}]";
    static ref VALID_IDENTIFIER_CHARACTER_REGEX: &'static str =
        r"[_\p{Letter}\p{Number}]";
    // ensure that the set of keywords matches the keywords defined in lexer.rs
    static ref KEYWORDS: HashSet<&'static str> = ["in", "let"].into();
}

impl Identifier {
    pub fn new(name: String) -> Result<Self, IdentifierError> {
        if Self::is_valid(&name) {
            Ok(Identifier { name })
        } else {
            Err(IdentifierError::InvalidIdentifier)
        }
    }

    fn is_valid(name: &str) -> bool {
        !KEYWORDS.contains(name) && VALID_IDENTIFIER_REGEX.is_match(name)
    }
}

impl FromStr for Identifier {
    type Err = IdentifierError;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Self::new(name.to_string())
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

impl<'a> arbitrary::Arbitrary<'a> for Identifier {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let (min_length, max_length) = Self::size_hint(0);
        let length = u.int_in_range(min_length..=max_length.unwrap())?;
        let mut name = "".to_string();
        while !Identifier::is_valid(&name) {
            let first = loop {
                if u.is_empty() {
                    break Err(arbitrary::Error::NotEnoughData);
                }
                let c = u.arbitrary::<char>()?;
                if c == '_' || c.is_alphabetic() {
                    break Ok(c);
                }
            }?;
            let rest = (1..length)
                .map(|_| loop {
                    if u.is_empty() {
                        return Err(arbitrary::Error::NotEnoughData);
                    }
                    let c = u.arbitrary::<char>()?;
                    if c == '_' || c.is_alphabetic() || c.is_numeric() {
                        return Ok(c);
                    }
                })
                .collect::<Result<String, arbitrary::Error>>()?;
            name = first.to_string() + &rest;
        }
        Identifier::new(name).map_err(|_| arbitrary::Error::IncorrectFormat)
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        (1, Some(16))
    }
}

impl Identifier {
    pub fn arbitrary_of_max_length(max_length: usize) -> impl Strategy<Value = Identifier> {
        proptest::string::string_regex(&format!(
            "{}{}{{0,{}}}",
            *VALID_IDENTIFIER_INITIAL_CHARACTER_REGEX,
            *VALID_IDENTIFIER_CHARACTER_REGEX,
            max_length - 1,
        ))
        .unwrap()
        .prop_map(|x| Identifier::new(x).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alphabetic_names_are_allowed() {
        assert_eq!(
            Identifier::from_str("name"),
            Ok(Identifier {
                name: "name".to_string()
            })
        );
    }

    #[test]
    fn test_numbers_are_allowed() {
        assert_eq!(
            Identifier::from_str("name123"),
            Ok(Identifier {
                name: "name123".to_string()
            })
        );
    }

    #[test]
    fn test_underscores_are_allowed() {
        assert_eq!(
            Identifier::from_str("x_y_z"),
            Ok(Identifier {
                name: "x_y_z".to_string()
            })
        );
    }

    #[test]
    fn test_empty_identifiers_are_rejected() {
        assert_eq!(
            Identifier::from_str(""),
            Err(IdentifierError::InvalidIdentifier)
        );
    }

    #[test]
    fn test_symbols_at_the_start_are_rejected() {
        assert_eq!(
            Identifier::from_str("!abc"),
            Err(IdentifierError::InvalidIdentifier)
        );
    }

    #[test]
    fn test_symbols_in_the_middle_are_rejected() {
        assert_eq!(
            Identifier::from_str("foo<bar"),
            Err(IdentifierError::InvalidIdentifier)
        );
    }

    #[test]
    fn test_hyphens_are_rejected() {
        assert_eq!(
            Identifier::from_str("x-y-z"),
            Err(IdentifierError::InvalidIdentifier)
        );
    }
}
