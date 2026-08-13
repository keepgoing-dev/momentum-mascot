//! The tray icon, which is plumbing (section 6.2).
//!
//! Its only jobs are opening the popover and holding Quit. It **does not encode state**: an
//! earlier design shipped four full-colour icons because four states cannot be told apart as
//! one-bit silhouettes at 16x16, and that requirement disappeared the moment the pet took
//! over carrying state ambiently. With nothing to encode, the simpler and better-behaved
//! option wins: a monochrome template image, which adapts to light and dark menu bars by
//! itself rather than by our arranging it.

use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager};

use crate::app::{self, AppState};

/// Derived from the pack at build time by `tools/build-app-assets.sh`, which is why it is not
/// in version control (section 4.2).
const TRAY_ICON: &[u8] = include_bytes!("../icons/tray.png");

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    // This menu is the only place Quit lives, because there is no menu bar and no settings
    // screen. Exactly two items, and adding a third is a spec change.
    let open = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;

    TrayIconBuilder::with_id("mascot")
        .icon(Image::from_bytes(TRAY_ICON)?)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => app::show_popover(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                rect,
                ..
            } = event
            else {
                return;
            };
            let app = tray.app_handle();
            remember_rect(app, &rect);
            app::toggle_popover(app);
        })
        .build(app)?;
    Ok(())
}

/// The popover is anchored under the icon, and the icon's position is only ever reported by
/// its own click event, so it is kept when it arrives rather than asked for when needed.
fn remember_rect(app: &AppHandle, rect: &tauri::Rect) {
    let scale = app
        .primary_monitor()
        .ok()
        .flatten()
        .map(|m| m.scale_factor())
        .unwrap_or(1.0);
    let position = rect.position.to_physical::<f64>(scale);
    let size = rect.size.to_physical::<f64>(scale);
    *app.state::<AppState>().tray_rect.lock().unwrap() =
        Some((position.x, position.y, size.width, size.height));
}
