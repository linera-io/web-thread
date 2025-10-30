<!-- cargo-rdme start -->

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

<!-- cargo-rdme end -->

## License

This project is available under the terms of the [Apache 2.0 license](../LICENSE).
