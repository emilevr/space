//! Implements a fast arena allocator that uses fixed size buckets and returns IDs for allocated objects.

use std::{marker::PhantomData, mem::size_of, ops::Index};

#[cfg(test)]
#[path = "./rapid_arena_test.rs"]
mod rapid_arena_test;

const DEFAULT_BUCKET_SIZE_IN_BYTES: usize = 64 * 1024;

/// An arena that can be used to allocate objects efficiently.
#[derive(Debug)]
pub struct RapIdArena<T> {
    buckets: Vec<Vec<T>>,
    items_per_bucket: usize,
    bucket_index: usize,
}

/// An ID that identifies an allocated object.
#[derive(Debug)]
pub struct RapId<T> {
    bucket_index: usize,
    index: usize,
    _t: PhantomData<T>,
}

impl<T> RapIdArena<T> {
    /// Creates a new arena for the specified type.
    pub fn new() -> Self {
        let items_per_bucket = DEFAULT_BUCKET_SIZE_IN_BYTES / size_of::<T>();
        RapIdArena::<T> {
            buckets: vec![Vec::<T>::with_capacity(items_per_bucket)],
            items_per_bucket,
            bucket_index: 0,
        }
    }

    /// Creates a new arena with each bucket able to hold the specified number of items.
    pub fn new_with_bucket_size(items_per_bucket: usize) -> Self {
        RapIdArena::<T> {
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
    pub fn alloc(&mut self, item: T) -> RapId<T> {
        let mut bucket = &mut self.buckets[self.bucket_index];
        if bucket.len() == self.items_per_bucket {
            self.buckets
                .push(Vec::<T>::with_capacity(self.items_per_bucket));
            self.bucket_index += 1;
            bucket = &mut self.buckets[self.bucket_index];
        }

        let item_index = bucket.len();

        bucket.push(item);

        RapId::<T> {
            bucket_index: self.bucket_index,
            index: item_index,
            _t: PhantomData,
        }
    }

    /// Returns a reference to the item identified by the specified ID.
    #[inline]
    pub fn get(&self, id: RapId<T>) -> Option<&T> {
        if let Some(bucket) = self.buckets.get(id.bucket_index) {
            bucket.get(id.index)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the item identified by the specified ID.
    #[inline]
    pub fn get_mut(&mut self, id: RapId<T>) -> Option<&mut T> {
        if let Some(bucket) = self.buckets.get_mut(id.bucket_index) {
            bucket.get_mut(id.index)
        } else {
            None
        }
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

impl<T> Default for RapIdArena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Index<RapId<T>> for RapIdArena<T> {
    type Output = T;

    #[inline]
    fn index(&self, id: RapId<T>) -> &Self::Output {
        &self.buckets[id.bucket_index][id.index]
    }
}

impl<T> Copy for RapId<T> {}

impl<T> Clone for RapId<T> {
    #[inline]
    fn clone(&self) -> RapId<T> {
        *self
    }
}
