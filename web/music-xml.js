// <music-xml> - a self-contained web component that renders a MusicXML
// document, one <canvas> per page, stacked by CSS.
//
// Usage:
//   <script type="module" src="/path/to/music-xml.js"></script>
//   <music-xml file="score.musicxml" style="--page-color: #ffffff;"></music-xml>
//
// All asset paths (the wasm module, the Bravura font, SMuFL metadata) are
// resolved relative to *this file's own URL* via import.meta.url, not
// relative to the page that imports it - so the tag works the same way
// regardless of where it's dropped into a host page, as long as this file
// stays in place relative to ../wasm/pkg and ../assets/smufl.

// Keep in sync with lib/src/drawable/canvas/flat_buffer/mod.rs.
const TAG_LINE = 0;
const TAG_RECT = 1;
const TAG_TEXT = 2;
const TAG_POLYGON = 3;
const TAG_CIRCLE = 4;
const TAG_GLYPH = 5;
// `output.page_table` (from wasm) is a parallel table of 4 f32s per page -
// [geometryStartIndex, textStartIndex, width, height]. Every page is
// engraved page-local (there is no page origin any more: the engine no
// longer arranges pages relative to each other, this component does), so
// `width`/`height` in tenths is all a page needs to be painted on its own.
// `textStartIndex` is how many `text_blob` entries (split on
// TEXT_DELIMITER) precede this page's own: TAG_TEXT/TAG_GLYPH records pull
// from that blob in stream order, so painting a page in isolation means
// skipping that many entries first. See _decode, which slices both
// `geometry` and the decoded `texts` array per page from this table.
// U+001F UNIT SEPARATOR, matching TEXT_DELIMITER in
// lib/src/drawable/canvas/flat_buffer/mod.rs. Built from its code point in
// plain ASCII rather than written as a literal control character, which is
// invisible in every editor and diff and so does not reliably survive a file
// being rewritten. Losing it is silent but total: the separator becomes "",
// String.split("") splits into single characters, and from there every font
// family resolves to its own first letter (so Bravura falls back to serif and
// no notation renders) while every glyph draws the wrong code point.
const TEXT_DELIMITER = String.fromCharCode(31);
const H_ALIGN = ["left", "center", "right"];
const V_ALIGN = ["hanging", "middle", "alphabetic"];
// Bits in a `font_styles` entry (see the flat-buffer module).
const FONT_STYLE_BOLD = 1;
const FONT_STYLE_ITALIC = 2;
// Generic fallback appended after every resolved family.
const FONT_FALLBACK = "serif";

// Upper bound on one page canvas's physical pixel count (width * height in
// device pixels). Moved here from wasm/src/lib.rs's MAX_CANVAS_PIXELS: now
// that every page is engraved page-local, only the browser knows how large a
// page is actually displayed, so the budget belongs to the thing that decides
// the backing-store size. Chosen the same way it was there: roughly "one big
// native display's worth of pixels" - a page that already fits renders at
// full native sharpness, only an oversized one gets scaled down.
const MAX_CANVAS_PIXELS = 12_000_000;
// Per-side cap, roughly Safari's ~16,384px backing-store limit per axis. The
// area cap alone can miss a very tall or very wide page that stays under the
// pixel budget while still exceeding this - which used to be the ROADMAP §2
// blank-canvas bug. Per-page, the two caps together are cheap to state and
// pages never approach either.
const MAX_CANVAS_SIDE_PX = 16_384;

const WASM_JS_URL = new URL("../wasm/pkg/wasm.js", import.meta.url).href;
const BRAVURA_METADATA_URL = new URL(
  "../assets/smufl/bravura-bravura-1.392/redist/bravura_metadata.json",
  import.meta.url,
);
const GLYPHNAMES_URL = new URL("../assets/smufl/metadata/glyphnames.json", import.meta.url);
const BRAVURA_WOFF2_URL = new URL(
  "../assets/smufl/bravura-bravura-1.392/redist/woff/Bravura.woff2",
  import.meta.url,
);
const BRAVURA_WOFF_URL = new URL(
  "../assets/smufl/bravura-bravura-1.392/redist/woff/Bravura.woff",
  import.meta.url,
);

async function loadBravuraFont() {
  // Canvas text rasterizes synchronously at fillText() time and never
  // repaints once a font arrives later, unlike SVG/DOM text, so it has to be
  // force-loaded up front via the Font Loading API rather than left to a
  // passive @font-face rule.
  const font = new FontFace(
    "Bravura",
    `url("${BRAVURA_WOFF2_URL}") format("woff2"), url("${BRAVURA_WOFF_URL}") format("woff")`,
  );
  await font.load();
  document.fonts.add(font);
}

