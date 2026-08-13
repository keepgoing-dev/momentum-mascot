//! Momentum Mascot: a retro pixel character who reflects side-project momentum.
//!
//! The spec is `docs/spec-v2.md`, and every module here names the section it implements.
//! Two things are worth knowing before reading any of it:
//!
//! **The character is the product.** Roughly 90% of the value is the art, the room, and the
//! personality. This binary is the other 10%: a file watcher and a timestamp comparison.
//!
//! **The mascot never dies. It waits.** There is no state past asleep, and no copy anywhere
//! in this program is allowed to make a tired person feel worse about themselves.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod clock;
mod commands;
mod copy;
mod momentum;
mod mood;
mod pet;
mod reflog;
mod repo;
mod store;
mod tray;
mod watcher;

use tauri::Manager;

use crate::app::AppState;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState::new(store::default_path()))
        .invoke_handler(tauri::generate_handler![
            commands::refresh,
            commands::toggle_popover,
            commands::hide_popover,
            commands::add_project,
            commands::untrack,
            commands::cycle_character,
            commands::copy_share_card,
        ])
        .setup(|tauri_app| {
            let handle = tauri_app.handle().clone();

            // No Dock icon and no app window, the direct equivalent of `LSUIElement`. This
            // also changes Space behaviour, which is why the pet spike ran in this mode too.
            #[cfg(target_os = "macos")]
            tauri_app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            tray::setup(&handle)?;
            pet::setup(&handle)?;

            // Startup reads every tracked project once, so commits made while the app was
            // not running are picked up rather than waiting for the next filesystem event.
            app::refresh(&handle);
            app::start_watcher(handle.clone());
            app::start_tick(handle.clone());

            if let Some(scale) = scaled_clock_note(&handle) {
                println!("{scale}");
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            // Click-outside closes the popover, except when the thing that took the focus is
            // the app's own folder picker. The pet has no such behaviour: it is never focused
            // in the first place.
            if let tauri::WindowEvent::Focused(false) = event {
                if window.label() == app::POPOVER && !app::picker_is_open(window.app_handle()) {
                    let _ = window.hide();
                    app::note_popover_hidden(window.app_handle());
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("failed to start Momentum Mascot");
}

/// Says so, loudly, when the demo clock is running. A recording made against a scaled clock
/// is honest about it; a build that quietly runs fast is not.
fn scaled_clock_note(app: &tauri::AppHandle) -> Option<String> {
    let scale = app.state::<AppState>().clock.scale();
    (scale != 1.0).then(|| {
        format!(
            "clock is running at {scale}x: 24h in {:.0}s, 72h in {:.0}s. Debug builds only.",
            24.0 * 3600.0 / scale,
            72.0 * 3600.0 / scale
        )
    })
}
