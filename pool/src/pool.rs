// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Mutex;

struct Inner<T, F = fn() -> T> {
    resources: Vec<T>,
    factory: F,
}

/// A pool of shared resources, each of which can only be used once at a time.
pub struct Pool<T, F = fn() -> T> {
    inner: Mutex<Inner<T, F>>,
    capacity: usize,
    sender: flume::Sender<*const T>,
    // we have to use an mpmc receiver here in order to be able to
    // receive using a reference: otherwise we would have to hold the
    // mutex guard over the await
    receiver: flume::Receiver<*const T>,
}

unsafe impl<T: Sync, F: Sync> Sync for Pool<T, F> {}
unsafe impl<T: Send, F: Send> Send for Pool<T, F> {}

/// A reference into the [`Pool`] that keeps its referent from being
/// used again until dropped.
pub struct Guard<'a, T> {
    resource: &'a T,
    sender: flume::Sender<*const T>,
}

unsafe impl<T: Sync> Send for Guard<'_, T> {}

impl<T> std::ops::Deref for Guard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.resource
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        let _ = self.sender.send(self.resource as *const _);
    }
}

impl<T, F: FnMut() -> T> Pool<T, F> {
    /// Create a new pool of `capacity` items, using `factory` to
    /// generate new items.
    pub fn new(capacity: usize, factory: F) -> Self {
        let (sender, receiver) = flume::unbounded();
        Self {
            inner: Mutex::new(Inner {
                resources: Vec::with_capacity(capacity),
                factory,
            }),
            capacity,
            sender,
            receiver,
        }
    }

    /// Get an item from the pool, waiting asynchronously if none is
    /// available.
    pub async fn get(&self) -> Guard<'_, T> {
        // the following is a false alarm
        #![allow(clippy::await_holding_lock)]

        let ptr = if let Ok(ptr) = self.receiver.try_recv() {
            Some(ptr)
        } else {
            let mut inner = self.inner.lock().unwrap();
            let len = inner.resources.len();
            if len < self.capacity {
                let resource = (inner.factory)();
                inner.resources.push(resource);
                Some(&inner.resources[len] as *const _)
            } else {
                drop(inner);
                self.receiver.recv_async().await.ok()
            }
        }.expect("we hold a sender");

        let resource = unsafe {
            // SAFETY:
            // This is safe because:
            // - all pointers we send around point into the data of
            //   the `Vec`, which is allocated on the heap
            // - while we sometimes get a `&mut` reference to the
            //   `Vec` we never use it to access an element other than
            //   one we have just created
            // - we never extend the `Vec` beyond its capacity, so
            //   never reallocate (i.e. invalidate pointers to) the
            //   existing elements
            &*ptr
        };

        Guard {
            resource,
            sender: self.sender.clone(),
        }
    }
}
