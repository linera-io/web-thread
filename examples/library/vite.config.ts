import { defineConfig, loadEnv } from 'vite';
import path from 'path';

import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { defineConfig } from 'vite'

export default defineConfig({
  base: '',
  resolve: {
    alias: {
      '@web-thread/wasm/shim': './bindgen/library.js',
    },
  },
  worker: {
    format: 'es',
  },
  build: {
    // lib: {
    //   formats: ['es'],
    //   entry: {
    //     'index': 'src/index.ts',
    //     // shouldn't need this?
    //     // secondary: resolve(__dirname, 'lib/secondary.js'),
    //   },
    //   name: '@web-thread/library',
    // },
    minify: false,
    rollupOptions: {
      input: {
        'index': 'src/index.ts',
      },
      output: {
        entryFileNames: '[name].mjs',
        manualChunks: ['']
      },
      preserveEntrySignatures: 'exports-only',
    },
  },
})
