import init, { load_score, render } from "../wasm/pkg/wasm.js";

// Keep in sync with lib/src/drawable/flat_buffer.rs.
const TAG_LINE = 0;
const TAG_RECT = 1;
const TAG_TEXT = 2;
const TAG_POLYGON = 3;
const TEXT_DELIMITER = "";
const H_ALIGN = ["left", "center", "right"];
const V_ALIGN = ["hanging", "middle", "alphabetic"];

const statusEl = document.getElementById("status");
const canvas = document.getElementById("score-canvas");
const ctx = canvas.getContext("2d");
const renderButton = document.getElementById("render-button");
const form = document.getElementById("render-form");

let metaJson;
let glyphNamesJson;
let musicxmlText;
let scoreLoaded = false;

function setStatus(message) {
  statusEl.textContent = message ?? "";
}

async function loadBravuraFont() {
  // Canvas text rasterizes synchronously at fillText() time and never repaints
  // once a font arrives later, unlike SVG/DOM text. It also won't lazy-load a
  // passive @font-face rule, since no DOM text is ever set in this font. So the
  // font has to be force-loaded up front via the Font Loading API.
  const font = new FontFace(
    "Bravura",
    'url("../smufl/bravura-bravura-1.392/redist/woff/Bravura.woff2") format("woff2"), ' +
      'url("../smufl/bravura-bravura-1.392/redist/woff/Bravura.woff") format("woff")',
  );
  await font.load();
  document.fonts.add(font);
}

async function bootstrap() {
  await Promise.all([
    init(),
    loadBravuraFont(),
    fetch("../smufl/bravura-bravura-1.392/redist/bravura_metadata.json")
      .then((r) => r.text())
      .then((text) => {
        metaJson = text;
      }),
    fetch("../smufl/metadata/glyphnames.json")
      .then((r) => r.text())
      .then((text) => {
        glyphNamesJson = text;
      }),
  ]);

  renderButton.disabled = false;
  renderButton.textContent = "Render";
}

function optionalText(id) {
  const value = document.getElementById(id).value.trim();
  return value === "" ? undefined : value;
}

function optionalNumber(id) {
  const value = document.getElementById(id).value.trim();
  return value === "" ? undefined : Number(value);
}

function rgba(r, g, b, a) {
  return `rgba(${r}, ${g}, ${b}, ${a})`;
}

// Assigning to these ctx properties is expensive even when the new value
// equals the current one (ctx.font in particular forces font re-resolution),
// and adjacent drawable records very often share style with their neighbor.
// Comparing the raw numeric components (rather than building an rgba()/font
// string first and comparing strings) avoids allocating and immediately
// discarding a throwaway string for every element that didn't change -
// that churn was enough to trigger unpredictable GC pauses given how many
// thousands of elements a score can have.
let currentFillR, currentFillG, currentFillB, currentFillA;
let currentStrokeR, currentStrokeG, currentStrokeB, currentStrokeA;
let currentLineWidth;
let currentFontSize;
let currentTextAlign;
let currentTextBaseline;

function setFillStyle(r, g, b, a) {
  if (currentFillR !== r || currentFillG !== g || currentFillB !== b || currentFillA !== a) {
    ctx.fillStyle = rgba(r, g, b, a);
    currentFillR = r;
    currentFillG = g;
    currentFillB = b;
    currentFillA = a;
  }
}

function setStrokeStyle(r, g, b, a) {
  if (
    currentStrokeR !== r ||
    currentStrokeG !== g ||
    currentStrokeB !== b ||
    currentStrokeA !== a
  ) {
    ctx.strokeStyle = rgba(r, g, b, a);
    currentStrokeR = r;
    currentStrokeG = g;
    currentStrokeB = b;
    currentStrokeA = a;
  }
}

function setLineWidth(width) {
  if (currentLineWidth !== width) {
    ctx.lineWidth = width;
    currentLineWidth = width;
  }
}

function setFont(fontSize) {
  if (currentFontSize !== fontSize) {
    ctx.font = `${fontSize}px Bravura`;
    currentFontSize = fontSize;
  }
}

function setTextAlign(align) {
  if (currentTextAlign !== align) {
    ctx.textAlign = align;
    currentTextAlign = align;
  }
}

function setTextBaseline(baseline) {
  if (currentTextBaseline !== baseline) {
    ctx.textBaseline = baseline;
    currentTextBaseline = baseline;
  }
}

