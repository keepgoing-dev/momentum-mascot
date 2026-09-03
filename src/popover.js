// The popover: the room, the quote, the project list, two buttons, and the credit line.

import { composeCard } from "./share.js";

import * as builder from "./builder.js";

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { getVersion } = window.__TAURI__.app;

const room = document.getElementById("room");
const charHit = document.getElementById("charHit");
const charsEl = document.getElementById("characters");
const quoteEl = document.getElementById("quote");
const list = document.getElementById("projects");
const emptyEl = document.getElementById("empty");
const errorEl = document.getElementById("error");
const clockEl = document.getElementById("clock");
const addButton = document.getElementById("add");
const shareButton = document.getElementById("share");

let current = null;

// The three shipped characters. Kept in one place so the picker, the room, and the backend
// all agree on what can be selected.
const CHARACTERS = ["07", "12", "20"];
const CUSTOM_ID = "custom";

/**
 * Where the character actually is, per state, in room pixels at 2x.
 *
 * These come from the compositor's own placements (tools/compose-rooms.sh) rather than from
 * looking at the picture, which is the same rule the crops follow. Awake is short because the
 * desk occludes the lower body, and that occlusion is the entire difference between reading
 * as sitting at the desk and standing behind it.
 */
const CHARACTER_AT = {
  awake: { left: 222, top: 82, width: 32, height: 34 },
  dozing: { left: 108, top: 112, width: 32, height: 64 },
  asleep: { left: 20, top: 70, width: 32, height: 32 },
  comeback: { left: 108, top: 128, width: 32, height: 64 },
};

function buildCharacters() {
  charsEl.replaceChildren(
    ...CHARACTERS.map((id) => {
      const btn = document.createElement("button");
      btn.className = "char-btn";
      btn.dataset.id = id;
      btn.type = "button";
      btn.title = `Character ${id}`;
      btn.setAttribute("aria-label", `Use character ${id}`);
      btn.style.backgroundImage = `url("assets/pet/${id}/dozing.png")`;
      btn.addEventListener("click", () => invoke("set_character", { id }));
      return btn;
    }),
    customButton(),
  );
}

/**
 * The fourth slot. A dashed "+" until something is built, then the mascot's own head. Clicking
 * it while it is already selected re-opens the builder, which is the only way back in to edit.
 */
function customButton() {
  const built = Boolean(current?.custom_character);
  const selected = current?.character_id === CUSTOM_ID;
  const btn = document.createElement("button");
  btn.className = built ? "char-btn" : "char-btn plus";
  btn.dataset.id = CUSTOM_ID;
  btn.type = "button";
  btn.title = built ? "Your mascot" : "Build your own";
  btn.setAttribute("aria-label", built ? "Use your built mascot" : "Build your own mascot");
  if (built) {
    btn.style.backgroundImage = `url("assets/swatches/skin/${current.custom_character.skin}.png")`;
  } else {
    btn.textContent = "+";
  }
  btn.addEventListener("click", () => {
    if (!built || selected) openBuilder();
    else invoke("set_character", { id: CUSTOM_ID });
  });
  return btn;
}

async function openBuilder() {
  await builder.open(current?.custom_character ?? null, async (build, blobs) => {
    if (!build) return;
    for (const [name, blob] of blobs) {
      const png = Array.from(new Uint8Array(await blob.arrayBuffer()));
      await invoke("write_custom_art", { name, png });
    }
    // The UI category is "skin"; the stored field is "body", after the pack's own folder.
    await invoke("save_custom_character", {
      body: build.skin,
      eyes: build.eyes,
      outfit: build.outfit,
      hair: build.hair,
      accessory: build.accessory,
    });
  });
}

function updateCharacters(selectedId) {
  for (const btn of charsEl.children) {
    btn.classList.toggle("selected", btn.dataset.id === selectedId);
    btn.setAttribute("aria-pressed", btn.dataset.id === selectedId ? "true" : "false");
  }
}

function render(payload) {
  current = payload;
  if (builder.isOpen()) return;

  room.dataset.mood = payload.mood;
  setRoomArt(payload);

  const at = CHARACTER_AT[payload.mood] ?? CHARACTER_AT.awake;
  Object.assign(charHit.style, {
    left: `${at.left}px`,
    top: `${at.top}px`,
    width: `${at.width}px`,
    height: `${at.height}px`,
  });

  updateCharacters(payload.character_id);

  quoteEl.textContent = payload.quote;

  list.replaceChildren(...payload.projects.map(row));
  emptyEl.hidden = payload.projects.length > 0;

  // Only ever visible while the demo clock is running, and only in a debug build. A recording
  // made against a scaled clock should say so on screen while it is being made.
  clockEl.hidden = payload.clock_scale === 1;
  clockEl.textContent = `debug clock: ${payload.clock_scale}x`;

  fitWindow();
}

/**
 * The room's art. A built mascot's strips live outside the bundle, so they come back through
 * the bridge as bytes and become a blob URL, which the existing CSP already permits.
 *
 * Falling back when the art is not ready is spec section 5.4: a half-written cache must show a
 * premade rather than a room with nobody in it.
 */