async function bootstrap() {
  // WASM_JS_URL is a runtime-computed URL, not a literal specifier, so
  // bundlers that try to statically analyze dynamic import() (e.g. Vite)
  // can't resolve it - tell them to leave it alone rather than warn/fail.
  const wasmPromise = import(/* @vite-ignore */ WASM_JS_URL);
  const metaJsonPromise = fetch(BRAVURA_METADATA_URL).then((r) => r.text());
  const glyphNamesJsonPromise = fetch(GLYPHNAMES_URL).then((r) => r.text());
  const fontPromise = loadBravuraFont();

  const wasm = await wasmPromise;
  await wasm.default();
  const [metaJson, glyphNamesJson] = await Promise.all([metaJsonPromise, glyphNamesJsonPromise]);
  await fontPromise;

  return { wasm, metaJson, glyphNamesJson };
}

// Shared across every <music-xml> instance on the page: the wasm module, the
// Bravura font, and the SMuFL metadata are all instance-independent, so
// there's no reason for a second element to re-fetch or re-instantiate them.
// Reset to undefined on failure so a later load can retry (e.g. after a
// transient network error) instead of every element being stuck forever.
let bootstrapPromise;
function ensureBootstrapped() {
  if (!bootstrapPromise) {
    bootstrapPromise = bootstrap().catch((err) => {
      bootstrapPromise = undefined;
      throw err;
    });
  }
  return bootstrapPromise;
}

function rgba(r, g, b, a) {
  return `rgba(${r}, ${g}, ${b}, ${a})`;
}

// Every layout option the wasm `Score.render()` accepts, as its path into the
// nested `layout` object (snake_case, matching Rust's UserLayout: the group,
// then the field). Each one is read from the CSS custom property spelled as
// that path in kebab-case: `tie.height_max` <- `--tie-height-max` - the same
// spelling as the CLI flag. Exposing a new UserLayout knob is a matter of
// adding its path here; the value's type is worked out at read time, so there
// is nothing else on this side to keep in step.
//
// Page arrangement (`--page-orientation`) is deliberately not here: it is a
// component-level CSS knob this element reads separately in the paint path
// (see _applyPageOrientation and the `.pages` stylesheet rule below), not a
// `UserLayout` field - the engine no longer arranges pages relative to each
// other, so there is nothing on the Rust side left for it to configure. The
// gap between pages is a fixed 10px in that same stylesheet, not a knob.
//
// A name that no UserLayout field matches is silently ignored rather than
// reported (serde-wasm-bindgen only looks up the fields it expects), so a typo
// here shows up as an option that quietly does nothing rather than an error -
// worth a look at the exhibition site's render page after adding one
// (site/render/index.html, built by scripts/build-site.sh).
const LAYOUT_OPTIONS = [
  "page.color",
  "foreground.color",
  "staff.line_width",
  "barline.light",
  "barline.heavy",
  "beam.thickness",
  "beam.spacing",
  "stem.thickness",
  "note_size.grace",
  "note_size.cue",
  "dot.radius",
  "dot.spacing",
  "measure_start.clef_padding",
  "measure_start.key_signature_padding",
  "measure_start.time_signature_padding",
  "tie.endpoint_thickness",
  "tie.midpoint_thickness",
  "tie.height_ratio",
  "tie.height_min",
  "tie.height_max",
  "tie.note_gap",
  "tie.vertical_offset",
  "tie.break_inset",
  "tie.break_fragment",
  "section.symbol",
  "section.symbol_gap",
  "part_group.symbol",
  "part_group.symbol_gap",
  "part.symbol",
  "part.symbol_gap",
  "group_bracket.thickness",
  "group_line.thickness",
  "group_square.thickness",
  "group_square.arm",
  "group_name.font",
  "group_name.size",
  "group_name.padding",
  "title.font",
  "lyric.font",
];

function cssPropertyFor(option) {
  return option.replace(/[._]/g, "-");
}

// These are CSS custom properties only (`--page-color`, etc. - via an inline
// `style="--page-color: ..."`, a class, or a plain stylesheet rule targeting
// the tag), not HTML attributes - see `_renderOptions`. `file` and `debug`
// are the only real HTML attributes this element has.
const OBSERVED_ATTRIBUTES = [
  "file",
  "debug",
  // "style" is observed (rather than each --custom-property individually,
  // which isn't possible - attributeChangedCallback only fires for real
  // attributes) so that `el.style.setProperty(...)` or a `style="..."` edit
  // re-renders. It does NOT catch custom properties changing via an
  // external stylesheet rule or a class toggle, since neither touches this
  // element's own `style` attribute - call `.refresh()` after those.
  "style",
];

export class MusicXmlElement extends HTMLElement {
  static get observedAttributes() {
    return OBSERVED_ATTRIBUTES;
  }

