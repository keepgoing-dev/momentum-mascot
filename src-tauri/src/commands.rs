//! Everything the webview is allowed to ask for. Seven commands, and no eighth without a
//! reason: this list is the whole API surface between the art and the machinery.

use tauri::{AppHandle, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_dialog::DialogExt;

use crate::app::{self, AppState};
use crate::pet;

/// Ask for the current mood without waiting for the next event. Called once when a window
/// finishes loading, because a window that opens between ticks would otherwise be blank.
#[tauri::command]
pub fn refresh(app: AppHandle) {
    app::publish(&app);
}

/// Clicking the pet. The pet is the primary way in; the tray icon is secondary.
#[tauri::command]
pub fn toggle_popover(app: AppHandle) {
    app::toggle_popover(&app);
}

/// The pet finished being dragged. The webview reports where it let go (its top-left in
/// physical pixels, which is the only coordinate the webview knows its own position in); the
/// backend snaps it to the nearest corner and remembers that corner, so the drag survives a
/// restart. No seventh parameter, no free placement: this is the whole drag API.
#[tauri::command]
pub fn snap_pet(app: AppHandle, x: f64, y: f64) {
    let Some(win) = app.get_webview_window(app::PET) else {
        return;
    };
    let Some(target) = pet::snap_to_nearest_corner(&win, (x, y)) else {
        return;
    };

    let state = app.state::<AppState>();
    let to_save = {
        let mut momentum = state.momentum.lock().unwrap();
        momentum.state.pet_position = Some(target);
        momentum.state.clone()
    };
    if let Err(e) = crate::store::save(&state.store_path, &to_save) {
        eprintln!("could not write state: {e}");
    }
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

    let state = app.state::<AppState>();
    let now = state.clock.now();
    state
        .momentum
        .lock()
        .unwrap()
        .add(&path, now)
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

/// Clicking the character cycles to the next of the three. This is the entire selection
/// mechanism: no picker UI and no settings screen, so the guardrail holds.
#[tauri::command]
pub fn cycle_character(app: AppHandle) {
    app.state::<AppState>()
        .momentum
        .lock()
        .unwrap()
        .cycle_character();
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
