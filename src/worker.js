// @ts-check

// This must be made available by the library consumer, since we don't
// know where it is yet.
import * as wasm from 'web-thread:wasm-shim';

self.onmessage = async (event) => {
  if (event.data.type === 'init') {
    await wasm.default(event.data);
    self.postMessage({ type: 'ready' });
  } else if (event.data.type === 'destroy') {
    self.close();
  } else if (event.data.type === 'run') {
    const { id, code, context } = event.data;
    try {
      const { message: result, transfer } = await wasm.__web_thread_worker_entry_point(code, context);
      self.postMessage({ type: 'response', id, result }, transfer);
    } catch (error) {
      console.error(error);
      self.postMessage({ type: 'response', id, error });
    }
  } else {
    console.error('[web-thread] malformed request', event.data);
  }
}
