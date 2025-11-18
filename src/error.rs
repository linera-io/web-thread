// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use super::{JsValue, js_sys, wasm_bindgen::JsCast as _};

#[derive(Debug)]
pub struct Error {
    description: String,
    source: Option<Box<Error>>,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.description)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|error| error.as_ref() as _)
    }
}

impl From<JsValue> for Error {
    fn from(value: JsValue) -> Self {
        let Some(error) = value.dyn_ref::<js_sys::Error>() else {
            return Error {
                description: format!(
                    "could not cast value of type {:?} to `Error`",
                    value.js_typeof()
                ),
                source: None,
            };
        };

        Error {
            description: error.message().into(),
            source: Some(error.cause())
                .filter(JsValue::is_undefined)
                .map(|x| Box::new(Error::from(x))),
        }
    }
}
