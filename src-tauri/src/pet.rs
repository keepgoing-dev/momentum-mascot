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
//!
//! The recipe itself lives in `appkit::show_over_fullscreen`, because the popover turned out to
//! need it too: it shipped as a plain `NSWindow`, so the pet was visible over a fullscreen app
//! and clicking it opened a popover nobody could see.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tauri::{AppHandle, LogicalSize, Manager, PhysicalPosition};

use crate::app::{AppState, PET};
use crate::store::PetAnchor;

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

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    // Built here rather than in tauri.conf.json because `app.windows` has no webview-less form,
    // so the config entry was deleted rather than converted. `WindowBuilder` itself is only
    // public under tauri's `unstable` feature.
    //
    // Per spec 4.1, `WindowBuilder::transparent()` is gated on `macos-private-api`, so the
    // window is opaque until `appkit::make_transparent` runs below.
    let win = tauri::window::WindowBuilder::new(app, PET)
        .inner_size(SIZE, SIZE)
        .resizable(false)
        .decorations(false)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .visible(false)
        .title("Momentum Mascot")
        .build()?;

    win.set_size(LogicalSize::new(SIZE, SIZE))?;
    place(&win, app)?;

    #[cfg(target_os = "macos")]
    {
        // `false`: clicking the character must never take the keyboard from whatever the user
        // is doing. The popover asks for the opposite, because Escape dismisses it.
        if !crate::appkit::show_over_fullscreen(win.ns_window()?, false) {
            eprintln!("NSPanel class not found; the pet will not show over fullscreen apps");
        }

        // Redundant while `macos-private-api` is on, because tao does it. Load-bearing the day
        // it is off, and silent if it is missing then: with the feature gone, tao's only
        // complaint is an eprintln gated on debug_assertions.
        crate::appkit::make_transparent(win.ns_window()?);

        // Task 3 only: the sprite view goes on top of the webview so the renderer can be judged
        // before the window type changes underneath it. Task 5 removes the webview.
        match crate::sprite::SpriteView::install(win.ns_window()?, app) {
            Some(view) => {
                let _ = SPRITE.set(crate::sprite::SpriteHandle::new(view));
            }
            None => eprintln!("the sprite view could not be installed"),
        }

    }

    win.show()?;

    #[cfg(target_os = "macos")]
    {
        let handle = app.clone();
        crate::appkit::observe_screen_changes(move || replace(&handle));
    }

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

/// A display's physical extent, in the same top-down coordinates as `Bounds`.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Rect {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

impl Rect {
    /// Squared distance from a point to the rect, and zero for a point inside it.
    fn distance_to(&self, p: (f64, f64)) -> f64 {
        let dx = (self.left - p.0).max(p.0 - self.right).max(0.0);
        let dy = (self.top - p.1).max(p.1 - self.bottom).max(0.0);
        dx * dx + dy * dy
    }
}

fn rect_of(mon: &tauri::window::Monitor) -> Rect {
    let p = mon.position();
    let s = mon.size();
    Rect {
        left: p.x as f64,
        top: p.y as f64,
        right: (p.x + s.width as i32) as f64,
        bottom: (p.y + s.height as i32) as f64,
    }
}

/// Which display a position off the edge of every one of them belongs to: the least far.
fn nearest_monitor(at: (f64, f64), screens: &[Rect]) -> Option<usize> {
    screens
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.distance_to(at).total_cmp(&b.distance_to(at)))
        .map(|(i, _)| i)
}

/// The display the pet is on, or the nearest one when it is on none. `current_monitor` is
/// `NSWindow.screen`, which is nil for a window overlapping no display, stranding a dragged pet.
fn monitor_at(
    win: &tauri::window::Window,
    near: Option<(f64, f64)>,
) -> Option<tauri::window::Monitor> {
    if let Some(mon) = win.current_monitor().ok().flatten() {
        return Some(mon);
    }
    let at = near?;
    let monitors = win.available_monitors().ok()?;
    let rects: Vec<Rect> = monitors.iter().map(rect_of).collect();
    monitors.into_iter().nth(nearest_monitor(at, &rects)?)
}

