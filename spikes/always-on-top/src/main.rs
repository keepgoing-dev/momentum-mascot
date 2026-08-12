//! Spike: can a 64x64 always-on-top window stay visible over a macOS fullscreen app?
//!
//! Answers docs/spec-v2.md section 11, wall 1. Throwaway code, not product code.
//!
//! Finding so far: level 25 (NSStatusWindowLevel) with collectionBehavior 273
//! (canJoinAllSpaces | stationary | fullScreenAuxiliary) is NOT enough. A fullscreen
//! Chrome window hides the dot completely. So the spike stopped guessing and started
//! measuring.
//!
//! Two things were added:
//!
//! 1. **Read-back.** After setting level and collectionBehavior, the values are read back
//!    off the NSWindow, along with `isVisible` and `isOnActiveSpace`. That separates
//!    "AppKit silently rejected the value" from "the value stuck but is insufficient",
//!    which are completely different problems.
//!
//! 2. **Combo cycling** (`AOT_CYCLE=1`). The terminal is invisible while another app owns
//!    the screen, so the window reports its own configuration: each combo gets a distinct
//!    colour and a digit. Go fullscreen, watch the corner, note which colour appears.
//!
//! The reporter thread logs every 2s regardless of mode, so the log can be read after the
//! fact to see exactly when `isOnActiveSpace` flipped.
//!
//! Environment:
//!   AOT_COMBO     index from the table below.   default 1
//!   AOT_LEVEL     override the combo's level.
//!   AOT_BEHAVIOR  override the combo's behavior.
//!   AOT_CYCLE     set to 1 to rotate every combo every 5s.
//!   AOT_PANEL     set to 1 to convert the NSWindow into a non-activating NSPanel.
//!   AOT_SIZE      window edge in px.            default 64
//!   AOT_DOCK      set to 1 to keep the Dock icon.

use std::time::Duration;
use tauri::{Manager, PhysicalPosition, PhysicalSize};

/// (level, collectionBehavior, css colour, label)
///
/// Behavior bits: 1 canJoinAllSpaces, 2 moveToActiveSpace, 4 managed, 8 transient,
/// 16 stationary, 64 ignoresCycle, 128 fullScreenPrimary, 256 fullScreenAuxiliary.
///
/// One option may be used from each of three groups: Space (1, 2), participation
/// (4, 8, 16), fullscreen (128, 256, 512). Mixing within a group makes AppKit ignore the
/// whole value, which is exactly what the read-back is there to catch.
const COMBOS: &[(isize, usize, &str, &str)] = &[
    (25, 273, "#ff00ff", "magenta  status level, canJoinAllSpaces|stationary|fsAux"),
    (1000, 273, "#00ffff", "cyan     screenSaver level, same behavior"),
    (1000, 17, "#ffff00", "yellow   screenSaver level, canJoinAllSpaces|stationary"),
    (1000, 1, "#00ff00", "lime     screenSaver level, canJoinAllSpaces only"),
    (1000, 257, "#ff8000", "orange   screenSaver level, canJoinAllSpaces|fsAux"),
    (1000, 81, "#ff0000", "red      screenSaver level, +ignoresCycle"),
    (2147483631, 273, "#ffffff", "white    kCGMaximumWindowLevel"),
    (3, 273, "#0080ff", "blue     floating level, the Tauri default"),
    (1000, 530, "#8000ff", "purple   screenSaver level, canJoinAllSpaces|stationary|fullScreenNone"),
    (1000, 18, "#00ff80", "spring   screenSaver level, moveToActiveSpace|stationary"),
];

/// Minimising set, used with `AOT_SET=min`, and always with `AOT_PANEL=1`.
///
/// Established so far: the NSPanel swap is necessary but not sufficient. As a panel at
/// level 25, behaviors 0, 1, and 17 are all visible on normal Spaces and all invisible over
/// a fullscreen app. Since the earlier ten-combo panel sweep did work, the missing
/// ingredient is one of the things this short list dropped.
///
/// Two hypotheses, both tested here. 1-4 walk the window level with behavior held constant,
/// to find the threshold. 5 and 6 hold the level at 25 and add the two fullscreen-specific
/// behavior bits, to see whether the level is a red herring.
const COMBOS_MIN: &[(isize, usize, &str, &str)] = &[
    (25, 17, "#ff00ff", "magenta  level 25    behavior 17  known-fail baseline"),
    (101, 17, "#00ffff", "cyan     level 101   behavior 17  popUpMenu level"),
    (500, 17, "#ffff00", "yellow   level 500   behavior 17  halfway"),
    (1000, 17, "#00ff00", "lime     level 1000  behavior 17  screenSaver level"),
    (25, 273, "#ff8000", "orange   level 25    behavior 273 +fullScreenAuxiliary"),
    (25, 530, "#8000ff", "purple   level 25    behavior 530 +fullScreenNone"),
];

