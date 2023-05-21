use std::marker::PhantomData;

#[derive(Debug, PartialEq)]
pub struct Pool<T>(Vec<T>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolRef<T> {
    index: usize,
    marker: PhantomData<T>,
}

impl<T> std::fmt::Display for PoolRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.index.fmt(f)
    }
}

impl<T> Pool<T> {
    pub fn new() -> Self {
        Pool(Vec::new())
    }

    pub fn root(&self) -> PoolRef<T> {
        if self.0.is_empty() {
            panic!("Tried to get the root of an empty pool.");
        }
        PoolRef {
            index: self.0.len() - 1,
            marker: PhantomData,
        }
    }

    pub fn get_root(&self) -> &T {
        self.get(self.root())
    }

    pub fn get(&self, value_ref: PoolRef<T>) -> &T {
        self.0.get(value_ref.index).unwrap()
    }

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

pub fn pool_with<T>(f: impl FnOnce(&mut Pool<T>)) -> Pool<T> {
    let mut value = Pool::new();
    f(&mut value);
    value
}
