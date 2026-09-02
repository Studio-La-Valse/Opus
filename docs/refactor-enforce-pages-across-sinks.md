# Refactor: enforce pages across all canvas sinks

## Why

The three canvas sinks are driven inconsistently:

| Sink | Compositor entry point | Shape | Cross-page shared state | Output |
|---|---|---|---|---|
| SVG (`lib/src/drawable/canvas/svg/mod.rs`) | `RenderCompositor::walk()` | one surface, global tenths | none | one `String` |
| FlatBuffer (`lib/src/drawable/canvas/flat_buffer/mod.rs`) | `RenderCompositor::walk()` | one surface, global tenths | none | one `FlatBuffer` |
| PDF (`lib/src/drawable/canvas/pdf/mod.rs`) | `RenderCompositor::walk_pages()` | one canvas per page + a stitch step | yes — one `FontSet`, glyphs subset/unioned across pages, embedded once | `Vec<PdfPage>` → `write_pdf` |

`RenderCompositor::walk()` is literally `walk_pages()` followed by a concat. The CLI
carries an `if pdf { … } else { … }` branch in `cli/src/commands/render.rs` where the
two writer functions have divergent signatures.

`walk_pages` is the real primitive; pagination is a genuine fact of the domain (the
layout engine already produces `score.pages`). Standardising every path on it removes
the asymmetry and lets the flat buffer carry page boundaries across the wasm boundary,
which the web side currently throws away.

## Scope

**In:**

1. Make `walk_pages` the only compositor entry point; delete `walk()`.
2. CLI: after `engrave`, walk pages once and dispatch per format; both writers take `&[RenderedPage]`.
3. SVG: emit one file per page (`--out score.svg` → `score-p1.svg`, `score-p2.svg`, …).
4. FlatBuffer: keep it a single buffer, but add a parallel page table (`[geom_start_index, origin_x, origin_y, width, height]` per page) to `RenderOutput` so JS *can* consume page boundaries. No behaviour change to the flat stream itself.
5. wasm: swap `walk()` for `walk_pages()`; populate the page table.
6. Tests + docs.

**Out (do not touch):**

- No `Canvas` trait redesign beyond the single default no-op method in step 4b.
- No `DocumentSink` trait — only two real implementors (multi-file SVG, PDF) with
  divergent needs (shared font state, single-blob vs multi-file, `String` vs `Vec<u8>`).
- No per-page-canvas rewrite of `web/music-xml.js`. The page table is additive; the
  current single-canvas renderer keeps working by ignoring it. That rewrite is
  separate, unblocked follow-up work.
- No changes to `ffi/`.

## Open decision — RESOLVED

`--out` is now an **optional directory** for every format (not a filename). When
given it is created if missing; when omitted, output is written to the directory
containing `--file`. Output filenames reuse the input file's stem:

- `render pdf` → `<stem>.pdf` (single file, no page suffix).
- `render svg` → one file per page, `<stem>-p1.svg`, `<stem>-p2.svg`, … (always
  suffixed, even for a single-page score).

SVG output is per-page only; there is no combined single-document option.

`--overwrite` (bool, default false) gates replacing an existing output file:
without it, `render` panics rather than overwrite a file it didn't create. This
guards the "omitted `--out` writes next to `--file`" default — e.g. `render pdf
--file assets/xmlsamples/X.musicxml` with no `--out` targets
`assets/xmlsamples/X.pdf`, one of the tracked reference PDFs — from silently
clobbering something. Pass `--out` to redirect, or `--overwrite` to accept the
replacement.

There are no in-repo consumers of the SVG output (checked `scripts/`, `packaging/`,
`README.md`, `platforms/`), so blast radius is the CLI surface only.

---

## Steps

### Step 1 — Compositor: `walk_pages` becomes the only entry point

File: `lib/src/score/visual/render_compositor.rs`

- Delete `RenderCompositor::walk()` (the flattening wrapper, ~lines 56–67).
- Fix the `RenderedPage` doc comment that references `walk` (it says "the same
  coordinates `RenderCompositor::walk` produces" — reword to describe the coordinate
  space directly).
- Keep `walk_pages` and `RenderedPage` exactly as they are (`origin: XY`, `width`,
  `height`, `elements: Vec<DrawableElement>`, all in global MusicXML tenths).

### Step 2 — wasm: inline the flatten + build the page table

File: `wasm/src/lib.rs`

2a. `RenderOutput` — add a field and getter:

