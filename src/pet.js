// The desktop pet. It has one job and one interaction, and it should stay that small.

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const pet = document.getElementById("pet");

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
