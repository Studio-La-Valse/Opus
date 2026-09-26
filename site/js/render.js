// The render page: loads a sample from ?doc=<id>, builds the options pane
// from js/options.js, and wires each control to the <music-xml> element's
// CSS custom properties / attributes.
import "/web/music-xml.js";
import { OPTIONS, OPTION_GROUPS, cssPropertyFor } from "/js/options.js";

// Coalesces bursts of "input" events (a slider drag, a color picker drag) to
// one write per animation frame, always carrying the latest value, so
// dragging a slider across a 1.2 MB orchestral score can't queue a write per
// pixel but still updates continuously rather than only once the drag
// pauses. "change" (commit) writes through immediately - see each
// build*Control function below; <music-xml>'s own option memoization
// (web/music-xml.js) makes that immediate extra write free when it lands on
// the same value the last queued frame already wrote.
function rafThrottle(fn) {
  let queued = null;
  let frame = 0;
  return (...args) => {
    queued = args;
    if (frame) return;
    frame = requestAnimationFrame(() => {
      frame = 0;
      const latest = queued;
      queued = null;
      fn(...latest);
    });
  };
}

function formatBytes(bytes) {
  if (!Number.isFinite(bytes)) return "";
  if (bytes < 1024) return `${bytes} B`;
  const kb = bytes / 1024;
  if (kb < 1024) return `${kb.toFixed(1)} KB`;
  return `${(kb / 1024).toFixed(1)} MB`;
}

// A control at its default value writes nothing (removeProperty) so the
// engine default applies - keeping "unset" and "set to the default value"
// honestly distinct, since UserLayout's fields are all Option. `cssState`
// tracks exactly the non-default properties currently set, for "Copy CSS".
function applyCssVar(scoreEl, cssState, prop, value, isDefault) {
  if (isDefault) {
    scoreEl.style.removeProperty(`--${prop}`);
    cssState.delete(prop);
  } else {
    scoreEl.style.setProperty(`--${prop}`, value);
    cssState.set(prop, value);
  }
}

function optionShell(option) {
  const root = document.createElement("div");
  root.className = "option";

  const label = document.createElement("label");
  label.className = "option-label";
  label.textContent = option.label;

  const controls = document.createElement("div");
  controls.className = "option-controls";

  const help = document.createElement("p");
  help.className = "option-help";
  help.textContent = option.help;

  root.append(label, controls, help);
  return { root, controls };
}

function buildNumberControl(scoreEl, cssState, option) {
  const { root, controls } = optionShell(option);
  const prop = cssPropertyFor(option.name);

  const range = document.createElement("input");
  range.type = "range";
  range.min = String(option.min);
  range.max = String(option.max);
  range.step = String(option.step);
  range.value = String(option.default);
  range.setAttribute("aria-label", option.label);

  const number = document.createElement("input");
  number.type = "number";
  number.min = String(option.min);
  number.max = String(option.max);
  number.step = String(option.step);
  number.placeholder = String(option.default);

  const write = (raw) => {
    if (raw === "") {
      applyCssVar(scoreEl, cssState, prop, "", true);
      return;
    }
    const value = Number(raw);
    applyCssVar(scoreEl, cssState, prop, String(value), value === option.default);
  };
  const throttledWrite = rafThrottle(write);

  range.addEventListener("input", () => {
    number.value = range.value;
    throttledWrite(range.value);
  });
  range.addEventListener("change", () => write(range.value));

  number.addEventListener("input", () => {
    range.value = number.value === "" ? String(option.default) : number.value;
    throttledWrite(number.value);
  });
  number.addEventListener("change", () => write(number.value));

  const unit = document.createElement("span");
  unit.className = "option-unit";
  unit.textContent = option.unit ?? "";

  controls.append(range, number, unit);
  root.reset = () => {
    range.value = String(option.default);
    number.value = "";
    applyCssVar(scoreEl, cssState, prop, "", true);
  };
  return root;
}

