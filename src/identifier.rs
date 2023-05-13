use std::collections::HashSet;
use std::str::FromStr;

use lazy_static::lazy_static;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    pub struct IdentifierStrategy(proptest::string::RegexGeneratorStrategy<String>);

    impl proptest::strategy::Strategy for IdentifierStrategy {
        type Tree = IdentifierValueTree;

        type Value = Identifier;

        fn new_tree(
            &self,
            runner: &mut proptest::test_runner::TestRunner,
        ) -> proptest::strategy::NewTree<Self> {
            self.0.new_tree(runner).map(IdentifierValueTree)
        }
    }

    pub struct IdentifierValueTree(proptest::string::RegexGeneratorValueTree<String>);

    impl proptest::strategy::ValueTree for IdentifierValueTree {
        type Value = Identifier;

        fn current(&self) -> Self::Value {
            Identifier::new(self.0.current()).unwrap()
        }

        fn simplify(&mut self) -> bool {
            self.0.simplify()
        }

        fn complicate(&mut self) -> bool {
            self.0.complicate()
        }
    }

    impl proptest::arbitrary::Arbitrary for Identifier {
        type Strategy = IdentifierStrategy;

        type Parameters = usize; // length

        fn arbitrary() -> Self::Strategy {
            Self::arbitrary_with(16)
        }

        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            IdentifierStrategy(
                proptest::string::string_regex(&format!(
                    "^{}{}{{0,{}}}$",
                    *VALID_IDENTIFIER_INITIAL_CHARACTER_REGEX,
                    *VALID_IDENTIFIER_CHARACTER_REGEX,
                    args - 1,
                ))
                .unwrap(),
            )
        }
    }

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
