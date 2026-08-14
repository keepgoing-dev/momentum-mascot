// The desktop pet. It has two interactions and no others: a click opens the popover, and a
// drag to one of the four screen corners moves it there. That is the whole surface, and it
// should stay that small.

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { getCurrentWindow } = window.__TAURI__.window;

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

// Clicking the pet opens the popover; dragging it to a corner moves it. The two are told
// apart by movement alone, and the click path runs on `mouseup` rather than `click`, because
// a browser still fires `click` after a drag that stays under its own threshold.
//
// The drag listens on `window`, not on the element. The pet is 64x64, so a cursor that leaves
// it almost immediately would otherwise end the drag; a window that received a button press
// keeps receiving mouse events until the button is released, however far the cursor travels,
// and `window` catches them wherever they land. Pointer capture was tried and it failed: the
// window moving out from under a captured pointer makes WebKit drop the events, so there is
// deliberately no `setPointerCapture` here.
//
// Movement is accumulated from `movementX`/`movementY`, the cursor's own delta since the last
// event. That is the one coordinate the drag can trust: the OS measures it in screen space, so
// it is unaffected by the window being moved out from under the cursor, whereas a window-
// relative read would cancel itself out and the pet would never budge. It is scaled by
// `devicePixelRatio` into the physical pixels the backend uses.
const DRAG_THRESHOLD = 4; // CSS px of movement below which a drag is a click.

let drag = null;

pet.addEventListener("mousedown", (e) => {
  if (e.button !== 0) return;
  e.preventDefault();
  // A drag that starts while the last glide is still landing would have its own movement
  // fought by the glide's remaining steps, so stop it before following the cursor.
  invoke("cancel_glide");
  getCurrentWindow()
    .outerPosition()
    .then((pos) => {
      drag = { x: pos.x, y: pos.y, ox: pos.x, oy: pos.y, moved: false };
      pet.style.cursor = "grabbing";
    })
    .catch(() => {
      drag = null;
    });
});

window.addEventListener("mousemove", (e) => {
  if (!drag) return;
  const scale = window.devicePixelRatio || 1;
  drag.x += e.movementX * scale;
  drag.y += e.movementY * scale;
  if (Math.hypot(drag.x - drag.ox, drag.y - drag.oy) >= DRAG_THRESHOLD * scale) {
    drag.moved = true;
  }
  const { PhysicalPosition } = window.__TAURI__.dpi;
  getCurrentWindow().setPosition(new PhysicalPosition(drag.x, drag.y));
});

window.addEventListener("mouseup", (e) => {
  if (!drag || e.button !== 0) return;
  const wasDrag = drag.moved;
  const { x, y } = drag;
  drag = null;
  pet.style.cursor = "";
  if (wasDrag) invoke("snap_pet", { x, y });
  else invoke("toggle_popover");
});

// The context menu and native HTML5 dragging both have to go, or a right-click turns the
// character into a web page and a slow drag turns it into selected text.
document.addEventListener("contextmenu", (e) => e.preventDefault());
document.addEventListener("dragstart", (e) => e.preventDefault());

listen("mood", (event) => render(event.payload));

// A window that opens between ticks would otherwise be blank until the next one.
invoke("refresh");
