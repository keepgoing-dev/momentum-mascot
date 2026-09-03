//! Wiring. Everything with real logic in it lives in the modules this one calls.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::clock::Clock;
use crate::momentum::Momentum;
use crate::popover::{self, Rect};
use crate::watcher::{ChangeEvent, Watcher};

pub const POPOVER: &str = "popover";
pub const PET: &str = "pet";

/// The one event the frontend listens to. Both windows get it, and both decide for
/// themselves what to do with it: the room shows the scene, the pet shows the character.
pub const MOOD_EVENT: &str = "mood";

pub struct AppState {
    pub momentum: Mutex<Momentum>,
    pub clock: Clock,
    pub store_path: PathBuf,
    pub watcher: Mutex<Option<Watcher>>,
    pub popover_hidden_at: Mutex<Option<std::time::Instant>>,
    /// Set while the app's own folder picker is on screen. See `picker_is_open`.
    pub picker_open: AtomicBool,
    /// The last mood announced, so the log can say what changed rather than repeating itself
    /// once a minute.
    pub last_published: Mutex<Option<crate::mood::Mood>>,
}

#[derive(Serialize, Clone)]
pub struct MoodPayload {
    pub mood: crate::mood::Mood,
    pub quote: String,
    pub character_id: String,
    pub custom_character: Option<CustomCharacterDto>,
    /// Whether the nine built-mascot strips are on disk. The id alone cannot say, so the
    /// popover falls back to a premade rather than painting a room with nobody in it.
    pub custom_art_ready: bool,
    pub projects: Vec<ProjectDto>,
    /// Only ever anything but 1.0 in a debug build with the demo clock running. The pet uses
    /// it for nothing; it is here so the popover can show that the clock is scaled, because
    /// a recording made against a scaled clock should say so on screen while it is being
    /// made rather than in a caption afterwards.
    pub clock_scale: f64,
}

