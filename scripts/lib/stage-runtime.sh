#!/usr/bin/env bash
# stage_runtime <dest> - copies the exact `web/` + `wasm/pkg/` +
# `assets/smufl/` relative layout that `music-xml.js` resolves against
# `import.meta.url`, dropping everything under wasm/ and assets/smufl/ that
# isn't actually loaded (installers, source crates, unused metadata/font
# formats). Sourced by both scripts/stage-package.sh and
# scripts/build-site.sh so the npm package and the exhibition site can't
# drift into two different runtime layouts.
#
# Assumes the caller has already `cd`'d to the repository root and already
# verified wasm/pkg/wasm.js exists.
stage_runtime() {
  local dest="$1"

  mkdir -p \
    "$dest/web" \
    "$dest/wasm/pkg" \
    "$dest/assets/smufl/bravura-bravura-1.392/redist/woff" \
    "$dest/assets/smufl/Leland-main" \
    "$dest/assets/smufl/Maestro-main" \
    "$dest/assets/smufl/Edwin-main"

  cp web/music-xml.js "$dest/web/"

  cp wasm/pkg/wasm.js wasm/pkg/wasm_bg.wasm wasm/pkg/wasm_bg.wasm.d.ts "$dest/wasm/pkg/"

  cp assets/smufl/bravura-bravura-1.392/redist/bravura_metadata.json \
     assets/smufl/bravura-bravura-1.392/redist/OFL.txt \
     "$dest/assets/smufl/bravura-bravura-1.392/redist/"
  cp assets/smufl/bravura-bravura-1.392/redist/woff/Bravura.woff \
     assets/smufl/bravura-bravura-1.392/redist/woff/Bravura.woff2 \
     "$dest/assets/smufl/bravura-bravura-1.392/redist/woff/"

  # The other music fonts in MUSIC_FONTS ship no web formats, so their OTFs
  # are served as they are.
  cp assets/smufl/Leland-main/leland_metadata.json \
     assets/smufl/Leland-main/Leland.otf \
     assets/smufl/Leland-main/LICENSE.txt \
     "$dest/assets/smufl/Leland-main/"
  cp "assets/smufl/Maestro-main/Finale Maestro.json" \
     assets/smufl/Maestro-main/FinaleMaestro.otf \
     assets/smufl/Maestro-main/OFL.txt \
     "$dest/assets/smufl/Maestro-main/"

  # Edwin, the text face in Leland's and Bravura's textFontFamily.
  cp assets/smufl/Edwin-main/Edwin-*.otf \
     assets/smufl/Edwin-main/LICENSE.txt \
     "$dest/assets/smufl/Edwin-main/"
}
