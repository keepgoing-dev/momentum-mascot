/**
 * Bakes a built mascot into the nine strips the rest of the app already reads.
 *
 * Everything here is source-over. The plates and the layer strips ship pre-tinted, so a
 * `filter`, a `globalAlpha` or any composite mode other than the default would be a bug:
 * the character would stop matching the room it stands in (spec section 4.5).
 */

const CAT_ORDER = ["skin", "eyes", "outfit", "hair", "accessory"];

/** Which strip frame a state draws at output frame `i`. Mirrors tools/assemble-frame.sh. */
export function frameIndex(manifest, state, surface, i) {
  const c = manifest.states[state][surface].char;
  const [lo, hi] = manifest.layerStrip.ranges[c.range];
  return c.frame === undefined ? lo + (i % (hi - lo)) : lo + c.frame;
}

/** Where the character's top-left sits at output frame `i`, hop included. */
export function framePlacement(manifest, state, surface, i) {
  const c = manifest.states[state][surface].char;
  return { x: c.x, y: c.y + c.hop[i % manifest.frames] };
}

function load(src) {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = () => reject(new Error(`cannot load ${src}`));
    img.src = src;
  });
}

/**
 * Loads every image one bake needs. Layers come from the build; plates, overlays and the
 * blanket are fixed.
 */
export async function loadAssets(manifest, build, base = "assets") {
  const layers = {};
  await Promise.all(
    CAT_ORDER.filter((c) => build[c]).map(async (c) => {
      layers[c] = await load(`${base}/layers/${c}/${build[c]}.png`);
    }),
  );

  const plates = {};
  const shared = {};
  await Promise.all(
    Object.entries(manifest.states).map(async ([state, s]) => {
      if (!s.room) return;
      plates[`${state}-back`] = await load(`${base}/plates/${state}-back.png`);
      if (s.room.front) plates[`${state}-front`] = await load(`${base}/plates/${state}-front.png`);
    }),
  );

  const names = new Set();
  for (const s of Object.values(manifest.states)) {
    for (const half of ["room", "pet"]) {
      for (const o of s[half]?.overlays ?? []) names.add(o.sprite);
    }
  }
  await Promise.all([...names].map(async (n) => { shared[n] = await load(`${base}/shared/${n}.png`); }));

  return { layers, plates, shared };
}

function drawCharacter(ctx, manifest, assets, state, surface, i, at) {
  const k = frameIndex(manifest, state, surface, i);
  const { w, h } = manifest.layerStrip.frame;
  for (const cat of CAT_ORDER) {
    const img = assets.layers[cat];
    if (img) ctx.drawImage(img, k * w, 0, w, h, at.x, at.y, w, h);
  }
}

function drawOverlays(ctx, assets, spec, i, at) {
  for (const o of spec.overlays ?? []) {
    const img = assets.shared[o.sprite];
    const ow = img.width / o.frames;
    ctx.drawImage(img, (i % o.frames) * ow, 0, ow, img.height,
      at.x + o.dx, at.y + o.dy, ow, img.height);
  }
}

function roomStrip(manifest, assets, state) {
  const { w, h } = manifest.room;
  const n = manifest.frames;
  const canvas = new OffscreenCanvas(w * n, h);
  const ctx = canvas.getContext("2d");
  ctx.imageSmoothingEnabled = false;

  const spec = manifest.states[state].room;
  for (let i = 0; i < n; i++) {
    const ox = i * w;
    ctx.drawImage(assets.plates[`${state}-back`], ox, 0, w, h, ox, 0, w, h);
    const at = framePlacement(manifest, state, "room", i);
    ctx.save();
    ctx.translate(ox, 0);
    drawCharacter(ctx, manifest, assets, state, "room", i, at);
    drawOverlays(ctx, assets, spec, i, at);
    ctx.restore();
    if (spec.front) ctx.drawImage(assets.plates[`${state}-front`], ox, 0, w, h, ox, 0, w, h);
  }
  return canvas;
}

function petStrip(manifest, assets, state) {
  const { w, h } = manifest.pet;
  const n = manifest.frames;
  const canvas = new OffscreenCanvas(w * n, h);
  const ctx = canvas.getContext("2d");
  ctx.imageSmoothingEnabled = false;

  const spec = manifest.states[state].pet;
  const frame = manifest.layerStrip.frame;
  for (let i = 0; i < n; i++) {
    const ox = i * w;
    const at = framePlacement(manifest, state, "pet", i);
    ctx.save();
    ctx.translate(ox, 0);
    drawCharacter(ctx, manifest, assets, state, "pet", i, at);
    // Only the body carries a blanket, so it comes from the skin strip.
    if (spec.blanketDy !== undefined) {
      const [bl] = manifest.layerStrip.ranges.blanket;
      ctx.drawImage(assets.layers.skin, bl * frame.w, 0, frame.w, frame.h,
        at.x, at.y + spec.blanketDy, frame.w, frame.h);
    }
    drawOverlays(ctx, assets, spec, i, at);
    ctx.restore();
  }
  return canvas;
}

/** The nine strips, keyed by the art names the Rust side allowlists. */
export async function bake(manifest, build, base = "assets") {
  const assets = await loadAssets(manifest, build, base);
  const out = new Map();
  for (const [state, s] of Object.entries(manifest.states)) {
    if (s.room) out.set(`rooms/${state}`, roomStrip(manifest, assets, state));
    if (s.pet) out.set(`pet/${state}`, petStrip(manifest, assets, state));
  }
  const blobs = new Map();
  for (const [name, canvas] of out) {
    blobs.set(name, await canvas.convertToBlob({ type: "image/png" }));
  }
  return blobs;
}
