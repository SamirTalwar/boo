//! A "pool" is a vector of nodes, into which a tree has been flattened.
//!
//! This is used to flatten an expression tree, though the data structure is
//! more general.
//!
//! See [Flattening ASTs](https://www.cs.cornell.edu/~asampson/blog/flattening.html),
//! by Adrian Sampson, for a thorough explanation of the approach.

use std::marker::PhantomData;
use std::sync::Arc;

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
        *self
    }
}

impl<T> Copy for PoolRef<T> {}

impl<T> std::fmt::Display for PoolRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.index.fmt(f)
    }
}

/// The pool of values.
#[derive(Debug, PartialEq)]
pub struct PoolBuilder<T> {
    inherited: Vec<(usize, Arc<Vec<T>>)>,
    offset: usize,
    owned: Vec<T>,
}

impl<T> Default for PoolBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> PoolBuilder<T> {
    /// Constructs a new pool.
    pub fn new() -> Self {
        PoolBuilder {
            inherited: vec![],
            offset: 0,
            owned: vec![],
        }
    }

    /// Adds a new node to the pool, and returns an opaque reference.
    pub fn add(&mut self, value: T) -> PoolRef<T> {
        let index = self.offset + self.owned.len();
        self.owned.push(value);
        PoolRef {
            index,
            marker: PhantomData,
        }
    }

    /// Splits a pool into many.
    ///
    /// Values for existing references are shared, but new values are inserted
    /// independently.
    pub fn build(self) -> Pool<T> {
        let Self {
            mut inherited,
            offset,
            owned,
        } = self;
        let new_offset = offset + owned.len();
        let previous = Arc::new(owned);
        inherited.push((offset, previous));
        Pool {
            pools: inherited,
            offset: new_offset,
            marker: PhantomData,
        }
    }
}

pub struct Pool<T> {
    pools: Vec<(usize, Arc<Vec<T>>)>,
    offset: usize,
    marker: PhantomData<T>,
}

impl<T> Pool<T> {
    /// Splits out a new pool from this one.
    ///
    /// Values for existing references are shared, but new values are inserted
    /// independently.
    pub fn fork(&self) -> PoolBuilder<T> {
        PoolBuilder {
            inherited: self.pools.clone(),
            offset: self.offset,
            owned: vec![],
        }
    }

    /// Gets a specific value from the pool by reference.
    pub fn get(&self, value_ref: PoolRef<T>) -> &T {
        for (inherited_offset, inherited_values) in self.pools.iter().rev() {
            if value_ref.index >= *inherited_offset {
                return inherited_values
                    .get(value_ref.index - inherited_offset)
                    .unwrap();
            }
        }
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insertion_and_lookup() {
        let (pool, a, b, c) = {
            let mut builder = PoolBuilder::new();
            let a = builder.add(1);
            let b = builder.add(3);
            let c = builder.add(5);
            (builder.build(), a, b, c)
        };

        assert_eq!(pool.get(a), &1);
        assert_eq!(pool.get(b), &3);
        assert_eq!(pool.get(c), &5);
    }

    #[test]
    fn test_forked_pool_shares_references() {
        let (pool, one, two) = {
            let mut builder = PoolBuilder::new();
            let one = builder.add("one");
            let two = builder.add("two");
            (builder.build(), one, two)
        };

        let pool_a = pool.fork().build();
        let pool_b = pool.fork().build();

        assert_eq!(pool_a.get(one), &"one");
        assert_eq!(pool_a.get(two), &"two");

        assert_eq!(pool_b.get(one), &"one");
        assert_eq!(pool_b.get(two), &"two");
    }

    #[test]
    fn test_forked_pools_are_modified_independently() {
        let pool = {
            let mut builder = PoolBuilder::new();
            let _ = builder.add("one");
            let _ = builder.add("two");
            builder.build()
        };

        let mut builder_a = pool.fork();
        let mut builder_b = pool.fork();
        let a_three = builder_a.add("three");
        let b_four = builder_b.add("four");
        let pool_a = builder_a.build();
        let pool_b = builder_b.build();

        assert_eq!(pool_a.get(a_three), &"three");
        assert_eq!(pool_b.get(b_four), &"four");
    }

    #[test]
    fn test_pools_can_be_forked_multiple_times() {
        let mut builder_a = PoolBuilder::new();
        let one = builder_a.add(1);
        let pool_a = builder_a.build();

        let mut builder_b = pool_a.fork();
        let two = builder_b.add(2);
        let pool_b = builder_b.build();

        let mut builder_c = pool_b.fork();
        let three = builder_c.add(3);
        let pool_c = builder_c.build();

        let mut builder_d = pool_c.fork();
        let four = builder_d.add(4);
        let pool_d = builder_d.build();

        assert_eq!(pool_a.get(one), &1);
        assert_eq!(pool_b.get(one), &1);
        assert_eq!(pool_c.get(one), &1);
        assert_eq!(pool_d.get(one), &1);

        assert_eq!(pool_b.get(two), &2);
        assert_eq!(pool_c.get(two), &2);
        assert_eq!(pool_d.get(two), &2);

        assert_eq!(pool_c.get(three), &3);
        assert_eq!(pool_d.get(three), &3);

        assert_eq!(pool_d.get(four), &4);
    }
}