  constructor() {
    super();

    const shadow = this.attachShadow({ mode: "open" });
    shadow.innerHTML = `
      <style>
        :host { display: block; }
        /* Pages stack along --page-orientation (vertical -> column, the
           default; horizontal -> row - see _applyPageOrientation), separated
           by a fixed 10px gap. Deliberately not configurable: the gap is
           chrome between pages, not a property of the engraving, and the
           tenth-valued gutters the engine used to arrange with are gone. */
        .pages {
          display: flex;
          flex-direction: column;
          gap: 10px;
        }
        /* width:100% + height:auto lets each canvas fill its container width
           while the browser derives the displayed height from the CSS
           aspect-ratio _reconcilePages sets per page (the page's own tenths
           width/height ratio) - so it scales without distorting, the same
           way a plain <img> does, even while the canvas is unpainted
           (width/height attributes both 0) between IntersectionObserver
           visits.

           flex: 0 0 auto is load-bearing in row mode. A flex item's default
           min-width is auto, which for a canvas resolves to its backing
           store's width: a painted page refuses to shrink while its unpainted
           neighbours collapse to nothing, so only the first page stays
           visible - and being zero-wide, the others never intersect enough to
           get painted, which makes the collapse self-sustaining. */
        .pages canvas { display: block; width: 100%; height: auto; flex: 0 0 auto; }
        /* Row mode: one page per container width, scrolled horizontally,
           rather than N pages crushed side by side into one screen. */
        .pages[data-flow="row"] { overflow-x: auto; align-items: flex-start; }
        .pages[data-flow="row"] canvas { flex: 0 0 100%; }
        .status {
          font: 0.85rem/1.4 -apple-system, BlinkMacSystemFont, sans-serif;
          color: #a00;
          white-space: pre-wrap;
          margin: 0 0 0.5rem;
        }
        .status:empty { display: none; }
      </style>
      <div part="status" class="status"></div>
      <div part="pages" class="pages"></div>
    `;
    this._statusEl = shadow.querySelector(".status");
    this._pagesEl = shadow.querySelector(".pages");

    // The wasm `Score` for this element's document, or undefined when nothing
    // is loaded. Each element owns its own - the wasm module is shared, the
    // engraved score is not - so there is no handle or registry to track.
    this._score = undefined;
    this._loadSeq = 0;
    this._renderDebounce = undefined;
    this._connected = false;

    // One entry per laid-out page, parallel to the decoded page metadata in
    // `_pages` (see _decode / _reconcilePages): `{ el, ctx, style, painted,
    // intersecting, backingWidth, backingHeight }`. `style` is this page's
    // own 2D-context style cache - each canvas has its own context state, so
    // this can't be shared across pages the way a single-canvas cache could.
    this._pageEls = [];
    this._pageIndexByEl = new Map();
    // Decoded page metadata: `{ geomStart, geomEnd, textStart, textEnd,
    // width, height }` per page, set by _decode. `_geometry`/`_texts`/
    // `_fontFamilies`/`_fontStyles` are the whole score's flat buffers those
    // records slice into.
    this._pages = [];
    this._geometry = undefined;
    this._texts = undefined;
    this._fontFamilies = undefined;
    this._fontStyles = undefined;

    // Painted visible-first: a page's canvas gets its backing store only
    // while it (or its ~1-viewport margin) is on screen, and gives it back
    // when it scrolls away - seventeen full-resolution pages held at once
    // would blow well past any reasonable memory budget. rootMargin covers
    // roughly one viewport on every side so a page already has pixels by the
    // time it's actually visible.
    this._pageObserver = new IntersectionObserver((entries) => this._onPageIntersect(entries), {
      rootMargin: "100% 100% 100% 100%",
    });
    // Repaints a visible page at its new displayed size - a window resize,
    // the host page's zoom slider, or a --page-orientation flip that changes
    // how wide a page is laid out. Never re-enters wasm: it redraws from the
    // buffers _decode already cached, which is what makes the zoom slider
    // sharpen the notation instead of upscaling stale pixels.
    this._resizeObserver = new ResizeObserver((entries) => this._onPageResize(entries));
    this._pendingResizeTargets = new Set();
    this._resizeRaf = undefined;

    // The JSON snapshot of the options used for the most recent completed
    // render, and (between _scheduleRender building them and _render
    // consuming them) the not-yet-rendered options a debounced render is
    // waiting on. See _renderOptions / _scheduleRender / _render.
    this._lastOptionsJson = undefined;
    this._pendingOptions = undefined;
    // JSON snapshot of the options that last caused a wasm trap (a Rust
    // panic), or undefined if none has. See _render's catch and
    // _scheduleRender's recovery branch.
    this._failedOptionsJson = undefined;
  }

  connectedCallback() {
    this._connected = true;
    this._applyPageOrientation();
    if (this.hasAttribute("file")) {
      this._loadFile();
    }
  }

  disconnectedCallback() {
    this._connected = false;
    this._loadSeq++; // invalidates any fetch/bootstrap still in flight
    clearTimeout(this._renderDebounce);
    if (this._resizeRaf !== undefined) {
      cancelAnimationFrame(this._resizeRaf);
      this._resizeRaf = undefined;
    }
    this._freeScore();
    this._clearPages();
  }

