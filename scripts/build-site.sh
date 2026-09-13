#!/usr/bin/env bash
# Assembles the exhibition site into dist/site: site/ verbatim, the wasm/web
# runtime (scripts/lib/stage-runtime.sh - the same one scripts/stage-package.sh
# uses, so the two can't drift into different runtime layouts), the current
# sample set, and samples.json. This is exactly what CI uploads to GitHub
# Pages, so a green local run of this script is the deploy rehearsal.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

if [ ! -f wasm/pkg/wasm.js ]; then
  echo "wasm/pkg is missing or stale - run scripts/build-wasm.sh first" >&2
  exit 1
fi

source scripts/lib/stage-runtime.sh

STAGE_DIR="dist/site"
rm -rf "$STAGE_DIR"
mkdir -p "$STAGE_DIR"
cp -R site/. "$STAGE_DIR/"

stage_runtime "$STAGE_DIR"

# The two UTF-16 samples are excluded by name, not sniffed at build time:
# web/music-xml.js does `response.text()`, which decodes as UTF-8, so both
# fail in the browser (they render fine through the CLI, which sniffs the
# BOM in read_musicxml). Delete this list once the component learns to do
# the same. Everything else in assets/xmlsamples/ is discovered below, so a
# new sample needs no edit here - and the __MACOSX/._*.musicxml AppleDouble
# resource forks are skipped for free, since they live one directory deeper
# than this non-recursive glob reaches.
EXCLUDED_SAMPLES=(
  "MozaChloSample.musicxml"
  "MozaVeilSample.musicxml"
)

is_excluded() {
  local name="$1"
  local excluded
  for excluded in "${EXCLUDED_SAMPLES[@]}"; do
    [ "$name" = "$excluded" ] && return 0
  done
  return 1
}

mkdir -p "$STAGE_DIR/assets/xmlsamples"
for file in assets/xmlsamples/*.musicxml; do
  name="$(basename "$file")"
  is_excluded "$name" && continue
  cp "$file" "$STAGE_DIR/assets/xmlsamples/$name"
done

python3 scripts/lib/generate-samples-json.py "$STAGE_DIR/assets/xmlsamples" "$STAGE_DIR/samples.json"

touch "$STAGE_DIR/.nojekyll"

echo "Staged site at $STAGE_DIR"
du -sh "$STAGE_DIR"
