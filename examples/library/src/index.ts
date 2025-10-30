export { run } from './wasm/library.js';

import { default as init } from './wasm/library.js';

import { ethers } from 'ethers';

export async function initialize() {
  const mnemonic = ethers.Wallet.createRandom().mnemonic.phrase;
  console.log('your cool mnemonic is', mnemonic);
  await init();
}