```rust
pub struct RenderOutput {
    // … existing fields …
    /// Parallel page table: 5 f32s per page —
    /// `[geometry_start_index, origin_x, origin_y, width, height]`.
    /// Page i's records span `geometry[page_table[5i] .. page_table[5(i+1)]]`
    /// (the last page runs to `geometry.len()`). Coordinates are in the same
    /// space as `geometry` (global tenths, already `render_scale`-adjusted).
    page_table: Vec<f32>,
}
```

Getter mirrors the other `mem::take` getters:

```rust
#[wasm_bindgen(getter)]
pub fn page_table(&mut self) -> Vec<f32> {
    std::mem::take(&mut self.page_table)
}
```

2b. `render()` — replace the two `RenderCompositor::base().walk(...)` /
`.debug().walk(...)` calls (lines ~266–271) with `walk_pages`. The `render_scale`
down-scaling below operates on the flattened `Vec<DrawableElement>` and must keep
working, and font dedup must stay global (one `FlatBufferCanvas` for the whole
document). Use the new `CanvasPainter::paint_pages` from step 4:

```rust
let mut pages = RenderCompositor::base().walk_pages(&cache.score, &fonts);
if debug {
    for (page, dbg) in pages.iter_mut().zip(RenderCompositor::debug().walk_pages(&cache.score, &fonts)) {
        page.elements.extend(dbg.elements);
    }
}

// render_scale: compute over all elements, then scale each page's elements in place.
let all_bounds = compute_bounds_over_pages(&pages); // helper, or flatten refs
// … existing MAX_CANVAS_PIXELS math on all_bounds …
if render_scale < 1.0 {
    for page in &mut pages {
        page.elements = page.elements.iter().map(|el| el.scale(render_scale, XY::ZERO)).collect();
        page.origin = /* scaled */;
        page.width *= render_scale;
        page.height *= render_scale;
    }
}

let flat = CanvasPainter::new(FlatBufferCanvas::new()).paint_pages(&pages);
```

Note: page `origin`/`width`/`height` must be scaled by `render_scale` too so the page
table stays consistent with the scaled geometry.

2c. Update the `render_pdf` doc-note in the same file — it currently says
`compositor.walk(..)`; there is no longer a `walk`.

### Step 3 — FlatBuffer canvas: record page boundaries

File: `lib/src/drawable/canvas/flat_buffer/mod.rs`

- Add to `FlatBuffer`:

```rust
/// 5 f32s per page — `[geometry_start_index, origin_x, origin_y, width, height]`.
pub page_table: Vec<f32>,
```

- Add to `FlatBufferCanvas` a `page_table: Vec<f32>` field and a method:

```rust
/// Marks the start of a page's records. Call before dispatching that page's
/// elements. Records the current `geometry` length as the page's start index.
pub fn begin_page(&mut self, origin_x: f32, origin_y: f32, width: f32, height: f32) {
    self.page_table.extend_from_slice(&[
        self.geometry.len() as f32, origin_x, origin_y, width, height,
    ]);
}
```

- `finish()` moves `page_table` into `FlatBuffer`.
- Doc-comment the record layout note that already lives here, and add `page_table`
  to the "Keep in sync with the decoder in `web/music-xml.js`" note.

### Step 4 — CanvasPainter: a page-aware drive

File: `lib/src/drawable/canvas.rs`

4a. Add a default no-op method to the `Canvas` trait:

```rust
/// Called once before each page's elements when driven via
/// [`CanvasPainter::paint_pages`]. Default: no-op. Sinks that track page
/// boundaries (the flat buffer) override this; SVG/PDF ignore it because they
/// build one surface per page instead.
fn begin_page(&mut self, _origin_x: f32, _origin_y: f32, _width: f32, _height: f32) {}
```

4b. Add `CanvasPainter::paint_pages`:

```rust
pub fn paint_pages(mut self, pages: &[RenderedPage<'_>]) -> C::Output {
    // Global bounds over every page's elements, same contract as `paint`.
    let all: Vec<&DrawableElement<'_>> = pages.iter().flat_map(|p| p.elements.iter()).collect();
    self.canvas.begin(compute_bounds_refs(&all)); // add a &[&DrawableElement] variant, or clone

    for page in pages {
        self.canvas.begin_page(page.origin.x, page.origin.y, page.width, page.height);
        for el in &page.elements {
            match el { /* same 5-arm dispatch as `paint` — factor into a private fn */ }
        }
    }

    self.canvas.finish()
}
```