function buildColorControl(scoreEl, cssState, option) {
  const { root, controls } = optionShell(option);
  const prop = cssPropertyFor(option.name);

  const swatch = document.createElement("input");
  swatch.type = "color";
  swatch.value = option.default;
  swatch.setAttribute("aria-label", `${option.label} colour`);

  const alpha = document.createElement("input");
  alpha.type = "range";
  alpha.min = "0";
  alpha.max = "100";
  alpha.value = "100";
  alpha.setAttribute("aria-label", `${option.label} alpha`);

  const alphaLabel = document.createElement("span");
  alphaLabel.className = "option-unit";
  alphaLabel.textContent = "100%";

  const currentValue = () => {
    const a = Number(alpha.value);
    alphaLabel.textContent = `${a}%`;
    if (a >= 100) return swatch.value;
    const alphaHex = Math.round((a / 100) * 255)
      .toString(16)
      .padStart(2, "0");
    return `${swatch.value}${alphaHex}`;
  };

  const write = () => {
    const value = currentValue();
    const isDefault = Number(alpha.value) >= 100 && value.toLowerCase() === option.default.toLowerCase();
    applyCssVar(scoreEl, cssState, prop, value, isDefault);
  };
  const throttledWrite = rafThrottle(write);

  swatch.addEventListener("input", () => {
    currentValue();
    throttledWrite();
  });
  swatch.addEventListener("change", write);
  alpha.addEventListener("input", () => {
    currentValue();
    throttledWrite();
  });
  alpha.addEventListener("change", write);

  controls.append(swatch, alpha, alphaLabel);
  root.reset = () => {
    swatch.value = option.default;
    alpha.value = "100";
    alphaLabel.textContent = "100%";
    applyCssVar(scoreEl, cssState, prop, "", true);
  };
  return root;
}

function buildEnumControl(scoreEl, cssState, option) {
  const { root, controls } = optionShell(option);
  const prop = cssPropertyFor(option.name);

  const select = document.createElement("select");
  for (const value of option.values) {
    const entry = document.createElement("option");
    entry.value = value;
    entry.textContent = value;
    select.append(entry);
  }
  select.value = option.default;

  const write = () => applyCssVar(scoreEl, cssState, prop, select.value, select.value === option.default);
  select.addEventListener("change", write);

  controls.append(select);
  root.reset = () => {
    select.value = option.default;
    applyCssVar(scoreEl, cssState, prop, "", true);
  };
  return root;
}

function buildFontControl(scoreEl, cssState, option) {
  const { root, controls } = optionShell(option);
  const prop = cssPropertyFor(option.name);

  const input = document.createElement("input");
  input.type = "text";
  input.placeholder = option.default;

  const write = () => {
    const value = input.value.trim();
    applyCssVar(scoreEl, cssState, prop, value, value === "");
  };
  const throttledWrite = rafThrottle(write);
  input.addEventListener("input", throttledWrite);
  input.addEventListener("change", write);

  controls.append(input);
  root.reset = () => {
    input.value = "";
    applyCssVar(scoreEl, cssState, prop, "", true);
  };
  return root;
}

function buildBooleanAttributeControl(scoreEl, option) {
  const { root, controls } = optionShell(option);

  const checkbox = document.createElement("input");
  checkbox.type = "checkbox";
  checkbox.checked = option.default;
  checkbox.addEventListener("change", () => scoreEl.toggleAttribute(option.name, checkbox.checked));

  controls.append(checkbox);
  root.reset = () => {
    checkbox.checked = option.default;
    scoreEl.toggleAttribute(option.name, option.default);
  };
  return root;
}

function buildEnumAttributeControl(scoreEl, option) {
  const { root, controls } = optionShell(option);

  const select = document.createElement("select");
  const unset = document.createElement("option");
  unset.value = "";
  unset.textContent = option.unsetLabel;
  select.append(unset);
  for (const value of option.values) {
    const entry = document.createElement("option");
    entry.value = value;
    entry.textContent = value;
    select.append(entry);
  }

  const write = () => {
    if (select.value === "") {
      scoreEl.removeAttribute(option.name);
    } else {
      scoreEl.setAttribute(option.name, select.value);
    }
  };
  select.addEventListener("change", write);

  controls.append(select);
  root.reset = () => {
    select.value = "";
    scoreEl.removeAttribute(option.name);
  };
  return root;
}

function buildControl(scoreEl, cssState, option) {
  switch (option.kind) {
    case "number":
      return buildNumberControl(scoreEl, cssState, option);
    case "color":
      return buildColorControl(scoreEl, cssState, option);
    case "enum":
      return buildEnumControl(scoreEl, cssState, option);
    case "font":
      return buildFontControl(scoreEl, cssState, option);
    case "boolean-attribute":
      return buildBooleanAttributeControl(scoreEl, option);
    case "enum-attribute":
      return buildEnumAttributeControl(scoreEl, option);
    default:
      throw new Error(`unknown option kind "${option.kind}"`);
  }
}

// Builds the whole pane by iterating OPTIONS - adding a knob later is one
// entry in options.js, nothing to touch here.
function buildPane(scoreEl, panelGroupsEl) {
  const cssState = new Map();
  const resets = [];

  const byGroup = new Map(OPTION_GROUPS.map((name) => [name, []]));
  for (const option of OPTIONS) byGroup.get(option.group).push(option);

  for (const groupName of OPTION_GROUPS) {
    const options = byGroup.get(groupName);
    if (!options || options.length === 0) continue;

    const section = document.createElement("section");
    section.className = "option-group";
    const heading = document.createElement("h2");
    heading.textContent = groupName;
    section.append(heading);

    for (const option of options) {
      const control = buildControl(scoreEl, cssState, option);
      resets.push(control.reset);
      section.append(control);
    }
    panelGroupsEl.append(section);
  }

  return { cssState, resets };
}

