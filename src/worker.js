// @ts-check

// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// This must be made available by the library consumer, since we don't
// know where it is yet.
import * as wasm from 'web-thread:wasm-shim';

const pending = new Map();

self.onmessage = async (event) => {
  if (event.data.type === 'init') {
    await wasm.default(event.data);
    self.postMessage({ type: 'ready' });
  } else if (event.data.type === 'destroy') {
    for (const id of pending.keys())
      self.postMessage({ type: 'response', id, error: new Error('thread destroyed during execution') });
    self.close();
  } else if (event.data.type === 'run') {
    const { id, code, context } = event.data;
    pending.set(id, null);
    try {
      const { message: result, transfer } = await wasm.__web_thread_worker_entry_point(code, context);
      self.postMessage({ type: 'response', id, result }, transfer);
    } catch (error) {
      console.error(error);
      self.postMessage({ type: 'response', id, error });
    } finally {
      pending.delete(id);
    }
  } else {
    console.error('[web-thread] malformed request', event.data);
  }
}
