// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// TODO: transitional
pub use web_thread_select as web_thread;

use std::sync::Arc;

mod pool;

pub type Guard<'a> = pool::Guard<'a, web_thread::Thread>;
pub type OwnedGuard = pool::OwnedGuard<web_thread::Thread>;

/// A lazily-initialized thread pool.
#[derive(Clone)]
pub struct Pool {
    pool: Arc<pool::Pool<web_thread::Thread, fn() -> web_thread::Thread>>,
}

impl Pool {
    /// Create a new thread pool with capacity for `capacity` threads.
    pub fn new(capacity: usize) -> Self {
        Self {
            pool: Arc::new(pool::Pool::new(capacity, web_thread::Thread::new))
        }
    }

    /// Get a reference to a free thread.
    pub async fn get(&self) -> Guard<'_> {
        self.pool.get().await
    }

    pub async fn get_owned(&self) -> OwnedGuard {
        Arc::clone(&self.pool).get_owned().await
    }
}