  attributeChangedCallback(name, oldValue, newValue) {
    // Attributes present at parse time fire this once during upgrade, before
    // connectedCallback - skip that call so the initial load only happens
    // once, from connectedCallback's explicit check.
    if (!this._connected || oldValue === newValue) return;

    if (name === "file") {
      this._loadFile();
    } else {
      // --page-orientation is component-level, not a UserLayout field, so a
      // style-only change applies it directly here - a pure CSS re-layout,
      // never a reason to re-enter wasm. _scheduleRender still runs after in
      // case a real UserLayout knob also changed.
      this._applyPageOrientation();
      this._scheduleRender();
    }
  }

  // Translates `--page-orientation: vertical | horizontal` (the same spelling
  // a CLI flag or a `<group-symbol>` would use, kept for the exhibition
  // site's existing `music-xml { --page-orientation: vertical; }` rule) into
  // the `.pages` container's flex-direction. Anything other than exactly
  // "horizontal" - unset included - leaves the stylesheet's own `column`
  // default in place.
  // Drives the `.pages` flex direction from --page-orientation. Sets a
  // `data-flow` attribute rather than flexDirection directly, because row
  // mode needs more than the direction: the stylesheet keys its sizing and
  // overflow rules off the same attribute.
  _applyPageOrientation() {
    const orientation = getComputedStyle(this).getPropertyValue("--page-orientation").trim();
    const row = orientation === "horizontal";
    this._pagesEl.style.flexDirection = row ? "row" : "";
    if (row) {
      this._pagesEl.dataset.flow = "row";
    } else {
      delete this._pagesEl.dataset.flow;
    }
  }

  async _loadFile() {
    const seq = ++this._loadSeq;
    const fileUrl = this.getAttribute("file");

    this._freeScore();
    this._clearPages();

    if (!fileUrl) {
      this._setStatus("");
      return;
    }

    this._setStatus("Loading…");

    try {
      const { wasm, metaJson, glyphNamesJson } = await ensureBootstrapped();
      if (seq !== this._loadSeq) return;

      const response = await fetch(fileUrl);
      if (!response.ok) {
        throw new Error(`failed to fetch "${fileUrl}": ${response.status} ${response.statusText}`);
      }
      const musicxml = await response.text();
      if (seq !== this._loadSeq) return;

      this._score = new wasm.Score(musicxml, metaJson, glyphNamesJson);
      this._setStatus("");
      this._render();
      this.dispatchEvent(new CustomEvent("load"));
    } catch (err) {
      if (seq !== this._loadSeq) return;
      this._setStatus(String(err));
      this.dispatchEvent(new CustomEvent("error", { detail: err }));
    }
  }

  // Releases the score's wasm memory. Nothing else does: wasm objects are only
  // finalized on GC, if ever, so an element that goes away without this leaks a
  // whole engraved score.
  _freeScore() {
    try {
      this._score?.free();
    } catch {
      // Already trapped (see _render's catch) - the leak beats a second throw.
    }
    this._score = undefined;
  }

  // Tears down every page canvas: unobserves it from both observers, drops it
  // from the DOM, and forgets the decoded buffers it was painted from.
  _clearPages() {
    for (const pageEl of this._pageEls) {
      this._pageObserver.unobserve(pageEl.el);
      this._resizeObserver.unobserve(pageEl.el);
      this._pageIndexByEl.delete(pageEl.el);
      pageEl.el.remove();
    }
    this._pageEls = [];
    this._pendingResizeTargets.clear();
    this._pages = [];
    this._geometry = undefined;
    this._texts = undefined;
    this._fontFamilies = undefined;
    this._fontStyles = undefined;
    this._lastOptionsJson = undefined;
    this._failedOptionsJson = undefined;
  }

  _setStatus(message) {
    this._statusEl.textContent = message ?? "";
  }

  // The full render() options object: debug, and every LAYOUT_OPTIONS entry
  // this element actually sets - all read off this element's CSS custom
  // properties (a stylesheet rule, class, or inline `style="--page-color:
  // ..."`), via a single getComputedStyle() call reused for all lookups rather
  // than one call per property.
  _renderOptions() {
    const computed = getComputedStyle(this);
    const cssVar = (name) => {
      const value = computed.getPropertyValue(`--${name}`).trim();
      return value === "" ? undefined : value;
    };

    const layout = {};
    for (const option of LAYOUT_OPTIONS) {
      const value = cssVar(cssPropertyFor(option));
      if (value === undefined) continue;
      // CSS custom properties are always strings, but the Rust side wants a
      // number for the numeric knobs - so send a number whenever the value is
      // one. That keeps this loop from having to know which option is which:
      // "20" parses, "#ffffff", "bracket" and "serif" don't.
      const asNumber = Number(value);
      const [group, field] = option.split(".");
      layout[group] ??= {};
      layout[group][field] = Number.isFinite(asNumber) ? asNumber : value;
    }

    return {
      debug: this.hasAttribute("debug"),
      layout,
    };
  }

