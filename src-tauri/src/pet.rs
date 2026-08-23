//! The desktop pet's window: 64x64, bottom-right, above everything, including fullscreen.
//!
//! This is the design's **one platform-specific exception** (section 10.3). The AppKit block
//! below is ported from `spikes/always-on-top/`, not rediscovered: ten `collectionBehavior`
//! values across four window levels up to `kCGMaximumWindowLevel` were all invisible over a
//! fullscreen Chrome window, and no `NSWindow` configuration works at all. What works is
//! changing the *kind* of window it is.
//!
//! **Applied once at window creation, and never adjusted afterwards.** The spike found that
//! reconfiguring a live window gives history-dependent results: the identical level and
//! behaviour was invisible over fullscreen in one run and visible in another, decided by what
//! had been applied minutes earlier. A configuration that measures differently depending on
//! its past is not one to shave, which is also why the recipe is not minimised further.
//!
//! The dead ends are kept in `spikes/always-on-top/RESULTS.md` so that a future macOS release
//! breaking this is re-diagnosed in minutes rather than re-explored from scratch.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tauri::{AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition};

use crate::app::{AppState, PET};

/// The pet's size in **logical** pixels, which is the same unit `pet.html` draws in.
///
/// The unit is the whole point of this constant, and it was wrong in the first version that
/// shipped. `PhysicalSize::new(64, 64)` here shrank the *webview* to 64 physical pixels, being
/// a 32x32 point viewport on a 2x display, while `"resizable": false` clamped the *window* back
/// to its configured 64x64 points. So the two disagreed by a factor of two and the character
/// was clipped to its own hat, on a window that still opened the popover when clicked.
///
/// It took three wrong diagnoses to find, because the obvious measurement is the wrong one:
/// `inner_size()` reports the window, and the window was correct. The viewport is only visible
/// from inside the webview, which is why `pet.js` is now the thing that measures it.
pub const SIZE: f64 = 64.0;

/// Clearance from the corner of the usable screen, in logical pixels.
const MARGIN: f64 = 20.0;

/// `NSStatusWindowLevel`.
#[cfg(target_os = "macos")]
const LEVEL: isize = 25;

/// `canJoinAllSpaces | stationary | fullScreenAuxiliary`.
#[cfg(target_os = "macos")]
const BEHAVIOR: usize = 273;

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let Some(win) = app.get_webview_window(PET) else {
        return Ok(());
    };

    win.set_size(LogicalSize::new(SIZE, SIZE))?;
    place(&win, app)?;

    #[cfg(target_os = "macos")]
    {
        let ns = win.ns_window()? as *mut objc2::runtime::AnyObject;
        if !macos::make_panel(ns) {
            eprintln!("NSPanel class not found; the pet will not show over fullscreen apps");
        }
        macos::apply(ns, LEVEL, BEHAVIOR);

        // Redundant while `macos-private-api` is on, because tao does it. Load-bearing the day
        // it is off, and silent if it is missing then: with the feature gone, tao's only
        // complaint is an eprintln gated on debug_assertions.
        crate::appkit::make_transparent(win.ns_window()?);

        // Task 3 only: the sprite view goes on top of the webview so the renderer can be judged
        // before the window type changes underneath it. Task 5 removes the webview.
        if crate::sprite::SpriteView::install(win.ns_window()?, app).is_none() {
            eprintln!("the sprite view could not be installed");
        }

        // Task 3 only. The webview pet is still present and still drawing, so the window shows
        // two characters at once: the native sprite and the old one. This hides the webview's
        // content so the native renderer can be judged on its own. Task 5 deletes the webview
        // outright and this goes with it.
        if std::env::var_os("MASCOT_HIDE_WEBVIEW").is_some() {
            let _ = win.eval(
                "document.documentElement.style.background='transparent';\
                 document.body.style.visibility='hidden'",
            );
        }
    }

    win.show()?;
    Ok(())
}

