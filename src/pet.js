// The desktop pet. It has one job and one interaction, and it should stay that small.

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const pet = document.getElementById("pet");

// The sprite's native frame, in source pixels. Everything the pet draws is a whole multiple
// of it, and never a fraction: a 1.5x character is not a smaller one, it is a blurry one.
const FRAME = 32;

// Size the character to the window it is actually in.
//
// The window is the authority here. Reading it rather than assuming it is what makes a
// mis-sized window show a small pet instead of a cropped one, and a cropped pet is the failure
// that got shipped: the head alone, on a window that still opened the popover when clicked.
//
// The floor is one whole frame, so a window too small for even 1x scales nothing down.
function fit() {
  const side = Math.min(window.innerWidth, window.innerHeight);
  const cell = Math.max(FRAME, Math.floor(side / FRAME) * FRAME);
  pet.style.setProperty("--cell", `${cell}px`);
}

fit();
// Moving between displays of different densities changes the viewport under a live page.
window.addEventListener("resize", fit);

function render(payload) {
  pet.dataset.mood = payload.mood;
  pet.style.backgroundImage = `url("assets/pet/${payload.character_id}/${payload.mood}.png")`;
}

// Clicking the pet opens the popover. Nothing else here does anything: no drag, no menu, no
// hover state. A pet with affordances is a widget, and a widget is a thing to configure.
pet.addEventListener("click", () => invoke("toggle_popover"));

// The context menu and dragging both have to go, or a right-click turns the character into a
// web page and a slow drag turns it into selected text.
document.addEventListener("contextmenu", (e) => e.preventDefault());
document.addEventListener("dragstart", (e) => e.preventDefault());

listen("mood", (event) => render(event.payload));

// A window that opens between ticks would otherwise be blank until the next one.
invoke("refresh");
