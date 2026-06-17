// klc_runtime::channel — Multi-producer, multi-consumer channel
//
// Reference: docs/08-async-runtime.md
// Channels enable communication between async tasks.

use std::marker::PhantomData;

/// A channel for sending values of type T between tasks.
pub struct Channel<T> {
    id: u64,
    capacity: usize,
    _marker: PhantomData<T>,
}

impl<T> Channel<T> {
    /// Create a new channel with the given capacity.
    pub fn new(capacity: usize) -> Self {
        static NEXT_ID: std::sync::atomic::AtomicU64 =
            std::sync::atomic::AtomicU64::new(1);
        Self {
            id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            capacity,
            _marker: PhantomData,
        }
    }

    /// Returns the channel ID.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns the channel capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}