  // Public escape hatch: re-renders using the current attributes/CSS custom
  // properties, for the cases attributeChangedCallback can't observe on its
  // own (a custom property changing via an external stylesheet rule or an
  // ancestor's class toggle). Clears the memoized option snapshot first, so
  // it still forces a render even though nothing this element can see has
  // actually changed.
  refresh() {
    this._lastOptionsJson = undefined;
    this._applyPageOrientation();
    if (!this._score) {
      this._failedOptionsJson = undefined;
      this._loadFile();
      return;
    }
    this._render();
  }

  _scheduleRender() {
    const options = this._renderOptions();
    if (!this._score) {
      if (this._failedOptionsJson === undefined) return; // no file loaded
      if (JSON.stringify(options) === this._failedOptionsJson) return; // same failing set
      this._failedOptionsJson = undefined;
      this._loadFile(); // refetch + rebuild; ends in _render() with the new options
      return;
    }
    // A render this element has already produced pixel-for-identical output
    // for - most commonly a `style` mutation that doesn't touch any of the
    // CSS custom properties above (e.g. the zoom slider's width) - has
    // nothing to gain from re-entering wasm, so skip it.
    if (JSON.stringify(options) === this._lastOptionsJson) return;
    this._pendingOptions = options;
    // Coalesces bursts of attribute changes (e.g. a host page driving a
    // color picker's `input` event) into a single render.
    clearTimeout(this._renderDebounce);
    this._renderDebounce = setTimeout(() => this._render(), 2);
  }

  _render() {
    if (!this._score) return;

    // _scheduleRender already built these; a direct call (from _loadFile or
    // refresh()) has not, so build them now.
    const options = this._pendingOptions ?? this._renderOptions();
    this._pendingOptions = undefined;

    let output;
    try {
      output = this._score.render(options);
      this._decode(output);
      this._reconcilePages();
      this._repaintVisiblePages();
      this._setStatus("");
      this._lastOptionsJson = JSON.stringify(options);
    } catch (err) {
      this._setStatus(String(err));
      this.dispatchEvent(new CustomEvent("error", { detail: err }));
      // panic = "abort" on wasm32 means a Rust panic traps without unwinding, so
      // the RefMut wasm-bindgen takes around this &mut self export never drops
      // and the Score stays flagged as borrowed - every later render() on it
      // would throw "recursive use of an object detected" instead of the real
      // error. Drop it and let the next *different* option set rebuild a clean
      // one. WebAssembly.RuntimeError is what distinguishes a trap from an
      // ordinary Err(JsValue) (e.g. an unknown option or unparseable color),
      // which returns normally through the glue and leaves the Score usable.
      if (err instanceof WebAssembly.RuntimeError) {
        this._failedOptionsJson = JSON.stringify(options);
        this._freeScore();
      }
    } finally {
      output?.free();
    }
  }

  // Reads every RenderOutput getter exactly once (each is a wasm-bindgen
  // mem::take - a second read comes back empty) and caches the buffers, plus
  // a derived per-page slice table, on the element. Decode is deliberately
  // separate from painting: a page can be repainted from these cached buffers
  // (on scroll, on resize) without going back to wasm.
  _decode(output) {
    const textBlob = output.text_blob;
    const texts = textBlob === "" ? [] : textBlob.split(TEXT_DELIMITER);
    const geometry = output.geometry;
    const fontBlob = output.font_blob;
    const fontFamilies = fontBlob === "" ? [] : fontBlob.split(TEXT_DELIMITER);
    const fontStyles = output.font_styles;
    const pageTable = output.page_table;

    const pages = [];
    for (let i = 0; i < pageTable.length; i += 4) {
      const geomStart = pageTable[i];
      const textStart = pageTable[i + 1];
      const width = pageTable[i + 2];
      const height = pageTable[i + 3];
      const hasNext = i + 4 < pageTable.length;
      pages.push({
        geomStart,
        geomEnd: hasNext ? pageTable[i + 4] : geometry.length,
        textStart,
        textEnd: hasNext ? pageTable[i + 5] : texts.length,
        width,
        height,
      });
    }

    this._geometry = geometry;
    this._texts = texts;
    this._fontFamilies = fontFamilies;
    this._fontStyles = fontStyles;
    this._pages = pages;
  }

