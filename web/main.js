import init, { render_score } from "../wasm/pkg/wasm.js";

const statusEl = document.getElementById("status");
const outputEl = document.getElementById("output");
const renderButton = document.getElementById("render-button");
const form = document.getElementById("render-form");

let metaJson;
let glyphNamesJson;

function setStatus(message) {
  statusEl.textContent = message ?? "";
}

async function bootstrap() {
  await init();

  [metaJson, glyphNamesJson] = await Promise.all([
    fetch("../smufl/bravura-bravura-1.392/redist/bravura_metadata.json").then((r) => r.text()),
    fetch("../smufl/metadata/glyphnames.json").then((r) => r.text()),
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

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  setStatus("");
  outputEl.innerHTML = "";

  const file = document.getElementById("musicxml-file").files[0];
  if (!file) {
    setStatus("Choose a MusicXML file first.");
    return;
  }

  try {
    const musicxml = await file.text();

    const svg = render_score(
      musicxml,
      metaJson,
      glyphNamesJson,
      document.getElementById("debug").checked,
      optionalText("page-color"),
      optionalText("foreground-color"),
      optionalText("page-orientation"),
      optionalNumber("horizontal-gutter-even"),
      optionalNumber("horizontal-gutter-uneven"),
      optionalNumber("vertical-gutter"),
    );

    outputEl.innerHTML = svg;
  } catch (err) {
    setStatus(String(err));
  }
});

bootstrap().catch((err) => {
  setStatus(`Failed to load wasm module: ${err}`);
});