/// The usable area the pet may occupy, in physical pixels, resolved the same way as the
/// original bottom-right placement: the **intersection** of Tauri's work area and AppKit's
/// `visibleFrame` on the right and bottom edges.
///
/// `work_area` alone is not enough, and this cost real time to diagnose. On the author's
/// display it returned a rect that reserved the menu bar and **not** the Dock band, so a pet
/// placed relative to it sits underneath the Dock, which draws at window level 20. Every
/// AppKit property reported the window healthy while it was hidden.
///
/// Taking the tighter of the two is deliberate: whichever of them accounts for the Dock, the
/// pet clears it, and if Tauri's behaviour changes later this does not silently start
/// double-counting. The left and top edges come from the work area alone, which already
/// reserves the menu bar; only the Dock, which sits on the right or bottom, needs the
/// `visibleFrame` clamp.
struct Bounds {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
    /// The pet's own extent (`SIZE * scale`) and its corner clearance (`MARGIN * scale`),
    /// both already in physical pixels.
    extent: f64,
    margin: f64,
}

fn usable_bounds(win: &tauri::WebviewWindow) -> Option<Bounds> {
    let Some(mon) = win.current_monitor().ok().flatten() else {
        return None;
    };
    let scale = mon.scale_factor();
    let area = mon.work_area();
    let left = area.position.x as f64;
    let top = area.position.y as f64;
    let mut right = (area.position.x + area.size.width as i32) as f64;
    let mut bottom = (area.position.y + area.size.height as i32) as f64;

    #[cfg(target_os = "macos")]
    if let Some((vr, vb)) = macos::visible_bottom_right(scale) {
        right = right.min(vr);
        bottom = bottom.min(vb);
    }

    Some(Bounds {
        left,
        top,
        right,
        bottom,
        extent: SIZE * scale,
        margin: MARGIN * scale,
    })
}

/// The four corner anchors, as the pet's top-left in physical pixels, in reading order:
/// top-left, top-right, bottom-left, bottom-right.
fn anchors(b: &Bounds) -> [(i32, i32); 4] {
    let e = b.extent;
    let m = b.margin;
    [
        ((b.left + m) as i32, (b.top + m) as i32),
        ((b.right - e - m) as i32, (b.top + m) as i32),
        ((b.left + m) as i32, (b.bottom - e - m) as i32),
        ((b.right - e - m) as i32, (b.bottom - e - m) as i32),
    ]
}

/// Whether a saved top-left still lies on a connected display. A position that does not (an
/// unplugged monitor, a resolution change) falls back to the bottom-right default rather than
/// leaving the pet off screen, which is the contract in section 13.
fn within_bounds(x: f64, y: f64, b: &Bounds) -> bool {
    x >= b.left && y >= b.top && x + b.extent <= b.right && y + b.extent <= b.bottom
}

/// The corner nearest to a position, by squared distance. All four corners are on screen by
/// construction, so "nearest" is the only rule and there is no edge case to be wrong in.
fn nearest(current: (f64, f64), corners: &[(i32, i32); 4]) -> (i32, i32) {
    let mut best = corners[0];
    let mut best_d = f64::INFINITY;
    for &(x, y) in corners {
        let d = (x as f64 - current.0).powi(2) + (y as f64 - current.1).powi(2);
        if d < best_d {
            best_d = d;
            best = (x, y);
        }
    }
    best
}