  // Adds/removes canvases so there is exactly one per decoded page, sizes
  // each one's CSS box via `aspect-ratio` so the layout is correct before
  // anything is rasterized, and marks every page unpainted - whatever a
  // reused canvas showed before belongs to geometry _decode just replaced.
  _reconcilePages() {
    const pages = this._pages;

    while (this._pageEls.length < pages.length) {
      const canvas = document.createElement("canvas");
      const index = this._pageEls.length;
      const pageEl = {
        el: canvas,
        ctx: canvas.getContext("2d"),
        style: this._blankStyle(),
        painted: false,
        intersecting: false,
        backingWidth: undefined,
        backingHeight: undefined,
      };
      this._pagesEl.appendChild(canvas);
      this._pageIndexByEl.set(canvas, index);
      this._pageObserver.observe(canvas);
      this._resizeObserver.observe(canvas);
      this._pageEls.push(pageEl);
    }

    while (this._pageEls.length > pages.length) {
      const pageEl = this._pageEls.pop();
      this._pageObserver.unobserve(pageEl.el);
      this._resizeObserver.unobserve(pageEl.el);
      this._pageIndexByEl.delete(pageEl.el);
      pageEl.el.remove();
    }

    for (let index = 0; index < pages.length; index++) {
      const page = pages[index];
      const pageEl = this._pageEls[index];
      pageEl.el.style.aspectRatio = `${page.width} / ${page.height}`;
      pageEl.painted = false;
      pageEl.backingWidth = undefined;
      pageEl.backingHeight = undefined;
    }
  }

  // Repaints every page the IntersectionObserver currently considers visible
  // (or near-visible, within its rootMargin). Called after _decode /
  // _reconcilePages, since a page already on screen won't get a fresh
  // IntersectionObserver callback on its own - only a visibility *change*
  // fires one - even though the geometry underneath it just changed.
  _repaintVisiblePages() {
    for (let index = 0; index < this._pageEls.length; index++) {
      if (this._pageEls[index].intersecting) {
        this._paintPage(index);
      }
    }
  }

  _onPageIntersect(entries) {
    for (const entry of entries) {
      const index = this._pageIndexByEl.get(entry.target);
      if (index === undefined) continue;

      const pageEl = this._pageEls[index];
      pageEl.intersecting = entry.isIntersecting;
      if (entry.isIntersecting) {
        this._paintPage(index);
      } else {
        this._releasePage(index);
      }
    }
  }

  // rAF-coalesced: a resize storm (dragging a splitter, a flex re-layout from
  // an orientation flip) can fire many ResizeObserver callbacks before the
  // next frame, and only the final size in each one matters.
  _onPageResize(entries) {
    for (const entry of entries) {
      this._pendingResizeTargets.add(entry.target);
    }
    if (this._resizeRaf !== undefined) return;

    this._resizeRaf = requestAnimationFrame(() => {
      this._resizeRaf = undefined;
      const targets = this._pendingResizeTargets;
      this._pendingResizeTargets = new Set();

      for (const target of targets) {
        const index = this._pageIndexByEl.get(target);
        if (index === undefined) continue;
        // Only a currently-visible page has anything to redraw; an
        // off-screen page is already released and repaints when it next
        // intersects, at whatever size is current by then.
        if (this._pageEls[index]?.intersecting) {
          this._paintPage(index);
        }
      }
    });
  }

  // Gives a released page's backing store back. The CSS aspect-ratio box set
  // by _reconcilePages holds its place in the stack either way, so nothing
  // about the surrounding layout moves.
  _releasePage(index) {
    const pageEl = this._pageEls[index];
    if (!pageEl) return;
    pageEl.el.width = 0;
    pageEl.el.height = 0;
    pageEl.painted = false;
    pageEl.backingWidth = undefined;
    pageEl.backingHeight = undefined;
  }

  // Sizes page `index`'s backing store to its current displayed width and
  // paints it from the buffers _decode cached - no wasm involved. A no-op if
  // the computed backing store would be unchanged from what's already there,
  // which is what lets a resize that doesn't actually change a page's pixel
  // size (most of them, in a page-per-canvas layout) skip repainting.
  _paintPage(index) {
    const page = this._pages[index];
    const pageEl = this._pageEls[index];
    if (!page || !pageEl) return;

    const canvas = pageEl.el;
    const cssWidth = canvas.clientWidth;
    if (cssWidth <= 0) return; // not laid out yet (e.g. a hidden ancestor)

    const dpr = window.devicePixelRatio || 1;
    let scale = (cssWidth * dpr) / page.width;
    // Two clamps replacing wasm's old render_scale, now evaluated per page
    // instead of once over every page's combined bounds: an area cap and a
    // per-side cap.
    const areaScale = Math.sqrt(MAX_CANVAS_PIXELS / (page.width * page.height));
    const sideScale = Math.min(MAX_CANVAS_SIDE_PX / page.width, MAX_CANVAS_SIDE_PX / page.height);
    scale = Math.min(scale, areaScale, sideScale);

    const backingWidth = Math.max(1, Math.round(page.width * scale));
    const backingHeight = Math.max(1, Math.round(page.height * scale));

    if (
      pageEl.painted &&
      pageEl.backingWidth === backingWidth &&
      pageEl.backingHeight === backingHeight
    ) {
      return;
    }

    canvas.width = backingWidth;
    canvas.height = backingHeight;
    pageEl.backingWidth = backingWidth;
    pageEl.backingHeight = backingHeight;
    // Setting canvas.width/height resets the entire 2D context state
    // (fillStyle, font, ...) back to browser defaults, so this page's cached
    // "current" style values would otherwise go stale.
    this._resetPageStyleCache(pageEl);

    const ctx = pageEl.ctx;
    ctx.setTransform(scale, 0, 0, scale, 0, 0);
    ctx.clearRect(0, 0, page.width, page.height);

    this._paintRecords(pageEl, page);
    pageEl.painted = true;
  }

