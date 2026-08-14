import init, { render_elements } from "../wasm/pkg/wasm.js";

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

function draw(output) {
  const texts = output.text_blob === "" ? [] : output.text_blob.split(TEXT_DELIMITER);
  const geometry = output.geometry;

  const dpr = window.devicePixelRatio || 1;
  canvas.width = output.bounds_width * dpr;
  canvas.height = output.bounds_height * dpr;
  canvas.style.width = `${output.bounds_width}px`;
  canvas.style.height = `${output.bounds_height}px`;

  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, output.bounds_width, output.bounds_height);
  ctx.translate(-output.bounds_min_x, -output.bounds_min_y);

  let i = 0;
  let textIndex = 0;

  while (i < geometry.length) {
    const tag = geometry[i++];

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

        ctx.strokeStyle = rgba(r, g, b, a);
        ctx.lineWidth = strokeWidth;
        ctx.beginPath();
        ctx.moveTo(x1, y1);
        ctx.lineTo(x2, y2);
        ctx.stroke();
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

        ctx.fillStyle = rgba(r, g, b, a);
        ctx.fillRect(x, y, w, h);
        if (strokeWidth >= 0) {
          ctx.strokeStyle = rgba(sr, sg, sb, sa);
          ctx.lineWidth = strokeWidth;
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

        ctx.fillStyle = rgba(r, g, b, a);
        ctx.font = `${fontSize}px Bravura`;
        ctx.textAlign = H_ALIGN[hAlign];
        ctx.textBaseline = V_ALIGN[vAlign];
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
        ctx.fillStyle = rgba(r, g, b, a);
        ctx.fill();
        if (strokeWidth >= 0) {
          ctx.strokeStyle = rgba(sr, sg, sb, sa);
          ctx.lineWidth = strokeWidth;
          ctx.stroke();
        }
        break;
      }

      default:
        throw new Error(`unknown drawable tag ${tag}`);
    }
  }
}

function renderNow(showFileError) {
  setStatus("");

  if (!musicxmlText) {
    if (showFileError) {
      setStatus("Choose a MusicXML file first.");
    }
    return;
  }

  let output;
  try {
    output = render_elements(
      musicxmlText,
      metaJson,
      glyphNamesJson,
      document.getElementById("debug").checked,
      document.getElementById("page-color").value,
      document.getElementById("foreground-color").value,
      optionalText("page-orientation"),
      optionalNumber("horizontal-gutter-even"),
      optionalNumber("horizontal-gutter-uneven"),
      optionalNumber("vertical-gutter"),
    );

    draw(output);
  } catch (err) {
    setStatus(String(err));
  } finally {
    output?.free();
  }
}

let debounceTimer;
function scheduleRender() {
  clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => renderNow(false), 200);
}

form.addEventListener("submit", (event) => {
  event.preventDefault();
  renderNow(true);
});

document.getElementById("musicxml-file").addEventListener("change", async () => {
  const file = document.getElementById("musicxml-file").files[0];
  musicxmlText = file ? await file.text() : undefined;
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
