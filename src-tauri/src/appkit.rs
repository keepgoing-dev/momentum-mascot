//! The public AppKit calls this app makes for itself.
//!
//! Every one of these is something a framework used to do for us and will not once
//! `macos-private-api` is off. `tauri-runtime-wry`'s `window.transparent(...)` is behind that
//! feature gate, `WindowBuilder::transparent()` carries the same gate, and with the feature off
//! the only feedback is an `eprintln!` gated on `debug_assertions`, which is **silent in release
//! builds**. Measured: dropping the feature printed "The window is set to be transparent but the
//! `macos-private-api` is not enabled" twice from a debug build, and nothing at all would have
//! been printed from a release one. So the premise that "a window can be transparent with public
//! API and only a webview cannot" is true of AppKit and false of Tauri, and the way out is to make
//! the AppKit calls here.
//!
//! The reclass is a different kind of thing from the rest: that one changes what the window *is*,
//! and it is the fix the fullscreen behaviour was won with. It lives here, rather than in the
//! window module it was written for, because **both** windows need it and a recipe this
//! history-sensitive should not exist twice. `pet.rs`'s module doc is still where the spike that
//! found it is written down.
//!
//! It is reachable only through `Panel`, whose constructor refuses a window that is *currently*
//! on screen. That is narrower than the spike's rule: a window shown and then hidden reports
//! `isVisible` false and would be adopted again. Nothing adopts after a show today, and the
//! tests at the foot of this file are what hold the call sites to that.
//!
//! Non-macOS builds get a handle with the same signature, so `appkit` exports no per-call stub.
//! The call sites still carry a `cfg`, because `ns_window()` is macOS-only.

/// `NSStatusWindowLevel`.
#[cfg(target_os = "macos")]
const FULLSCREEN_LEVEL: isize = 25;

/// `canJoinAllSpaces | stationary | fullScreenAuxiliary`.
#[cfg(target_os = "macos")]
const FULLSCREEN_BEHAVIOR: usize = 273;

/// `NSWindowStyleMaskNonactivatingPanel`, the property a plain `NSWindow` lacks: a panel with it
/// is shown without activating its app, so clicking it neither switches Space nor steals focus.
#[cfg(target_os = "macos")]
const NONACTIVATING_PANEL: usize = 1 << 7;

/// The right to show a window that was reclassed and configured while it was still hidden.
///
/// The handle owns that right, not the window: both surfaces need the window itself afterwards,
/// the pet to install the sprite view into and the popover to hide and re-show for the process.
///
/// `adopt_key` and `adopt_non_key` are the only things that make one.
pub struct Panel {
    configured: bool,
}

impl Panel {
    /// Whether the recipe reached the window. A `false` is an ordinary `NSWindow` that will be
    /// invisible over fullscreen apps, and the caller says so rather than pretending otherwise.
    pub fn is_configured(&self) -> bool {
        self.configured
    }

    /// Show a window through the handle. The handle carries no window, so this is the intended
    /// route to the screen rather than the only one; the tests below hold the call sites to it.
    pub fn show<R: tauri::Runtime>(&self, win: &tauri::Window<R>) -> tauri::Result<()> {
        win.show()
    }
}

#[cfg(target_os = "macos")]
impl Panel {
    /// Adopt a window the popover may take the keyboard through, because Escape dismisses it.
    pub fn adopt_key(ns: *mut std::ffi::c_void, radius: Option<f64>) -> Option<Self> {
        Self::adopt(ns, true, radius)
    }

    /// Adopt a window that must never take the keyboard from whatever the user is doing.
    pub fn adopt_non_key(ns: *mut std::ffi::c_void, radius: Option<f64>) -> Option<Self> {
        Self::adopt(ns, false, radius)
    }

    /// `None` means there is no window to adopt, or it is on screen now: reconfiguring a live
    /// window is what the spike measured as history-dependent, so it is refused rather than done.
    fn adopt(ns: *mut std::ffi::c_void, keyboard: bool, radius: Option<f64>) -> Option<Self> {
        use objc2::runtime::{AnyObject, Bool};

        let ns = ns as *mut AnyObject;
        if ns.is_null() {
            return None;
        }
        let visible: Bool = unsafe { objc2::msg_send![ns, isVisible] };
        if visible.as_bool() {
            return None;
        }
        let configured = reclass(ns, keyboard);
        make_transparent(ns);
        if let Some(radius) = radius {
            round_corners(ns, radius);
        }
        Some(Self { configured })
    }
}

/// Off macOS there is no window to be missing or already up, so the answer is always `Some`.
#[cfg(not(target_os = "macos"))]
impl Panel {
    pub fn adopt_key(ns: *mut std::ffi::c_void, radius: Option<f64>) -> Option<Self> {
        let _ = (ns, radius);
        Some(Self { configured: false })
    }

