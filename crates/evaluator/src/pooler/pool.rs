//! A "pool" is a vector of nodes, into which a tree has been flattened.
//!
//! This is used to flatten an expression tree, though the data structure is
//! more general.
//!
//! See [Flattening ASTs](https://www.cs.cornell.edu/~asampson/blog/flattening.html),
//! by Adrian Sampson, for a thorough explanation of the approach.

use std::marker::PhantomData;

/// The pool of values.
#[derive(Debug, PartialEq)]
pub struct Pool<T>(Vec<T>);

/// A reference into a pool, scoped to a specific type.
///
/// Internally, this is just a vector offset.
///
/// When used with the same pool that yielded the reference, the value is
/// guaranteed to exist.
///
/// Note that if a reference is used with the wrong pool, the behavior is
/// very much undefined.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct PoolRef<T> {
    index: usize,
    marker: PhantomData<T>,
}

impl<T> Clone for PoolRef<T> {
    fn clone(&self) -> Self {
        PoolRef { ..*self }
    }
}

impl<T> Copy for PoolRef<T> {}

impl<T> std::fmt::Display for PoolRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.index.fmt(f)
    }
}

impl<T> Pool<T> {
    /// Constructs a new pool.
    pub fn new() -> Self {
        Pool(Vec::new())
    }

    /// Gets a specific value from the pool by reference.
    pub fn get(&self, value_ref: PoolRef<T>) -> &T {
        self.0.get(value_ref.index).unwrap()
    }

    /// Adds a new node to the pool, and returns an opaque reference.
    pub fn add(&mut self, value: T) -> PoolRef<T> {
        let index = self.0.len();
        self.0.push(value);
        PoolRef {
            index,
            marker: PhantomData,
        }
    }
}

impl<T> Default for Pool<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Constructs a new pool in a functional style, leaking data.
pub fn pool_with<T, Leak>(f: impl FnOnce(&mut Pool<T>) -> Leak) -> (Pool<T>, Leak) {
    let mut new_pool = Pool::new();
    let leak = f(&mut new_pool);
    (new_pool, leak)
}
