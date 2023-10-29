//! Implements a fast arena allocator that uses fixed size buckets and returns IDs for allocated objects.

use std::{
    mem::size_of,
    ops::{Deref, DerefMut},
    ptr,
    sync::RwLock,
};

#[cfg(test)]
#[path = "./rapid_arena_test.rs"]
mod rapid_arena_test;

const DEFAULT_BUCKET_SIZE_IN_BYTES: usize = 64 * 1024;

/// An arena that can be used to allocate objects efficiently.
#[derive(Debug)]
pub struct RapIdArena<T> {
    items_per_bucket: usize,
    internals: RwLock<ArenaInternals<T>>,
}

#[derive(Debug)]
struct ArenaInternals<T> {
    buckets: Vec<Vec<T>>,
    bucket_index: usize,
}

impl<T> RapIdArena<T> {
    /// Creates a new arena for the specified type.
    pub fn new() -> Self {
        let items_per_bucket = DEFAULT_BUCKET_SIZE_IN_BYTES / size_of::<T>();
        RapIdArena::<T> {
            items_per_bucket,
            internals: RwLock::new(ArenaInternals {
                buckets: vec![Vec::<T>::with_capacity(items_per_bucket)],
                bucket_index: 0,
            }),
        }
    }

    /// Creates a new arena with each bucket able to hold the specified number of items.
    pub fn new_with_bucket_size(items_per_bucket: usize) -> Self {
        if items_per_bucket == 0 {
            panic!("The specified number of items per bucket is invalid! The value must be greater than 0.")
        }
        RapIdArena::<T> {
            items_per_bucket,
            internals: RwLock::new(ArenaInternals {
                buckets: vec![Vec::<T>::with_capacity(items_per_bucket)],
                bucket_index: 0,
            }),
        }
    }

    /// The maximum number of items per bucket.
    pub fn items_per_bucket(&self) -> usize {
        self.items_per_bucket
    }

    /// Allocates the specified item inside the arena.
    #[inline]
    pub fn alloc(&mut self, item: T) -> RapId<T> {
        let mut internals = self.internals.write().unwrap();
        let mut bucket_index = internals.bucket_index;
        let mut bucket = &mut internals.buckets[bucket_index];

        if bucket.len() == self.items_per_bucket {
            internals
                .buckets
                .push(Vec::<T>::with_capacity(self.items_per_bucket));
            bucket_index += 1;
            internals.bucket_index = bucket_index;
            bucket = &mut internals.buckets[bucket_index];
        }

        let item_index = bucket.len();

        bucket.push(item);

        RapId {
            p: ptr::NonNull::from(&bucket[item_index]),
        }
    }

    /// Returns the number of allocated items in the arena.
    pub fn len(&self) -> usize {
        let internals = self.internals.read().unwrap();
        let bucket_index = internals.bucket_index;
        bucket_index * self.items_per_bucket + internals.buckets[bucket_index].len()
    }

    /// Returns true is the arena is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returs an iterator for the arena contents. This iterator is threadsafe.
    pub fn iter(&self) -> RapIdArenaIterator<T> {
        let mut data = vec![];
        let arena_internals = self.internals.read().unwrap();
        for bucket in &arena_internals.buckets {
            for item in bucket {
                data.push(RapId {
                    p: ptr::NonNull::from(item),
                })
            }
        }
        RapIdArenaIterator::<T> { data, index: 0 }
    }
}

impl<T> Default for RapIdArena<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Safety: items_per_bucket is immutable and all the internal values are protected via a RwLock.
unsafe impl<T> Send for RapIdArena<T> {}

// Safety: items_per_bucket is immutable and all the internal values are protected via a RwLock.
unsafe impl<T> Sync for RapIdArena<T> {}

/// An iterator for a RapIdArena instance.
#[derive(Debug)]
pub struct RapIdArenaIterator<T> {
    data: Vec<RapId<T>>,
    index: usize,
}

impl<T> Iterator for RapIdArenaIterator<T> {
    type Item = RapId<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.data.len() {
            let value = self.data[self.index];
            self.index += 1;
            Some(value)
        } else {
            None
        }
    }
}

/// An ID that contains an allocated object.
#[derive(Debug)]
pub struct RapId<T> {
    p: ptr::NonNull<T>,
}

impl<T> Copy for RapId<T> {}

impl<T> Clone for RapId<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self { p: self.p }
    }
}

impl<T> Deref for RapId<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            // Safety: The pointer is aligned, initialized, and dereferenceable by the guarantees made by Vec.
            // We require readers to borrow the RapId, and the lifetime of the return value is elided to the
            // lifetime of the input. This means the borrow checker will enforce that no one can mutate the
            // contents of the RapId until the reference returned is dropped.
            self.p.as_ref()
        }
    }
}

impl<T> DerefMut for RapId<T> {
    /// NOTE! If the mutable reference is used concurrently from multiple threads, then T has to be threadsafe
    ///       or race conditions may occur. Wrap T in Mutex or RwLock rather than storing T instances directly
    ///       in the arena.
    ///       However, it is safe to modify instances in a single parallel iterator as each item is accessed
    ///       only by a single thread at a time.
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            // Safety: The pointer is aligned, initialized, and dereferenceable by the guarantees made by Vec.
            // We require readers to borrow the RapId, and the lifetime of the return value is elided to the
            // lifetime of the input. This means the borrow checker will enforce that no one can mutate the
            // contents of the RapId until the reference returned is dropped.
            self.p.as_mut()
        }
    }
}

// Safety: No one besides us has the raw pointer, so we can safely transfer the RapId to another thread if T
// can be safely transferred.
unsafe impl<T: Send> Send for RapId<T> {}

// Safety: Since there exists a public way to go from a `&RapId<T>` to a `&T` in an unsynchronized fashion
// (such as `Deref`), then `RapId<T>` can't be `Sync` if `T` isn't. Conversely, `RapId` itself does not use
// any interior mutability whatsoever: all the mutations are performed through an exclusive reference
// (`&mut`). This means it suffices that `T` be `Sync` for `RapId<T>` to be `Sync`.
unsafe impl<T: Sync> Sync for RapId<T> {}
