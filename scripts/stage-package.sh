#!/usr/bin/env bash
# Assembles the publishable @studiolavalse/music-xml package into dist/npm,
# mirroring the exact relative layout web/music-xml.js expects at runtime
# (../wasm/pkg/..., ../smufl/...) while dropping everything under wasm/ and
# smufl/ that isn't actually loaded (installers, source crates, unused
# metadata/font formats). This only stages files locally - it never publishes
# or uploads anything.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

if [ ! -f wasm/pkg/wasm.js ]; then
  echo "wasm/pkg is missing or stale - run scripts/build-wasm.sh first" >&2
  exit 1
fi

STAGE_DIR="dist/npm"
rm -rf "$STAGE_DIR"
mkdir -p \
  "$STAGE_DIR/web" \
  "$STAGE_DIR/wasm/pkg" \
  "$STAGE_DIR/smufl/bravura-bravura-1.392/redist/woff" \
  "$STAGE_DIR/smufl/metadata"

cp web/music-xml.js "$STAGE_DIR/web/"

cp wasm/pkg/wasm.js wasm/pkg/wasm_bg.wasm wasm/pkg/wasm_bg.wasm.d.ts "$STAGE_DIR/wasm/pkg/"

cp smufl/bravura-bravura-1.392/redist/bravura_metadata.json \
   smufl/bravura-bravura-1.392/redist/OFL.txt \
   "$STAGE_DIR/smufl/bravura-bravura-1.392/redist/"
cp smufl/bravura-bravura-1.392/redist/woff/Bravura.woff \
   smufl/bravura-bravura-1.392/redist/woff/Bravura.woff2 \
   "$STAGE_DIR/smufl/bravura-bravura-1.392/redist/woff/"
cp smufl/metadata/glyphnames.json "$STAGE_DIR/smufl/metadata/"

cp packaging/package.json "$STAGE_DIR/package.json"
cp LICENSE "$STAGE_DIR/LICENSE"

echo "Staged package at $STAGE_DIR"
du -sh "$STAGE_DIR"
