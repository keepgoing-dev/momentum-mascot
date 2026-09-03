/**
 * The mascot builder: a takeover of the panel below the room, with the room as its preview.
 *
 * Hair and outfit are style by colour. The stored id encodes both (`Hairstyle_11_03`), so the
 * two pickers compose one string and the state schema never learns there were two axes.
 */

import { bake } from "./baker.js";

const CATS = ["skin", "eyes", "hair", "outfit", "accessory"];
const TAB_LABEL = { skin: "SKIN", eyes: "EYES", hair: "HAIR", outfit: "WEAR", accessory: "EXTRA" };
const SPLIT = { hair: "Hairstyle", outfit: "Outfit" };

const el = {
  room: document.getElementById("room"),
  charHit: document.getElementById("charHit"),
  chars: document.getElementById("characters"),
  preview: document.getElementById("preview"),
  panel: document.getElementById("builder"),
  tabs: document.getElementById("builderTabs"),
  colours: document.getElementById("builderColours"),
  grid: document.getElementById("builderGrid"),
  cancel: document.getElementById("builderCancel"),
  shuffle: document.getElementById("builderShuffle"),
  done: document.getElementById("builderDone"),
};

let index = null;
let manifest = null;
let build = null;
let tab = "skin";
let onFinish = null;

const pick = (a) => a[Math.floor(Math.random() * a.length)];
const styleOf = (id) => id.split("_")[1];
const colourOf = (id) => id.split("_")[2];

/** The ids of one category that share a style, in colour order. */
function coloursFor(cat, style) {
  return index[cat].filter((id) => styleOf(id) === style);
}

/** One representative id per style, so the style grid shows each style once. */
function stylesFor(cat) {
  const seen = new Set();
  return index[cat].filter((id) => {
    const s = styleOf(id);
    if (seen.has(s)) return false;
    seen.add(s);
    return true;
  });
}

function randomBuild() {
  return {
    skin: pick(index.skin),
    eyes: pick(index.eyes),
    hair: pick(index.hair),
    outfit: pick(index.outfit),
    accessory: Math.random() < 0.4 ? null : pick(index.accessory),
  };
}

function renderPreview() {
  el.preview.replaceChildren(
    ...CATS.filter((c) => build[c]).map((c) => {
      const d = document.createElement("div");
      d.className = "layer pixels";
      d.style.backgroundImage = `url("assets/layers/${c}/${build[c]}.png")`;
      return d;
    }),
  );
}

function swatch(cat, id, selected, label) {
  const b = document.createElement("button");
  b.type = "button";
  b.className = "swatch pixels";
  b.setAttribute("aria-pressed", selected ? "true" : "false");
  if (id === null) {
    b.classList.add("none");
    b.textContent = "none";
    b.setAttribute("aria-label", `No ${cat}`);
  } else {
    b.style.backgroundImage = `url("assets/swatches/${cat}/${id}.png")`;
    b.setAttribute("aria-label", label ?? id);
  }
  return b;
}

function renderTab() {
  const cat = tab;
  const split = SPLIT[cat];

  if (split) {
    const current = styleOf(build[cat]);
    el.colours.hidden = false;
    el.colours.replaceChildren(
      ...coloursFor(cat, current).map((id) => {
        const b = swatch(cat, id, id === build[cat], `colour ${colourOf(id)}`);
        b.addEventListener("click", () => {
          build[cat] = id;
          renderPreview();
          renderTab();
        });
        return b;
      }),
    );
    el.grid.replaceChildren(
      ...stylesFor(cat).map((rep) => {
        const style = styleOf(rep);
        // Keep the chosen colour when switching style, falling back if that style lacks it.
        const same = coloursFor(cat, style);
        const wanted = same.find((id) => colourOf(id) === colourOf(build[cat])) ?? same[0];
        const b = swatch(cat, wanted, style === current, `style ${style}`);
        b.addEventListener("click", () => {
          build[cat] = wanted;
          renderPreview();
          renderTab();
        });
        return b;
      }),
    );
  } else {
    el.colours.hidden = true;
    const ids = cat === "accessory" ? [null, ...index[cat]] : index[cat];
    el.grid.replaceChildren(
      ...ids.map((id) => {
        const b = swatch(cat, id, id === build[cat], id ?? undefined);
        b.addEventListener("click", () => {
          build[cat] = id;
          renderPreview();
          renderTab();
        });
        return b;
      }),
    );
  }

  for (const b of el.tabs.children) {
    b.setAttribute("aria-selected", b.dataset.cat === cat ? "true" : "false");
  }
}

function buildTabs() {
  el.tabs.replaceChildren(
    ...CATS.map((c) => {
      const b = document.createElement("button");
      b.type = "button";
      b.dataset.cat = c;
      b.textContent = TAB_LABEL[c];
      b.addEventListener("click", () => {
        tab = c;
        renderTab();
      });
      return b;
    }),
  );
}

async function ensureData() {
  if (index && manifest) return;
  [index, manifest] = await Promise.all([
    fetch("assets/layers/index.json").then((r) => r.json()),
    fetch("assets/character-layout.json").then((r) => r.json()),
  ]);
}

export function isOpen() {
  return !el.panel.hidden;
}

/**
 * Opens the builder. `existing` is the stored build to edit, or null to start from a shuffle
 * rather than from an empty grid.
 */
export async function open(existing, finish) {
  await ensureData();
  onFinish = finish;
  build = existing ? { ...existing } : randomBuild();
  tab = "skin";

  // The preview is the untinted back plate: judging a hairstyle through a 34% blue wash is
  // not judging a hairstyle.
  el.preview.style.backgroundImage = 'url("assets/plates/awake-back.png")';
  el.preview.hidden = false;
  el.room.style.visibility = "hidden";
  el.charHit.hidden = true;
  el.chars.hidden = true;
  el.panel.hidden = false;
  for (const n of document.querySelectorAll(".quote, .projects, .buttons, .empty")) n.hidden = true;

  buildTabs();
  renderTab();
  renderPreview();
}

export function close() {
  el.preview.hidden = true;
  el.room.style.visibility = "";
  el.charHit.hidden = false;
  el.chars.hidden = false;
  el.panel.hidden = true;
  for (const n of document.querySelectorAll(".quote, .projects, .buttons")) n.hidden = false;
}

el.cancel.addEventListener("click", () => {
  close();
  onFinish?.(null);
});

el.shuffle.addEventListener("click", () => {
  build = randomBuild();
  renderPreview();
  renderTab();
});

el.done.addEventListener("click", async () => {
  el.done.disabled = true;
  try {
    const blobs = await bake(manifest, build);
    onFinish?.(build, blobs);
    close();
  } finally {
    el.done.disabled = false;
  }
});
