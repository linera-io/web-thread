// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/*!
# `web-thread-select`

This crate allows selecting between `web-thread` and `web-thread-shim`
using a feature flag.

If the target is a `wasm32` architecture and the `web` feature flag is
passed, we use the Web implementation of `web-thread`; otherwise, we
transparently substitute in the shim.
*/

cfg_if::cfg_if! {
    if #[cfg(all(target_arch = "wasm32", feature = "web"))] {
        pub use web_thread::*;
    } else {
        pub use web_thread_shim::*;
    }
}
