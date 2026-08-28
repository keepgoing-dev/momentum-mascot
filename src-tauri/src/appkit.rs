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
//! `show_over_fullscreen` is a different kind of thing from the rest: that one changes what the
//! window *is*, and it is the fix the fullscreen behaviour was won with. It lives here, rather
//! than in the window module it was written for, because **both** windows need it and a recipe
//! this history-sensitive should not exist twice. `pet.rs`'s module doc is still where the spike
//! that found it is written down.
//!
//! Non-macOS builds get no-ops rather than a `cfg` at every call site, which is the same shape
//! `store::default_path` already uses for Windows.

/// `NSStatusWindowLevel`.
#[cfg(target_os = "macos")]
const FULLSCREEN_LEVEL: isize = 25;

/// `canJoinAllSpaces | stationary | fullScreenAuxiliary`.
#[cfg(target_os = "macos")]
const FULLSCREEN_BEHAVIOR: usize = 273;

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
            builder.add_method(objc2::sel!(canBecomeKeyWindow), yes as extern "C" fn(_, _) -> _);
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
/// **Call once, before the window is first shown.** The spike found that reconfiguring a live
/// window gives history-dependent results: the identical level and behaviour was invisible over
/// fullscreen in one run and visible in another, decided by what had been applied minutes
/// earlier. That is also why the recipe is not minimised further.
///
/// `keyboard` is the one thing the two windows disagree on. The pet must never take the keyboard
/// from whatever the user is doing, and a stock `NSPanel` will not. The popover must, because
/// Escape dismisses it through a JS `keydown`, so it gets `key_capable_panel` instead.
///
/// Returns whether the reclass happened. A `false` means the window is still an ordinary
/// `NSWindow` and will be invisible over fullscreen apps; the caller says so rather than
/// pretending otherwise.
#[cfg(target_os = "macos")]
pub fn show_over_fullscreen(ns: *mut std::ffi::c_void, keyboard: bool) -> bool {
    use objc2::runtime::{AnyClass, AnyObject, Bool};

    let ns = ns as *mut AnyObject;
    if ns.is_null() {
        return false;
    }
    let Some(cls) = (if keyboard {
        key_capable_panel()
    } else {
        AnyClass::get(c"NSPanel")
    }) else {
        return false;
    };
    unsafe {
        objc2::ffi::object_setClass(ns.cast(), (cls as *const AnyClass).cast());
        // Preserve borderless (0) and add nonactivatingPanel (1 << 7), which is the property a
        // plain NSWindow lacks: a panel with it can be shown without activating its application,
        // so clicking it over a fullscreen app neither switches Space nor steals focus.
        let mask: usize = objc2::msg_send![ns, styleMask];
        let _: () = objc2::msg_send![ns, setStyleMask: mask | (1usize << 7)];
        let _: () = objc2::msg_send![ns, setFloatingPanel: Bool::YES];
        let _: () = objc2::msg_send![ns, setBecomesKeyOnlyIfNeeded: Bool::new(!keyboard)];

        let _: () = objc2::msg_send![ns, setCollectionBehavior: FULLSCREEN_BEHAVIOR];
        let _: () = objc2::msg_send![ns, setLevel: FULLSCREEN_LEVEL];
        // An accessory app is never "active", so a window that hides on deactivation would
        // vanish for a reason that has nothing to do with Spaces.
        let _: () = objc2::msg_send![ns, setHidesOnDeactivate: Bool::NO];
    }
    true
}

#[cfg(not(target_os = "macos"))]
pub fn show_over_fullscreen(_ns: *mut std::ffi::c_void, _keyboard: bool) -> bool {
    false
}

/// `setOpaque: NO` plus a clear `backgroundColor`, which is exactly what
/// `tao/window.rs:544-561` does behind the private feature.
///
/// Takes tauri's own `ns_window()` return type, so there is no cast at the call site. Verified by
/// reading the properties back afterwards: `isOpaque=false backgroundColorAlpha=0`.
#[cfg(target_os = "macos")]
pub fn make_transparent(ns: *mut std::ffi::c_void) {
    use objc2::runtime::{AnyObject, Bool};

    let ns = ns as *mut AnyObject;
    if ns.is_null() {
        return;
    }
    unsafe {
        let _: () = objc2::msg_send![ns, setOpaque: Bool::NO];
        let clear = objc2_app_kit::NSColor::clearColor();
        let _: () = objc2::msg_send![ns, setBackgroundColor: &*clear];
    }
}

#[cfg(not(target_os = "macos"))]
pub fn make_transparent(_ns: *mut std::ffi::c_void) {}

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
pub fn round_corners(ns: *mut std::ffi::c_void, radius: f64) {
    use objc2::runtime::{AnyObject, Bool};

    let ns = ns as *mut AnyObject;
    if ns.is_null() {
        return;
    }
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

#[cfg(not(target_os = "macos"))]
pub fn round_corners(_ns: *mut std::ffi::c_void, _radius: f64) {}

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