    pub fn adopt_non_key(ns: *mut std::ffi::c_void, radius: Option<f64>) -> Option<Self> {
        let _ = (ns, radius);
        Some(Self { configured: false })
    }
}

#[cfg(target_os = "macos")]
/// The style mask the reclass writes back: borderless (0) and whatever else the window had are
/// preserved, and the non-activating bit is added.
fn panel_style_mask(current: usize) -> usize {
    current | NONACTIVATING_PANEL
}

#[cfg(target_os = "macos")]
/// `becomesKeyOnlyIfNeeded` is the inverse of the surface's answer to "may I take the keyboard".
fn becomes_key_only_if_needed(keyboard: bool) -> bool {
    !keyboard
}

/// An `NSPanel` that can still take the keyboard.
///
/// The reclass below throws away the class tao installed, and tao's window class overrides
/// `canBecomeKeyWindow` to return YES. `NSWindow`'s own answer for a borderless window is NO, and
/// `NSPanel` does not change it, so a reclassed panel silently stops accepting key events: the
/// popover would open over a fullscreen app and then ignore Escape. Measured both ways before
/// this subclass existed - `isKeyWindow` stayed false for as long as the window was up, where the
/// unreclassed window reported true within a second.
///
/// Registered once. `class_addMethod` refuses a duplicate, and the pointer is stable for the
/// life of the process.
#[cfg(target_os = "macos")]
fn key_capable_panel() -> Option<&'static objc2::runtime::AnyClass> {
    use objc2::runtime::{AnyClass, AnyObject, Bool, ClassBuilder, Sel};
    use std::sync::OnceLock;

    extern "C" fn yes(_this: &AnyObject, _sel: Sel) -> Bool {
        Bool::YES
    }

    static CLASS: OnceLock<Option<&'static AnyClass>> = OnceLock::new();
    *CLASS.get_or_init(|| {
        let mut builder = ClassBuilder::new(c"MomentumKeyPanel", AnyClass::get(c"NSPanel")?)?;
        unsafe {
            builder.add_method(
                objc2::sel!(canBecomeKeyWindow),
                yes as extern "C" fn(_, _) -> _,
            );
        }
        Some(builder.register())
    })
}

/// Make a window visible over a fullscreen app, by changing the kind of window it is.
///
/// The recipe is from `spikes/always-on-top/` and is described in `pet.rs`'s module doc: no
/// `NSWindow` configuration works, at any level, and swapping the class for a non-activating
/// `NSPanel` is the step that does. Hand-rolled rather than taking `tauri-nspanel`, because it is
/// twenty verified lines against a dependency with its own Tauri-version coupling and its own
/// plugin surface, in a project whose stated failure mode is sprawl.
///
/// **Applied once, to a window that has not been shown.** The spike found that reconfiguring a
/// live window gives history-dependent results: the identical level and behaviour was invisible
/// over fullscreen in one run and visible in another, decided by what had been applied minutes
/// earlier. That is also why the recipe is not minimised further, and why nothing outside `adopt`
/// can reach it.
///
/// `keyboard` is the one thing the two windows disagree on. The pet must never take the keyboard
/// from whatever the user is doing, and a stock `NSPanel` will not. The popover must, because
/// Escape dismisses it through a JS `keydown`, so it gets `key_capable_panel` instead.
///
/// Returns whether the reclass happened. A `false` means the window is still an ordinary
/// `NSWindow` and will be invisible over fullscreen apps; the caller says so rather than
/// pretending otherwise.
#[cfg(target_os = "macos")]
fn reclass(ns: *mut objc2::runtime::AnyObject, keyboard: bool) -> bool {
    use objc2::runtime::{AnyClass, Bool};

    let Some(cls) = (if keyboard {
        key_capable_panel()
    } else {
        AnyClass::get(c"NSPanel")
    }) else {
        return false;
    };
    unsafe {
        objc2::ffi::object_setClass(ns.cast(), (cls as *const AnyClass).cast());
        let mask: usize = objc2::msg_send![ns, styleMask];
        let _: () = objc2::msg_send![ns, setStyleMask: panel_style_mask(mask)];
        let _: () = objc2::msg_send![ns, setFloatingPanel: Bool::YES];
        let key_only: Bool = Bool::new(becomes_key_only_if_needed(keyboard));
        let _: () = objc2::msg_send![ns, setBecomesKeyOnlyIfNeeded: key_only];

        let _: () = objc2::msg_send![ns, setCollectionBehavior: FULLSCREEN_BEHAVIOR];
        let _: () = objc2::msg_send![ns, setLevel: FULLSCREEN_LEVEL];
        // An accessory app is never "active", so a window that hides on deactivation would
        // vanish for a reason that has nothing to do with Spaces.
        let _: () = objc2::msg_send![ns, setHidesOnDeactivate: Bool::NO];
    }
    true
}

