// Builds the navbar (brand, samples dropdown, Docs/Roadmap/GitHub links)
// into a <header id="site-nav"></header> present on every page, so the
// samples list only has to be fetched and rendered in one place.
const GITHUB_REPO = "https://github.com/Studio-La-Valse/Opus";

function el(tag, props = {}) {
  const node = document.createElement(tag);
  Object.assign(node, props);
  return node;
}

async function loadSamples() {
  try {
    const response = await fetch("/samples.json");
    if (!response.ok) return [];
    return await response.json();
  } catch {
    return [];
  }
}

async function initNav() {
  const nav = document.getElementById("site-nav");
  if (!nav) return;

  const samples = await loadSamples();

  const brand = el("a", { href: "/", className: "nav-brand", textContent: "Opus" });

  const samplesWrap = el("div", { className: "nav-samples" });
  const samplesButton = el("button", {
    type: "button",
    className: "nav-samples-button",
    textContent: "Samples",
    "aria-expanded": "false",
  });
  const samplesMenu = el("div", { className: "nav-samples-menu" });
  samplesMenu.hidden = true;
  for (const sample of samples) {
    samplesMenu.append(
      el("a", { href: `/render/?doc=${encodeURIComponent(sample.id)}`, textContent: sample.title }),
    );
  }
  samplesButton.addEventListener("click", () => {
    const open = samplesMenu.hidden;
    samplesMenu.hidden = !open;
    samplesButton.setAttribute("aria-expanded", String(open));
  });
  document.addEventListener("click", (event) => {
    if (!samplesWrap.contains(event.target)) {
      samplesMenu.hidden = true;
      samplesButton.setAttribute("aria-expanded", "false");
    }
  });
  samplesWrap.append(samplesButton, samplesMenu);

  const links = el("div", { className: "nav-links" });
  links.append(
    el("a", {
      href: `${GITHUB_REPO}/blob/main/README.md`,
      textContent: "Docs",
      target: "_blank",
      rel: "noopener",
    }),
    el("a", {
      href: `${GITHUB_REPO}/blob/main/ROADMAP.md`,
      textContent: "Roadmap",
      target: "_blank",
      rel: "noopener",
    }),
    el("a", { href: GITHUB_REPO, textContent: "GitHub", target: "_blank", rel: "noopener" }),
  );

  nav.append(brand, samplesWrap, links);
}

initNav();
