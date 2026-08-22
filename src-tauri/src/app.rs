//! Wiring. Everything with real logic in it lives in the modules this one calls.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::clock::Clock;
use crate::momentum::Momentum;
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
    /// Where the tray icon last reported itself, so the popover can be anchored to it.
    pub tray_rect: Mutex<Option<(f64, f64, f64, f64)>>,
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
    pub projects: Vec<ProjectDto>,
    /// Only ever anything but 1.0 in a debug build with the demo clock running. The pet uses
    /// it for nothing; it is here so the popover can show that the clock is scaled, because
    /// a recording made against a scaled clock should say so on screen while it is being
    /// made rather than in a caption afterwards.
    pub clock_scale: f64,
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
            tray_rect: Mutex::new(None),
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

    let _ = app.emit(MOOD_EVENT, payload);
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
/// Two calls, both public AppKit, both previously done for us by the private-API feature or not
/// needed at all. `transparent: true` is gone from the window's config: the room art fills the
/// whole surface, so the popover never needed a see-through webview. What it needed was rounded
/// corners, and those come from the layer.
#[cfg(target_os = "macos")]
pub fn setup_popover(app: &AppHandle) {
    let Some(win) = app.get_webview_window(POPOVER) else {
        return;
    };
    if let Ok(ns) = win.ns_window() {
        crate::appkit::make_transparent(ns);
        // 12pt, matching `.panel`'s `border-radius: 12px` in popover.css. If these ever disagree,
        // the border draws a different curve than the mask cuts.
        crate::appkit::round_corners(ns, 12.0);
    }
}

#[cfg(not(target_os = "macos"))]
pub fn setup_popover(_app: &AppHandle) {}

/// Show the popover under the tray icon.
pub fn show_popover(app: &AppHandle) {
    let Some(win) = app.get_webview_window(POPOVER) else {
        return;
    };
    let state = app.state::<AppState>();

    // Opening the popover is what resolves the comeback (section 4.5), and it is also when
    // the quote rotates, so two consecutive looks are not the same line.
    {
        let mut momentum = state.momentum.lock().unwrap();
        momentum.resolve_comeback();
        momentum.next_quote();
    }

    // Anchored to the tray icon, which is the only reason the tray rect is kept at all.
    if let Some((x, y, w, h)) = *state.tray_rect.lock().unwrap() {
        if let Ok(size) = win.outer_size() {
            let gap = 6.0;
            let left = x + w / 2.0 - size.width as f64 / 2.0;
            let top = y + h + gap;
            let _ = win.set_position(tauri::PhysicalPosition::new(left.max(8.0), top));
        }
    }

    let _ = win.show();
    let _ = win.set_focus();
    publish(app);
}

/// Remember that the popover just closed itself.
///
/// Clicking the tray icon while the popover is open produces two events in this order: the
/// window loses focus and hides itself, then the click arrives and asks to toggle. Without a
/// memory of the first, the second reopens what the user was closing, and the popover appears
/// to be un-closable. A short window is enough, because the two events are consecutive.
pub fn note_popover_hidden(app: &AppHandle) {
    *app.state::<AppState>().popover_hidden_at.lock().unwrap() = Some(std::time::Instant::now());
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

pub fn toggle_popover(app: &AppHandle) {
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
    show_popover(app);
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