Factor the 5-arm `match` in `paint` into a private `fn draw_one(canvas: &mut C, el: &DrawableElement)`
and call it from both `paint` and `paint_pages`.

`RenderedPage` lives in `lib/src/score/visual/render_compositor.rs`; `canvas.rs` will
need to `use` it (or accept a more generic `&[(rect, &[DrawableElement])]` — prefer
importing `RenderedPage`, it is already a `lib`-internal type).

`compute_bounds` currently takes `&[DrawableElement]`; add an `&[&DrawableElement]`
variant or just build a flattened owned `Vec` for the bounds call (cheap relative to
the render).

### Step 5 — CLI SVG canvas: explicit per-page viewBox

File: `lib/src/drawable/canvas/svg/mod.rs`

- `SvgCanvas::new()` → `SvgCanvas::new(view_box: (f32, f32, f32, f32))` carrying
  `(origin_x, origin_y, width, height)`. Store it.
- `begin()` ignores its `bounds` argument and emits the stored `viewBox` /
  `width` / `height` (mirror `PdfPageCanvas::begin`, which ignores `_bounds`).
- This also fixes a latent inconsistency: SVG currently sizes to element bounds
  (clipping page margins / trailing whitespace); the page rect is the correct box and
  matches what PDF does.
- No coordinate translation — SVG `viewBox` with a non-zero origin handles the page
  offset natively (`viewBox="1400 0 1360 1760"` with elements at their global coords).

### Step 6 — CLI SVG writer: loop pages, one file per page

File: `cli/src/commands/render/svg.rs`

- `write(...)` takes `pages: &[RenderedPage]` (pass it in from `render.rs`, do not
  re-walk).
- Debug overlay: merge via `zip(walk_pages(debug()))` in `render.rs` (step 8), so the
  writer just receives fully-populated pages.
- For each page:

```rust
let canvas = SvgCanvas::new((page.origin.x, page.origin.y, page.width, page.height));
let svg = CanvasPainter::new(canvas).paint(&page.elements);
let path = page_path(out, page_number); // "score.svg" + 1 -> "score-p1.svg"
fs::write(&path, svg).unwrap();
```

- Add `fn page_path(out: &str, n: usize) -> PathBuf` splitting `out` into stem +
  extension and inserting `-p{n}` before the extension. `n` is 1-based; use the page's
  own `number` if `RenderedPage` exposes it, otherwise the enumerate index + 1.
  (`RenderedPage` does not currently carry `number`; either add it — `Page::number`
  exists — or use the loop index. Adding it to `RenderedPage` is cleaner and cheap.)
- Print each written path (keep the existing "Written to:" style, one line per file).

### Step 7 — CLI PDF writer: accept `&[RenderedPage]`

File: `cli/src/commands/render/pdf.rs`

- Move the `RenderCompositor::base().walk_pages(...)` + debug-overlay merge out of
  `write()`; take `pages: &[RenderedPage]` in.
- Font-set construction (`fontdb`, system fonts, `EmbeddedFont`, `FontSet`),
  `pt_per_tenth`, the per-page `PdfPageCanvas` loop, and `write_pdf` stitching all
  stay exactly as they are — that is legitimately PDF-specific.

### Step 8 — CLI `render.rs`: flatten the branch

File: `cli/src/commands/render.rs`

- Drop the `(args, pdf)` bool tuple and the `if pdf { … } else { … }` at the bottom.
- After `engrave`, walk once:

```rust
let compositor = RenderCompositor::base();
let mut pages = compositor.walk_pages(&visual, &fonts_for_walk);
if debug {
    for (p, d) in pages.iter_mut().zip(RenderCompositor::debug().walk_pages(&visual, &fonts_for_walk)) {
        p.elements.extend(d.elements);
    }
}
match format {
    RenderCommand::Svg(_) => svg::write(&pages, /* title/lyric font, out */),
    RenderCommand::Pdf(_) => pdf::write(&pages, &font, title_font, lyric_font, &layout.defaults, &out),
}
```