fn env_num<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(default)
}

#[cfg(target_os = "macos")]
mod appkit {
    use objc2::runtime::{AnyObject, Bool};

    pub struct Report {
        pub level: isize,
        pub behavior: usize,
        pub visible: bool,
        pub on_active_space: bool,
        /// NSWindowOcclusionState. Bit 1 (value 2) set means the WindowServer considers at
        /// least part of the window visible to the user. 0 means it is being hidden or
        /// fully covered, which distinguishes "drawn underneath" from "not composited".
        pub occlusion: usize,
    }

    pub fn apply(ns: *mut AnyObject, level: isize, behavior: usize) {
        unsafe {
            let _: () = objc2::msg_send![ns, setCollectionBehavior: behavior];
            let _: () = objc2::msg_send![ns, setLevel: level];
            // Cheap insurance: an accessory app is never "active", so a window that hides
            // on deactivation would vanish for a reason unrelated to Spaces.
            let _: () = objc2::msg_send![ns, setHidesOnDeactivate: Bool::NO];
        }
    }

    /// Swap the NSWindow's class for a non-activating NSPanel.
    ///
    /// This is the trick the `tauri-nspanel` community plugin uses, and it is how
    /// Spotlight-style HUDs float over fullscreen apps. An NSPanel with
    /// NSWindowStyleMaskNonactivatingPanel (1 << 7) can be shown without activating its
    /// application, which is the property a plain NSWindow lacks.
    pub fn make_panel(ns: *mut AnyObject) -> bool {
        unsafe {
            let Some(panel) = objc2::runtime::AnyClass::get(c"NSPanel") else {
                return false;
            };
            let cls = panel as *const objc2::runtime::AnyClass;
            objc2::ffi::object_setClass(ns.cast(), cls.cast());
            // Preserve borderless (0) and add nonactivatingPanel.
            let mask: usize = objc2::msg_send![ns, styleMask];
            let _: () = objc2::msg_send![ns, setStyleMask: mask | (1usize << 7)];
            let _: () = objc2::msg_send![ns, setFloatingPanel: Bool::YES];
            let _: () = objc2::msg_send![ns, setBecomesKeyOnlyIfNeeded: Bool::YES];
            true
        }
    }

    pub fn report(ns: *mut AnyObject) -> Report {
        unsafe {
            let level: isize = objc2::msg_send![ns, level];
            let behavior: usize = objc2::msg_send![ns, collectionBehavior];
            let visible: Bool = objc2::msg_send![ns, isVisible];
            let on: Bool = objc2::msg_send![ns, isOnActiveSpace];
            let occlusion: usize = objc2::msg_send![ns, occlusionState];
            Report {
                level,
                behavior,
                visible: visible.as_bool(),
                on_active_space: on.as_bool(),
                occlusion,
            }
        }
    }
}

