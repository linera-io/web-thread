// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

pub use web_thread_select as web_thread;

mod pool;

pub type Guard<'a> = pool::Guard<'a, web_thread::Thread>;

pub struct Pool {
    pool: pool::Pool<web_thread::Thread, fn() -> web_thread::Thread>,
}

impl Pool {
    pub fn new(capacity: usize) -> Self {
        Self {
            pool: pool::Pool::new(capacity, web_thread::Thread::new)
        }
    }

    pub async fn get(&self) -> Guard<'_> {
        self.pool.get().await
    }
}