/// `setOpaque: NO` plus a clear `backgroundColor`, which is exactly what
/// `tao/window.rs:544-561` does behind the private feature.
///
/// Redundant while `macos-private-api` is on, because tao does it. Load-bearing the day it is
/// off, and silent if it is missing then. Verified by reading the properties back afterwards:
/// `isOpaque=false backgroundColorAlpha=0`.
#[cfg(target_os = "macos")]
fn make_transparent(ns: *mut objc2::runtime::AnyObject) {
    use objc2::runtime::Bool;

    unsafe {
        let _: () = objc2::msg_send![ns, setOpaque: Bool::NO];
        let clear = objc2_app_kit::NSColor::clearColor();
        let _: () = objc2::msg_send![ns, setBackgroundColor: &*clear];
    }
}

/// Round the window's content view, so the popover reads as a panel against the desktop rather
/// than a rectangle with a drawn-on radius. `layer.cornerRadius` plus `masksToBounds`, both
/// public, which is what replaces the transparent webview.
///
/// Masking on the content view **does** clip the WKWebView's remote-hosted layer. That was the
/// open question and it was measured: rounded corners with the desktop showing through on both a
/// light and a dark backdrop, and the window's drop shadow followed the rounded shape, so no
/// `invalidateShadow` call is needed. The documented fallback, rounding the webview's own layer
/// through `with_webview`, is not required.
#[cfg(target_os = "macos")]
fn round_corners(ns: *mut objc2::runtime::AnyObject, radius: f64) {
    use objc2::runtime::{AnyObject, Bool};

    unsafe {
        let view: *mut AnyObject = objc2::msg_send![ns, contentView];
        if view.is_null() {
            return;
        }
        let _: () = objc2::msg_send![view, setWantsLayer: Bool::YES];
        let layer: *mut AnyObject = objc2::msg_send![view, layer];
        if layer.is_null() {
            return;
        }
        let _: () = objc2::msg_send![layer, setCornerRadius: radius];
        let _: () = objc2::msg_send![layer, setMasksToBounds: Bool::YES];
    }
}

/// Open a URL in the user's browser. `NSWorkspace`, not a shellout: `/usr/bin/open` would be an
/// `exec` of a program outside the bundle, which Apple's sandbox documentation puts out of reach
/// of the file-access entitlements this app has.
///
/// Returns whether AppKit accepted it. The caller does nothing with a `false` except not lie
/// about it: a link that will not open is a cosmetic failure, and the same page is also linked
/// from the App Store listing.
#[cfg(target_os = "macos")]
pub fn open_url(url: &str) -> bool {
    use objc2_app_kit::NSWorkspace;
    use objc2_foundation::{NSString, NSURL};

    match NSURL::URLWithString(&NSString::from_str(url)) {
        Some(url) => NSWorkspace::sharedWorkspace().openURL(&url),
        None => false,
    }
}

#[cfg(not(target_os = "macos"))]
pub fn open_url(_url: &str) -> bool {
    false
}

/// Run `f` on the main thread whenever the display arrangement changes: a monitor plugged or
/// unplugged, a lid closed, a resolution change, a wake from sleep. Never removed.
#[cfg(target_os = "macos")]
pub fn observe_screen_changes<F: Fn() + 'static>(f: F) {
    use objc2_app_kit::NSApplicationDidChangeScreenParametersNotification;
    use objc2_foundation::{NSNotification, NSNotificationCenter, NSOperationQueue};

    let block = block2::RcBlock::new(move |_: std::ptr::NonNull<NSNotification>| f());
    let observer = unsafe {
        NSNotificationCenter::defaultCenter().addObserverForName_object_queue_usingBlock(
            Some(NSApplicationDidChangeScreenParametersNotification),
            None,
            Some(&NSOperationQueue::mainQueue()),
            &block,
        )
    };
    std::mem::forget(observer);
}

#[cfg(not(target_os = "macos"))]
pub fn observe_screen_changes<F: Fn() + 'static>(_f: F) {}

#[cfg(test)]
mod tests {
    use super::*;

    /// The deliverable is that a reclass can no longer reach a window through a free function,
    /// so it needs a check that fails when the hole reopens.
    #[test]
    fn the_constructor_is_the_only_thing_taking_a_window_pointer() {
        // Spelled in pieces so the needles cannot match this test's own text.
        let pointers = [
            format!("*mut {}c_void", ""),
            format!("*mut {}c_void", "std::ffi::"),
            format!("*mut {}c_void", "core::ffi::"),
            format!("*mut {}AnyObject", ""),
            format!("*mut {}AnyObject", "objc2::runtime::"),
        ];
        let source = include_str!("appkit.rs");
        let taking: Vec<&str> = source
            .lines()
            .map(str::trim)
            .filter(|line| pointers.iter().any(|p| line.contains(p)))
            .collect();

        assert!(!taking.is_empty(), "nothing takes a window now");
        for line in taking {
            let exported = line.contains(&format!("{}fn ", "pub "));
            let constructor = line.contains("fn adopt_key") || line.contains("fn adopt_non_key");
            assert!(
                !exported || constructor,
                "appkit.rs exposes `{line}`, so a live window can be reclassed again"
            );
        }
    }

