// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::{
    pin::Pin,
    sync::RwLock,
    task::{Context, Poll},
};

use web_thread_select as web_thread;

type Id = usize;

pub use web_thread::Error;
pub type Task<T> = Guard<web_thread::Task<T>>;
pub type SendTask<T> = Guard<web_thread::SendTask<T>>;

struct ResourceHandle {
    id: Id,
    sender: flume::Sender<Id>,
}

impl Drop for ResourceHandle {
    fn drop(&mut self) {
        let _ = self.sender.send(self.id);
    }
}

/// A pool of shared resources, each of which can only be used once at a time.
pub struct Pool {
    threads: RwLock<Vec<web_thread::Thread>>,
    capacity: usize,
    sender: flume::Sender<Id>,
    // we have to use an mpmc receiver here in order to be able to
    // receive using a reference: otherwise we would have to hold the
    // mutex guard over the await
    receiver: flume::Receiver<Id>,
}

pin_project_lite::pin_project! {
    /// A future that, while running, causes the thread to be considered
    /// claimed.
    pub struct Guard<F> {
        #[pin]
        future: F,
        handle: ResourceHandle,
    }
}

impl<F: Future> Future for Guard<F> {
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        self.project().future.poll(context)
    }
}

impl Pool {
    /// Create a new pool of `capacity` items, using `factory` to
    /// generate new items.
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = flume::unbounded();
        Self {
            threads: RwLock::new(Vec::with_capacity(capacity)),
            capacity,
            sender,
            receiver,
        }
    }

    async fn get(&self) -> Id {
        let mut id = self.receiver.try_recv().ok();

        if id.is_none() {
            let mut threads = self.threads.write().unwrap();
            let len = threads.len();
            if len < self.capacity {
                threads.push(web_thread::Thread::new());
                id = Some(len);
            }
        }

        if id.is_none() {
            id = self.receiver.recv_async().await.ok();
        }

        id.expect("we hold a sender")
    }

    /// Run a job, creating a new thread if necessary or waiting for one to become available.
    pub async fn run<Context: web_thread::Post, F: Future<Output: web_thread::Post> + 'static>(
        &self,
        context: Context,
        code: impl FnOnce(Context) -> F + Send + 'static,
    ) -> Task<F::Output> {
        let id = self.get().await;
        Guard {
            future: self.threads.read().unwrap()[id].run(context, code),
            handle: ResourceHandle {
                sender: self.sender.clone(),
                id,
            },
        }
    }

    /// Like [`Pool::run`], but the output can be sent through Rust
    /// memory without `Post`ing.
    pub async fn run_send<Context: web_thread::Post, F: Future<Output: Send> + 'static>(
        &self,
        context: Context,
        code: impl FnOnce(Context) -> F + Send + 'static,
    ) -> SendTask<F::Output> {
        let id = self.get().await;
        Guard {
            future: self.threads.read().unwrap()[id].run_send(context, code),
            handle: ResourceHandle {
                sender: self.sender.clone(),
                id,
            },
        }
    }
}
