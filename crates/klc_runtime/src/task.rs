// klc_runtime::task — Async task handle
//
// Reference: docs/08-async-runtime.md

use std::marker::PhantomData;

/// A handle to an asynchronous task that will yield a value of type T.
pub struct Task<T> {
    id: u64,
    _marker: PhantomData<T>,
}

impl<T> Task<T> {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }
}
