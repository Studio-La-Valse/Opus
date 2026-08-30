// <music-xml> - a self-contained web component that renders a MusicXML
// document to a canvas via the `wasm` crate.
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
const TEXT_DELIMITER = "";
const H_ALIGN = ["left", "center", "right"];
const V_ALIGN = ["hanging", "middle", "alphabetic"];
// Bits in a `font_styles` entry (see the flat-buffer module).
const FONT_STYLE_BOLD = 1;
const FONT_STYLE_ITALIC = 2;
// Generic fallback appended after every resolved family.
const FONT_FALLBACK = "serif";

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

// These are CSS custom properties only (`--page-color`, etc. - via an inline
// `style="--page-color: ..."`, a class, or a plain stylesheet rule targeting
// the tag), not HTML attributes - see `_cssVar` / `_cssNumberVar`. `file` and
// `debug` are the only real HTML attributes this element has.
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
        /* width:100% + height:auto (rather than a fixed pixel height) lets
           the canvas fill its container width while the browser derives the
           displayed height from the backing store's width/height *attribute*
           ratio (bounds_width*dpr : bounds_height*dpr, which reduces to the
           same ratio as the logical bounds) - so it scales without
           distorting, the same way a plain <img> does. */
        canvas { display: block; width: 100%; height: auto; }
        .status {
          font: 0.85rem/1.4 -apple-system, BlinkMacSystemFont, sans-serif;
          color: #a00;
          white-space: pre-wrap;
          margin: 0 0 0.5rem;
        }
        .status:empty { display: none; }
      </style>
      <div part="status" class="status"></div>
      <canvas part="canvas"></canvas>
    `;
    this._statusEl = shadow.querySelector(".status");
    this._canvas = shadow.querySelector("canvas");
    this._ctx = this._canvas.getContext("2d");

    this._wasm = undefined;
    this._handle = undefined;
    this._loadSeq = 0;
    this._renderDebounce = undefined;
    this._connected = false;
    this._resetStyleCache();
  }

  connectedCallback() {
    this._connected = true;
    if (this.hasAttribute("file")) {
      this._loadFile();
    }
  }

  disconnectedCallback() {
    this._connected = false;
    this._loadSeq++; // invalidates any fetch/bootstrap still in flight
    clearTimeout(this._renderDebounce);
    this._freeHandle();
  }

  attributeChangedCallback(name, oldValue, newValue) {
    // Attributes present at parse time fire this once during upgrade, before
    // connectedCallback - skip that call so the initial load only happens
    // once, from connectedCallback's explicit check.
    if (!this._connected || oldValue === newValue) return;

    if (name === "file") {
      this._loadFile();
    } else {
      this._scheduleRender();
    }
  }

  async _loadFile() {
    const seq = ++this._loadSeq;
    const fileUrl = this.getAttribute("file");

    this._freeHandle();
    this._clearCanvas();

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

      this._wasm = wasm;
      this._handle = wasm.load_score(musicxml, metaJson, glyphNamesJson);
      this._setStatus("");
      this._render();
      this.dispatchEvent(new CustomEvent("load"));
    } catch (err) {
      if (seq !== this._loadSeq) return;
      this._setStatus(String(err));
      this.dispatchEvent(new CustomEvent("error", { detail: err }));
    }
  }

  _freeHandle() {
    if (this._handle !== undefined && this._wasm) {
      this._wasm.free_score(this._handle);
    }
    this._handle = undefined;
  }

  _clearCanvas() {
    this._canvas.width = 0;
    this._canvas.height = 0;
    this._resetStyleCache();
  }

  _setStatus(message) {
    this._statusEl.textContent = message ?? "";
  }

  // Reads the CSS custom property `--${name}` off this element's own
  // computed style, so it picks up whatever a stylesheet rule, class, or
  // inline `style="--page-color: ..."` on the tag resolves to.
  _cssVar(name) {
    const value = getComputedStyle(this).getPropertyValue(`--${name}`).trim();
    return value === "" ? undefined : value;
  }

  _cssNumberVar(name) {
    const value = this._cssVar(name);
    return value === undefined ? undefined : Number(value);
  }

  // Public escape hatch: re-renders using the current attributes/CSS custom
  // properties, for the cases attributeChangedCallback can't observe on its
  // own (a custom property changing via an external stylesheet rule or an
  // ancestor's class toggle).
  refresh() {
    this._render();
  }

  _scheduleRender() {
    if (this._handle === undefined) return;
    // Coalesces bursts of attribute changes (e.g. a host page driving a
    // color picker's `input` event) into a single render.
    clearTimeout(this._renderDebounce);
    this._renderDebounce = setTimeout(() => this._render(), 2);
  }

  _render() {
    if (this._handle === undefined || !this._wasm) return;

    let output;
    try {
      output = this._wasm.render(
        this._handle,
        this.hasAttribute("debug"),
        this._cssVar("page-color"),
        this._cssVar("foreground-color"),
        this._cssVar("page-orientation"),
        this._cssNumberVar("horizontal-gutter-even"),
        this._cssNumberVar("horizontal-gutter-uneven"),
        this._cssNumberVar("vertical-gutter"),
        this._cssVar("title-font"),
        this._cssVar("lyric-font"),
        window.devicePixelRatio || 1,
      );
      this._draw(output);
      this._setStatus("");
    } catch (err) {
      this._setStatus(String(err));
      this.dispatchEvent(new CustomEvent("error", { detail: err }));
    } finally {
      output?.free();
    }
  }

  _resetStyleCache() {
    // Assigning to ctx properties is expensive even when the new value
    // equals the current one (ctx.font in particular forces font
    // re-resolution), and adjacent drawable records very often share style
    // with their neighbor, so every _setXStyle() below compares against
    // this cache first instead of writing unconditionally.
    this._style = {
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

  _setFillStyle(r, g, b, a) {
    const s = this._style;
    if (s.fillR !== r || s.fillG !== g || s.fillB !== b || s.fillA !== a) {
      this._ctx.fillStyle = rgba(r, g, b, a);
      s.fillR = r;
      s.fillG = g;
      s.fillB = b;
      s.fillA = a;
    }
  }

  _setStrokeStyle(r, g, b, a) {
    const s = this._style;
    if (s.strokeR !== r || s.strokeG !== g || s.strokeB !== b || s.strokeA !== a) {
      this._ctx.strokeStyle = rgba(r, g, b, a);
      s.strokeR = r;
      s.strokeG = g;
      s.strokeB = b;
      s.strokeA = a;
    }
  }

  _setLineWidth(width) {
    if (this._style.lineWidth !== width) {
      this._ctx.lineWidth = width;
      this._style.lineWidth = width;
    }
  }

  _setFont(fontSize, family, styleFlags) {
    const prefix =
      (styleFlags & FONT_STYLE_ITALIC ? "italic " : "") +
      (styleFlags & FONT_STYLE_BOLD ? "bold " : "");
    const font = `${prefix}${fontSize}px ${family}, ${FONT_FALLBACK}`;
    if (this._style.font !== font) {
      this._ctx.font = font;
      this._style.font = font;
    }
  }

  _setTextAlign(align) {
    if (this._style.textAlign !== align) {
      this._ctx.textAlign = align;
      this._style.textAlign = align;
    }
  }

  _setTextBaseline(baseline) {
    if (this._style.textBaseline !== baseline) {
      this._ctx.textBaseline = baseline;
      this._style.textBaseline = baseline;
    }
  }

  _draw(output) {
    const ctx = this._ctx;
    const canvas = this._canvas;

    // Each property read below is a wasm-bindgen getter call that takes
    // (mem::take) rather than clones the underlying buffer, so it must be
    // read exactly once - a second read would come back empty.
    const textBlob = output.text_blob;
    const texts = textBlob === "" ? [] : textBlob.split(TEXT_DELIMITER);
    const geometry = output.geometry;

    // Font table: families joined by TEXT_DELIMITER, parallel style-flag array.
    // A TAG_TEXT record's trailing fontIndex points into both.
    const fontBlob = output.font_blob;
    const fontFamilies = fontBlob === "" ? [] : fontBlob.split(TEXT_DELIMITER);
    const fontStyles = output.font_styles;

    // The wasm side already scales elements down (see render()'s
    // device_pixel_ratio param in wasm/src/lib.rs) so that bounds_width/height
    // times dpr stays within its canvas pixel budget - this is a plain,
    // budget-agnostic consumer of whatever bounds it's given.
    const dpr = window.devicePixelRatio || 1;
    const width = output.bounds_width * dpr;
    const height = output.bounds_height * dpr;
    if (canvas.width !== width || canvas.height !== height) {
      // Only the backing-store attributes are set here - the displayed size
      // comes from the CSS `width: 100%; height: auto;` rule above, which
      // derives its aspect ratio from these same attributes.
      canvas.width = width;
      canvas.height = height;
      // Setting canvas.width/height resets the entire 2D context state
      // (fillStyle, font, ...) back to browser defaults, so the cached
      // "current" style values above would otherwise go stale.
      this._resetStyleCache();
    }

    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, output.bounds_width, output.bounds_height);
    ctx.translate(-output.bounds_min_x, -output.bounds_min_y);

    let i = 0;
    let textIndex = 0;

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

    while (i < geometry.length) {
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
            this._setStrokeStyle(r, g, b, a);
            this._setLineWidth(strokeWidth);
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

          this._setFillStyle(r, g, b, a);
          ctx.fillRect(x, y, w, h);
          if (strokeWidth >= 0) {
            this._setStrokeStyle(sr, sg, sb, sa);
            this._setLineWidth(strokeWidth);
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
          this._setFillStyle(r, g, b, a);
          ctx.fill();
          if (strokeWidth >= 0) {
            this._setStrokeStyle(sr, sg, sb, sa);
            this._setLineWidth(strokeWidth);
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

          this._setFillStyle(r, g, b, a);
          this._setFont(
            fontSize,
            fontFamilies[fontIndex] ?? "Bravura",
            fontStyles[fontIndex] ?? 0,
          );
          this._setTextAlign(H_ALIGN[hAlign]);
          this._setTextBaseline(V_ALIGN[vAlign]);
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
          this._setFillStyle(r, g, b, a);
          ctx.fill();
          if (strokeWidth >= 0) {
            this._setStrokeStyle(sr, sg, sb, sa);
            this._setLineWidth(strokeWidth);
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
