#!/usr/bin/env bash
# Assembles the publishable @studiolavalse/music-xml package into dist/npm.
# This only stages files locally - it never publishes or uploads anything.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

if [ ! -f wasm/pkg/wasm.js ]; then
  echo "wasm/pkg is missing or stale - run scripts/build-wasm.sh first" >&2
  exit 1
fi

source scripts/lib/stage-runtime.sh

STAGE_DIR="dist/npm"
rm -rf "$STAGE_DIR"
stage_runtime "$STAGE_DIR"

cp packaging/package.json "$STAGE_DIR/package.json"
cp LICENSE "$STAGE_DIR/LICENSE"

echo "Staged package at $STAGE_DIR"
du -sh "$STAGE_DIR"
