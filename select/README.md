<!-- cargo-rdme start -->


# `web-thread-select`

This crate allows selecting between `web-thread` and `web-thread-shim`
using a feature flag.

If the target is a `wasm32` architecture and the `web` feature flag is
passed, we use the Web implementation of `web-thread`; otherwise, we
transparently substitute in the shim.

<!-- cargo-rdme end -->

## License

This project is available under the terms of the [Apache 2.0 license](../LICENSE).
