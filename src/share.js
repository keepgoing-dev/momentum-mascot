// The share card, drawn in the webview at 1200x630.
//
// This is the executable twin of tools/compose-share.sh, which is where the composition was
// settled and argued (Phase 2, spec section 5.2). The constants below are that script's
// constants, and if the two ever disagree the script is the one that was looked at.
//
// The rule that came out of building it, and the one thing to preserve if this file is ever
// rewritten: **anything drawn on the room matches the room's pixel unit, and anything drawn
// on the mat is free.** The label obeys the first half at 5x, the quote the second at 2x, and
// the quote had to move off the room entirely to get there.

const CARD = {
  w: 1200,
  h: 630,
  scale: 5, // 160x112 -> 800x560
  roomX: 200, // (1200 - 800) / 2, so 200px of mat either side
  roomY: 12,
  frame: 5, // a one-room-pixel mount line
  mat: "#191924",
  mount: "#3a3a50",
  cream: "#f0e6d2",
  dimCream: "#b6ab98",
  shadow: "#14141c",
  muted: "#7c7c96",
  url: "keepgoing.dev",
  credit: "art: limezu.itch.io",
};

const LABELS = {
  awake: "AWAKE",
  dozing: "DOZING",
  asleep: "DREAMING",
  comeback: "BACK!!!",
};

// Pushed to the light end of each hue rather than using the state's own tint at full
// strength: #8f9ec8 on a dimmed grey wall was unreadable. The outline does the contrast work,
// not the fill.
const ACCENTS = {
  awake: "#f5c65c",
  dozing: "#dce4f4",
  asleep: "#e4ecff",
  comeback: "#ffd45e",
};

// Departure Mono's own cell, and its advance at that cell. Every size on the card is a whole
// multiple of the first, and the second is what the layout arithmetic assumes.
const CELL = 11;
const ADVANCE = 7;

/**
 * Register the font here rather than relying on a stylesheet.
 *
 * This started as a bug worth keeping the fix for. The card composed silently in a fallback
 * serif when it was called from a page that had not linked the stylesheet holding the
 * `@font-face` rule: hard-edged, correctly laid out, entirely the wrong typeface, and no
 * error anywhere. A module that draws type should own the type it draws, so that the card
 * cannot depend on what its caller happens to have loaded.
 */
let fontReady;
function ensureFont() {
  if (!fontReady) {
    const face = new FontFace(
      "Departure Mono",
      'url("assets/fonts/DepartureMono-Regular.woff2")',
    );
    fontReady = face.load().then((loaded) => document.fonts.add(loaded));
  }
  return fontReady;
}

/**
 * Refuse to compose a card in the wrong font.
 *
 * A silent fallback is the worst available outcome here: the artifact still looks finished,
 * still goes on someone's clipboard, and is wrong in the one way the whole of section 6.4 was
 * about. Measuring the advance catches it, because no fallback matches a 7px advance on an
 * 11px cell by accident.
 */
function assertFontLoaded() {
  const probe = document.createElement("canvas").getContext("2d");
  probe.font = `${CELL}px "Departure Mono"`;
  const advance = probe.measureText("MMMMMMMMMM").width / 10;
  if (Math.abs(advance - ADVANCE) > 0.5) {
    throw new Error(
      `the pixel font did not load (advance ${advance.toFixed(2)}px, expected ${ADVANCE}px)`,
    );
  }
}

/**
 * Render a line of type as a hard-edged bitmap.
 *
 * Canvas has no equivalent of ImageMagick's `+antialias`, and `+antialias` is not optional:
 * an antialiased pixel font is just a blurry font. So the text is drawn once at the font's
 * native 11px cell, its alpha channel is thresholded to make it a true one-bit bitmap, and
 * the bitmap is then scaled by a whole number with smoothing off. That is the same operation
 * the compositor performs, done in the only way a canvas allows.
 */
function typeBitmap(text, colour) {
  const probe = document.createElement("canvas").getContext("2d");
  probe.font = `${CELL}px "Departure Mono"`;
  const w = Math.max(1, Math.ceil(probe.measureText(text).width));
  const h = 16; // the 11px cell plus room for a descender

  const canvas = document.createElement("canvas");
  canvas.width = w;
  canvas.height = h;
  const ctx = canvas.getContext("2d", { willReadFrequently: true });
  ctx.font = `${CELL}px "Departure Mono"`;
  ctx.textBaseline = "alphabetic";
  ctx.fillStyle = colour;
  ctx.fillText(text, 0, CELL);

  const image = ctx.getImageData(0, 0, w, h);
  for (let i = 3; i < image.data.length; i += 4) {
    image.data[i] = image.data[i] >= 128 ? 255 : 0;
  }
  ctx.putImageData(image, 0, 0);
  return canvas;
}

