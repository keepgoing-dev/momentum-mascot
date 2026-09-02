//! The tray icon, which is plumbing (section 6.2).
//!
//! Its only jobs are opening the popover, reaching support, and holding Quit. It **does not
//! encode state**: an earlier design shipped four full-colour icons because four states cannot
//! be told apart as one-bit silhouettes at 16x16, and that requirement disappeared the moment
//! the pet took over carrying state ambiently. With nothing to encode, the simpler and
//! better-behaved option wins: a monochrome template image, which adapts to light and dark menu
//! bars by itself rather than by our arranging it.

use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::AppHandle;

use crate::{app, appkit};

/// Derived from the pack at build time by `tools/build-app-assets.sh`, which is why it is not
/// in version control (section 4.2).
const TRAY_ICON: &[u8] = include_bytes!("../icons/tray.png");

/// The landing page's contact footer, which is also the App Store Support URL (guideline 1.5).
const SUPPORT_URL: &str = "https://keepgoing.dev/#support";

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    // This menu is the only place Quit and support live, because there is no menu bar and no
    // settings screen. Items are a spec change (section 6.2), not a free addition.
    let open = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
    let support = MenuItem::with_id(app, "support", "Support", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &support, &quit])?;

    TrayIconBuilder::with_id("mascot")
        .icon(Image::from_bytes(TRAY_ICON)?)
        .icon_as_template(true)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => app::show_popover(app, app::OpenedBy::Tray),
            "support" => {
                if !appkit::open_url(SUPPORT_URL) {
                    eprintln!("could not open {SUPPORT_URL}");
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            else {
                return;
            };
            app::toggle_popover(tray.app_handle(), app::OpenedBy::Tray);
        })
        .build(app)?;
    Ok(())
}

/// Where the icon is, in physical pixels, asked for when the popover needs it.
///
/// This used to be a copy kept from the icon's own click event, which was the only place the
/// rect was known to arrive. The consequence was invisible while the tray was how anyone opened
/// the popover and wrong the moment the pet became the other way in: nothing had ever clicked
/// the icon, so there was no rect, so `show_popover` skipped positioning entirely and the panel
/// opened in the middle of the screen. `TrayIcon::rect` answers on demand on macOS - it reads
/// the status item button's own window - so the copy is gone and with it the question of
/// whether it was ever filled in.
pub fn rect(app: &AppHandle) -> Option<(f64, f64, f64, f64)> {
    let rect = app.tray_by_id("mascot")?.rect().ok()??;
    let scale = app
        .primary_monitor()
        .ok()
        .flatten()
        .map(|m| m.scale_factor())
        .unwrap_or(1.0);
    let position = rect.position.to_physical::<f64>(scale);
    let size = rect.size.to_physical::<f64>(scale);
    Some((position.x, position.y, size.width, size.height))
}
