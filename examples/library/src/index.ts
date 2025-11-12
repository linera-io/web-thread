export { run } from './wasm/index.js';

import { default as init } from './wasm/index.js';

import { ethers } from 'ethers';

export async function initialize() {
  const mnemonic = ethers.Wallet.createRandom().mnemonic.phrase;
  console.log('your cool mnemonic is', mnemonic);
  await init();
}
