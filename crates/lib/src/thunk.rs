#[derive(Debug, PartialEq, Eq)]
pub enum Thunk<Unresolved, Resolved> {
    Unresolved(Unresolved),
    Resolved(Resolved),
}

impl<Unresolved, Resolved> Thunk<Unresolved, Resolved> {
    pub fn unresolved(value: Unresolved) -> Self {
        Self::Unresolved(value)
    }

    pub fn resolve(&mut self, value: Resolved) {
        match self {
            Self::Unresolved(_) => {
                *self = Self::Resolved(value);
            }
            Self::Resolved(_) => {}
        }
    }

    pub fn resolve_by(&mut self, compute: impl FnOnce(&mut Unresolved) -> Resolved) -> &Resolved {
        match self {
            Self::Unresolved(input) => {
                let value = compute(input);
                *self = Self::Resolved(value);
                match self {
                    Self::Resolved(value) => value,
                    Self::Unresolved(_) => unreachable!("Thunk::resolve_by"),
                }
            }
            Self::Resolved(value) => value,
        }
    }

    pub fn value(&self) -> Option<&Resolved> {
        match self {
            Self::Unresolved(_) => None,
            Self::Resolved(value) => Some(value),
        }
    }
}

impl<Unresolved, Resolved> From<Unresolved> for Thunk<Unresolved, Resolved> {
    fn from(value: Unresolved) -> Self {
        Self::Unresolved(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unresolved_thunks() {
        let thunk = Thunk::<i32, i32>::unresolved(0);
        assert_eq!(thunk.value(), None);
    }

    #[test]
    fn test_resolve_a_thunk_with_a_value() {
        let mut thunk = Thunk::<(), i32>::unresolved(());
        thunk.resolve(7);
        assert_eq!(thunk.value(), Some(&7));
        assert_eq!(thunk, Thunk::Resolved(7));
    }

    #[test]
    fn test_never_resolve_a_thunk_with_a_value_twice() {
        let mut thunk = Thunk::<(), i32>::unresolved(());
        thunk.resolve(1);
        thunk.resolve(2);
        assert_eq!(thunk.value(), Some(&1));
        assert_eq!(thunk, Thunk::Resolved(1));
    }

    #[test]
    fn test_resolve_a_thunk_with_a_compuation() {
        let mut thunk = Thunk::<Box<dyn Fn() -> i32>, i32>::unresolved(Box::new(|| 1 + 1));
        assert_eq!(thunk.resolve_by(|f| f()), &2);
        assert_eq!(thunk.value(), Some(&2));
    }

    #[test]
    fn test_never_resolve_a_thunk_with_a_compuation_twice() {
        let mut thunk = Thunk::<Box<dyn Fn() -> i32>, i32>::unresolved(Box::new(|| 2 + 3));
        thunk.resolve_by(|f| f());
        thunk.resolve_by(|f| f() + 4);
        assert_eq!(thunk.value(), Some(&5));
    }
}