    /// A tripwire, not a proof: `tauri::Window::show` is public, so `win.show( )` still slips by.
    #[test]
    fn no_surface_shows_its_window_without_going_through_the_handle() {
        // Assembled at runtime, and `panel.show(&win)` reads `.show(` rather than this.
        let bare = format!(".{}()", "show");
        let surfaces = [
            ("pet.rs", include_str!("pet.rs")),
            ("app.rs", include_str!("app.rs")),
            ("commands.rs", include_str!("commands.rs")),
            ("main.rs", include_str!("main.rs")),
            ("tray.rs", include_str!("tray.rs")),
        ];

        for (name, source) in surfaces {
            // Comments are stripped first: this file's own prose quotes the call it forbids.
            let code = source
                .lines()
                .filter(|l| !l.trim_start().starts_with("//"))
                .collect::<Vec<_>>()
                .join("\n");
            assert!(
                !code.contains(&bare),
                "{name} shows a window itself, so the handle is optional rather than the way"
            );
        }
    }

    /// A tripwire like the one above: it can say where a window is adopted and which keyboard
    /// answer that site asked for, never whether the window was live.
    #[test]
    fn each_surface_adopts_its_own_window_only_in_the_setup_that_built_it() {
        // Assembled at runtime, as above.
        let call = format!("Panel::{}", "adopt");
        let key = format!("{}_key", call);
        let non_key = format!("{}_non_key", call);
        let surfaces = [
            ("pet.rs", include_str!("pet.rs"), "fn setup(", &non_key),
            ("app.rs", include_str!("app.rs"), "fn setup_popover(", &key),
        ];

        for (name, source, setup, constructor) in surfaces {
            let mut enclosing = "";
            let mut calls = 0;
            for line in source.lines() {
                let head = line.trim_start();
                if head.starts_with("fn ") || head.contains(&format!("{} ", " fn")) {
                    enclosing = head;
                } else if line == "}" {
                    enclosing = "";
                }
                if !line.contains(&call) {
                    continue;
                }
                calls += 1;
                assert!(enclosing.contains(setup), "{name} adopts in `{enclosing}`");
                assert!(line.contains(constructor), "{name} wants {constructor}");
            }
            assert!(calls > 0, "{name} adopts no window at all");
        }
    }

    /// The two constructors differ in one argument, and transposing it is invisible to every
    /// test that does not own a real window: the helper tests below call the helper directly.
    #[test]
    fn each_constructor_asks_for_the_keyboard_answer_its_name_promises() {
        // Assembled at runtime, as the tripwires above are.
        let source = include_str!("appkit.rs");
        let body_of = |name: &str| -> String {
            let head = format!("{} {}(", "fn", name);
            source
                .lines()
                .skip_while(|l| !l.contains(&head))
                .skip(1)
                .take_while(|l| !l.trim_start().starts_with('}'))
                .collect::<Vec<_>>()
                .join(" ")
        };

        let key = body_of("adopt_key");
        let non_key = body_of("adopt_non_key");
        assert!(
            key.contains("true") && !key.contains("false"),
            "`adopt_key` does not ask for the keyboard, so Escape will not dismiss the popover: `{key}`"
        );
        assert!(
            non_key.contains("false") && !non_key.contains("true"),
            "`adopt_non_key` takes the keyboard from whatever the user is typing in: `{non_key}`"
        );
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn a_null_window_yields_no_handle_because_there_is_nothing_to_show() {
        let panel = Panel::adopt_non_key(std::ptr::null_mut(), None);
        assert!(panel.is_none(), "nothing was sent to nil");
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn a_panel_becomes_key_only_if_needed_exactly_when_it_may_not_take_the_keyboard() {
        // The pet may not take the keyboard; the popover must, because Escape dismisses it.
        assert!(becomes_key_only_if_needed(false));
        assert!(!becomes_key_only_if_needed(true));
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn the_style_mask_keeps_what_the_window_already_had() {
        assert_eq!(panel_style_mask(0), NONACTIVATING_PANEL, "borderless is 0");
        assert_eq!(panel_style_mask(1 << 3), (1 << 3) | NONACTIVATING_PANEL);
        assert_eq!(panel_style_mask(NONACTIVATING_PANEL), NONACTIVATING_PANEL);
    }
}