  _blankStyle() {
    return {
      fillR: undefined,
      fillG: undefined,
      fillB: undefined,
      fillA: undefined,
      strokeR: undefined,
      strokeG: undefined,
      strokeB: undefined,
      strokeA: undefined,
      lineWidth: undefined,
      font: undefined,
      textAlign: undefined,
      textBaseline: undefined,
    };
  }

  _resetPageStyleCache(pageEl) {
    pageEl.style = this._blankStyle();
  }

  _setFillStyle(pageEl, r, g, b, a) {
    const s = pageEl.style;
    if (s.fillR !== r || s.fillG !== g || s.fillB !== b || s.fillA !== a) {
      pageEl.ctx.fillStyle = rgba(r, g, b, a);
      s.fillR = r;
      s.fillG = g;
      s.fillB = b;
      s.fillA = a;
    }
  }

  _setStrokeStyle(pageEl, r, g, b, a) {
    const s = pageEl.style;
    if (s.strokeR !== r || s.strokeG !== g || s.strokeB !== b || s.strokeA !== a) {
      pageEl.ctx.strokeStyle = rgba(r, g, b, a);
      s.strokeR = r;
      s.strokeG = g;
      s.strokeB = b;
      s.strokeA = a;
    }
  }

  _setLineWidth(pageEl, width) {
    if (pageEl.style.lineWidth !== width) {
      pageEl.ctx.lineWidth = width;
      pageEl.style.lineWidth = width;
    }
  }

  _setFont(pageEl, fontSize, family, styleFlags) {
    const prefix =
      (styleFlags & FONT_STYLE_ITALIC ? "italic " : "") +
      (styleFlags & FONT_STYLE_BOLD ? "bold " : "");
    const font = `${prefix}${fontSize}px ${family}, ${FONT_FALLBACK}`;
    if (pageEl.style.font !== font) {
      pageEl.ctx.font = font;
      pageEl.style.font = font;
    }
  }

  _setTextAlign(pageEl, align) {
    if (pageEl.style.textAlign !== align) {
      pageEl.ctx.textAlign = align;
      pageEl.style.textAlign = align;
    }
  }

  _setTextBaseline(pageEl, baseline) {
    if (pageEl.style.textBaseline !== baseline) {
      pageEl.ctx.textBaseline = baseline;
      pageEl.style.textBaseline = baseline;
    }
  }

