#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Identifier<'a> {
    name: &'a str,
}

impl<'a> Identifier<'a> {
    pub fn new(name: &'a str) -> Self {
        Identifier { name }
    }
}

impl<'a> std::fmt::Display for Identifier<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}