- `RenderFonts` is needed by both `walk_pages` and the writers. `svg::write` currently
  builds its own `RenderFonts`; `pdf::write` builds `RenderFonts` *and* the fontdb
  `EmbeddedFont`s. Decide where `RenderFonts::create` lives — simplest is to build it
  once in `render.rs` and pass `&fonts` to both `walk_pages` and the writers, leaving
  only the PDF-specific fontdb/embedding in `pdf::write`. Watch the borrow: `pages`
  borrows from `fonts`, so `fonts` must outlive both writer calls (it already does in
  `pdf.rs`'s current structure).
- Keep the per-format `RenderArgs` structs and the `RenderCommand` enum as they are —
  they exist so svg-only / pdf-only options can diverge later.

### Step 9 — Tests

Rust tests live in the `tests` crate (`tests/src/`), not inline in `lib`.

- `tests/src/test_canvas.rs`
  - `svg_canvas_emits_a_document_for_every_shape` (line ~133): update
    `SvgCanvas::new()` → `SvgCanvas::new((0.0, 0.0, 10.0, 5.0))`; the `viewBox="0 0 10 5"`
    assertion still holds.
  - Add a test that a different `view_box` (e.g. `(1400.0, 0.0, 1360.0, 1760.0)`) is
    emitted verbatim regardless of element positions.
  - Add a `paint_pages` test with a `RecordingCanvas` that also records `begin_page`,
    asserting order: `begin` → `begin_page` → elements → `begin_page` → elements → `finish`.

- `tests/src/test_flat_buffer.rs`
  - Add a test: two pages through `CanvasPainter::paint_pages` produce a `page_table`
    of `[0, ox0, oy0, w0, h0, split, ox1, oy1, w1, h1]` where `split` is the f32 length
    of page 0's records, and `geometry` is the concatenation in page order.
  - Existing single-surface `paint` tests are unchanged.

- `tests/src/test_pdf.rs`
  - `walk_pages_keeps_one_bundle_per_page_while_walk_flattens` (line ~271): drop the
    `compositor.walk(&score, &fonts)` half and its assertion; rename to
    `walk_pages_keeps_one_bundle_per_page`.

- CLI path derivation: add a unit test for `page_path` (`"score.svg"`, `3` →
  `"score-p3.svg"`; `"out/score.svg"`, `1` → `"out/score-p1.svg"`; no extension →
  append `-p{n}` then `.svg` or document that an extension is required).

- Add a small end-to-end-ish test if feasible: `render svg` over a known 2-page
  fixture writes two files. Use a MusicXML fixture already sanctioned for tests — see
  the repo's test-fixtures notes; do not introduce a new sample file.

### Step 10 — Docs

- `lib/src/drawable/canvas/pdf/mod.rs` — the "# Why this canvas is shaped differently
  from the SVG / flat-buffer ones" section: SVG is now also per-page. Reword so the
  remaining real difference is PDF's shared font embedding + the `write_pdf` stitch
  step, not pagination.
- `cli/src/commands/render.rs` — the `RenderCommand` doc comment and the `Svg` /
  `Pdf` variant docs: describe per-page SVG output and the `-p{n}` filename scheme.
- `web/music-xml.js` — the "Keep in sync with lib/src/drawable/canvas/flat_buffer"
  comment near the tag constants: mention `page_table` and its layout, even though JS
  does not consume it yet.
- `lib/src/drawable/canvas.rs` — doc the new `begin_page` hook and `paint_pages` on
  the `Canvas` / `CanvasPainter` doc comments.

---

## Verification

- `cargo build --workspace`
- `cargo test --workspace` (the `tests` crate is where the assertions live)
- `cargo clippy --workspace --all-targets`
- `scripts/build-wasm.sh` (the wasm crate must still compile to the `wasm32` target)
- Manual: `opus render svg --file <fixture> --out /tmp/x.svg …` produces
  `/tmp/x-p1.svg` … and `opus render pdf …` is byte-compatible with `main` for a
  single-page score (diffing the content stream, modulo any intentional change).
  The user does their own visual verification — do not launch web/dev servers.

## Risks / call-outs

- **Only externally visible change:** SVG output filenames. Gated on the open decision
  above.
- **wasm `render_scale`:** must scale page `origin`/`width`/`height` alongside the
  elements, or the page table will disagree with the geometry.
- **Borrow lifetimes in `render.rs`:** `Vec<RenderedPage>` borrows from `RenderFonts`;
  keep `fonts` alive across the `match`.
- **`RenderedPage` gaining a `number` field** (optional, for `page_path`): touches the
  `walk_pages` constructor and the existing `test_pdf.rs` assertions on `pages[i]`.
- **`compute_bounds` over `&[&DrawableElement]`:** add a variant or flatten; don't
  regress the single-surface `paint` path.
