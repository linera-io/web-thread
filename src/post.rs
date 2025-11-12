use super::{JsValue, js_sys};

/// Objects that can be sent via `postMessage`.  A type that is `Post`
/// supports being serialized into a JavaScript object that can be
/// sent using `postMessage`, and also getting an array of subobjects
/// that must be
/// [transferred](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Transferable_objects).
pub trait Post: AsJs {
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

        self.post_message_with_transferable(&message.to_js()?, &message.transferables())
    }
}

impl PostExt for web_sys::Worker {
    fn post(&self, message: impl Post) -> Result<(), JsValue> {
        // While not syntactically consumed, the use of `postMessage`
        // here may leave `Context` in an invalid state (setting
        // transferred JavaScript values to `undefined`).
        #![allow(clippy::needless_pass_by_value)]

        self.post_message_with_transfer(&message.to_js()?, &message.transferables())
    }
}

/// A serializable (JS-friendly) representation of a message plus its
/// transferables.
#[derive(serde::Serialize)]
pub struct Postable {
    #[serde(with = "serde_wasm_bindgen::preserve")]
    message: JsValue,
    #[serde(with = "serde_wasm_bindgen::preserve")]
    transfer: js_sys::Array,
}

impl Postable {
    pub fn new(message: impl Post) -> Result<Self, JsValue> {
        // While not syntactically consumed, the use of `postMessage`
        // may leave `Context` in an invalid state (setting
        // transferred JavaScript values to `undefined`).
        #![allow(clippy::needless_pass_by_value)]

        Ok(Self {
            message: message.to_js()?,
            transfer: message.transferables(),
        })
    }
}

/// An object-safe version of `std::convert::Into`.
pub trait AsJs {
    fn to_js(&self) -> Result<JsValue, JsValue>;
    fn from_js(js_value: JsValue) -> Result<Self, JsValue> where Self: Sized;
}

impl<T: serde::Serialize + serde::de::DeserializeOwned> AsJs for T {
    fn to_js(&self) -> Result<JsValue, JsValue> {
        Ok(serde_wasm_bindgen::to_value(self)?)
    }

    fn from_js(value: JsValue) -> Result<Self, JsValue> where Self: Sized {
        Ok(serde_wasm_bindgen::from_value(value)?)
    }
}

impl Post for () {}
impl Post for u8 {}
impl Post for u16 {}
impl Post for u32 {}
impl Post for u64 {}
impl Post for u128 {}
impl Post for i8 {}
impl Post for i16 {}
impl Post for i32 {}
impl Post for i64 {}
impl Post for i128 {}
impl Post for String {}

impl<T: Post, E: Post> Post for Result<T, E> where Result<T, E>: AsJs {
    fn transferables(&self) -> js_sys::Array {
        match self {
            Ok(x) => x.transferables(),
            Err(e) => e.transferables(),
        }
    }
}
