#!/usr/bin/env bash
# Builds the wasm crate into wasm/pkg via wasm-pack. wasm/pkg is a build
# artifact (git-ignored, see wasm/pkg/.gitignore) - this script is the one
# source of truth for how it's produced, so `web/music-xml.js`'s relative
# imports (`../wasm/pkg/wasm.js`) always resolve to a fresh, matching build.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "wasm-pack not found - install it: https://rustwasm.github.io/wasm-pack/installer/" >&2
  exit 1
fi

wasm-pack build wasm --target web --out-dir pkg --out-name wasm