function copyCss(cssState) {
  const lines = [];
  for (const option of OPTIONS) {
    if (option.kind === "boolean-attribute" || option.kind === "enum-attribute") continue;
    const prop = cssPropertyFor(option.name);
    if (cssState.has(prop)) lines.push(`  --${prop}: ${cssState.get(prop)};`);
  }
  const block =
    lines.length > 0
      ? `music-xml {\n${lines.join("\n")}\n}`
      : "music-xml {\n  /* nothing set - every option is at its default */\n}";

  const copyStatus = document.getElementById("copy-status");
  navigator.clipboard.writeText(block).then(
    () => {
      copyStatus.textContent = "Copied";
      setTimeout(() => (copyStatus.textContent = ""), 1500);
    },
    () => {
      copyStatus.textContent = "Copy failed";
    },
  );
}

function wirePanelChrome(resets, cssState) {
  const tab = document.getElementById("panel-tab");
  const panel = document.getElementById("panel");

  tab.addEventListener("click", () => {
    const open = panel.classList.toggle("open");
    tab.setAttribute("aria-expanded", String(open));
    panel.setAttribute("aria-hidden", String(!open));
  });

  document.getElementById("reset-all").addEventListener("click", () => {
    for (const reset of resets) reset();
  });

  document.getElementById("copy-css").addEventListener("click", () => copyCss(cssState));
}

// Pure CSS: width on the element scales the canvas (which is
// width:100%/height:auto inside the component's shadow root). A block
// element's auto width already fills its container, so max-width could only
// ever shrink it - an explicit width is what lets it grow past 100% and
// overflow into .stage's own scrollbars. This mutates the element's `style`
// attribute, which the component does observe and re-evaluate - but its
// option memoization (web/music-xml.js) recognizes that no CSS custom
// property the component reads has actually changed, and skips the repaint
// that would otherwise imply.
function wireZoom(scoreEl) {
  const zoom = document.getElementById("zoom");
  const zoomValue = document.getElementById("zoom-value");
  const apply = () => {
    scoreEl.style.width = `${zoom.value}%`;
    zoomValue.textContent = `${zoom.value}%`;
  };
  zoom.addEventListener("input", apply);
  apply();
}

function showEmptyState(doc, samples) {
  const emptyState = document.getElementById("empty-state");
  const emptyDocName = document.getElementById("empty-doc-name");
  const emptySampleList = document.getElementById("empty-sample-list");

  emptyDocName.textContent = doc ?? "";
  emptySampleList.innerHTML = "";
  for (const sample of samples) {
    const item = document.createElement("li");
    const link = document.createElement("a");
    link.href = `/render/?doc=${encodeURIComponent(sample.id)}`;
    link.textContent = sample.title;
    item.append(link);
    emptySampleList.append(item);
  }
  emptyState.hidden = false;
}

async function main() {
  const scoreEl = document.getElementById("score");
  const titleEl = document.getElementById("doc-title");
  const bytesEl = document.getElementById("doc-bytes");
  const statusEl = document.getElementById("doc-status");

  let samples = [];
  try {
    const response = await fetch("/samples.json");
    samples = await response.json();
  } catch (err) {
    statusEl.textContent = `Failed to load the sample list: ${err}`;
  }

  const params = new URLSearchParams(location.search);
  let doc = params.get("doc");

  // Bare /render/ picks the first sample rather than showing an empty page.
  if (!doc && samples.length > 0) {
    doc = samples[0].id;
    const url = new URL(location.href);
    url.searchParams.set("doc", doc);
    history.replaceState(null, "", url);
  }

  const sample = samples.find((s) => s.id === doc);

  if (!sample) {
    scoreEl.hidden = true;
    showEmptyState(doc, samples);
  } else {
    titleEl.textContent = sample.title;
    bytesEl.textContent = formatBytes(sample.bytes);
    scoreEl.addEventListener("error", (e) => {
      statusEl.textContent = String(e.detail);
    });
    scoreEl.addEventListener("load", () => {
      statusEl.textContent = "";
    });
    scoreEl.setAttribute("file", `/assets/xmlsamples/${sample.file}`);
  }

  const { cssState, resets } = buildPane(scoreEl, document.getElementById("panel-groups"));
  wirePanelChrome(resets, cssState);
  wireZoom(scoreEl);
}

main();
