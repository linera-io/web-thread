/*!
# `web-thread`

A crate for long-running, shared-memory threads in a browser context
for use with
[`wasm-bindgen`](https://github.com/wasm-bindgen/wasm-bindgen).
Supports sending non-`Send` data across the boundary using
`postMessage` and
[transfer](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Transferable_objects).

## Requirements

Like all Web threading solutions, this crate requires Wasm atomics,
bulk memory, and mutable globals:

`.cargo/config.toml`

```toml
[target.wasm32-unknown-unknown]
rustflags = [
    "-C", "target-feature=+atomics,+bulk-memory,+mutable-globals",
]
```

as well as cross-origin isolation on the serving Web page in order to
[enable the use of
`SharedArrayBuffer`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer#security_requirements),
i.e. the HTTP headers

```text
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

The `credentialless` value for `Cross-Origin-Embedder-Policy` should
also work, but at the time of writing is not supported in Safari.

## Linking the binary

Since this crate can't know the location of your shim script and Wasm
binary ahead of time, you must make the module identifier
`web-thread:wasm-shim` resolve to the path of your `wasm-bindgen` shim
script.  This can be done with a bundler such as
[Vite](https://vite.dev/) or [Webpack](https://webpack.js.org/), or by
using a source-transformation tool such as
[`tsc-alias`](https://www.npmjs.com/package/tsc-alias?activeTab=readme):

`tsconfig.json`

```json
{
    "compilerOptions": {
        "baseUrl": "./",
        "paths": {
            "web-thread:wasm-shim": ["./src/wasm/my-library.js"]
        }
    },
    "tsc-alias": {
        "resolveFullPaths": true
    }
}
```

Turbopack is currently not supported due to an open issue when
processing cyclic dependencies.  See the following discussions for
more information:

* [Turbopack: dynamic cyclical import causes infinite loop (#85119)](https://github.com/vercel/next.js/issues/85119)
* [Next.js v15.2.2 Turbopack Dev server stuck in compiling + extreme CPU/memory usage (#77102)](https://github.com/vercel/next.js/discussions/77102)
* [Eliminate the circular dependency between the main loader and the worker (#20580)](https://github.com/emscripten-core/emscripten/issues/20580)

*/

use futures::future;
use wasm_bindgen::prelude::{JsValue, wasm_bindgen};
use wasm_bindgen_futures::JsFuture;
use web_sys::{js_sys, wasm_bindgen};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[wasm_bindgen(module = "/src/Client.js")]
extern "C" {
    type Client;
    #[wasm_bindgen(catch, constructor)]
    fn new(module: JsValue, memory: JsValue) -> Result<Client, JsValue>;

    #[wasm_bindgen(method)]
    fn run(
        this: &Client,
        code: JsValue,
        context: JsValue,
        transfer: js_sys::Array,
    ) -> js_sys::Promise;

    #[wasm_bindgen(method)]
    fn destroy(this: &Client);
}

/// A representation of a JavaScript thread (Web worker with shared memory).
pub struct Thread(Client);

impl Thread {
    /// Spawn a new thread.
    ///
    /// # Errors
    ///
    /// If the worker can't be started or initialized with the shared memory.
    pub fn new() -> Result<Self, Error> {
        Ok(Self(Client::new(
            wasm_bindgen::module(),
            wasm_bindgen::memory(),
        )?))
    }

    /// Execute a function on a thread.
    ///
    /// In JavaScript style, the function will begin executing
    /// immediately.  The resulting `Future` can be awaited to
    /// retrieve the result.
    ///
    /// # Arguments
    ///
    /// ## `context`
    ///
    /// A [`Post`]able context that will be sent across the thread
    /// boundary using `postMessage` and passed to the function on the
    /// other side.
    ///
    /// ## `code`
    ///
    /// A `FnOnce` implementation containing the code in question.
    /// The function is async, but will run on a `Worker` so may block
    /// (though doing so will block the thread!).  The function itself
    /// must be `Send`, and `Send` values can be sent through in its
    /// closure, but once executed the resulting [`Future`] will not
    /// be moved, so needn't be `Send`.
    pub fn run<Context: Post, F: Future<Output: Post> + 'static>(
        &self,
        context: Context,
        code: impl FnOnce(Context) -> F + Send + 'static,
    ) -> impl Future<Output = Result<F::Output>> {
        // While not syntactically consumed, the use of `postMessage`
        // here may leave `Context` in an invalid state (setting
        // transferred JavaScript values to `undefined`).
        #![allow(clippy::needless_pass_by_value)]

        let transfer = context.transferables();
        let context = match serde_wasm_bindgen::to_value(&context) {
            Ok(context) => context,
            Err(error) => return future::Either::Left(async { Err(error.into()) }),
        };
        let promise = self.0.run(Code::new(code).into(), context, transfer);
        future::Either::Right(async {
            Ok(serde_wasm_bindgen::from_value(
                JsFuture::from(promise).await?,
            )?)
        })
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        self.0.destroy();
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_wasm_bindgen::Error),
    #[error("JavaScript error: {0:?}")]
    Js(JsValue),
}