fn usable_bounds(win: &tauri::window::Window, near: Option<(f64, f64)>) -> Option<Bounds> {
    let mon = monitor_at(win, near)?;
    let scale = mon.scale_factor();
    let area = mon.work_area();
    let left = area.position.x as f64;
    let top = area.position.y as f64;
    let mut right = (area.position.x + area.size.width as i32) as f64;
    let mut bottom = (area.position.y + area.size.height as i32) as f64;

    #[cfg(target_os = "macos")]
    if let Some((vr, vb)) = macos::visible_bottom_right(&mon) {
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

/// The corner nearest to a position, by squared distance. All four corners are on screen by
/// construction, so "nearest" is the only rule and there is no edge case to be wrong in.
fn nearest_index(current: (f64, f64), corners: &[(i32, i32); 4]) -> usize {
    let mut best = 0;
    let mut best_d = f64::INFINITY;
    for (i, &(x, y)) in corners.iter().enumerate() {
        let d = (x as f64 - current.0).powi(2) + (y as f64 - current.1).powi(2);
        if d < best_d {
            best_d = d;
            best = i;
        }
    }
    best
}

/// An anchor against a set of corners: where the pet goes, and which corner that was. The
/// index is what a `Legacy` anchor is upgraded to once bounds exist to resolve it.
fn resolve(anchor: Option<PetAnchor>, corners: &[(i32, i32); 4]) -> ((i32, i32), u8) {
    let i = match anchor {
        Some(PetAnchor::Corner(i)) if (i as usize) < corners.len() => i as usize,
        Some(PetAnchor::Legacy(x, y)) => nearest_index((x as f64, y as f64), corners),
        _ => 3,
    };
    (corners[i], i as u8)
}

/// The saved corner against the live bounds, or the bottom-right default. Absolute pixels are
/// derived here and never stored, which is the whole point: see section 13.
///
/// **`outer_position()` read straight after `set_position` returns the OLD position**, and
/// anyone checking this function's work needs to know that before they start. It reports the
/// macOS default, which is near the middle of the display, and it keeps reporting it however
/// many times the call is repeated. A few hundred milliseconds later the same read returns the
/// corner. This cost an entire wrong diagnosis, a restructure of `main.rs` around
/// `RunEvent::Ready`, and a fix for a bug that did not exist: placement worked correctly the
/// whole time and the measurement was lying. **Read the position from a delayed thread, or do
/// not read it.**
fn place(win: &tauri::window::Window, app: &AppHandle) -> tauri::Result<()> {
    // Its own position as the fallback hint: once a display is unplugged the window overlaps
    // none, `current_monitor` is nil, and without this `usable_bounds` gives up on the pet.
    let at = win
        .outer_position()
        .ok()
        .map(|p| (p.x as f64, p.y as f64));
    let Some(b) = usable_bounds(win, at) else {
        return Ok(());
    };
    let corners = anchors(&b);

    let state = app.state::<AppState>();
    let (target, _) = {
        let mut momentum = state.momentum.lock().unwrap();
        let resolved = resolve(momentum.state.pet_anchor, &corners);
        momentum.state.pet_anchor = Some(PetAnchor::Corner(resolved.1));
        resolved
    };

    win.set_position(PhysicalPosition::new(target.0, target.1))?;
    Ok(())
}

/// Put the pet back on its corner after the displays changed underneath it. `place` otherwise
/// runs only at creation, so an unplugged screen would strand the window where it used to be.
pub fn replace(app: &AppHandle) {
    if let Some(win) = app.get_window(PET) {
        if let Err(e) = place(&win, app) {
            eprintln!("could not re-place the pet after a display change: {e}");
        }
    }
}

/// The corner nearest `current`, without moving the window.
///
/// `current` is the pet's top-left in physical pixels, handed over by the webview rather than
/// read back here, because the webview is the one that just moved it and its own last word on
/// where it is is more recent than anything `outer_position()` would report. The backend owns
/// the geometry (the display, the Dock-aware bounds, the anchors) and the motion; the frontend
/// owns the pointer. Called once on drag end.
pub fn nearest_corner(
    win: &tauri::window::Window,
    current: (f64, f64),
) -> Option<((i32, i32), u8)> {
    let b = usable_bounds(win, Some(current))?;
    let corners = anchors(&b);
    let i = nearest_index(current, &corners);
    Some((corners[i], i as u8))
}

/// Bumped whenever a glide starts or is cancelled. A running glide checks it against the value
/// it started under and stops the moment it is no longer the latest, so the tail of an old
/// glide can never fight a newer drag.
static GLIDE_GENERATION: AtomicU64 = AtomicU64::new(0);

/// Glide the window from `from` to `to`, both physical pixels, in **two phases**: a quick move to
/// the target corner's edge, then a slower horizontal run along that edge into the corner.
///
/// The single ease-out this replaced covered the whole diagonal in ~250ms, which reads as the pet
/// being flung at the corner rather than travelling there. Splitting it means the run animation
/// the drag already switched on has something to do: phase two is horizontal, at a walking-ish
/// speed, along the bottom or top edge, which is what a side-view run sprite is for. Phase one is
/// mostly vertical and stays fast, so the whole thing still resolves promptly.
///
/// The motion is driven from a thread here rather than from the webview, because the pet's window
/// is never focused and WebKit throttles the webview's own timers for exactly that window. That
/// reason is now historical, since there is no webview, but a thread is still the right shape: the
/// alternative is blocking the main thread for the length of the glide.
///
/// Both phases check the generation, so a new drag cuts the glide short wherever it has got to.
/// The final step of phase two lands exactly on `to`, so the corner is reached even if an earlier
/// step was coalesced away.
///
/// Landing calls `end_run`, and only landing does: a glide cut short by `cancel_glide` stays
/// silent, so a re-grab does not get its run cut short out from under it. That replaces the
/// `glide-done` event, whose only listener was `pet.js`.
pub fn glide_to(app: &AppHandle, win: &tauri::window::Window, from: (f64, f64), to: (i32, i32)) {
    let generation = GLIDE_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    let win = win.clone();
    let app = app.clone();
    std::thread::spawn(move || {
        const STEP: Duration = Duration::from_millis(16);

        // Phase one: to the corner's own edge, keeping the x the drag left it at. Fast, and
        // ease-out so it settles rather than stopping dead.
        let waypoint = (from.0, to.1 as f64);
        const APPROACH_STEPS: u32 = 12;
        for i in 1..=APPROACH_STEPS {
            if GLIDE_GENERATION.load(Ordering::SeqCst) != generation {
                return;
            }
            let t = i as f64 / APPROACH_STEPS as f64;
            let eased = 1.0 - (1.0 - t).powi(3);
            let x = from.0 + (waypoint.0 - from.0) * eased;
            let y = from.1 + (waypoint.1 - from.1) * eased;
            if win.set_position(PhysicalPosition::new(x, y)).is_err() {
                return;
            }
            std::thread::sleep(STEP);
        }

        // Phase two: the run along the edge. Duration comes from the distance rather than being
        // fixed, so a long way to travel actually takes longer, which is what makes it read as
        // running rather than sliding. Clamped at both ends: below the floor there is no run to
        // see, above the ceiling the user is waiting on an animation.
        const RUN_PX_PER_SEC: f64 = 900.0;
        const MIN_RUN: Duration = Duration::from_millis(280);
        const MAX_RUN: Duration = Duration::from_millis(1500);
        let distance = (to.0 as f64 - waypoint.0).abs();
        let run = Duration::from_secs_f64(distance / RUN_PX_PER_SEC).clamp(MIN_RUN, MAX_RUN);
        let steps = ((run.as_secs_f64() / STEP.as_secs_f64()).round() as u32).max(1);
        for i in 1..=steps {
            if GLIDE_GENERATION.load(Ordering::SeqCst) != generation {
                return;
            }
            // Linear, unlike phase one. A run at a steady pace is the point; easing it would
            // make the character appear to slow down without its legs slowing down with it.
            let t = i as f64 / steps as f64;
            let x = waypoint.0 + (to.0 as f64 - waypoint.0) * t;
            if win
                .set_position(PhysicalPosition::new(x, to.1 as f64))
                .is_err()
            {
                return;
            }
            std::thread::sleep(STEP);
        }

        // Runs on the glide thread, so it hops. Only a glide that COMPLETES reaches this line:
        // one cancelled by a newer drag returned above, so a re-grab does not get its run cut
        // short. That was the contract of the `glide-done` event and it is unchanged.
        end_run(&app);
    });
}

/// Cancel any glide still in flight. Called when a new drag begins, before the cursor has a
/// chance to be fought by the previous glide's remaining steps.
pub fn cancel_glide() {
    GLIDE_GENERATION.fetch_add(1, Ordering::SeqCst);
}

/// The pet's sprite view, which is main-thread-only. Held here rather than in `AppState` so the
/// main-thread rule lives next to the code that has to honour it.
#[cfg(target_os = "macos")]
static SPRITE: std::sync::OnceLock<crate::sprite::SpriteHandle> = std::sync::OnceLock::new();

/// Tell the pet what mood to show. Safe to call from any thread.
pub fn set_mood(app: &AppHandle, mood: &str, character_id: &str) {
    #[cfg(target_os = "macos")]
    {
        let mood = mood.to_string();
        let character_id = character_id.to_string();
        let _ = app.run_on_main_thread(move || {
            if let Some(handle) = SPRITE.get() {
                handle.set_mood(&mood, &character_id);
            }
        });
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, mood, character_id);
    }
}

/// The glide landed, so the pet stops running. Safe to call from any thread.
pub fn end_run(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    {
        let _ = app.run_on_main_thread(|| {
            if let Some(handle) = SPRITE.get() {
                handle.end_run();
            }
        });
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = app;
    }
}

/// The pet was clicked rather than dragged.
pub fn on_click(app: &AppHandle) {
    crate::app::toggle_popover(app, crate::app::OpenedBy::Pet);
}

/// Follow the cursor during a drag. Physical pixels.
pub fn move_to(app: &AppHandle, to: (f64, f64)) {
    if let Some(win) = app.get_window(PET) {
        let _ = win.set_position(PhysicalPosition::new(to.0, to.1));
    }
}

/// The drag ended: resolve the nearest corner, glide there, remember it, and report the corner so
/// the view can face the run that way. This is `commands::snap_pet` minus the command wrapper.
pub fn on_drag_end(app: &AppHandle, at: (f64, f64)) -> Option<(i32, i32)> {
    let win = app.get_window(PET)?;
    let (target, corner) = nearest_corner(&win, at)?;
    glide_to(app, &win, at, target);

    let state = app.state::<AppState>();
    let to_save = {
        let mut momentum = state.momentum.lock().unwrap();
        momentum.state.pet_anchor = Some(PetAnchor::Corner(corner));
        momentum.state.clone()
    };
    if let Err(e) = crate::store::save(&state.store_path, &to_save) {
        eprintln!("could not write state: {e}");
    }
    Some(target)
}

#[cfg(target_os = "macos")]
mod macos {
    use objc2_app_kit::NSScreen;
    use objc2_foundation::MainThreadMarker;

    /// The bottom-right of `mon`'s `visibleFrame`, in Tauri's physical pixels.
    ///
    /// AppKit measures from the bottom-left of the *primary* screen with y increasing
    /// upwards; Tauri measures from the top-left with y increasing downwards. The flip needs
    /// the primary screen's height, which is the first entry in `NSScreen.screens`.
    ///
    /// Matched to `mon` by origin, not by name: tao names every display `Monitor #<model>`.
    pub fn visible_bottom_right(mon: &tauri::window::Monitor) -> Option<(f64, f64)> {
        let mtm = MainThreadMarker::new()?;
        let screens = NSScreen::screens(mtm);
        let primary_height = screens.iter().next()?.frame().size.height;
        // tao scales `CGDisplayBounds` by the display's own backing scale, so every point
        // below is scaled the same way and the comparison stays in tao's units.
        let scale = mon.scale_factor();
        let want = mon.position();
        let screen = screens.iter().find(|s| {
            let f = s.frame();
            let left = f.origin.x * scale;
            let top = (primary_height - (f.origin.y + f.size.height)) * scale;
            (left - want.x as f64).abs() < 1.0 && (top - want.y as f64).abs() < 1.0
        })?;

        let visible = screen.visibleFrame();
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
        assert_eq!(nearest_index((10.0, 10.0), &c), 0);
        assert_eq!(nearest_index((1900.0, 10.0), &c), 1);
        assert_eq!(nearest_index((10.0, 1050.0), &c), 2);
        assert_eq!(nearest_index((1900.0, 1050.0), &c), 3);
    }

    fn screen(left: f64, top: f64, right: f64, bottom: f64) -> Bounds {
        Bounds {
            left,
            top,
            right,
            bottom,
            extent: 64.0,
            margin: 20.0,
        }
    }

    /// The lid-closed case, in the arrangement it was measured in: a laptop bottom-aligned with
    /// an ultrawide, then unplugged. No bounds check catches it; the stale point is on screen.
    #[test]
    fn a_corner_outlives_the_display_it_was_chosen_on() {
        let laptop = screen(88.0, 436.0, 1600.0, 1418.0);
        let dell = screen(0.0, 0.0, 3360.0, 1418.0);
        let stale = anchors(&laptop)[3];

        assert_eq!(stale, (1516, 1334), "the position the pet was found at");
        assert!(
            stale.0 as f64 >= dell.left && (stale.0 as f64) + dell.extent <= dell.right,
            "still over the surviving display, so no off-screen check could catch it"
        );
        assert_ne!(stale, anchors(&dell)[3]);

        assert_eq!(
            resolve(Some(PetAnchor::Corner(3)), &anchors(&dell)),
            ((3276, 1334), 3)
        );
    }

    #[test]
    fn an_anchor_resolves_to_a_corner_and_the_index_it_landed_on() {
        let c = anchors(&b());
        assert_eq!(
            resolve(None, &c),
            ((1836, 996), 3),
            "the default is bottom right"
        );
        assert_eq!(resolve(Some(PetAnchor::Corner(0)), &c), ((20, 20), 0));
        assert_eq!(
            resolve(Some(PetAnchor::Corner(9)), &c),
            ((1836, 996), 3),
            "out of range"
        );
        assert_eq!(
            resolve(Some(PetAnchor::Legacy(10, 1050)), &c),
            ((20, 996), 2),
            "a pre-3.3 absolute position takes the corner nearest it"
        );
    }

    /// A wide display at the origin with a narrower one below it, inset on both sides. The
    /// inset is what leaves dead space between them for a pet to be stranded in.
    fn two_screens() -> [Rect; 2] {
        [
            Rect {
                left: 0.0,
                top: 0.0,
                right: 6720.0,
                bottom: 2836.0,
            },
            Rect {
                left: 1760.0,
                top: 2836.0,
                right: 4960.0,
                bottom: 4836.0,
            },
        ]
    }

    #[test]
    fn a_point_on_a_display_is_no_distance_from_it() {
        let s = two_screens();
        assert_eq!(s[0].distance_to((100.0, 100.0)), 0.0);
        assert_eq!(s[1].distance_to((2000.0, 3000.0)), 0.0);
        assert_eq!(nearest_monitor((100.0, 100.0), &s), Some(0));
        assert_eq!(nearest_monitor((2000.0, 3000.0), &s), Some(1));
    }

    #[test]
    fn a_pet_dragged_off_the_bottom_edge_belongs_to_the_display_it_left() {
        let s = two_screens();
        // Below the wide display's bottom edge, right of where the narrow one ends.
        assert_eq!(nearest_monitor((6162.0, 2952.0), &s), Some(0));
        // The mirror case, off the narrow display's own bottom edge.
        assert_eq!(nearest_monitor((3000.0, 5000.0), &s), Some(1));
    }
}