#[derive(Serialize, Clone)]
pub struct CustomCharacterDto {
    pub skin: String,
    pub eyes: String,
    pub outfit: String,
    pub hair: String,
    pub accessory: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct ProjectDto {
    pub id: String,
    pub name: String,
    pub relative: String,
    pub available: bool,
    pub operating: bool,
    /// Why it is unavailable, when there is something specific to say. The popover prefers this
    /// over its own generic line.
    pub reason: Option<&'static str>,
}

impl AppState {
    pub fn new(store_path: PathBuf) -> Self {
        let clock = Clock::from_env();
        AppState {
            momentum: Mutex::new(Momentum::load(&store_path, clock)),
            clock,
            store_path,
            watcher: Mutex::new(None),
            popover_hidden_at: Mutex::new(None),
            picker_open: AtomicBool::new(false),
            last_published: Mutex::new(None),
        }
    }
}

/// Take a reading of the world and tell both windows about it.
///
/// Every path that can change the mood ends here: the watcher, the tick, adding a project,
/// untracking one, and opening the popover. One function means one place where state is
/// persisted and one place where the UI is told, rather than five that drift apart.
pub fn publish(app: &AppHandle) {
    let state = app.state::<AppState>();
    let now = state.clock.now();
    let real_now = crate::clock::real_unix_now();

    let (payload, to_save) = {
        let mut momentum = state.momentum.lock().unwrap();
        let snap = momentum.snapshot(now, real_now);
        let payload = MoodPayload {
            mood: snap.mood,
            quote: snap.quote.to_string(),
            character_id: snap.character_id,
            custom_character: snap.custom_character.map(|c| CustomCharacterDto {
                skin: c.body,
                eyes: c.eyes,
                outfit: c.outfit,
                hair: c.hair,
                accessory: c.accessory,
            }),
            // Whether the nine strips are actually on disk, which the id alone cannot say.
            custom_art_ready: crate::custom::has_art(&crate::custom::dir(&state.store_path)),
            projects: snap
                .projects
                .into_iter()
                .map(|p| ProjectDto {
                    id: p.id,
                    name: p.name,
                    relative: p.relative,
                    available: p.available,
                    operating: p.operating,
                    reason: p.reason,
                })
                .collect(),
            clock_scale: state.clock.scale(),
        };
        (payload, momentum.state.clone())
    };

    if let Err(e) = crate::store::save(&state.store_path, &to_save) {
        // A failed write costs continuity across a restart, not the running app.
        eprintln!("could not write state: {e}");
    }

    // One line per actual change, never per tick. This is the only window onto the state
    // machine from outside, and it is what makes a transition verifiable under an accelerated
    // clock instead of merely assumed: the demo in Phase 4 is recorded against this same
    // machine, so it should be able to say what it is doing while it does it.
    {
        let mut last = state.last_published.lock().unwrap();
        if *last != Some(payload.mood) {
            println!("{} -> {}", elapsed_hours(&state, now), payload.mood.as_str());
            *last = Some(payload.mood);
        }
    }

    // `MOOD_EVENT` stays: `src/popover.js` is still a listener for it.
    let mood = payload.mood.as_str();
    let character_id = payload.character_id.clone();
    let _ = app.emit(MOOD_EVENT, payload);

    // The pet has no webview to listen any more, so it is told directly. Both direct callers
    // arrive off the main thread: the tick runs on `start_tick`'s thread and the watcher on its
    // own, and touching an NSView off the main thread crashes. `app.emit` marshalled for free;
    // a direct setter does not, which is why `pet::set_mood` hops.
    crate::pet::set_mood(app, mood, &character_id);
}

/// How long since the newest commit anywhere, in simulated hours. The only number worth
/// printing, because it is the only number the state machine actually reads.
fn elapsed_hours(state: &AppState, now: i64) -> String {
    match state.momentum.lock().unwrap().latest() {
        Some(latest) => format!("{:>7.2}h", (now - latest) as f64 / 3600.0),
        None => "      -".to_string(),
    }
}

/// Re-read every tracked project, then publish. This is what a filesystem event and the
/// startup pass both call.
pub fn refresh(app: &AppHandle) {
    {
        let state = app.state::<AppState>();
        let mut momentum = state.momentum.lock().unwrap();
        let changed = momentum.refresh_all();
        if crate::watcher::debugging() {
            eprintln!(
                "refresh: changed={changed} latest={:?} now={}",
                momentum.latest(),
                state.clock.now()
            );
        }
    }
    publish(app);
}

pub fn sync_watcher(app: &AppHandle) {
    let state = app.state::<AppState>();
    let (git_dirs, work_trees) = state.momentum.lock().unwrap().watch_paths();
    let mut watcher = state.watcher.lock().unwrap();
    if let Some(w) = watcher.as_mut() {
        w.sync(&git_dirs, &work_trees);
    }
}

/// The popover's window chrome, which the app owns now.
///
/// Two of the three calls are cosmetic and were previously done for us by the private-API feature
/// or not needed at all. `transparent: true` is gone from the window's config: the room art fills
/// the whole surface, so the popover never needed a see-through webview. What it needed was
/// rounded corners, and those come from the layer.
///
/// The third changes what kind of window this is, and has to run before the first `show`.
#[cfg(target_os = "macos")]
pub fn setup_popover(app: &AppHandle) {
    let Some(win) = app.get_webview_window(POPOVER) else {
        return;
    };
    if let Ok(ns) = win.ns_window() {
        // Section 9 item 1, found by eye and not by test: the pet showed over a fullscreen app
        // and the popover did not, which is worse than neither working, because the pet is still
        // clickable and clicking it looks like the app is broken. `alwaysOnTop` in the window
        // config is a *level*, and the spike proved no level is enough.
        //
        // This runs in `setup`, so it is still the ordering the spike requires: the window is
        // created from `tauri.conf.json` with `visible: false`, configured here, and shown for
        // the first time later. Nothing reconfigures it afterwards.
        //
        // `true`, unlike the pet: the popover has to be able to take the keyboard, because
        // Escape dismisses it through a JS `keydown`.
        if !crate::appkit::show_over_fullscreen(ns, true) {
            eprintln!("NSPanel class not found; the popover will not show over fullscreen apps");
        }
        crate::appkit::make_transparent(ns);
        // 12pt, matching `.panel`'s `border-radius: 12px` in popover.css. If these ever disagree,
        // the border draws a different curve than the mask cuts.
        crate::appkit::round_corners(ns, 12.0);
    }
}

#[cfg(not(target_os = "macos"))]
pub fn setup_popover(_app: &AppHandle) {}

/// Which surface opened the popover, and therefore what the panel hangs off.
///
/// It used to always be the tray icon, which is what the spec says (section 6.3) and was
/// written when the tray icon was the only way in. The pet came later and is now the primary
/// entry point (section 6.1), and on a two-display desktop the two are not always on the same
/// screen: macOS moves the menu bar's status items to whichever display is active, so clicking
/// the pet in the corner of one display opened the popover on the other one.
#[derive(Clone, Copy)]
pub enum OpenedBy {
    Pet,
    Tray,
}

/// What the popover hangs off and the work area of the display that thing is on, both in
/// physical pixels, plus that display's scale so the gap is a fixed number of points.
fn anchor(app: &AppHandle, by: OpenedBy) -> Option<(Rect, Rect, f64)> {
    let anchor = match by {
        OpenedBy::Pet => {
            let win = app.get_window(PET)?;
            let at = win.outer_position().ok()?;
            let size = win.outer_size().ok()?;
            Rect::new(at.x as f64, at.y as f64, size.width as f64, size.height as f64)
        }
        OpenedBy::Tray => {
            let (x, y, w, h) = crate::tray::rect(app)?;
            Rect::new(x, y, w, h)
        }
    };
    let monitor = monitor_holding(app, anchor)?;
    let area = monitor.work_area();
    let area = Rect::new(
        area.position.x as f64,
        area.position.y as f64,
        area.size.width as f64,
        area.size.height as f64,
    );
    Some((anchor, area, monitor.scale_factor()))
}

/// The display an anchor is on, found by its centre. The centre rather than a corner because
/// the tray icon's own top edge is the display's top edge, and a point exactly on a boundary
/// belongs to whichever display the runtime feels like naming.
fn monitor_holding(app: &AppHandle, anchor: Rect) -> Option<tauri::Monitor> {
    let (cx, cy) = (anchor.x + anchor.w / 2.0, anchor.y + anchor.h / 2.0);
    app.available_monitors()
        .ok()?
        .into_iter()
        .find(|m| {
            let (at, size) = (m.position(), m.size());
            Rect::new(at.x as f64, at.y as f64, size.width as f64, size.height as f64)
                .contains(cx, cy)
        })
        .or_else(|| app.primary_monitor().ok().flatten())
}

/// Show the popover, hung off whichever surface opened it.
pub fn show_popover(app: &AppHandle, by: OpenedBy) {
    let Some(win) = app.get_webview_window(POPOVER) else {
        return;
    };
    let state = app.state::<AppState>();

    // The quote rotates on open, so two consecutive looks are not the same line. The comeback
    // is deliberately NOT resolved here: section 4.5 puts the resolution on close, because this
    // function ends in `publish`, and resolving first would evaluate the room as `awake` and
    // render the celebration away in the same call that was supposed to show it.
    {
        let mut momentum = state.momentum.lock().unwrap();
        momentum.next_quote();
    }

    // Anchored to whichever surface was clicked, asked for every time. Without this the window
    // keeps whatever `tauri.conf.json` gave it, which is the centre of the screen.
    if let (Some((anchor, area, scale)), Ok(size)) = (anchor(app, by), win.outer_size()) {
        let size = (size.width as f64, size.height as f64);
        let (x, y) = popover::anchored(anchor, size, area, 6.0 * scale);
        let _ = win.set_position(tauri::PhysicalPosition::new(x, y));
    }

    let _ = win.show();
    let _ = win.set_focus();
    publish(app);
}

/// The popover just closed. Remember when, and settle the comeback.
///
/// Clicking the tray icon while the popover is open produces two events in this order: the
/// window loses focus and hides itself, then the click arrives and asks to toggle. Without a
/// memory of the first, the second reopens what the user was closing, and the popover appears
/// to be un-closable. A short window is enough, because the two events are consecutive.
///
/// Closing is also where the comeback resolves (section 4.5: "the user sees the full-room
/// celebration, and on close it settles into `awake`"). Every way the popover goes away comes
/// through here, which is the reason the resolution lives in this function rather than at the
/// three call sites.
pub fn note_popover_hidden(app: &AppHandle) {
    let state = app.state::<AppState>();
    *state.popover_hidden_at.lock().unwrap() = Some(std::time::Instant::now());
    state.momentum.lock().unwrap().resolve_comeback();
}

/// Whether the popover has been pinned open for a screenshot.
///
/// Reads `KEEPGOING_PIN_POPOVER`, and **only in a debug build**, for the same reason as the
/// clock and the state path: a release binary ignores it, so a popover that will not dismiss
/// cannot ship.
///
/// The problem it solves has no solution from outside the app. The popover closes when it
/// loses focus, which is correct, and every way of triggering a screen capture takes the focus
/// first: the shift-cmd-5 panel is an app, and a capture invoked from a terminal means the
/// terminal is frontmost. So the App Store shots of the popover were not merely awkward to
/// take, they were unobtainable, and the comeback one doubly so because closing the popover
/// also resolves the celebration.
///
/// Escape and the tray icon still close it, so there is always a way out.
pub fn popover_is_pinned() -> bool {
    cfg!(debug_assertions) && std::env::var_os("KEEPGOING_PIN_POPOVER").is_some()
}

/// Whether the app's own folder picker is on screen, in which case a focus loss on the popover
/// is not a click outside and must not close it.
///
/// The picker is a macOS *sheet*, and a sheet belongs to a window: closing that window takes the
/// sheet off screen with it. So the click-outside rule, applied to the focus loss the picker
/// itself causes, hid the popover and made the folder picker appear to close itself the instant
/// it opened, with Add Project consequently doing nothing at all.
pub fn picker_is_open(app: &AppHandle) -> bool {
    app.state::<AppState>().picker_open.load(Ordering::SeqCst)
}

/// Marks the picker as open for as long as this value is alive.
///
/// A guard rather than two calls, because the flag has to be cleared on every way out of the add
/// flow, including the two that return an error, and a missed one leaves a popover that can no
/// longer be dismissed.
pub struct PickerGuard(AppHandle);

impl PickerGuard {
    pub fn new(app: &AppHandle) -> Self {
        app.state::<AppState>()
            .picker_open
            .store(true, Ordering::SeqCst);
        PickerGuard(app.clone())
    }
}

impl Drop for PickerGuard {
    fn drop(&mut self) {
        self.0
            .state::<AppState>()
            .picker_open
            .store(false, Ordering::SeqCst);
    }
}

const REOPEN_GUARD: std::time::Duration = std::time::Duration::from_millis(250);

pub fn toggle_popover(app: &AppHandle, by: OpenedBy) {
    // The other way into the same bug: the pet and the tray icon are still clickable while the
    // picker is up, and hiding the popover from here would take the sheet with it.
    if picker_is_open(app) {
        return;
    }

    let Some(win) = app.get_webview_window(POPOVER) else {
        return;
    };
    if win.is_visible().unwrap_or(false) {
        let _ = win.hide();
        note_popover_hidden(app);
        return;
    }

    let state = app.state::<AppState>();
    let mut hidden_at = state.popover_hidden_at.lock().unwrap();
    if hidden_at.is_some_and(|t| t.elapsed() < REOPEN_GUARD) {
        *hidden_at = None;
        return;
    }
    drop(hidden_at);
    show_popover(app, by);
}

/// The 60-simulated-second tick (section 8.2).
///
/// Two sources move the state and both must be handled: a commit landing, which is
/// event-driven, and **time passing**, which produces no event at all. The second one is
/// easy to forget and is the more common transition in practice.
pub fn start_tick(app: AppHandle) {
    let interval = app.state::<AppState>().clock.tick_interval();
    std::thread::spawn(move || loop {
        std::thread::sleep(interval);
        publish(&app);
    });
}

pub fn start_watcher(app: AppHandle) {
    let handle = app.clone();
    match Watcher::new(move |event| on_change(&handle, event)) {
        Ok(w) => {
            *app.state::<AppState>().watcher.lock().unwrap() = Some(w);
            sync_watcher(&app);
        }
        Err(e) => eprintln!("file watching is unavailable, falling back to the tick: {e}"),
    }
}

fn on_change(app: &AppHandle, event: ChangeEvent) {
    match event {
        ChangeEvent::ReflogChanged => refresh(app),
        ChangeEvent::TreeChanged(id) => {
            let state = app.state::<AppState>();
            let now = state.clock.now();
            {
                let mut momentum = state.momentum.lock().unwrap();
                let changed = momentum.touch_activity(&id, now);
                if crate::watcher::debugging() {
                    eprintln!(
                        "tree changed: id={id} changed={changed} latest={:?} now={}",
                        momentum.latest(),
                        now
                    );
                }
            }
            // Publish even if the timestamp did not move: the UI may need to refresh for
            // other reasons, and publish is cheap and idempotent.
            publish(app);
        }
    }
}