/** Draw a bitmap so its baseline lands on `y`, scaled by a whole number. */
function drawType(ctx, bitmap, x, y, unit) {
  ctx.drawImage(bitmap, x, y - CELL * unit, bitmap.width * unit, bitmap.height * unit);
}

function drawOutlined(ctx, text, x, y, unit, colour) {
  // A full one-unit outline on all four sides, which is how the pack outlines its own art.
  // A drop shadow was not enough: the label sits on the wall, and the wall runs from
  // full-brightness beige when awake to dimmed grey-blue when asleep, so any single fill is
  // low-contrast against one end of that range. An outline makes the label independent of
  // what is behind it, so one colour rule covers all four states instead of four.
  const shadow = typeBitmap(text, CARD.shadow);
  for (const [dx, dy] of [
    [-unit, 0],
    [unit, 0],
    [0, -unit],
    [0, unit],
  ]) {
    drawType(ctx, shadow, x + dx, y + dy, unit);
  }
  drawType(ctx, typeBitmap(text, colour), x, y, unit);
}

function drawShadowed(ctx, text, x, y, unit, colour) {
  drawType(ctx, typeBitmap(text, CARD.shadow), x + 2, y + 2, unit);
  drawType(ctx, typeBitmap(text, colour), x, y, unit);
}

function drawRight(ctx, text, right, y, unit, colour) {
  const bitmap = typeBitmap(text, colour);
  drawType(ctx, bitmap, right - bitmap.width * unit, y, unit);
}

function loadImage(src) {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = () => reject(new Error(`could not load ${src}`));
    img.src = src;
  });
}

/**
 * Compose the card for a state and return it as PNG bytes.
 *
 * Nothing here can carry a project name, a path, a commit message, a hash, or a timestamp,
 * and that is the point rather than an oversight: the privacy rule in section 5.3 is enforced
 * by there being no way to express a violation, not by remembering not to.
 */
export async function composeCard({ mood, quote, characterId }) {
  await ensureFont();
  assertFontLoaded();

  const canvas = document.createElement("canvas");
  canvas.width = CARD.w;
  canvas.height = CARD.h;
  const ctx = canvas.getContext("2d");
  ctx.imageSmoothingEnabled = false;

  ctx.fillStyle = CARD.mat;
  ctx.fillRect(0, 0, CARD.w, CARD.h);

  // The room at exactly 5x, from frame 0 of the state's strip. A 10:7 room in a 1.91:1 crop
  // gets a mat, never fractional scaling to fill.
  const strip = await loadImage(`assets/rooms/${characterId}/${mood}.png`);
  const rw = 160 * CARD.scale;
  const rh = 112 * CARD.scale;
  ctx.fillStyle = CARD.mount;
  ctx.fillRect(
    CARD.roomX - CARD.frame,
    CARD.roomY - CARD.frame,
    rw + CARD.frame * 2,
    rh + CARD.frame * 2,
  );
  ctx.drawImage(strip, 0, 0, 160, 112, CARD.roomX, CARD.roomY, rw, rh);

  // The state label, on the upper-left wall. Room space x 0..62 by y 8..33 is plain
  // wallpaper: the trim ends at y=8, the map starts at x=64, the bed at y=34. At 5x that is a
  // clear 310x125 block, and DREAMING is the longest label at 280px.
  drawOutlined(
    ctx,
    LABELS[mood] ?? "",
    CARD.roomX + 20,
    CARD.roomY + 108,
    CARD.scale,
    ACCENTS[mood] ?? CARD.cream,
  );

  // The footer band: one baseline across the full 1200px, reading credit, quote, URL.
  //
  // The band spanning the whole width rather than the room's 800px column is the fix for the
  // one measurement that did not work out. A 5x room leaves 70px of vertical slack; a 22px
  // quote plus a meta row plus workable margins needs 78px. Moving the credit and the URL
  // into the side mats, which were dead space in every candidate, gives the quote the band's
  // whole height and buys the missing 8px.
  const band = CARD.roomY + rh + CARD.frame + 32;
  drawRight(ctx, CARD.credit, CARD.roomX - 16, band, 1, CARD.muted);
  drawShadowed(ctx, quote, CARD.roomX, band, 2, CARD.cream);
  drawRight(ctx, CARD.url, CARD.w - 16, band, 2, CARD.dimCream);

  const blob = await new Promise((resolve) => canvas.toBlob(resolve, "image/png"));
  return new Uint8Array(await blob.arrayBuffer());
}