/// The saved corner, if there is one and it still lies on a connected display, otherwise the
/// bottom-right default. This is the whole persistence story: placement reads the one field
/// and asks no questions of the geometry it does not need.
///
/// **`outer_position()` read straight after `set_position` returns the OLD position**, and
/// anyone checking this function's work needs to know that before they start. It reports the
/// macOS default, which is near the middle of the display, and it keeps reporting it however
/// many times the call is repeated. A few hundred milliseconds later the same read returns the
/// corner. This cost an entire wrong diagnosis, a restructure of `main.rs` around
/// `RunEvent::Ready`, and a fix for a bug that did not exist: placement worked correctly the
/// whole time and the measurement was lying. **Read the position from a delayed thread, or do
/// not read it.**
fn place(win: &tauri::WebviewWindow, app: &AppHandle) -> tauri::Result<()> {
    let Some(b) = usable_bounds(win) else {
        return Ok(());
    };
    let corners = anchors(&b);

    let saved = app
        .state::<AppState>()
        .momentum
        .lock()
        .unwrap()
        .state
        .pet_position;
    let target = saved
        .filter(|&(x, y)| within_bounds(x as f64, y as f64, &b))
        .unwrap_or(corners[3]);

    win.set_position(PhysicalPosition::new(target.0, target.1))?;
    Ok(())
}

/// The corner nearest `current`, without moving the window.
///
/// `current` is the pet's top-left in physical pixels, handed over by the webview rather than
/// read back here, because the webview is the one that just moved it and its own last word on
/// where it is is more recent than anything `outer_position()` would report. The backend owns
/// the geometry (the display, the Dock-aware bounds, the anchors) and the motion; the frontend
/// owns the pointer. Called once on drag end.
pub fn nearest_corner(
    win: &tauri::WebviewWindow,
    current: (f64, f64),
) -> Option<(i32, i32)> {
    let b = usable_bounds(win)?;
    Some(nearest(current, &anchors(&b)))
}

/// Bumped whenever a glide starts or is cancelled. A running glide checks it against the value
/// it started under and stops the moment it is no longer the latest, so the tail of an old
/// glide can never fight a newer drag.
static GLIDE_GENERATION: AtomicU64 = AtomicU64::new(0);

/// The event the pet webview listens for to know a glide has landed and it can stop running.
const GLIDE_DONE_EVENT: &str = "glide-done";

/// Glide the window from `from` to `to`, both physical pixels, easing out over ~250ms.
///
/// The motion is driven from a thread here rather than from the webview, because the pet's
/// window is never focused and WebKit throttles the webview's own timers — `requestAnimationFrame`
/// included — for exactly that window. An animation the frontend runs may silently not run; a
/// Rust thread is not subject to that throttling. The final step lands exactly on `to`, so the
/// corner is reached even if an earlier step was coalesced away.
///
/// Landing emits `glide-done`, and only landing does: a glide cut short by `cancel_glide` stays
/// silent, so a re-grab does not get the frontend's run cut short out from under it.
pub fn glide_to(win: &tauri::WebviewWindow, from: (f64, f64), to: (i32, i32)) {
    let generation = GLIDE_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    let win = win.clone();
    std::thread::spawn(move || {
        const STEPS: u32 = 16;
        const STEP: Duration = Duration::from_millis(16);
        for i in 1..=STEPS {
            if GLIDE_GENERATION.load(Ordering::SeqCst) != generation {
                return;
            }
            let t = i as f64 / STEPS as f64;
            let eased = 1.0 - (1.0 - t).powi(3);
            let x = from.0 + (to.0 as f64 - from.0) * eased;
            let y = from.1 + (to.1 as f64 - from.1) * eased;
            if win.set_position(PhysicalPosition::new(x, y)).is_err() {
                return;
            }
            std::thread::sleep(STEP);
        }
        let _ = win.emit(GLIDE_DONE_EVENT, ());
    });
}

/// Cancel any glide still in flight. Called when a new drag begins, before the cursor has a
/// chance to be fought by the previous glide's remaining steps.
pub fn cancel_glide() {
    GLIDE_GENERATION.fetch_add(1, Ordering::SeqCst);
}

#[cfg(target_os = "macos")]
mod macos {
    use objc2::runtime::{AnyClass, AnyObject, Bool};
    use objc2_app_kit::NSScreen;
    use objc2_foundation::MainThreadMarker;

