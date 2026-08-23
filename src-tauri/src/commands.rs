//! Everything the popover is allowed to ask for. Nine commands, and no tenth without a
//! reason: this list is the whole API surface between the art and the machinery.
//!
//! Three left when the pet stopped being a webview: `snap_pet` and `cancel_glide`, whose only
//! caller was `pet.js`'s drag, and `toggle_popover`. That third one is the non-obvious one, since
//! `tray.rs:50` still opens the popover: it calls the Rust function `app::toggle_popover`
//! directly, so deleting `pet.js` left the *command* wrapper callerless.

use tauri::{AppHandle, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_dialog::DialogExt;

use crate::app::{self, AppState};
use crate::appkit;
use crate::scoped;
use crate::store;

/// Ask for the current mood without waiting for the next event. Called once when a window
/// finishes loading, because a window that opens between ticks would otherwise be blank.
#[tauri::command]
pub fn refresh(app: AppHandle) {
    app::publish(&app);
}

#[tauri::command]
pub fn hide_popover(app: AppHandle) {
    if let Some(win) = app.get_webview_window(app::POPOVER) {
        let _ = win.hide();
    }
}

/// Section 7's add flow: native folder picker, validate, append, watch, re-evaluate.
///
/// On failure this returns a short line that the popover shows inline. No modal and no alert
/// dialog: the app has one surface, and an error is a sentence in it.
#[tauri::command]
pub async fn add_project(app: AppHandle) -> Result<(), String> {
    // The popover must stay open for as long as the picker is up: the picker is a sheet on it,
    // and a sheet does not outlive the window it is attached to (`app::picker_is_open`).
    let _picker = app::PickerGuard::new(&app);

    let (tx, rx) = std::sync::mpsc::channel();
    let mut dialog = app.dialog().file();
    // Named explicitly rather than left to the dialog crate, which otherwise picks whichever
    // window happens to be macOS's main one and would hang the sheet off the pet, or off the
    // status bar, depending on what was focused when.
    if let Some(popover) = app.get_webview_window(app::POPOVER) {
        dialog = dialog.set_parent(&popover);
    }
    dialog.pick_folder(move |picked| {
        let _ = tx.send(picked);
    });
    // The picker answers on the main thread, so the wait for it must not happen there.
    let picked = tauri::async_runtime::spawn_blocking(move || rx.recv().ok().flatten())
        .await
        .map_err(|_| "The folder picker didn't come back.".to_string())?;

    let Some(folder) = picked else { return Ok(()) };
    let path = folder
        .into_path()
        .map_err(|_| "That folder has a path this app can't read.".to_string())?;

    // Created here and nowhere else: the bookmark has to be made while the picker's grant is
    // live, and this is the only moment the app knows it is. A failure costs the bookmark, not
    // the project.
    let bookmark = scoped::create(&path);

    let state = app.state::<AppState>();
    let now = state.clock.now();
    state
        .momentum
        .lock()
        .unwrap()
        .add(&path, now, bookmark)
        .map_err(|e| e.to_string())?;

    app::sync_watcher(&app);
    app::refresh(&app);
    Ok(())
}

/// The hover-`x`. It earns its place because the alternative is hand-editing JSON.
#[tauri::command]
pub fn untrack(app: AppHandle, id: String) {
    app.state::<AppState>().momentum.lock().unwrap().remove(&id);
    app::sync_watcher(&app);
    app::publish(&app);
}

/// Toggle whether a project is in operating mode. Operating projects are excluded from the
/// mascot's mood evaluation and show an "operating" label in the project list.
#[tauri::command]
pub fn toggle_operating(app: AppHandle, id: String) -> Option<bool> {
    let new_state = app.state::<AppState>().momentum.lock().unwrap().toggle_operating(&id);
    app::publish(&app);
    new_state
}

/// Clicking the character cycles to the next of the three. This is the original selection
/// mechanism, kept alongside the visible picker so the room itself still responds to a click.
#[tauri::command]
pub fn cycle_character(app: AppHandle) {
    app.state::<AppState>()
        .momentum
        .lock()
        .unwrap()
        .cycle_character();
    app::publish(&app);
}

/// The visible character picker sets the mascot directly. Unknown ids are ignored rather than
/// relaxed, because the only valid characters are the three shipped ones.
#[tauri::command]
pub fn set_character(app: AppHandle, id: String) {
    let state = app.state::<AppState>();
    let mut momentum = state.momentum.lock().unwrap();
    if store::CHARACTERS.contains(&id.as_str()) {
        momentum.state.character_id = id;
    }
    drop(momentum);
    app::publish(&app);
}

/// Share Status. The webview draws the 1200x630 card on a canvas and hands over the PNG
/// bytes; this puts them on the clipboard as a real image rather than a file or a data URL,
/// because a real image is the only thing that pastes into a chat app and a social composer
/// without negotiation (section 5.1).
#[tauri::command]
pub fn copy_share_card(app: AppHandle, png: Vec<u8>) -> Result<(), String> {
    let image = tauri::image::Image::from_bytes(&png).map_err(|e| e.to_string())?;
    app.clipboard().write_image(&image).map_err(|e| e.to_string())
}

/// The privacy policy, opened in the user's browser.
///
/// Narrow on purpose: it takes no URL. Guideline 5.1.1(i) wants the policy reachable from inside
/// the app, and an `open_url(url)` command would hand the webview the ability to open anything,
/// which is a larger API than the requirement. One constant, one destination.
///
/// This is the eleventh command, and this module's own rule is that there is no eleventh without
/// a reason. The reason is a review guideline.
#[tauri::command]
pub fn open_privacy_policy() {
    if !appkit::open_url(PRIVACY_POLICY_URL) {
        eprintln!("could not open {PRIVACY_POLICY_URL}");
    }
}

/// Kept next to the command that opens it, and it must stay in step with the URL in App Store
/// Connect: guideline 5.1.1(i) asks for the policy in both places.
pub const PRIVACY_POLICY_URL: &str = "https://keepgoing.dev/privacy";
