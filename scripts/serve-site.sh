#!/usr/bin/env bash
# Serves the staged exhibition site (dist/site) over http://localhost for local
# inspection of the render page.
#
# Why a real static server and not `file://`: music-xml.js is an ES module that
# dynamic-imports wasm/pkg/wasm.js and fetches the SMuFL metadata, and browsers
# block both under file:// as cross-origin. The site also uses absolute paths
# (/js/nav.js, /samples.json), so it has to be served from dist/site as the
# document root -- serving the repo root instead is why those 404.
#
# Why http-server rather than `python3 -m http.server`: Python's server guesses
# content types from a table that has no .wasm entry on every platform, and a
# wasm file served as application/octet-stream makes the browser reject
# WebAssembly.instantiateStreaming outright. http-server resolves
# application/wasm correctly.
#
# -c-1 disables caching. This matters more than it looks: rebuilding wasm
# rewrites wasm_bg.wasm at the same URL, and a cached copy is the usual reason
# a rebuilt change appears not to have taken effect.
#
# Usage:
#   ./scripts/serve-site.sh            # serve dist/site on :8000
#   ./scripts/serve-site.sh 3000       # ... on :3000
#   ./scripts/serve-site.sh --build    # rebuild wasm + site first, then serve
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

PORT=8000
BUILD=0

for arg in "$@"; do
  case "$arg" in
    --build)
      BUILD=1
      ;;
    ''|*[!0-9]*)
      echo "usage: $0 [port] [--build]" >&2
      exit 1
      ;;
    *)
      PORT="$arg"
      ;;
  esac
done

if [ "$BUILD" -eq 1 ]; then
  ./scripts/build-wasm.sh
  ./scripts/build-site.sh
fi

if [ ! -f dist/site/index.html ]; then
  echo "dist/site is missing - run ./scripts/build-site.sh first (or pass --build)" >&2
  exit 1
fi

if ! command -v npx >/dev/null 2>&1; then
  echo "npx not found - install Node.js to use this script" >&2
  exit 1
fi

echo
echo "  Site:   http://localhost:$PORT/"
echo "  Render: http://localhost:$PORT/render/"
echo
echo "Ctrl-C to stop."
echo

# --yes keeps npx from prompting before it fetches http-server the first time;
# the major version is pinned so a future release can't change the flags this
# script passes.
exec npx --yes http-server@14 dist/site -p "$PORT" -c-1
