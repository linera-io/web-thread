// @ts-check

// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// We prefix this class with the crate name to reduce the chance of
// conflicts with `Client` classes in other crates, since
// `wasm-bindgen` doesn't qualify imports
// (https://github.com/wasm-bindgen/wasm-bindgen/issues/2798).

export class web_thread$Client {
  constructor(module, memory) {
    this.nextId = 0;
    this.promises = new Map();
    this.worker = new Worker(
      new URL('./worker.js', import.meta.url),
      { type: 'module' },
    );
    this.ready = new Promise(resolve => {
      this.setReady = resolve;
    });
    this.worker.onmessage = event => this.handleResponse(event);
    this.worker.postMessage({ type: 'init', module, memory });
  }

  async run(code, context, transfer) {
    await this.ready;
    return await new Promise((resolve, reject) => {
      const id = this.nextId++;
      if (id === Number.MAX_SAFE_INTEGER) this.nextId = 0;
      this.worker.postMessage({ type: 'run', id, code, context }, transfer);
      this.promises.set(id, { resolve, reject });
    });
  }

  destroy() {
    this.worker.postMessage({ type: 'destroy' });
  }

  handleResponse(event) {
    if (event.data.type === 'ready')
      this.setReady(null);
    else if (event.data.type === 'response') {
      let id = event.data.id;
      let { resolve, reject } = this.promises.get(id);
      this.promises.delete(id);
      if ('result' in event.data)
        resolve(event.data.result);
      else if ('error' in event.data)
        reject(event.data.error);
    } else {
      console.error('[web-thread] malformed response', event.data);
    }
  }
}
