#!/usr/bin/env bash

set -eu

shopt -s extglob

cd $(dirname -- "${BASH_SOURCE[0]}")

if [ "${1-}" = "--release" ]
then
    profile_flag=--release
    profile_dir=release
else
    profile_flag=
    profile_dir=debug
fi

wasm_bindgen_cli_version=$(wasm-bindgen --version)
wasm_bindgen_cli_version=${wasm_bindgen_cli_version##* }

wasm_bindgen_cargo_version=$(cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "wasm-bindgen").version')
target_dir=$(cargo metadata --format-version 1 | jq -r .target_directory)
package_name=$(cargo metadata --format-version 1 | jq -r '.packages[]|select(.manifest_path == $manifest).name' --arg manifest "$(realpath Cargo.toml)")
binary_name="${package_name//-/_}"
binary_path="$target_dir"/wasm32-unknown-unknown/$profile_dir/"${binary_name}".wasm

if [[ "$wasm_bindgen_cargo_version" != "$wasm_bindgen_cli_version" ]]
then
    cargo update --package wasm-bindgen --precise "$wasm_bindgen_cli_version"
fi

cargo build --lib --target wasm32-unknown-unknown $profile_flag

wasm-bindgen \
    $binary_path \
    --out-dir src/wasm \
    --typescript \
    --target web

mkdir -p dist/wasm
cp src/wasm/"${binary_name}"_bg.wasm{,.d.ts} dist/wasm

pnpm exec tsc
pnpm exec tsc-alias