    /// Swap the window's class for a non-activating `NSPanel`.
    ///
    /// This is the load-bearing step and the same approach `tauri-nspanel` takes. It is
    /// hand-rolled rather than taken as a dependency because it is fifteen lines already
    /// written and verified, against a dependency with its own Tauri-version coupling and its
    /// own plugin surface, in a project whose stated failure mode is sprawl.
    ///
    /// A panel with `NSWindowStyleMaskNonactivatingPanel` can be shown without activating its
    /// application, which is exactly the property a plain `NSWindow` lacks and exactly why
    /// clicking the pet over a fullscreen app neither switches Space nor steals focus.
    pub fn make_panel(ns: *mut AnyObject) -> bool {
        unsafe {
            let Some(panel) = AnyClass::get(c"NSPanel") else {
                return false;
            };
            let cls = panel as *const AnyClass;
            objc2::ffi::object_setClass(ns.cast(), cls.cast());
            // Preserve borderless (0) and add nonactivatingPanel (1 << 7).
            let mask: usize = objc2::msg_send![ns, styleMask];
            let _: () = objc2::msg_send![ns, setStyleMask: mask | (1usize << 7)];
            let _: () = objc2::msg_send![ns, setFloatingPanel: Bool::YES];
            let _: () = objc2::msg_send![ns, setBecomesKeyOnlyIfNeeded: Bool::YES];
            true
        }
    }

    pub fn apply(ns: *mut AnyObject, level: isize, behavior: usize) {
        unsafe {
            let _: () = objc2::msg_send![ns, setCollectionBehavior: behavior];
            let _: () = objc2::msg_send![ns, setLevel: level];
            // An accessory app is never "active", so a window that hides on deactivation
            // would vanish for a reason that has nothing to do with Spaces.
            let _: () = objc2::msg_send![ns, setHidesOnDeactivate: Bool::NO];
        }
    }

    /// The bottom-right of the main screen's `visibleFrame`, in Tauri's physical pixels.
    ///
    /// AppKit measures from the bottom-left of the *primary* screen with y increasing
    /// upwards; Tauri measures from the top-left with y increasing downwards. The flip needs
    /// the primary screen's height, which is the first entry in `NSScreen.screens`.
    pub fn visible_bottom_right(scale: f64) -> Option<(f64, f64)> {
        let mtm = MainThreadMarker::new()?;
        let screens = NSScreen::screens(mtm);
        let primary_height = screens.iter().next()?.frame().size.height;
        let visible = NSScreen::mainScreen(mtm)?.visibleFrame();

        let right = visible.origin.x + visible.size.width;
        let bottom_from_top = primary_height - visible.origin.y;
        Some((right * scale, bottom_from_top * scale))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b() -> Bounds {
        Bounds {
            left: 0.0,
            top: 0.0,
            right: 1920.0,
            bottom: 1080.0,
            extent: 64.0,
            margin: 20.0,
        }
    }

    #[test]
    fn the_four_corners_are_where_a_corner_is() {
        assert_eq!(
            anchors(&b()),
            [(20, 20), (1836, 20), (20, 996), (1836, 996)]
        );
    }

    #[test]
    fn the_nearest_corner_is_chosen_by_distance() {
        let c = anchors(&b());
        assert_eq!(nearest((10.0, 10.0), &c), (20, 20));
        assert_eq!(nearest((1900.0, 10.0), &c), (1836, 20));
        assert_eq!(nearest((10.0, 1050.0), &c), (20, 996));
        assert_eq!(nearest((1900.0, 1050.0), &c), (1836, 996));
    }

    #[test]
    fn an_off_screen_position_falls_back_rather_than_leaving_the_pet_stranded() {
        let b = b();
        assert!(within_bounds(0.0, 0.0, &b));
        assert!(
            within_bounds(1856.0, 1016.0, &b),
            "the far corner is the far edge"
        );
        assert!(!within_bounds(-1.0, 0.0, &b));
        assert!(!within_bounds(0.0, -1.0, &b));
        assert!(
            !within_bounds(1857.0, 0.0, &b),
            "an extent poking past the right edge"
        );
        assert!(!within_bounds(0.0, 1017.0, &b));
    }
}
