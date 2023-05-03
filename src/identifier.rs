#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Identifier<'a> {
    name: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentifierError {
    InvalidIdentifier,
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
        let mut chars = name.chars();
        match chars.next() {
            None => false,
            Some(initial_char) => {
                Self::is_valid_initial_char(initial_char) && chars.all(Self::is_valid_char)
            }
        }
    }

    fn is_valid_char(c: char) -> bool {
        c == '_' || c.is_alphanumeric()
    }

    fn is_valid_initial_char(c: char) -> bool {
        c == '_' || c.is_alphabetic()
    }
}

impl<'a> std::fmt::Display for Identifier<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<'a> arbitrary::Arbitrary<'a> for Identifier<'a> {
        fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
            let mut attempt = u.arbitrary::<&'a str>()?;
            // drop characters until it works out
            while !Self::is_valid(attempt) {
                let mut indices = attempt.char_indices();
                indices.next();
                match indices.next() {
                    None => {
                        // if we run out of characters, do something
                        return Identifier::new("fallback")
                            .map_err(|_| arbitrary::Error::IncorrectFormat);
                    }
                    Some(index) => {
                        attempt = &attempt[index.0..];
                    }
                };
            }
            Identifier::new(attempt).map_err(|_| arbitrary::Error::IncorrectFormat)
        }
    }

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
