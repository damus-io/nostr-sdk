#!/bin/bash
#
# Build the JavaScript modules
#
# This script is really a workaround for https://github.com/rustwasm/wasm-pack/issues/1074.
#
# Currently, the only reliable way to load WebAssembly in all the JS
# environments we want to target (web-via-webpack, web-via-browserify, jest)
# seems to be to pack the WASM into base64, and then unpack it and instantiate
# it at runtime.
#
# Hopefully one day, https://github.com/rustwasm/wasm-pack/issues/1074 will be
# fixed and this will be unnecessary.

set -e

cd $(dirname "$0")/..

WASM_BINDGEN_WEAKREF=1 wasm-pack build --target nodejs --scope rust-nostr --out-dir pkg "${WASM_PACK_ARGS[@]}"

# Convert the Wasm into a JS file that exports the base64'ed Wasm.
echo "module.exports = \`$(base64 pkg/nostr_js_bg.wasm)\`;" > pkg/nostr_js_bg.wasm.js

# In the JavaScript:
#  1. Strip out the lines that load the WASM, and our new epilogue.
#  2. Remove the imports of `TextDecoder` and `TextEncoder`. We rely on the global defaults.
{
  sed -e '/Text..coder.*= require(.util.)/d' \
      -e '/^const path = /,$d' pkg/nostr_js.js
  cat scripts/epilogue.js
} > pkg/nostr_js.js.new
mv pkg/nostr_js.js.new pkg/nostr_js.js

# also extend the typescript
cat scripts/epilogue.d.ts >> pkg/nostr_js.d.ts