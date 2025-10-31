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
    task::LocalFutureObj,
};

/// The type of errors that may arise from operations in this crate.
/// In this shim, this is an empty enum (as the operations are
/// infallible), but this type is provided anyway for compatibility
/// with `web-thread`.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {}

/// Convenience alias for `Result<T, Error>`.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// A thread running a local future executor ([`futures::executor::LocalPool`]).
pub struct Thread {
    sender: mpsc::UnboundedSender<Request>,
}

type Request = Box<dyn FnOnce() -> LocalFutureObj<'static, ()> + Send>;

impl Thread {
    /// Create a new background thread to run tasks.
    ///
    /// # Errors
    ///
    /// Never, actually: the error is here just for compatibility with
    /// `web-thread`.
    pub fn new() -> Result<Self, Error> {
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
        Ok(Self { sender })
    }

    /// Run a future on a separate thread.
    ///
    /// # Errors
    ///
    /// Never, actually: the error is here just for compatibility with
    /// `web-thread`.
    pub fn run<Context: Post, F: Future<Output: Post> + 'static>(
        &self,
        context: Context,
        code: impl FnOnce(Context) -> F + Send + 'static,
    ) -> impl Future<Output = Result<F::Output>> + '_ {
        let (sender, receiver) = oneshot::channel::<F::Output>();
        self.sender
            .unbounded_send(Box::new(move || {
                Box::new(async move {
                    let _ = sender.send(code(context).await);
                })
                .into()
            }))
            .unwrap_or_else(|_| panic!("worker shouldn't die unless dropped"));
        async move {
            Ok(receiver
                .await
                .unwrap_or_else(|_| panic!("worker shouldn't die unless dropped")))
        }
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
