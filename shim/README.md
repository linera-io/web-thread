<!-- cargo-rdme start -->

# `web-thread-shim`

This crate mimics the public API of `web-thread`, but using native
futures and channels, to be substituted in when conditionally
compiling cross-platform software.

If you aren't using `web-thread`, you probably don't want this crate!
Just use `std::thread`.

<!-- cargo-rdme end -->

## License

This project is available under the terms of the [Apache 2.0 license](../LICENSE).
