//! Implements a fast arena allocator that uses fixed size buckets and returns IDs for allocated objects.

use page_size;
use std::{cmp::max, marker::PhantomData, mem::size_of, ops::Index};

#[cfg(test)]
#[path = "./fast_arena_test.rs"]
mod fast_arena_test;

/// An arena that can be used to allocate objects efficiently.
#[derive(Debug)]
pub struct FastIdArena<T> {
    buckets: Vec<Vec<T>>,
    items_per_bucket: usize,
    bucket_index: usize,
}

/// An ID that identifies an allocated object.
#[derive(Debug)]
pub struct FastId<T> {
    index: usize,
    _t: PhantomData<T>,
}

impl<T> FastIdArena<T> {
    /// Creates a new arena for the specified type.
    pub fn new() -> Self {
        let items_per_bucket = max(page_size::get(), page_size::get_granularity()) / size_of::<T>();
        FastIdArena::<T> {
            buckets: vec![Vec::<T>::with_capacity(items_per_bucket)],
            items_per_bucket,
            bucket_index: 0,
        }
    }

    /// Creates a new arena with each bucket able to hold the specified number of items.
    pub fn new_with_bucket_size(items_per_bucket: usize) -> Self {
        FastIdArena::<T> {
            buckets: vec![Vec::<T>::with_capacity(items_per_bucket)],
            items_per_bucket,
            bucket_index: 0,
        }
    }

    /// The maximum number of items per bucket.
    pub fn items_per_bucket(&self) -> usize {
        self.items_per_bucket
    }

    /// Allocates the specified item inside the arena.
    #[inline]
    pub fn alloc(&mut self, item: T) -> FastId<T> {
        let mut bucket = &mut self.buckets[self.bucket_index];
        if bucket.len() == self.items_per_bucket {
            self.buckets
                .push(Vec::<T>::with_capacity(self.items_per_bucket));
            self.bucket_index += 1;
            bucket = &mut self.buckets[self.bucket_index];
        }

        let index = bucket.len();

        bucket.push(item);

        FastId::<T> {
            index,
            _t: PhantomData,
        }
    }

    /// Returns a reference to the item identified by the specified ID.
    #[inline]
    pub fn get(&self, id: FastId<T>) -> Option<&T> {
        self.buckets[self.bucket_index].get(id.index())
    }

    /// Returns a mutable reference to the item identified by the specified ID.
    #[inline]
    pub fn get_mut(&mut self, id: FastId<T>) -> Option<&mut T> {
        self.buckets[self.bucket_index].get_mut(id.index())
    }

    /// Returns the number of allocated items in the arena.
    pub fn len(&self) -> usize {
        self.bucket_index * self.items_per_bucket + self.buckets[self.bucket_index].len()
    }

    /// Returns true is the arena is empty.
    pub fn is_empty(&self) -> bool {
        self.bucket_index == 0 && self.buckets[self.bucket_index].len() == 0
    }

    /// Resets the arena to the default state, with a single empty bucket.
    pub fn reset(&mut self) {
        self.bucket_index = 0;
        self.buckets.truncate(1);
    }
}

impl<T> Default for FastIdArena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Index<FastId<T>> for FastIdArena<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: FastId<T>) -> &Self::Output {
        &self.buckets[self.bucket_index][index.index()]
    }
}

impl<T> FastId<T> {
    #[inline]
    fn index(&self) -> usize {
        self.index
    }
}

impl<T> Copy for FastId<T> {}

impl<T> Clone for FastId<T> {
    #[inline]
    fn clone(&self) -> FastId<T> {
        *self
    }
}
