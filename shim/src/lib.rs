#![allow(clippy::missing_panics_doc)]

/*!
# `web-thread-shim`

This crate mimics the public API of `web-thread`, but using native
futures and channels, to be substituted in when conditionally
compiling cross-platform software.

If you aren't using `web-thread`, you probably don't want this crate!
Just use `std::thread`.
 */

use futures::{
    channel::{mpsc, oneshot},
    future::FutureExt as _,
    task::LocalFutureObj,
};

use std::pin::Pin;
use std::task::{
    Context,
    Poll,
};

/// The type of errors that may arise from operations in this crate.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("thread killed before task completed")]
    Killed(#[from] oneshot::Canceled),
}

/// Convenience alias for `Result<T, Error>`.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// A thread running a local future executor ([`futures::executor::LocalPool`]).
pub struct Thread {
    sender: mpsc::UnboundedSender<Request>,
}

type Request = Box<dyn FnOnce() -> LocalFutureObj<'static, ()> + Send>;

/// A task that's been spawned on a [`Thread`] that should eventually
/// compute a `T`.
pub struct Task<T> {
    receiver: oneshot::Receiver<T>,
}


/// A [`Task`] with a `Send` output.
/// See [`Task::run_send`] for usage.
pub struct SendTask<T>(Task<T>);

impl<T: Send> Future for SendTask<T> {
    type Output = Result<T>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.poll_unpin(context)
    }
}

impl<T> Future for Task<T> {
    type Output = Result<T>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        self.receiver.poll_unpin(context).map(|ready| ready.map_err(Into::into))
    }
}

impl Thread {
    /// Create a new background thread to run tasks.
    pub fn new() -> Self {
        let (sender, mut receiver) = mpsc::unbounded::<Request>();
        std::thread::spawn(|| {
            use futures::{StreamExt as _, executor::LocalPool, task::LocalSpawn as _};
            let mut executor = LocalPool::new();
            let spawner = executor.spawner();
            executor.run_until(async move {
                while let Some(task) = receiver.next().await {
                    spawner
                        .spawn_local_obj(task())
                        .expect("executor should exist until destroyed");
                }
            });
        });
        Self { sender }
    }

    /// Execute a function on a thread.
    ///
    /// The function will begin executing immediately.  The resulting
    /// [`Task`] can be awaited to retrieve the result.
    pub fn run<Context: Post, F: Future<Output: Post> + 'static>(
        &self,
        context: Context,
        code: impl FnOnce(Context) -> F + Send + 'static,
    ) -> Task<F::Output> {
        let (sender, receiver) = oneshot::channel::<F::Output>();
        self.sender
            .unbounded_send(Box::new(move || {
                Box::new(async move {
                    let _ = sender.send(code(context).await);
                })
                .into()
            }))
            .unwrap_or_else(|_| panic!("worker shouldn't die unless dropped"));
        Task {
            receiver,
        }
    }

    /// Like [`Thread::run`], but the output can be sent through Rust
    /// memory without `Post`ing.
    ///
    /// In this shim, this is equivalent to [`Thread::run`].
    pub fn run_send<Context: Post, F: Future<Output: Send> + 'static>(
        &self,
        context: Context,
        code: impl FnOnce(Context) -> F + Send + 'static,
    ) -> SendTask<F::Output> {
        SendTask(self.run(context, code))
    }
}

/// Types that can be sent to another thread.  In this shim, this
/// trait is just an alias for `Send + 'static`, but in `web-thread`
/// some types can be sent only by performing an explicit transfer
/// operation.
pub trait Post: Send + 'static {}
impl<T: Send + 'static> Post for T {}

#[test]
fn basic_functionality() {
    assert_eq!(
        8u8,
        futures::executor::LocalPool::new()
            .run_until(
                Thread::new()
                    .unwrap()
                    .run(3u8, |three| async move { three + 5 })
            )
            .unwrap(),
    );
}
