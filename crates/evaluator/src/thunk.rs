//! Provides infrastructure for thread-safe thunks.

use std::sync::{Arc, RwLock};

/// A thunk is a value that is left as unresolved until needed, and then
/// resolved a single time when required. Subsequent accesses will get the same
/// value that was resolved the first time.
///
/// Thunks are thread-safe.
#[derive(Debug, Clone)]
pub struct Thunk<Unresolved, Resolved>(Arc<RwLock<ThunkValue<Unresolved, Arc<Resolved>>>>);

/// A thunk can be either unresolved or resolved.
#[derive(Debug)]
pub enum ThunkValue<Unresolved, Resolved> {
    Unresolved(Unresolved),
    Resolved(Resolved),
}

impl<Unresolved, Resolved> From<Unresolved> for Thunk<Unresolved, Resolved> {
    fn from(value: Unresolved) -> Self {
        Self::unresolved(value)
    }
}

impl<Unresolved, Resolved> Thunk<Unresolved, Resolved> {
    /// Constructs a new unresolved thunk.
    pub fn unresolved(value: Unresolved) -> Self {
        Self(Arc::new(RwLock::new(ThunkValue::Unresolved(value))))
    }

    /// Resolves a thunk with a specific value.
    pub fn resolve(&mut self, value: Resolved) {
        match (*self.0).write() {
            Ok(mut inner) => match *inner {
                ThunkValue::Unresolved(_) => {
                    *inner = ThunkValue::Resolved(value.into());
                }
                ThunkValue::Resolved(_) => {}
            },
            Err(err) => panic!("Poisoned mutex in thunk: {}", err),
        }
    }

    /// Resolves a thunk by computing something over the unresolved value.
    pub fn resolve_by(
        &mut self,
        compute: impl FnOnce(&mut Unresolved) -> Resolved,
    ) -> Arc<Resolved> {
        {
            // return without locking if possible
            match self.value() {
                None => {}
                Some(value) => {
                    return value;
                }
            }

            match (*self.0).write() {
                Ok(mut inner) => match *inner {
                    ThunkValue::Unresolved(ref mut input) => {
                        let value = Arc::new(compute(input));
                        *inner = ThunkValue::Resolved(Arc::clone(&value));
                        value
                    }
                    ThunkValue::Resolved(ref value) => Arc::clone(value),
                },
                Err(err) => panic!("Poisoned mutex in thunk: {}", err),
            }
        }
    }

    /// Returns the resolve value if it has already been computed, or `None`
    /// otherwise.
    pub fn value(&self) -> Option<Arc<Resolved>> {
        match (*self.0).read() {
            Ok(inner) => match *inner {
                ThunkValue::Unresolved(_) => None,
                ThunkValue::Resolved(ref value) => Some(Arc::clone(value)),
            },
            Err(err) => panic!("Poisoned mutex in thunk: {}", err),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::thread;

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
        assert_eq!(thunk.value(), Some(7.into()));
    }

    #[test]
    fn test_never_resolve_a_thunk_with_a_value_twice() {
        let mut thunk = Thunk::<(), i32>::unresolved(());
        thunk.resolve(1);
        thunk.resolve(2);
        assert_eq!(thunk.value(), Some(1.into()));
    }

    #[test]
    fn test_resolve_a_thunk_with_a_computation() {
        let mut thunk = Thunk::<Box<dyn Fn() -> i32>, i32>::unresolved(Box::new(|| 1 + 1));
        assert_eq!(thunk.resolve_by(|f| f()), 2.into());
        assert_eq!(thunk.value(), Some(2.into()));
    }

    #[test]
    fn test_never_resolve_a_thunk_with_a_compuation_twice() {
        let mut thunk = Thunk::<Box<dyn Fn() -> i32>, i32>::unresolved(Box::new(|| 2 + 3));
        thunk.resolve_by(|f| f());
        thunk.resolve_by(|f| f() + 4);
        assert_eq!(thunk.value(), Some(5.into()));
    }

    #[test]
    fn test_thunks_can_be_shared_across_threads() {
        let thunk = Thunk::<_, i32>::unresolved((7, 6));
        let handles = (0..16).map(|_| {
            let mut t = thunk.clone();
            thread::spawn(move || {
                t.resolve_by(|(a, b)| *a * *b);
            })
        });
        handles.into_iter().try_for_each(|t| t.join()).unwrap();
        assert_eq!(thunk.value(), Some(42.into()));
    }

    #[test]
    fn test_thunks_are_only_resolved_once() {
        let counter = Arc::new(AtomicU32::new(0));
        let thunk = Thunk::<_, i32>::unresolved((6, 9));
        let handles = (0..16).map(|_| {
            let c = Arc::clone(&counter);
            let mut t = thunk.clone();
            thread::spawn(move || {
                t.resolve_by(|(a, b)| {
                    c.fetch_add(1, Ordering::Relaxed);
                    *a * *b
                });
            })
        });
        handles.into_iter().try_for_each(|t| t.join()).unwrap();
        assert_eq!(thunk.value(), Some(54.into()));
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }
}