  // The tag-record switch, bounded to one page's slice of the shared
  // `geometry` stream (`page.geomStart .. page.geomEnd`) and seeded with
  // `page.textStart` so TAG_TEXT/TAG_GLYPH pull the right strings out of the
  // shared `texts` array even though painting starts partway through it.
  _paintRecords(pageEl, page) {
    const ctx = pageEl.ctx;
    const geometry = this._geometry;
    const texts = this._texts;
    const fontFamilies = this._fontFamilies;
    const fontStyles = this._fontStyles;

    let i = page.geomStart;
    const end = page.geomEnd;
    let textIndex = page.textStart;

    // Adjacent TAG_LINE records overwhelmingly share the same stroke style
    // (five staff lines, a run of beam/ledger segments, ...), so instead of
    // beginPath()+stroke() per line - each stroke() is a real rasterization
    // pass - accumulate consecutive same-style segments into one path and
    // stroke it once. This only merges records that are already adjacent in
    // the stream, so draw order (and therefore z-stacking) is unchanged.
    let lineOpen = false;
    let lineR, lineG, lineB, lineA, lineWidth;

    const flushLine = () => {
      if (lineOpen) {
        ctx.stroke();
        lineOpen = false;
      }
    };

    while (i < end) {
      const tag = geometry[i++];

      if (tag !== TAG_LINE) {
        flushLine();
      }

      switch (tag) {
        case TAG_LINE: {
          const x1 = geometry[i++];
          const y1 = geometry[i++];
          const x2 = geometry[i++];
          const y2 = geometry[i++];
          const r = geometry[i++];
          const g = geometry[i++];
          const b = geometry[i++];
          const a = geometry[i++];
          const strokeWidth = geometry[i++];

          const styleChanged =
            !lineOpen ||
            lineR !== r ||
            lineG !== g ||
            lineB !== b ||
            lineA !== a ||
            lineWidth !== strokeWidth;

          if (styleChanged) {
            flushLine();
            this._setStrokeStyle(pageEl, r, g, b, a);
            this._setLineWidth(pageEl, strokeWidth);
            ctx.beginPath();
            lineOpen = true;
            lineR = r;
            lineG = g;
            lineB = b;
            lineA = a;
            lineWidth = strokeWidth;
          }
          ctx.moveTo(x1, y1);
          ctx.lineTo(x2, y2);
          break;
        }

        case TAG_RECT: {
          const x = geometry[i++];
          const y = geometry[i++];
          const w = geometry[i++];
          const h = geometry[i++];
          const r = geometry[i++];
          const g = geometry[i++];
          const b = geometry[i++];
          const a = geometry[i++];
          const strokeWidth = geometry[i++];
          const sr = geometry[i++];
          const sg = geometry[i++];
          const sb = geometry[i++];
          const sa = geometry[i++];

          this._setFillStyle(pageEl, r, g, b, a);
          ctx.fillRect(x, y, w, h);
          if (strokeWidth >= 0) {
            this._setStrokeStyle(pageEl, sr, sg, sb, sa);
            this._setLineWidth(pageEl, strokeWidth);
            ctx.strokeRect(x, y, w, h);
          }
          break;
        }

        case TAG_CIRCLE: {
          const x = geometry[i++];
          const y = geometry[i++];
          const radius = geometry[i++];
          const r = geometry[i++];
          const g = geometry[i++];
          const b = geometry[i++];
          const a = geometry[i++];
          const strokeWidth = geometry[i++];
          const sr = geometry[i++];
          const sg = geometry[i++];
          const sb = geometry[i++];
          const sa = geometry[i++];

          ctx.beginPath();
          ctx.arc(x, y, radius, 0, 2 * Math.PI);
          this._setFillStyle(pageEl, r, g, b, a);
          ctx.fill();
          if (strokeWidth >= 0) {
            this._setStrokeStyle(pageEl, sr, sg, sb, sa);
            this._setLineWidth(pageEl, strokeWidth);
            ctx.stroke();
          }
          break;
        }

        case TAG_TEXT: {
          const x = geometry[i++];
          const y = geometry[i++];
          const fontSize = geometry[i++];
          const r = geometry[i++];
          const g = geometry[i++];
          const b = geometry[i++];
          const a = geometry[i++];
          const hAlign = geometry[i++];
          const vAlign = geometry[i++];
          const fontIndex = geometry[i++];

          this._setFillStyle(pageEl, r, g, b, a);
          this._setFont(
            pageEl,
            fontSize,
            fontFamilies[fontIndex] ?? "Bravura",
            fontStyles[fontIndex] ?? 0,
          );
          this._setTextAlign(pageEl, H_ALIGN[hAlign]);
          this._setTextBaseline(pageEl, V_ALIGN[vAlign]);
          ctx.fillText(texts[textIndex++] ?? "", x, y);
          break;
        }

        case TAG_GLYPH: {
          const x = geometry[i++];
          const y = geometry[i++];
          const fontSize = geometry[i++];
          const r = geometry[i++];
          const g = geometry[i++];
          const b = geometry[i++];
          const a = geometry[i++];
          const fontIndex = geometry[i++];

          this._setFillStyle(pageEl, r, g, b, a);
          this._setFont(
            pageEl,
            fontSize,
            fontFamilies[fontIndex] ?? "Bravura",
            fontStyles[fontIndex] ?? 0,
          );
          // A glyph is placed on its own origin, so it always draws from the
          // left on the alphabetic baseline - no alignment to decode.
          this._setTextAlign(pageEl, "left");
          this._setTextBaseline(pageEl, "alphabetic");
          ctx.fillText(texts[textIndex++] ?? "", x, y);
          break;
        }

        case TAG_POLYGON: {
          const nPts = geometry[i++];
          const r = geometry[i++];
          const g = geometry[i++];
          const b = geometry[i++];
          const a = geometry[i++];
          const strokeWidth = geometry[i++];
          const sr = geometry[i++];
          const sg = geometry[i++];
          const sb = geometry[i++];
          const sa = geometry[i++];

          ctx.beginPath();
          for (let p = 0; p < nPts; p++) {
            const px = geometry[i++];
            const py = geometry[i++];
            if (p === 0) {
              ctx.moveTo(px, py);
            } else {
              ctx.lineTo(px, py);
            }
          }
          ctx.closePath();
          this._setFillStyle(pageEl, r, g, b, a);
          ctx.fill();
          if (strokeWidth >= 0) {
            this._setStrokeStyle(pageEl, sr, sg, sb, sa);
            this._setLineWidth(pageEl, strokeWidth);
            ctx.stroke();
          }
          break;
        }

        default:
          throw new Error(`unknown drawable tag ${tag}`);
      }
    }

    flushLine();
  }
}

if (!customElements.get("music-xml")) {
  customElements.define("music-xml", MusicXmlElement);
}