let roomUrl = null;
async function setRoomArt(payload) {
  const custom = payload.character_id === CUSTOM_ID;
  const id = custom && !payload.custom_art_ready ? CHARACTERS[0] : payload.character_id;

  if (roomUrl) {
    URL.revokeObjectURL(roomUrl);
    roomUrl = null;
  }
  if (id !== CUSTOM_ID) {
    room.style.backgroundImage = `url("assets/rooms/${id}/${payload.mood}.png")`;
    return;
  }
  const png = await invoke("read_custom_art", { name: `rooms/${payload.mood}` });
  roomUrl = URL.createObjectURL(new Blob([new Uint8Array(png)], { type: "image/png" }));
  room.style.backgroundImage = `url("${roomUrl}")`;
}

function row(project) {
  const li = document.createElement("li");
  if (!project.available) li.classList.add("away");
  if (project.operating) li.classList.add("operating");

  const name = document.createElement("span");
  name.className = "name";
  name.textContent = project.name;
  // The backend's specific reason wins when there is one. Under sandbox a linked worktree or a
  // submodule is unavailable for a reason worth naming, and "not reachable right now" covered
  // four different causes.
  name.title = project.available
    ? project.name
    : `${project.name} (${project.reason ?? "not reachable right now"})`;

  const op = document.createElement("button");
  op.className = "op";
  op.type = "button";
  op.textContent = project.operating ? "●" : "○";
  op.title = project.operating
    ? "Operating: excluded from mascot mood"
    : "Mark as operating";
  op.addEventListener("click", () => invoke("toggle_operating", { id: project.id }));

  const when = document.createElement("span");
  when.className = "when";
  when.textContent = project.available ? project.relative : "unavailable";

  const drop = document.createElement("button");
  drop.className = "drop";
  drop.textContent = "x";
  drop.title = `Stop tracking ${project.name}`;
  drop.addEventListener("click", () => invoke("untrack", { id: project.id }));

  li.append(name, op, when, drop);

  // The reason goes on its own line, visibly, and not in a tooltip.
  //
  // It was a tooltip first, and that was measured not to work: a `title` on a span never renders
  // in this webview, though one on a button does. An explanation nobody can see is not an
  // explanation, and under sandbox a linked worktree reads as unavailable for a reason the user
  // cannot otherwise guess. The `title` above is kept as well, because it costs nothing and does
  // work in some contexts.
  if (!project.available && project.reason) {
    const why = document.createElement("span");
    why.className = "reason";
    why.textContent = project.reason;
    li.append(why);
  }

  return li;
}

/** Height is sized to content (section 6.3), so it is measured rather than guessed. */
async function fitWindow() {
  try {
    const { getCurrentWindow } = window.__TAURI__.window;
    const { LogicalSize } = window.__TAURI__.dpi;
    const height = Math.ceil(document.querySelector(".panel").getBoundingClientRect().height);
    await getCurrentWindow().setSize(new LogicalSize(352, height));
  } catch {
    // A window that will not resize is a cosmetic problem, not a reason to stop.
  }
}

function showError(message) {
  errorEl.textContent = message;
  errorEl.hidden = !message;
  fitWindow();
}

addButton.addEventListener("click", async () => {
  showError("");
  try {
    await invoke("add_project");
  } catch (e) {
    // One short line, inline. No modal and no alert dialog: the app has one surface, and an
    // error is a sentence in it.
    showError(String(e));
  }
});

shareButton.addEventListener("click", async () => {
  if (!current) return;
  showError("");
  shareButton.disabled = true;
  const label = shareButton.textContent;
  try {
    const png = await composeCard({
      mood: current.mood,
      quote: current.quote,
      characterId: current.character_id,
    });
    await invoke("copy_share_card", { png: Array.from(png) });
    shareButton.textContent = "Art copied";
    setTimeout(() => (shareButton.textContent = label), 1600);
  } catch (e) {
    showError(`Couldn't copy that: ${e}`);
    shareButton.textContent = label;
  } finally {
    shareButton.disabled = false;
  }
});

charHit.addEventListener("click", () => invoke("cycle_character"));

// Guideline 5.1.1(i): the policy has to be reachable from inside the app. The tray is not the
// place (tray.rs holds the line at exactly two items), so it lives on the credit line.
document
  .getElementById("privacy")
  .addEventListener("click", () => invoke("open_privacy_policy"));

// Read from the bundle rather than written down here, so it cannot disagree with what
// release.sh bumped. Allowed by core:app:default, which core:default already grants.
getVersion().then((v) => {
  document.getElementById("version").textContent = `v${v}`;
});

document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") invoke("hide_popover");
});
document.addEventListener("contextmenu", (e) => e.preventDefault());

buildCharacters();

listen("mood", (event) => render(event.payload));

// Only a refresh, deliberately. Resolving the comeback and rotating the quote belong to the
// window being *shown*, which the backend owns, and this page loads once at startup while the
// window is still hidden. Calling the open path from here would quietly resolve a comeback
// that had survived a restart, before anyone had seen it.
invoke("refresh");