fn main() {
    let size: u32 = env_num("AOT_SIZE", 64);
    let combos: &'static [(isize, usize, &'static str, &'static str)] =
        if matches!(std::env::var("AOT_SET").as_deref(), Ok("min")) {
            COMBOS_MIN
        } else {
            COMBOS
        };
    let combo_ix: usize = env_num::<usize>("AOT_COMBO", 1).saturating_sub(1) % combos.len();
    let (def_level, def_behavior, _, label) = combos[combo_ix];
    let level: isize = env_num("AOT_LEVEL", def_level);
    let behavior: usize = env_num("AOT_BEHAVIOR", def_behavior);
    let cycle = matches!(std::env::var("AOT_CYCLE").as_deref(), Ok("1"));
    let panel = matches!(std::env::var("AOT_PANEL").as_deref(), Ok("1"));
    let keep_dock = std::env::var("AOT_DOCK").is_ok();

    println!("aot-spike");
    println!("  combo    {} ({label})", combo_ix + 1);
    println!("  level    {level}");
    println!("  behavior {behavior}");
    println!("  panel    {panel}");
    println!("  cycle    {cycle}");
    println!();

    tauri::Builder::default()
        .setup(move |app| {
            // An accessory app has no Dock icon and no menu bar presence, which is what the
            // real pet needs, and it changes Space behaviour. The spike must run in the same
            // mode the product will.
            #[cfg(target_os = "macos")]
            if !keep_dock {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            let win = app
                .get_webview_window("pet")
                .expect("window 'pet' is missing from tauri.conf.json");

            win.set_size(PhysicalSize::new(size, size))?;

            // Bottom-right of the monitor's WORK AREA, not its full frame.
            //
            // This matters more than it looks. The full frame includes the Dock band and
            // the menu bar, and the Dock draws at window level 20. Any pet placed inside
            // that band is invisible at Tauri's default floating level 3, which cost real
            // time to diagnose: AppKit reported the window visible and on the active
            // space, and it was, underneath the Dock.
            //
            // Using the work area means the pet never fights the Dock, which in turn means
            // the window level does not have to be raised to win that fight.
            if let Some(mon) = win.current_monitor()? {
                let inset = 24i32;
                let scale = mon.scale_factor();
                let wa = mon.work_area();
                let (wp, ws) = (wa.position, wa.size);
                println!(
                    "monitor: frame pos=({}, {}) size=({}, {})  work_area pos=({}, {}) size=({}, {})  scale={scale}",
                    mon.position().x,
                    mon.position().y,
                    mon.size().width,
                    mon.size().height,
                    wp.x,
                    wp.y,
                    ws.width,
                    ws.height
                );
                win.set_position(PhysicalPosition::new(
                    wp.x + ws.width as i32 - size as i32 - inset,
                    wp.y + ws.height as i32 - size as i32 - inset,
                ))?;
            }

            #[cfg(target_os = "macos")]
            {
                let ns = win.ns_window()? as *mut objc2::runtime::AnyObject;
                if panel && !appkit::make_panel(ns) {
                    eprintln!("warning: NSPanel class not found, staying an NSWindow");
                }
                appkit::apply(ns, level, behavior);
                let r = appkit::report(ns);
                println!(
                    "applied: level={} behavior={} visible={} onActiveSpace={} occlusion={}",
                    r.level, r.behavior, r.visible, r.on_active_space, r.occlusion
                );
                if r.level != level {
                    println!("  !! level was NOT accepted (asked {level})");
                }
                if r.behavior != behavior {
                    println!(
                        "  !! collectionBehavior was NOT accepted (asked {behavior}). \
                         Almost always means two options from the same group were combined."
                    );
                }
            }

            // Reporter. Runs in every mode, so the log can be read after a fullscreen test
            // to see exactly when onActiveSpace flipped.
            {
                let handle = app.handle().clone();
                let w = win.clone();
                std::thread::spawn(move || {
                    let mut tick = 0u32;
                    loop {
                        std::thread::sleep(Duration::from_secs(2));
                        tick += 1;
                        let w = w.clone();
                        let _ = handle.run_on_main_thread(move || {
                            #[cfg(target_os = "macos")]
                            if let Ok(p) = w.ns_window() {
                                let r = appkit::report(p as *mut objc2::runtime::AnyObject);
                                // Tauri's own view of geometry, not AppKit's: this is what
                                // production code would rely on, and it distinguishes "the
                                // window is not rendering" from "the window is off-screen".
                                let pos = w.outer_position().ok();
                                let sz = w.outer_size().ok();
                                println!(
                                    "t+{:>4}s  level={:<11} behavior={:<4} visible={:<5} onActiveSpace={:<5} occlusion={:<5} pos={:?} size={:?}",
                                    tick * 2,
                                    r.level,
                                    r.behavior,
                                    r.visible,
                                    r.on_active_space,
                                    r.occlusion,
                                    pos.map(|p| (p.x, p.y)),
                                    sz.map(|s| (s.width, s.height))
                                );
                            }
                        });
                    }
                });
            }

            if cycle {
                let handle = app.handle().clone();
                let w = win.clone();
                std::thread::spawn(move || {
                    let mut i = 0usize;
                    loop {
                        let ix = i % combos.len();
                        let (lvl, beh, colour, name) = combos[ix];
                        let w = w.clone();
                        let _ = handle.run_on_main_thread(move || {
                            #[cfg(target_os = "macos")]
                            if let Ok(p) = w.ns_window() {
                                appkit::apply(p as *mut objc2::runtime::AnyObject, lvl, beh);
                            }
                            let _ = w.eval(&format!(
                                "window.setCombo({}, '{}')",
                                ix + 1,
                                colour
                            ));
                        });
                        println!("--> combo {} {name}", ix + 1);
                        std::thread::sleep(Duration::from_secs(5));
                        i += 1;
                    }
                });
            }

            println!("\nBottom-right corner. Work through RESULTS.md. Ctrl-C to quit.\n");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run the spike");
}
