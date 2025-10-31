#![allow(dead_code, unused_imports)]

use wasm_bindgen::prelude::{JsCast as _, JsError, JsValue, wasm_bindgen};
use web_sys::{js_sys, wasm_bindgen};

use web_thread::PostExt as _;

#[derive(serde::Serialize, serde::Deserialize)]
struct Context {
    #[serde(with = "serde_wasm_bindgen::preserve")]
    port: web_sys::MessagePort,
}

impl Context {
    async fn receive_message(&self) -> Result<u8, JsValue> {
        let promise = js_sys::Promise::new(&mut |resolve, reject| {
            self.port.set_onmessage(Some(&resolve));
            self.port.set_onmessageerror(Some(&reject));
        });
        self.port.post(0u8).unwrap();
        let message = wasm_bindgen_futures::JsFuture::from(promise).await?;
        Ok(
            serde_wasm_bindgen::from_value(message.dyn_into::<web_sys::MessageEvent>()?.data())
                .unwrap(),
        )
    }
}

impl web_thread::Post for Context {
    fn transferables(&self) -> js_sys::Array {
        std::iter::once(&self.port).collect()
    }
}

async fn calculate(context: Context) -> u8 {
    web_sys::console::log_1(&"[child] began execution".into());
    context.receive_message().await.unwrap() + 3
}

#[wasm_bindgen]
pub async fn run() -> Result<u8, JsValue> {
    let thread = web_thread::Thread::new().map_err(JsError::from)?;
    let channel = web_sys::MessageChannel::new()?;
    let message = js_sys::Promise::new(&mut |resolve, reject| {
        channel.port2().set_onmessage(Some(&resolve));
        channel.port2().set_onmessageerror(Some(&reject));
    });
    let job = thread.run(
        Context {
            port: channel.port1(),
        },
        calculate,
    );
    let _zero = wasm_bindgen_futures::JsFuture::from(message).await.unwrap();
    channel.port2().post(12u8)?;
    Ok(job.await.map_err(JsError::from)?)
}

#[wasm_bindgen(start)]
fn start() {
    console_error_panic_hook::set_once();
}