impl From<JsValue> for Error {
    fn from(js_value: JsValue) -> Self {
        Self::Js(js_value)
    }
}

/// Objects that can be sent via `postMessage`.  A type that is `Post`
/// supports being serialized into a JavaScript object that can be
/// sent using `postMessage`, and also getting an array of subobjects
/// that must be
/// [transferred](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Transferable_objects).
pub trait Post: serde::ser::Serialize + serde::de::DeserializeOwned {
    /// Get a list of the objects that must be
    /// transferred when calling `postMessage`.
    ///
    /// The default implementation returns an empty array.
    fn transferables(&self) -> js_sys::Array {
        js_sys::Array::new()
    }
}

/// Convenience trait for something that can have messages posted to
/// it, including
/// [transferables](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Transferable_objects).
pub trait PostExt {
    /// Send a value, transferring subobjects as necessary.
    ///
    /// This function consumes `message`, as in general it may leave
    /// the object in an incoherent state.
    ///
    /// # Errors
    ///
    /// If the message could not be sent.
    fn post(&self, message: impl Post) -> Result<(), JsValue>;
}

impl PostExt for web_sys::MessagePort {
    fn post(&self, message: impl Post) -> Result<(), JsValue> {
        // While not syntactically consumed, the use of `postMessage`
        // here may leave `Context` in an invalid state (setting
        // transferred JavaScript values to `undefined`).
        #![allow(clippy::needless_pass_by_value)]

        self.post_message_with_transferable(
            &serde_wasm_bindgen::to_value(&message)?,
            &message.transferables(),
        )
    }
}

impl PostExt for web_sys::Worker {
    fn post(&self, message: impl Post) -> Result<(), JsValue> {
        // While not syntactically consumed, the use of `postMessage`
        // here may leave `Context` in an invalid state (setting
        // transferred JavaScript values to `undefined`).
        #![allow(clippy::needless_pass_by_value)]

        self.post_message_with_transfer(
            &serde_wasm_bindgen::to_value(&message)?,
            &message.transferables(),
        )
    }
}

impl Post for u8 {}

/// A serializable (JS-friendly) representation of a message plus its
/// transferables.
#[derive(serde::Serialize)]
struct Postable {
    #[serde(with = "serde_wasm_bindgen::preserve")]
    message: JsValue,
    #[serde(with = "serde_wasm_bindgen::preserve")]
    transfer: js_sys::Array,
}

impl Postable {
    fn new(message: impl Post) -> Result<Self, serde_wasm_bindgen::Error> {
        // While not syntactically consumed, the use of `postMessage`
        // may leave `Context` in an invalid state (setting
        // transferred JavaScript values to `undefined`).
        #![allow(clippy::needless_pass_by_value)]

        Ok(Self {
            message: serde_wasm_bindgen::to_value(&message)?,
            transfer: message.transferables(),
        })
    }
}

type Task = std::pin::Pin<Box<dyn Future<Output = Result<Postable, JsValue>>>>;
type RemoteTask = Box<dyn FnOnce(JsValue) -> Task + Send>;

struct Code {
    // The second box allows us to represent this as a thin pointer
    // (Wasm: u32) which, unlike fat pointers (Wasm: u64) is within
    // the [JavaScript safe integer
    // range](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/isSafeInteger).
    code: Option<Box<RemoteTask>>,
}

impl Code {
    fn new<F: Future<Output: Post> + 'static, Context: Post>(
        code: impl FnOnce(Context) -> F + Send + 'static,
    ) -> Self {
        Self {
            code: Some(Box::new(Box::new(|context| {
                Box::pin(async move {
                    Ok(Postable::new(
                        code(serde_wasm_bindgen::from_value(context)?).await,
                    )?)
                })
            }))),
        }
    }

    async fn call_once(mut self, context: JsValue) -> Result<Postable, JsValue> {
        (*self.code.take().expect("code called more than once"))(context).await
    }

    /// # Safety
    ///
    /// Must only be called on `JsValue`s created with the
    /// `Into<JsValue>` implementation.
    unsafe fn from_js_value(js_value: &JsValue) -> Self {
        // We know this doesn't truncate or lose sign as the `f64` is
        // a representation of a 32-bit pointer.
        #![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

        Self {
            code: Some(unsafe { Box::from_raw(js_value.as_f64().unwrap() as u32 as _) }),
        }
    }
}

impl From<Code> for JsValue {
    fn from(code: Code) -> Self {
        (Box::into_raw(code.code.expect("serializing consumed code")) as u32).into()
    }
}

#[doc(hidden)]
#[wasm_bindgen]
pub async unsafe fn __web_thread_worker_entry_point(
    code: JsValue,
    context: JsValue,
) -> Result<JsValue, JsValue> {
    let code = unsafe { Code::from_js_value(&code) };
    serde_wasm_bindgen::to_value(&code.call_once(context).await?).map_err(Into::into)
}

#[wasm_bindgen(module = "/src/worker.js")]
extern "C" {
    // This is here just to ensure `/src/worker.js` makes it into the
    // bundle produced by `wasm-bindgen`.
    fn _non_existent_function();
}