// Temporary profiling instrumentation (see the perf investigation in web/main.js
// history): logs a decode/draw timing breakdown to the console on every render
// so we can see where the 100ms-for-10k-items time actually goes before
// deciding between a glyph atlas, draw-call batching, etc. Safe to delete once
// that's resolved.
function draw(output) {
  const tDrawStart = performance.now();
  // Each property read below is a wasm-bindgen getter call that now takes
  // (mem::take) rather than clones the underlying buffer, so it must be read
  // exactly once - a second read would come back empty.
  const textBlob = output.text_blob;
  const texts = textBlob === "" ? [] : textBlob.split(TEXT_DELIMITER);
  const geometry = output.geometry;
  const tDecoded = performance.now();

  // The wasm side already scales elements down (see render()'s
  // device_pixel_ratio param in wasm/src/lib.rs) so that bounds_width/height
  // times dpr stays within its canvas pixel budget - this is a plain,
  // budget-agnostic consumer of whatever bounds it's given.
  const dpr = window.devicePixelRatio || 1;
  const width = output.bounds_width * dpr;
  const height = output.bounds_height * dpr;
  console.log(`[perf] canvas device pixels: ${width.toFixed(0)}x${height.toFixed(0)} (dpr=${dpr})`);
  if (canvas.width !== width || canvas.height !== height) {
    canvas.width = width;
    canvas.height = height;
    canvas.style.width = `${output.bounds_width}px`;
    canvas.style.height = `${output.bounds_height}px`;
    // Setting canvas.width/height resets the entire 2D context state
    // (fillStyle, font, ...) back to browser defaults, so the tracked
    // "current" values above would otherwise go stale.
    currentFillR = currentFillG = currentFillB = currentFillA = undefined;
    currentStrokeR = currentStrokeG = currentStrokeB = currentStrokeA = undefined;
    currentLineWidth = undefined;
    currentFontSize = undefined;
    currentTextAlign = undefined;
    currentTextBaseline = undefined;
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
  let lineRecordCount = 0;
  let lineStrokeCallCount = 0;

  function flushLine() {
    if (lineOpen) {
      ctx.stroke();
      lineStrokeCallCount++;
      lineOpen = false;
    }
  }

  while (i < geometry.length) {
    const tag = geometry[i++];

    if (tag !== TAG_LINE) {
      flushLine();
    }

    switch (tag) {
      case TAG_LINE: {
        lineRecordCount++;
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
          setStrokeStyle(r, g, b, a);
          setLineWidth(strokeWidth);
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

        setFillStyle(r, g, b, a);
        ctx.fillRect(x, y, w, h);
        if (strokeWidth >= 0) {
          setStrokeStyle(sr, sg, sb, sa);
          setLineWidth(strokeWidth);
          ctx.strokeRect(x, y, w, h);
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

        setFillStyle(r, g, b, a);
        setFont(fontSize);
        setTextAlign(H_ALIGN[hAlign]);
        setTextBaseline(V_ALIGN[vAlign]);
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
        setFillStyle(r, g, b, a);
        ctx.fill();
        if (strokeWidth >= 0) {
          setStrokeStyle(sr, sg, sb, sa);
          setLineWidth(strokeWidth);
          ctx.stroke();
        }
        break;
      }

      default:
        throw new Error(`unknown drawable tag ${tag}`);
    }
  }

  flushLine();

  const tDrawEnd = performance.now();
  console.log(
    `[perf] draw: split+fetch ${(tDecoded - tDrawStart).toFixed(2)}ms, ` +
      `decode+canvas ${(tDrawEnd - tDecoded).toFixed(2)}ms, ` +
      `total ${(tDrawEnd - tDrawStart).toFixed(2)}ms ` +
      `(${geometry.length} floats, ${texts.length} text records, ` +
      `${lineRecordCount} line records batched into ${lineStrokeCallCount} stroke() calls)`,
  );
}

function renderNow(showFileError) {
  setStatus("");

  if (!scoreLoaded) {
    if (showFileError) {
      setStatus("Choose a MusicXML file first.");
    }
    return;
  }

  let output;
  try {
    const tRenderStart = performance.now();
    output = render(
      document.getElementById("debug").checked,
      document.getElementById("page-color").value,
      document.getElementById("foreground-color").value,
      optionalText("page-orientation"),
      optionalNumber("horizontal-gutter-even"),
      optionalNumber("horizontal-gutter-uneven"),
      optionalNumber("vertical-gutter"),
      window.devicePixelRatio || 1,
    );
    const tRenderEnd = performance.now();
    console.log(`[perf] wasm render() (rust + marshalling): ${(tRenderEnd - tRenderStart).toFixed(2)}ms`);

    draw(output);

    // draw()'s own timer only covers building the display list - on an
    // accelerated 2D canvas, Chrome defers the actual rasterization/composite
    // to later, off that synchronous call stack (visible as "Composite" in
    // the DevTools timeline). A double rAF callback runs after the browser
    // has presented the frame, so this captures the time the user actually
    // sees, including that deferred raster/composite work.
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        const tPainted = performance.now();
        console.log(
          `[perf] paint-inclusive: ${(tPainted - tRenderStart).toFixed(2)}ms from render() start to ` +
            `actual on-screen paint`,
        );
      });
    });
  } catch (err) {
    setStatus(String(err));
  } finally {
    output?.free();
  }
}

let debounceTimer;
function scheduleRender() {
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => renderNow(false), 2);
}

form.addEventListener("submit", (event) => {
  event.preventDefault();
  renderNow(true);
});

document.getElementById("musicxml-file").addEventListener("change", async () => {
  const file = document.getElementById("musicxml-file").files[0];
  musicxmlText = file ? await file.text() : undefined;
  scoreLoaded = false;

  if (musicxmlText) {
    try {
      load_score(musicxmlText, metaJson, glyphNamesJson);
      scoreLoaded = true;
    } catch (err) {
      setStatus(String(err));
      return;
    }
  }

  renderNow(true);
});

const settingIds = [
  "page-color",
  "foreground-color",
  "page-orientation",
  "horizontal-gutter-even",
  "horizontal-gutter-uneven",
  "vertical-gutter",
  "debug",
];
for (const id of settingIds) {
  document.getElementById(id).addEventListener("input", scheduleRender);
}

bootstrap().catch((err) => {
  setStatus(`Failed to load wasm module: ${err}`);
});
