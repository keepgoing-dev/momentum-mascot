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

use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize};

use crate::app::PET;

pub const SIZE: u32 = 64;

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

    win.set_size(PhysicalSize::new(SIZE, SIZE))?;
    place(&win)?;

    #[cfg(target_os = "macos")]
    {
        let ns = win.ns_window()? as *mut objc2::runtime::AnyObject;
        if !macos::make_panel(ns) {
            eprintln!("NSPanel class not found; the pet will not show over fullscreen apps");
        }
        macos::apply(ns, LEVEL, BEHAVIOR);
    }

    win.show()?;
    Ok(())
}

/// Bottom-right of the **usable** screen.
///
/// `work_area` alone is not enough, and this cost real time to diagnose. On the author's
/// display it returned a rect that reserved the menu bar and **not** the Dock band, so a pet
/// placed relative to it sits underneath the Dock, which draws at window level 20. Every
/// AppKit property reported the window healthy while it was hidden.
///
/// So the usable corner is the **intersection** of Tauri's work area and AppKit's own
/// `visibleFrame`. Taking the tighter of the two is deliberate: whichever of them accounts
/// for the Dock, the pet clears it, and if Tauri's behaviour changes later this does not
/// silently start double-counting.
fn place(win: &tauri::WebviewWindow) -> tauri::Result<()> {
    let Some(mon) = win.current_monitor()? else {
        return Ok(());
    };
    let scale = mon.scale_factor();
    let area = mon.work_area();
    let mut right = (area.position.x + area.size.width as i32) as f64;
    let mut bottom = (area.position.y + area.size.height as i32) as f64;

    #[cfg(target_os = "macos")]
    if let Some((vr, vb)) = macos::visible_bottom_right(scale) {
        right = right.min(vr);
        bottom = bottom.min(vb);
    }

    let margin = MARGIN * scale;
    win.set_position(PhysicalPosition::new(
        (right - SIZE as f64 - margin) as i32,
        (bottom - SIZE as f64 - margin) as i32,
    ))?;
    Ok(())
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
