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
//! `pet::macos` keeps the NSPanel reclass, which is a different kind of thing: that one changes
//! what the window *is*, and it is the fix the fullscreen behaviour was won with. This module is
//! only public API, only cosmetic, and safe to call on either window.
//!
//! Non-macOS builds get no-ops rather than a `cfg` at every call site, which is the same shape
//! `store::default_path` already uses for Windows.

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
