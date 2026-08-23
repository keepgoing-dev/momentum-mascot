# Native AppKit pet Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the pet's webview with a native AppKit sprite view, so that dropping tauri's `macos-private-api` feature costs the character nothing, and the Mac App Store submission stops being blocked on two private KVC keys.

**Architecture:** The pet window stops being a webview window and becomes a plain `Window` built in Rust. A layer-hosting `NSView` subclass is added as a subview of tao's content view, and the sprite is a **sublayer** of that view's root layer, animated with a discrete `CAKeyframeAnimation` over `contentsRect`. Interaction (click, drag, corner snap) moves from `pet.js` into the view's `mouseDown`/`mouseDragged`/`mouseUp`. Window geometry, placement and the glide stay in `pet.rs` exactly as they are.

**Tech Stack:** Rust 2021, Tauri 2.11.5 with the **`unstable`** feature, objc2 0.6 / objc2-app-kit 0.3 / objc2-foundation 0.3 / objc2-quartz-core 0.3.

**Spec:** `docs/superpowers/specs/2026-08-22-mac-app-store-design.md`, section 4.

**Parent plan:** `docs/superpowers/plans/2026-08-22-mac-app-store-submission.md`. Phases 1 to 3 of that plan are done. This plan is the section 4 work it deferred, and it slots between that plan's Phase 3 and Phase 5.

## Why this plan exists

The parent plan's Task 3 ran the spec's section 4.0 probe, which existed to try to delete this
entire body of work. **It failed.** With `macos-private-api` dropped and `setOpaque: NO`,
`setBackgroundColor: clearColor` and `underPageBackgroundColor = clearColor` all applied by hand
with public API, the pet rendered as a **visible opaque square**. `underPageBackgroundColor` does
not reach the page's own backdrop, which is what wry's comment at `wkwebview/mod.rs:429-431`
implies: it covers the overscroll region only. `_drawsBackground` is the only thing that makes a
WKWebView see-through and it is private.

Two things that probe established, which this plan depends on:

- **The window is genuinely transparent with public API.** Read back in the same run:
  `isOpaque=false backgroundColorAlpha=0`. `appkit::make_transparent` already exists and is
  already called on the pet in `pet::setup`. So only the *content* needs replacing.
- **Dropping the feature really does remove the two strings.** Measured on the same build:
  `drawsBackground` 0, `fullScreenEnabled` 0, while `allowsPictureInPictureMediaPlayback` and
  `_wantsKeyDownForEvent` remain and are not removable without forking wry and tao.

Read `spikes/app-store/RESULTS.md` before starting. It holds every measurement behind this plan.

## Global Constraints

- **No em dashes anywhere.** House rule; applies to code, comments, docs and commit messages.
- **`tauri` needs the `unstable` feature added.** Not optional, and the spec did not mention it. `tauri::window::WindowBuilder` is `pub` only under `#[cfg(feature = "unstable")]` and `pub(crate)` otherwise, and `Manager::get_window` / `get_focused_window` / `get_windows` are all gated the same way. **Measured by compiling both ways:** without it, `error[E0603]: struct WindowBuilder is private`; with it, the full `WindowBuilder::new(app, "pet").inner_size(64.0, 64.0)...build()` chain plus `win.ns_window()` and `Manager::get_window` compile clean. The feature forwards only to `tauri-runtime-wry?/unstable`, so the blast radius is small.
- **`objc2-quartz-core = "0.3"` becomes a direct dependency.** It is **already in `Cargo.lock` at 0.3.2**, pulled in transitively by objc2-app-kit, so this costs a `Cargo.toml` line and no new compilation.
- **Sprites live in two places on purpose.** `bundle.resources` for the native view, and `frontendDist` as now, because `src/popover.js:49` sets `backgroundImage = url("assets/pet/${id}/dozing.png")` for the three character-picker buttons. Deleting them from `src/assets` breaks the popover.
- **Sprite geometry:** each mood is one PNG, **384x32**, being 12 frames of 32x32. Verified with `file`. Durations, from `src/pet.html:45-64`: awake 4s, dozing 6s, asleep 6s, comeback 1.5s, run 0.75s.
- **Minimum system version 10.15**, from `tauri.conf.json`. Everything here is older than that.
- **Test command:** `cargo test --manifest-path src-tauri/Cargo.toml`. The 73 tests passing at the end of the parent plan's Phase 3 must all keep passing.
- **Commit messages:** imperative, sentence case, no conventional-commit prefixes.

## Verified API surface

Every signature below was read out of the vendored crate sources at the versions in
`Cargo.lock`. If something does not compile, suspect the surrounding code and not these.

| What | Signature |
|---|---|
| Main-thread hop | `AppHandle::run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()>` |
| Webview-less window | `tauri::window::WindowBuilder::new(app, label)` then `.build() -> Result<Window<R>>` |
| Window handle | `tauri::window::Window<tauri::Wry>`; has `ns_window()`, `set_size`, `set_position`, `outer_position`, `current_monitor`, `show` |
| Keyframe animation | `CAKeyframeAnimation::animationWithKeyPath(Option<&NSString>)`, `setValues(Option<&NSArray>)` (unsafe), `setKeyTimes(Option<&NSArray<NSNumber>>)`, `setCalculationMode(&CAAnimationCalculationMode)` |
| Timing | `CAMediaTiming::setDuration(CFTimeInterval)`, `setRepeatCount(c_float)` |
| Discrete mode | `kCAAnimationDiscrete: &'static CAAnimationCalculationMode` |
| Layer | `CALayer::setContents(Option<&AnyObject>)` (unsafe), `setContentsRect(CGRect)`, `setContentsScale(CGFloat)`, `setMagnificationFilter(&CALayerContentsFilter)`, `setFrame(CGRect)`, `setAnchorPoint(CGPoint)`, `setTransform(CATransform3D)`, `addSublayer(&CALayer)`, `addAnimation_forKey(&CAAnimation, Option<&NSString>)` |
| Nearest filter | `kCAFilterNearest: &'static CALayerContentsFilter` |
| Flip | `CATransform3DMakeScale(sx, sy, sz) -> CATransform3D`, and `CATransform3DIdentity` |
| Rect in an array | `NSValue::valueWithRect(NSRect) -> Retained<NSValue>` (unsafe) |
| Sprite image | `NSImage::initWithContentsOfFile(...)` |
| Cursors | `NSCursor::openHandCursor()`, `NSCursor::closedHandCursor()`, `NSCursor::set(&self)` |
| View | `NSView::setAutoresizingMask`, `bounds()`, `convertPoint_fromView` |
| Subclassing | `objc2::define_class!` with `#[unsafe(super(NSView))]`, `#[thread_kind = MainThreadOnly]`, `#[name = "..."]`, `#[ivars = ...]`, and `#[unsafe(method(sel:))]` on each override |

## File Structure

| File | Responsibility |
|---|---|
| Create `src-tauri/src/sprite.rs` | Two halves, deliberately separated. A **pure** top half: frame rects, the N+1 keyTimes rule, cell sizing and centring, per-mood durations, sprite paths. All unit tested. A **native** bottom half: the layer-hosting `NSView` subclass, the animation, and the mouse handling. |
| Modify `src-tauri/src/pet.rs` | Window type changes from `WebviewWindow` to `Window` in six places. Geometry, anchors, placement and the glide are untouched. Gains the sprite view's installation and the mood/glide entry points. |
| Modify `src-tauri/src/app.rs` | `publish` tells the pet directly instead of relying on the pet listening to `MOOD_EVENT`. |
| Modify `src-tauri/src/commands.rs` | Three commands go: `toggle_popover`, `snap_pet`, `cancel_glide`. |
| Modify `src-tauri/src/main.rs` | New module, three commands removed from `generate_handler!`, the pet window built in Rust. |
| Modify `src-tauri/Cargo.toml` | `unstable` on, `macos-private-api` off, `objc2-quartz-core` added. |
| Modify `src-tauri/tauri.conf.json` | The `pet` window entry is deleted; `macOSPrivateApi` false; `bundle.resources` added. |
| Modify `src-tauri/capabilities/default.json` | `"pet"` leaves `windows`; three window permissions go, `allow-set-size` stays. |
| Delete `src/pet.html`, `src/pet.js` | Replaced entirely. |

---

## Task 1: The pure sprite arithmetic

Everything that can be wrong about the animation without involving Core Animation at all. The
N+1 keyTimes rule and the whole-multiple cell are the two rules an implementer would not infer,
and both are pure functions, so both get real tests.

**Files:**
- Create: `src-tauri/src/sprite.rs`
- Modify: `src-tauri/src/main.rs` (module list)
- Test: inline `mod tests` in `src-tauri/src/sprite.rs`

**Interfaces:**
- Produces:
  - `sprite::FRAME: f64` (32.0), `sprite::FRAMES: usize` (12)
  - `sprite::frame_rect(i: usize) -> (f64, f64, f64, f64)`
  - `sprite::key_times() -> Vec<f64>`
  - `sprite::frame_at(progress: f64) -> usize`
  - `sprite::cell_side(view_side: f64) -> f64`
  - `sprite::cell_origin(view_side: f64, cell: f64) -> f64`
  - `sprite::duration(mood: &str) -> f64`
  - `sprite::relative_path(character_id: &str, mood: &str) -> PathBuf`

- [x] **Step 1: Write the failing tests**

Create `src-tauri/src/sprite.rs`:

```rust
//! The pet's sprite: the arithmetic, and the AppKit view that draws it.
//!
//! Split deliberately. Everything above the `// native` divider is pure and tested; everything
//! below it is FFI and is covered by the manual acceptance test. The two rules most likely to be
//! got wrong, the N+1 keyTimes rule and the whole-multiple cell, are both in the pure half.

use std::path::PathBuf;

/// One frame of the strip, in source pixels. Every sprite PNG is 12 of these side by side, so
/// 384x32. Verified with `file` on all fifteen of them.
pub const FRAME: f64 = 32.0;
pub const FRAMES: usize = 12;

/// `contentsRect` for frame `i`, in CALayer's unit coordinate space.
///
/// `contentsRect` is the right mechanism for this: unit coordinates, animatable, and
/// `contentsGravity` defaults to `resize`, which stretches the selected cell to fill the layer.
pub fn frame_rect(i: usize) -> (f64, f64, f64, f64) {
    (i as f64 / FRAMES as f64, 0.0, 1.0 / FRAMES as f64, 1.0)
}

/// The keyTimes for a discrete keyframe animation over `FRAMES` values.
///
/// **There are N+1 of them, and this is the rule that is not guessable.**
/// `CAKeyframeAnimation.keyTimes` documents that with `calculationMode = .discrete` "the array
/// should have one more entry than appears in the values array", and
/// `CAAnimationCalculationMode.discrete` says each value/keyTime pair "represents the value from
/// the specified time until the next keyframe". CSS `steps(12)` is `floor(p * 12) / 12`, which is
/// exactly 12 plateaus of D/12.
///
/// 12 values with `nil` keyTimes yields **eleven** visible frames of D/11 each, with the twelfth
/// holding for zero time. These strips are walk cycles, so a dropped frame reads as a limp. Apple
/// documents no `nil`-keyTimes fallback for discrete mode, only that "the timing of your
/// animation might not be what you expect."
pub fn key_times() -> Vec<f64> {
    (0..=FRAMES).map(|i| i as f64 / FRAMES as f64).collect()
}

/// The CSS oracle: which frame `steps(12)` shows at normalised progress `p`. Kept so the
/// keyTimes can be checked against the thing they are replacing rather than against themselves.
pub fn frame_at(progress: f64) -> usize {
    ((progress * FRAMES as f64).floor().max(0.0) as usize).min(FRAMES - 1)
}

/// The displayed side of one frame: a whole multiple of the source frame, never a fraction.
///
/// From `pet.js:22-26`. A 1.5x character is not a smaller character, it is a blurry one. The
/// floor of one whole frame is why a window too small for even 1x shows a **small** pet rather
/// than a cropped one, which is the bug that actually shipped once: a 32pt cell in a 64pt window
/// drawn at full size and clipped to the character's hat.
pub fn cell_side(view_side: f64) -> f64 {
    FRAME.max((view_side / FRAME).floor() * FRAME)
}

/// Where the cell sits inside the view: centred.
///
/// **This is one rule with `cell_side`, not two.** `pet.html:10-13` centres the cell with
/// `display: grid; place-items: center`, and dropping the centring breaks exactly the case the
/// floor exists for: uncentred, a 32pt cell in a 64pt window sits in a corner. Worse,
/// `pet.html:66-68` ties the flip to the centring explicitly, so without it
/// `CATransform3DMakeScale(-1, 1, 1)` shunts the character sideways on every leftward run
/// instead of turning it in place.
pub fn cell_origin(view_side: f64, cell: f64) -> f64 {
    ((view_side - cell) / 2.0).max(0.0)
}

/// Seconds for one full 12-frame cycle, per mood. Carried over from `pet.html:45-64` unchanged.
///
/// Motion is reserved on the pet by design: it sits in peripheral vision all day, so the idle
/// moods run at the bottom of the range and only the comeback and the run are loud.
pub fn duration(mood: &str) -> f64 {
    match mood {
        "dozing" | "asleep" => 6.0,
        "comeback" => 1.5,
        "run" => 0.75,
        // "awake" and anything unrecognised. An unknown mood animating slowly is a better
        // failure than one that does not animate at all.
        _ => 4.0,
    }
}

/// Where a sprite lives under the bundle's resource directory.
pub fn relative_path(character_id: &str, mood: &str) -> PathBuf {
    PathBuf::from("pet").join(character_id).join(format!("{mood}.png"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn there_is_one_more_key_time_than_there_are_frames() {
        // The whole bug, asserted directly. This is what fails loudly on the nil-keyTimes
        // mistake, by construction, without a window or a display or Core Animation.
        let times = key_times();
        assert_eq!(times.len(), FRAMES + 1, "N+1 rule broken");
        assert_eq!(times[0], 0.0);
        assert_eq!(*times.last().unwrap(), 1.0);
        for pair in times.windows(2) {
            assert!(pair[1] > pair[0], "keyTimes must increase: {pair:?}");
        }
    }

    #[test]
    fn every_frame_selects_its_own_twelfth_of_the_strip() {
        for i in 0..FRAMES {
            let (x, y, w, h) = frame_rect(i);
            assert!((x - i as f64 / 12.0).abs() < 1e-12, "frame {i} x");
            assert_eq!(y, 0.0);
            assert!((w - 1.0 / 12.0).abs() < 1e-12, "frame {i} width");
            assert_eq!(h, 1.0);
        }
        // The strip is fully covered and nothing runs off the end.
        let (x, _, w, _) = frame_rect(FRAMES - 1);
        assert!((x + w - 1.0).abs() < 1e-12);
    }

    /// Which value a discrete keyframe animation displays at normalised progress `p`, given its
    /// keyTimes. Discrete mode holds `values[i]` from `keyTimes[i]` until `keyTimes[i + 1]`.
    fn displayed(times: &[f64], p: f64) -> usize {
        let mut i = 0;
        while i + 1 < times.len() && times[i + 1] <= p {
            i += 1;
        }
        i.min(FRAMES - 1)
    }

    #[test]
    fn twelve_plateaus_agree_with_the_css_oracle_and_eleven_do_not() {
        // The N+1 keyTimes must reproduce CSS `steps(12)` exactly, and the eleven-plateau
        // mistake must not.
        //
        // Comparing the two schemes at their own boundaries proves nothing: `floor(12i/11) == i`
        // for every `i` below 11, because `12i/11 = i + i/11` and `i/11 < 1` there, so both
        // schemes name the right frame at every boundary. The difference is what is on screen
        // BETWEEN the boundaries, so sample the timeline rather than the boundaries.
        let right = key_times();
        // The mistake: 12 values with 12 keyTimes, giving eleven plateaus of D/11 and a twelfth
        // frame that holds for no time at all.
        let wrong: Vec<f64> = (0..FRAMES).map(|i| i as f64 / (FRAMES - 1) as f64).collect();

        let samples = 1200;
        let mut wrong_disagreements = 0;
        for n in 0..samples {
            let p = n as f64 / samples as f64;
            assert_eq!(displayed(&right, p), frame_at(p), "N+1 keyTimes at p={p}");
            if displayed(&wrong, p) != frame_at(p) {
                wrong_disagreements += 1;
            }
        }

        // The two schemes are on the same frame for exactly half the cycle and disagree for the
        // other half: agreement on frame k is the overlap of [k/12, (k+1)/12) with
        // [k/11, (k+1)/11), which is (11-k)/132, and those sum to 66/132.
        let fraction = wrong_disagreements as f64 / samples as f64;
        assert!(
            (fraction - 0.5).abs() < 0.02,
            "eleven plateaus should be wrong half the time, got {fraction}"
        );

        // And the frame the mistake never shows for any measurable time is the twelfth.
        assert!(
            (0..samples).all(|n| displayed(&wrong, n as f64 / samples as f64) != FRAMES - 1),
            "the eleven-plateau mistake should drop the twelfth frame entirely"
        );
    }

    #[test]
    fn the_cell_is_always_a_whole_multiple_of_the_source_frame() {
        assert_eq!(cell_side(64.0), 64.0, "the normal case, 2x");
        assert_eq!(cell_side(32.0), 32.0, "1x");
        assert_eq!(cell_side(96.0), 96.0, "3x");
        assert_eq!(cell_side(100.0), 96.0, "never a fraction");
        assert_eq!(cell_side(63.0), 32.0, "just under 2x is 1x, not 1.97x");
        assert_eq!(cell_side(20.0), 32.0, "the floor: a small pet, never a cropped one");
        assert_eq!(cell_side(0.0), 32.0, "a degenerate view still has a floor");
    }

    #[test]
    fn a_cell_smaller_than_its_view_is_centred() {
        // The clipped-to-a-hat bug is a 32pt cell in a 64pt window. Uncentred it sits in a
        // corner, and the flip shunts it sideways.
        assert_eq!(cell_origin(64.0, 32.0), 16.0);
        assert_eq!(cell_origin(64.0, 64.0), 0.0);
        assert_eq!(cell_origin(96.0, 64.0), 16.0);
        assert_eq!(cell_origin(20.0, 32.0), 0.0, "never negative");
    }

    #[test]
    fn the_durations_are_the_ones_pet_html_shipped() {
        assert_eq!(duration("awake"), 4.0);
        assert_eq!(duration("dozing"), 6.0);
        assert_eq!(duration("asleep"), 6.0);
        assert_eq!(duration("comeback"), 1.5);
        assert_eq!(duration("run"), 0.75);
        assert_eq!(duration("nonsense"), 4.0, "an unknown mood still animates");
    }

    #[test]
    fn a_sprite_path_is_character_then_mood() {
        assert_eq!(
            relative_path("07", "awake"),
            PathBuf::from("pet/07/awake.png")
        );
        assert_eq!(relative_path("20", "run"), PathBuf::from("pet/20/run.png"));
    }
}
```

- [x] **Step 2: Declare the module and run the tests to see them fail**

In `src-tauri/src/main.rs`, add `mod sprite;` to the module list, between `mod scoped;` and
`mod store;`.

Run: `cargo test --manifest-path src-tauri/Cargo.toml sprite::`
Expected: the seven tests run. Because the implementation is given in full above, they should
pass.

**Correction, found by running this step.** As first written, this task's
`twelve_plateaus_agree_with_the_css_oracle_and_eleven_do_not` failed on correct code with "got 0".
Its premise was arithmetically false: it compared the two schemes at the eleven-plateau scheme's
own boundaries, and `floor(12i/11) == i` for every `i` below 11, so the schemes agree at every
one of them. The version above is the corrected test, which samples the timeline instead and
measures the half of the cycle where the two schemes are on different frames. Confirmed to bite:
with `key_times` set to the eleven-plateau scheme it fails at p=1/12, the first boundary. Confirm they bite by breaking the rule this task exists for: change `key_times` to
`(0..FRAMES)` instead of `(0..=FRAMES)`, watch `there_is_one_more_key_time_than_there_are_frames`
fail with "N+1 rule broken", then change it back.

- [x] **Step 3: Run the tests to verify they pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: PASS, 80 tests (73 from the parent plan plus 7).

- [x] **Step 4: Commit**

```bash
git add src-tauri/src/sprite.rs src-tauri/src/main.rs
git commit -m "Add the pet sprite's frame arithmetic"
```

---

## Task 2: Ship the sprites as bundle resources

The native view reads PNGs off disk, so they have to be in the bundle. They also have to stay in
`frontendDist`, because the popover's character picker still loads them over the custom protocol.

**Files:**
- Modify: `src-tauri/tauri.conf.json` (`bundle.resources`)
- Modify: `src-tauri/src/sprite.rs` (the loader)
- Test: inline `mod tests`, plus a build-and-inspect step

**Interfaces:**
- Consumes: `sprite::relative_path` from Task 1.
- Produces: `sprite::resolve_path(app: &AppHandle, character_id: &str, mood: &str) -> Option<PathBuf>`

- [x] **Step 1: Add the resources key**

`grep resources src-tauri/tauri.conf.json` returns nothing today: the key does not exist and must
be added. In `src-tauri/tauri.conf.json`, inside `bundle`, after `"icon"`:

```json
    "resources": {
      "../src/assets/pet": "pet"
    },
```

The **map form** is deliberate. The array form would land the files at
`Contents/Resources/_up_/src/assets/pet/...`, because Tauri encodes the `../` in the path. The map
form puts them at `Contents/Resources/pet/...`, which is what `sprite::relative_path` expects.

- [x] **Step 2: Write the loader**

Append to the pure half of `src-tauri/src/sprite.rs`, above the tests:

```rust
/// The absolute path of a sprite inside the running bundle, or `None` if the resource directory
/// cannot be resolved.
///
/// Separate from `relative_path` so the layout is testable without an `AppHandle`.
pub fn resolve_path(app: &tauri::AppHandle, character_id: &str, mood: &str) -> Option<PathBuf> {
    use tauri::Manager;
    let dir = app.path().resource_dir().ok()?;
    Some(dir.join(relative_path(character_id, mood)))
}
```

- [x] **Step 3: Build and confirm the sprites actually land where the code looks**

Run:

```sh
cd src-tauri && cargo tauri build --debug --bundles app && cd ..
find "src-tauri/target/debug/bundle/macos/Momentum Mascot.app/Contents/Resources/pet" -name '*.png' | sort
```

Expected: fifteen files, `pet/07/{asleep,awake,comeback,dozing,run}.png` and the same for `12`
and `20`. If they are under `Resources/_up_/` instead, the map form of the key was not applied.

Also confirm the popover's copies are still served. **Corrected while running this step:** the
`ls` originally written here pointed at `Contents/Resources/pet/07/dozing.png`, which is the new
bundle resource, not the popover's copy, so it could not have detected the failure it was for.
`frontendDist` assets are embedded in the binary and served over the custom protocol, so that is
where to look:

```sh
B="src-tauri/target/debug/bundle/macos/Momentum Mascot.app/Contents/MacOS/momentum-mascot"
strings -a "$B" | grep -E '^/assets/pet/[0-9]+/dozing\.png$' | sort
grep -n 'assets/pet' src/popover.js
```

Expected: three embedded paths, `/assets/pet/{07,12,20}/dozing.png`, and `popover.js:49` still
referencing `assets/pet/${id}/dozing.png`. That is a different copy from the bundle resource and
both are needed.

- [x] **Step 4: Run the tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: PASS, 80 tests.

- [x] **Step 5: Commit**

```bash
git add src-tauri/tauri.conf.json src-tauri/src/sprite.rs
git commit -m "Ship the pet sprites as bundle resources"
```

---

## Task 3: The sprite view, drawn on top of the existing webview

The first task that draws anything. It deliberately does **not** touch the window yet: the view is
installed as a subview of tao's content view, in front of the still-present webview, so the
animation, the pixel crispness and the centring can all be seen and judged before the window type
changes underneath them. That keeps the largest change in this plan from landing on top of an
unverified renderer.

**Files:**
- Modify: `src-tauri/Cargo.toml` (add `objc2-quartz-core`)
- Modify: `src-tauri/src/sprite.rs` (the native half)
- Modify: `src-tauri/src/pet.rs` (install the view)

**Interfaces:**
- Consumes: all of Task 1 and Task 2.
- Produces:
  - `sprite::SpriteView`, an `NSView` subclass, `MainThreadOnly`
  - `sprite::SpriteView::install(window_ns: *mut c_void, app: &AppHandle) -> Option<Retained<SpriteView>>`
  - `SpriteView::set_mood(&self, mood: &str, character_id: &str)`

- [x] **Step 1: Add the Core Animation dependency**

In `src-tauri/Cargo.toml`, under the macOS target dependencies:

```toml
# Core Animation, for the pet's sprite layer. Already in Cargo.lock at 0.3.2 as a transitive
# dependency of objc2-app-kit, so naming it here costs a line and no compilation.
objc2-quartz-core = "0.3"
```

Run: `cargo build --manifest-path src-tauri/Cargo.toml`
Expected: compiles, and `Cargo.lock` is unchanged apart from the direct-dependency edge.

- [x] **Step 2: Write the native half**

Append to `src-tauri/src/sprite.rs`:

```rust
// --------------------------------------------------------------------------------------
// native
// --------------------------------------------------------------------------------------

#[cfg(target_os = "macos")]
mod view {
    use std::cell::RefCell;
    use std::ffi::c_void;

    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2::{define_class, msg_send, DefinedClass, MainThreadOnly};
    use objc2_app_kit::{NSImage, NSView};
    use objc2_foundation::{
        MainThreadMarker, NSArray, NSNumber, NSPoint, NSRect, NSSize, NSString, NSValue,
    };
    use objc2_quartz_core::{
        kCAAnimationDiscrete, kCAFilterNearest, CAKeyframeAnimation, CALayer,
        CATransform3DIdentity, CATransform3DMakeScale,
    };

    use super::{cell_origin, cell_side, duration, frame_rect, key_times, resolve_path, FRAMES};

    #[derive(Default)]
    pub struct SpriteState {
        pub mood: String,
        pub character_id: String,
        /// Which way the character faces. Only the run strip is ever flipped.
        pub facing_left: bool,
    }

    pub struct Ivars {
        /// **The sprite is a SUBLAYER, never the root layer.**
        ///
        /// The hosting view's root layer stays a plain container, and this carries `contents`,
        /// `contentsRect`, `transform`, `magnificationFilter` and `contentsScale`. Layer-hosting
        /// buys the right to build the layer *tree*; it does not buy a fight with AppKit over the
        /// root layer's geometry, which AppKit must still position to honour the view's frame. The
        /// 10.13 AppKit Release Notes clobber list names `transform`, `bounds`, `position` and
        /// `frame` on "an NSView's layer" with no carve-out for hosting, and `NSView` exposes no
        /// `transform` cover property, so the horizontal flip has nowhere legal to live on a root
        /// layer. Whether a transform on a hosting root layer actually breaks is genuinely
        /// ambiguous; the sublayer costs nothing and makes the question moot, because AppKit never
        /// touches sublayers under any reading of any document.
        ///
        /// Three wins fall out of it: the centring is free, the flip is exact because the default
        /// `anchorPoint` of (0.5, 0.5) mirrors about the cell's own centre, and `contentsScale`
        /// is unambiguously ours to maintain.
        sprite: Retained<CALayer>,
        app: tauri::AppHandle,
        state: RefCell<SpriteState>,
    }

    define_class!(
        // SAFETY: NSView has no subclassing requirements beyond the main thread, and this class
        // implements no Drop.
        #[unsafe(super(NSView))]
        #[thread_kind = MainThreadOnly]
        #[name = "MomentumSpriteView"]
        #[ivars = Ivars]
        pub struct SpriteView;

        impl SpriteView {
            /// **Must return YES or the pet stops responding to clicks entirely.**
            ///
            /// `NSView.acceptsFirstMouse(for:)` "ignores event and returns false" by default, and
            /// a mouse-down in a non-key window "isn't sent to the NSView object over which the
            /// mouse click occurs". The panel is nonactivating (`pet.rs`'s `make_panel`),
            /// `becomesKeyOnlyIfNeeded` is YES, and the app is `ActivationPolicy::Accessory`, so
            /// the panel is **never key** and every click is structurally first-mouse. This works
            /// today only because tao already handles it (`tao/view.rs:255-257` registers the
            /// selector, `:1148` returns YES); a fresh subclass inherits `false`.
            ///
            /// This is the single most likely way the rewrite presents as "the panel regressed".
            #[unsafe(method(acceptsFirstMouse:))]
            fn accepts_first_mouse(&self, _event: *mut AnyObject) -> bool {
                true
            }

            /// `NSView` "passes the event up the responder chain" if a context menu is
            /// unhandled, so returning nil is not full suppression. `menu` is also set to nil at
            /// install time; this is the belt to that braces.
            #[unsafe(method(rightMouseDown:))]
            fn right_mouse_down(&self, _event: *mut AnyObject) {
                // Deliberately does not call super.
            }

            /// Moving between displays of different densities. `pet.js:30` had a `resize`
            /// listener for this. Natively it is this, **plus** re-setting `contentsScale`:
            /// `CALayer.contentsScale` says "for layers you create and manage yourself, you must
            /// set the value of this property yourself". Without it the pixel art blurs on a
            /// second display.
            #[unsafe(method(viewDidChangeBackingProperties))]
            fn backing_changed(&self) {
                self.relayout();
            }

            /// The cell is sized from the view, so a frame change re-lays it out. The window is
            /// resized by `pet::setup` *after* the view exists, so this fires at least once.
            #[unsafe(method(setFrameSize:))]
            fn set_frame_size(&self, size: NSSize) {
                let _: () = unsafe { msg_send![super(self), setFrameSize: size] };
                self.relayout();
            }

            /// `cursor: grab` from `pet.html:28`. `NSCursor` has no "grab", and the open and
            /// closed hand cursors are its native equivalents.
            #[unsafe(method(resetCursorRects))]
            fn reset_cursor_rects(&self) {
                let bounds = self.bounds();
                let cursor = objc2_app_kit::NSCursor::openHandCursor();
                unsafe { self.addCursorRect_cursor(bounds, &cursor) };
            }
        }
    );

    impl SpriteView {
        /// Add the sprite view as a **subview** of the window's content view.
        ///
        /// **It must not replace the content view.** `tao/window.rs:535-536` calls
        /// `setContentView` *and* `setInitialFirstResponder`, and `:539` builds the IME input
        /// context on that view. The class it installs (`tao/view.rs:222-258`) is where
        /// `mouseDown:`, `mouseDragged:`, `scrollWheel:`, `frameDidChange:`, `cancelOperation:`
        /// and `acceptsFirstMouse:` all live. `setContentView:` would throw all of that away,
        /// including the `frameDidChange` plumbing the pet's sizing depends on.
        ///
        /// Hit-testing prefers subviews, so this receives the mouse events. **Never override
        /// `hitTest:` on this class**: if the sprite view ever loses the hit test the click falls
        /// through to tao's view, which returns YES from `tao/view.rs:1148` and routes `mouseDown`
        /// into tao's own handler, so the pet appears to accept clicks while the drag never
        /// starts, with nothing logged.
        pub fn install(window_ns: *mut c_void, app: &tauri::AppHandle) -> Option<Retained<Self>> {
            let mtm = MainThreadMarker::new()?;
            let ns = window_ns as *mut AnyObject;
            if ns.is_null() {
                return None;
            }
            let content: *mut AnyObject = unsafe { msg_send![ns, contentView] };
            if content.is_null() {
                return None;
            }
            let bounds: NSRect = unsafe { msg_send![content, bounds] };

            let sprite = unsafe { CALayer::new() };
            sprite.setMagnificationFilter(unsafe { kCAFilterNearest });

            let this = Self::alloc(mtm).set_ivars(Ivars {
                sprite: sprite.clone(),
                app: app.clone(),
                state: RefCell::new(SpriteState {
                    mood: "awake".into(),
                    character_id: "07".into(),
                    // `..Default::default()` so Task 4 can add `busy` and `drag` without
                    // coming back to edit this literal.
                    ..Default::default()
                }),
            });
            let this: Retained<Self> = unsafe { msg_send![super(this), initWithFrame: bounds] };

            // Layer-HOSTING, in this order: assign our own layer, then ask for layer backing.
            // `NSView.wantsLayer` says "do not add subviews to a layer-hosting view"; this view
            // adds none, so hosting is available to it. That constraint is about a hosting view's
            // own subviews and says nothing about the hosting view being someone else's subview.
            let root = unsafe { CALayer::new() };
            root.addSublayer(&sprite);
            unsafe { this.setLayer(Some(&root)) };
            this.setWantsLayer(true);

            // `pet::setup` calls `set_size` AFTER the window exists, so a subview added with a
            // fixed frame would be the wrong size and would leave a dead margin owned by tao,
            // which is the silent way to lose the hit test.
            this.setAutoresizingMask(
                objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable
                    | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable,
            );

            // `title="Momentum Mascot"` from `pet.html:78`. Native tooltips work on an NSView,
            // unlike the `title` attribute on a span in the popover's webview, which was measured
            // not to render at all.
            unsafe { this.setToolTip(Some(&NSString::from_str("Momentum Mascot"))) };
            unsafe { this.setMenu(None) };

            unsafe { let _: () = msg_send![content, addSubview: &*this] };

            this.relayout();
            Some(this)
        }

        /// Size and centre the cell, keep `contentsScale` current, and restart the animation.
        fn relayout(&self) {
            let bounds = self.bounds();
            let side = bounds.size.width.min(bounds.size.height);
            let cell = cell_side(side);
            let x = cell_origin(bounds.size.width, cell);
            let y = cell_origin(bounds.size.height, cell);

            let sprite = &self.ivars().sprite;
            sprite.setFrame(NSRect::new(NSPoint::new(x, y), NSSize::new(cell, cell)));

            // From the WINDOW, not the view: `CALayer.contentsScale` says "for layers you
            // create and manage yourself, you must set the value of this property yourself".
            sprite.setContentsScale(self.backing_scale());

            let (mood, character_id, facing_left) = {
                let s = self.ivars().state.borrow();
                (s.mood.clone(), s.character_id.clone(), s.facing_left)
            };
            self.paint(&mood, &character_id, facing_left);
        }

        /// Load the strip, apply the flip, and start the discrete keyframe animation.
        fn paint(&self, mood: &str, character_id: &str, facing_left: bool) {
            let sprite = &self.ivars().sprite;

            if let Some(path) = resolve_path(&self.ivars().app, character_id, mood) {
                let s = NSString::from_str(&path.to_string_lossy());
                if let Some(image) = NSImage::alloc().initWithContentsOfFile(&s) {
                    let obj: *const AnyObject = (&*image as *const NSImage).cast();
                    unsafe { sprite.setContents(Some(&*obj)) };
                }
            }

            // The strip is centred in its cell, so with the sublayer's default anchorPoint of
            // (0.5, 0.5) this mirrors about the cell's own centre and turns the character in
            // place rather than shunting it across the window.
            sprite.setTransform(if facing_left {
                unsafe { CATransform3DMakeScale(-1.0, 1.0, 1.0) }
            } else {
                unsafe { CATransform3DIdentity }
            });

            let values: Vec<Retained<NSValue>> = (0..FRAMES)
                .map(|i| {
                    let (x, y, w, h) = frame_rect(i);
                    unsafe {
                        NSValue::valueWithRect(NSRect::new(
                            NSPoint::new(x, y),
                            NSSize::new(w, h),
                        ))
                    }
                })
                .collect();
            let times: Vec<Retained<NSNumber>> =
                key_times().into_iter().map(NSNumber::new_f64).collect();

            let anim = CAKeyframeAnimation::animationWithKeyPath(Some(&NSString::from_str(
                "contentsRect",
            )));
            unsafe { anim.setValues(Some(&NSArray::from_retained_slice(&values))) };
            anim.setKeyTimes(Some(&NSArray::from_retained_slice(&times)));
            anim.setCalculationMode(unsafe { kCAAnimationDiscrete });
            anim.setDuration(duration(mood));
            anim.setRepeatCount(f32::INFINITY);
            anim.setRemovedOnCompletion(false);

            sprite.addAnimation_forKey(&anim, Some(&NSString::from_str("walk")));
        }

        /// Called from `pet.rs` on the main thread only.
        pub fn set_mood(&self, mood: &str, character_id: &str) {
            {
                let mut s = self.ivars().state.borrow_mut();
                s.mood = mood.to_string();
                s.character_id = character_id.to_string();
                s.facing_left = false;
            }
            self.paint(mood, character_id, false);
        }
    }
}

#[cfg(target_os = "macos")]
pub use view::SpriteView;
```

**Note for the implementer.** `relayout` and `paint` call `self.backing_scale()`, which Task 4
step 2 defines alongside the mouse handlers. Add that one helper now rather than waiting, or this
task does not compile:

```rust
        fn backing_scale(&self) -> f64 {
            self.window().map(|w| w.backingScaleFactor()).unwrap_or(1.0)
        }
```

Every signature in this file was read out of the vendored crates and is listed in the
Verified API surface table above. Expect the first compile of this task to need small fixes around
`define_class!` and `Retained` conversions; that is what step 3 is for, and the table is what to
work from rather than guessing.

**Corrections found while executing this task.** The code committed in `src-tauri/src/sprite.rs`
is the source of truth; the block above is what was written before compiling it. Every item here
was a real compile error or a real deprecation, not a style preference.

- **`objc2-quartz-core` needs `default-features = false` and six features, not zero.** Every class
  is behind its own feature. Worse, `default` turns on *all* of them, which drags in
  `objc2-metal`, `objc2-core-video` and `objc2-core-graphics`, one of them entirely new to the
  lock file, contradicting this step's claim that the lock would be unchanged. With defaults off
  and `std`, `CALayer`, `CAAnimation`, `CAMediaTiming`, `CATransform3D`, `CATransaction` and
  `objc2-core-foundation` named, the lock gains only the dependency edge, as promised.
- **There is no `CAKeyframeAnimation` feature.** That class lives under `CAAnimation`. Naming it
  fails resolution outright.
- **`objc2-core-foundation` is the non-obvious one.** It is not a class, it gates every method
  whose signature mentions a Core Foundation type: `setFrame`, `setContentsScale`, `setTransform`,
  `setDuration` and all of `CATransform3D`. Without it the errors read as missing *methods*, which
  sends you looking in the wrong place entirely.
- **`CAMediaTiming` must be imported as a trait** for `setDuration` and `setRepeatCount`, which are
  protocol methods rather than inherent ones.
- **Several calls are safe, not `unsafe`.** `CALayer::new`, `setLayer`, `setToolTip`,
  `addCursorRect_cursor`, `addSublayer`, `setFrame`, `setContentsRect`, `setContentsScale`,
  `setMagnificationFilter`, `setTransform` and `addAnimation_forKey` are all safe in these crate
  versions. Wrapping them costs only an `unused_unsafe` warning, but the reverse would not
  compile, so the list is worth having.
- **`CATransform3DMakeScale` is deprecated** in favour of `CATransform3D::new_scale`. Same
  arguments, and it is safe.
- **`NSImage::initWithContentsOfFile` is an associated function**, not a method: it takes
  `this: Allocated<Self>`, so the call is
  `NSImage::initWithContentsOfFile(NSImage::alloc(), &s)`, and `objc2::AllocAnyThread` has to be
  in scope for `alloc()`.
- **`setValues` takes an untyped `NSArray`.** `NSArray::from_retained_slice(&values)` on
  `Vec<Retained<NSValue>>` yields `NSArray<NSValue>`, which does not coerce. The rects are erased
  with `Retained::cast_unchecked::<AnyObject>` as they are built.
- **A missing semicolon.** `unsafe { let _: () = msg_send![content, addSubview: &*this] }` does not
  parse: the `let` needs its terminator inside the block.

- [x] **Step 3: Install it from `pet::setup` and build until it compiles**

In `src-tauri/src/pet.rs`, inside `setup`'s macOS block, after
`crate::appkit::make_transparent(win.ns_window()?);`:

```rust
        // Task 3 only: the sprite view goes on top of the webview so the renderer can be judged
        // before the window type changes underneath it. Task 5 removes the webview.
        if crate::sprite::SpriteView::install(win.ns_window()?, app).is_none() {
            eprintln!("the sprite view could not be installed");
        }
```

Run: `cd src-tauri && cargo tauri build --debug --bundles app`
Expected: compiles. Fix compile errors against the Verified API surface table.

- [x] **Step 4: Look at it**

**Done, and this step's premise was wrong in a way worth recording.** It says "The old webview pet
is behind it and invisible." It is not invisible. It draws, so the window shows **two characters at
once**, the native sprite and the old one, offset from each other. That alone is confusing enough
to read as a rendering bug.

It is worse than cosmetic, because of the responder chain. The sprite view wins hit-testing, being
the frontmost subview, but at this point in the plan it implements no `mouseDown`, so the event
propagates to its **superview, which is tao's content view**, and `pet.js` handles the drag as
before. The window therefore still moves while the native sprite ignores the drag entirely: no run
sprite, no flip. The honest description of this intermediate state is "the pet is duplicated and
dragging half-works", and that is what it was reported as on first look.

None of it is a defect in the renderer, and Task 4 resolves it by handling the mouse natively,
which also stops the propagation. To judge the native renderer alone before then, hide the
webview's content rather than reasoning about the composite:

```rust
        if std::env::var_os("MASCOT_HIDE_WEBVIEW").is_some() {
            let _ = win.eval(
                "document.documentElement.style.background='transparent';\
                 document.body.style.visibility='hidden'",
            );
        }
```

With that set, the verdict was that the pet looks the way it did before the rewrite, in its normal
working state. Combined with step 5's instrument this is what the task needed.

**Also expected at this step, and not a defect:** changing the character in the popover does not
change the sprite. Nothing wires a publish to the native view until Task 5 step 5, and `pet.js`
was the only listener.

**A trap for anyone hand-writing a state file to place the pet for a probe.** `pet_position`
serialises as an object, `{"x": 80, "y": 120}`, not as an array. An array parses as absent, the
field falls back to `None`, and `place` uses its bottom-right default, so the window looks like it
ignored the position when in fact the file did.

- [x] **Step 5: Verify the frame count without an eye test**

Counting 11 against 12 in a 0.75s cycle by eye is exactly the measurement that is easy to get
wrong, which is why Task 1 asserts the arrays directly. Task 1 cannot assert what Core Animation
*does* with those arrays, though, and that is this plan's central claim, so it was measured.

**Done, and the seek-based design this step originally proposed does not work.** Setting
`layer.speed = 0` and driving `layer.timeOffset` across the twelve boundaries reads frame 0 for
every seek, whether the seek and the read share a run-loop turn or not, and whether
`CATransaction::flush()` is called or not. That reading is a **false negative that looks like a
real one**: it is indistinguishable from a sprite that never animates. Diagnostics ruled out
every obvious cause (`animationKeys=1`, `contents=true`, `presentationLayer=true`, cell frame
64x64), so the seek itself is what does not take.

What works is sampling the running animation, and it measures the claim more directly anyway: the
eleven-plateau mistake holds its twelfth frame for zero time, so it is precisely the scheme under
which twelve distinct frames can never be observed. `sprite::view::probeFrames`, run with
`MASCOT_PROBE_FRAMES=1`, samples the presentation layer 125 times at 40ms and reports the distinct
frames and any transition that is not "next frame" or "wrap to zero".

Result, on the signed debug bundle: `distinct={0..11}`, `out_of_order_transitions=0`, **PASS**.
Counter-test with `key_times` set to the eleven-plateau scheme, same build: `distinct={0..10}`,
`out_of_order_transitions=1`, **FAIL**, frame 11 never reaching the screen and the cycle going
10 to 0. Both recorded in `spikes/app-store/RESULTS.md`.

Two smaller things the same probe settled without an eye test: `magnificationFilter` reads back as
`nearest`, and `contentsScale` is 2 and equals the window's `backingScaleFactor`. Those are the
two ways the pixel art goes blurry, so neither needs looking at.

Note for anyone extending the probe: `f64::NAN as i64` is 0 in Rust, so a nil presentation layer
must not be folded into the reading with `unwrap_or(f64::NAN)`. It records as -1.

- [x] **Step 6: Run the tests and commit**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
git add src-tauri/Cargo.toml src-tauri/src/sprite.rs src-tauri/src/pet.rs
git commit -m "Draw the pet with a native sprite layer"
```

---

## Task 4: Interaction

`pet.js:86-135` becomes `mouseDown` / `mouseDragged` / `mouseUp`, keeping the 4pt threshold, the
`cancel_glide` on mouse down, and the `busy` flag that stops a mood tick from walking back the run
sprite mid-glide.

**Files:**
- Modify: `src-tauri/src/sprite.rs` (the mouse handlers)
- Modify: `src-tauri/src/pet.rs` (the direct entry points the handlers call)

**Interfaces:**
- Produces: `pet::on_drag_end(app: &AppHandle, at: (f64, f64)) -> Option<(i32, i32)>`, `pet::on_click(app: &AppHandle)`, both callable from the view.

- [x] **Step 1: What ports and what does not**

Two of the three "it gets simpler" claims in the spec hold, and one does not. Read this before
writing the handlers.

**Accumulate-deltas ports legitimately.** `NSEvent.deltaX`/`deltaY` are documented valid for
mouse-drag events, so `pet.js:106-107`'s accumulation carries over. Note that `pet.js` used
`movementX`/`movementY`, the same quantity by another name.

**AppKit's implicit mouse capture is real.** A `mouseDown:` responder receives the whole drag
through `mouseUp:`, so `pet.js:70-75`'s window-level listener and its deliberate avoidance of
`setPointerCapture` genuinely disappear. The pet is 64x64 and the cursor leaves it immediately;
AppKit keeps delivering anyway.

**The scaling does NOT disappear.** The spec's claim that "the view works in backing coordinates"
is wrong: an `NSView`'s `bounds` is in **points**, and everything downstream is in Tauri physical
pixels (`commands.rs:32`, `pet.rs`'s `place`, `nearest_corner`, `glide_to`). So `pet.js:105-107`'s
`* scale` does not vanish, it becomes `window.backingScaleFactor()`.

**The sign is settled by Apple's own sample, not by documentation.** Apple documents neither units
nor sign nor pointer acceleration for mouse-event deltas. Handling Mouse Events, Listing 4-4 uses
`windowOrigin.y - [theEvent deltaY]` against a y-up `NSWindow.frame.origin`, which establishes
that `deltaY` on a drag is already y-down, and therefore feeds Tauri's y-down `PhysicalPosition`
**unnegated**.

**Considered and rejected: `NSWindow.performDrag(with:)`.** It is Apple's recommended way to drag a
window from a view and it participates in space switching, but Apple notes "a mouse-up event may
not get sent", and both the 4pt click/drag discrimination and the corner snap need `mouseUp`.
Recorded so a reader does not wonder why the harder path was taken.

- [x] **Step 2: Add the handlers**

In `src-tauri/src/sprite.rs`, add to `SpriteState`:

```rust
        /// Set from the moment a drag is recognised to the moment its glide lands. While it is
        /// set, a mood publish updates the stored mood but leaves the running sprite alone, so a
        /// tick cannot walk the idle back mid-glide.
        pub busy: bool,
        pub drag: Option<Drag>,
```

and:

```rust
    #[derive(Clone, Copy)]
    pub struct Drag {
        /// The window's top-left in Tauri physical pixels, accumulated from event deltas.
        x: f64,
        y: f64,
        origin_x: f64,
        origin_y: f64,
        moved: bool,
    }

    /// Points of movement below which a drag is a click. From `pet.js:82`.
    const DRAG_THRESHOLD: f64 = 4.0;
```

Add to the `define_class!` block:

```rust
            #[unsafe(method(mouseDown:))]
            fn mouse_down(&self, event: *mut AnyObject) {
                // A drag that starts while the last glide is still landing would have its own
                // movement fought by the glide's remaining steps.
                crate::pet::cancel_glide();

                let Some(origin) = self.window_origin() else { return };
                self.ivars().state.borrow_mut().drag = Some(Drag {
                    x: origin.0,
                    y: origin.1,
                    origin_x: origin.0,
                    origin_y: origin.1,
                    moved: false,
                });
                let _ = event;
                objc2_app_kit::NSCursor::closedHandCursor().set();
            }

            #[unsafe(method(mouseDragged:))]
            fn mouse_dragged(&self, event: *mut AnyObject) {
                let scale = self.backing_scale();
                let dx: f64 = unsafe { msg_send![event, deltaX] };
                let dy: f64 = unsafe { msg_send![event, deltaY] };

                let (drag, crossed) = {
                    let mut s = self.ivars().state.borrow_mut();
                    let Some(d) = s.drag.as_mut() else { return };
                    d.x += dx * scale;
                    d.y += dy * scale;
                    let crossed = !d.moved
                        && ((d.x - d.origin_x).powi(2) + (d.y - d.origin_y).powi(2)).sqrt()
                            >= DRAG_THRESHOLD * scale;
                    if crossed {
                        d.moved = true;
                        s.busy = true;
                    }
                    if d.moved && dx != 0.0 {
                        s.facing_left = dx < 0.0;
                    }
                    (*d, crossed)
                };

                if crossed {
                    let character_id = self.ivars().state.borrow().character_id.clone();
                    self.paint("run", &character_id, self.ivars().state.borrow().facing_left);
                }
                crate::pet::move_to(&self.ivars().app, (drag.x, drag.y));
            }

            #[unsafe(method(mouseUp:))]
            fn mouse_up(&self, _event: *mut AnyObject) {
                objc2_app_kit::NSCursor::openHandCursor().set();
                let Some(drag) = self.ivars().state.borrow_mut().drag.take() else { return };

                if !drag.moved {
                    self.end_run();
                    crate::pet::on_click(&self.ivars().app);
                    return;
                }

                // Keep running: the backend glides the window, and the facing comes from the
                // corner it reports. `busy` clears when the glide lands.
                match crate::pet::on_drag_end(&self.ivars().app, (drag.x, drag.y)) {
                    Some(target) => {
                        let facing_left = (target.0 as f64) < drag.x;
                        let (mood, character_id) = {
                            let mut s = self.ivars().state.borrow_mut();
                            s.facing_left = facing_left;
                            (s.mood.clone(), s.character_id.clone())
                        };
                        let _ = mood;
                        self.paint("run", &character_id, facing_left);
                    }
                    None => self.end_run(),
                }
            }
```

And the helpers, in `impl SpriteView`:

```rust
        /// The window's top-left in Tauri physical pixels. AppKit measures from the bottom-left
        /// of the primary screen with y increasing upwards; Tauri measures from the top-left with
        /// y increasing downwards, so the flip needs the primary screen's height. This is the
        /// same conversion `pet::macos::visible_bottom_right` already does.
        fn window_origin(&self) -> Option<(f64, f64)> {
            let window = self.window()?;
            let frame = window.frame();
            let mtm = MainThreadMarker::new()?;
            let screens = objc2_app_kit::NSScreen::screens(mtm);
            let primary_height = screens.iter().next()?.frame().size.height;
            let scale = window.backingScaleFactor();
            let top_from_top = primary_height - (frame.origin.y + frame.size.height);
            Some((frame.origin.x * scale, top_from_top * scale))
        }

        fn backing_scale(&self) -> f64 {
            self.window().map(|w| w.backingScaleFactor()).unwrap_or(1.0)
        }

        /// The glide landed, or a click happened: stop running and show the idle mood again.
        pub fn end_run(&self) {
            let (mood, character_id) = {
                let mut s = self.ivars().state.borrow_mut();
                s.busy = false;
                s.facing_left = false;
                (s.mood.clone(), s.character_id.clone())
            };
            self.paint(&mood, &character_id, false);
        }
```

And make `set_mood` respect `busy`:

```rust
        pub fn set_mood(&self, mood: &str, character_id: &str) {
            let busy = {
                let mut s = self.ivars().state.borrow_mut();
                s.mood = mood.to_string();
                s.character_id = character_id.to_string();
                s.busy
            };
            // While a drag or glide is in flight the run sprite owns the screen. The mood is
            // stored and painted when the glide lands.
            if !busy {
                self.paint(mood, character_id, false);
            }
        }
```

- [x] **Step 3: Add the entry points in `pet.rs`**

```rust
/// The pet was clicked rather than dragged.
pub fn on_click(app: &AppHandle) {
    crate::app::toggle_popover(app);
}

/// Follow the cursor during a drag. Physical pixels.
pub fn move_to(app: &AppHandle, to: (f64, f64)) {
    if let Some(win) = app.get_window(PET) {
        let _ = win.set_position(PhysicalPosition::new(to.0, to.1));
    }
}

/// The drag ended: resolve the nearest corner, glide there, remember it, and report the corner so
/// the view can face the run that way. This is `commands::snap_pet` minus the command wrapper.
pub fn on_drag_end(app: &AppHandle, at: (f64, f64)) -> Option<(i32, i32)> {
    let win = app.get_window(PET)?;
    let target = nearest_corner(&win, at)?;
    // `glide_to` takes the handle because its completion hops to the main thread. See Task 5.
    glide_to(app, &win, at, target);

    let state = app.state::<AppState>();
    let to_save = {
        let mut momentum = state.momentum.lock().unwrap();
        momentum.state.pet_position = Some(target);
        momentum.state.clone()
    };
    if let Err(e) = crate::store::save(&state.store_path, &to_save) {
        eprintln!("could not write state: {e}");
    }
    Some(target)
}
```

**Corrections found while executing this task.** The committed code in `src-tauri/src/sprite.rs`
is the source of truth.

- **The sprite strip is authored facing LEFT, and `pet.html:66-68` says the opposite.** Its comment
  claims the run strip is "composed once facing right and flipped here for leftward travel", and
  `pet.js:113` mirrors on `movementX < 0` accordingly. Built that way, the character faces *away*
  from its direction of travel. `contentsRect` selects a sub-rect and cannot mirror anything, so
  the native render of the unmirrored case is pixel-identical to the webview's, which means **the
  old build faced the wrong way too** and this is not a regression this plan introduced. The field
  is now called `flipped` rather than `facing_left`, because "flipped" is what the transform does
  and the facing it produces depends on how the art was drawn. Mirror for **rightward** travel.
- **Repaint on a direction change, not only when the 4pt threshold is crossed.** The handler as
  first written painted once, at the crossing, which fixed the facing for the whole drag: reported
  as "runs, but doesn't turn". Repainting on *every* drag event is the other wrong answer, because
  each `paint` restarts the walk cycle at frame 0 and the run visibly stutters.
- **`addCursorRect:cursor:` does nothing here.** `resetCursorRects` is driven by the active
  window, and the pet's panel is nonactivating with `becomesKeyOnlyIfNeeded` inside an accessory
  app, so it is never key and the override never runs. Replaced with an `NSTrackingArea` using
  `MouseEnteredAndExited | CursorUpdate | ActiveAlways | InVisibleRect`. `ActiveAlways` is the part
  that matters, and `InVisibleRect` means the area follows the view through `pet::setup`'s later
  `set_size`.
- **The drag cursor is pushed, not set.** The pet is 64x64 and the pointer leaves it within a few
  pixels of a drag starting, at which point a merely `set` cursor reverts to the arrow. `push` in
  `mouseDown` and `pop_class` in `mouseUp`, taken before the early return so the stack cannot be
  left unbalanced.
- **Known limitation, and not a regression: the cursor only changes once the app has been
  activated.** Measured with `MASCOT_TRACE=1`: `mouseEntered` fires and `apply_cursor` runs while
  `app_active=false`, but `NSCursor` state belongs to the *active* application, so nothing visible
  happens until the user has clicked the pet or the popover once. The old build's `cursor: grab`
  CSS sat behind the same constraint, in the same nonactivating panel in the same accessory app.
  Fixing it properly is not reachable from public API.

**Beyond the plan, at the user's request: the glide is now two phases.** The plan's premise was
that "the glide stays exactly as it is", and the single ~250ms ease-out over the whole diagonal was
described on sight as the pet being "flung" at the corner. `glide_to` now moves quickly to the
target corner's edge, then runs **horizontally** along it into the corner, linearly at 900 physical
px/s clamped to 280ms..1500ms. Two reasons for that shape: a side-view run sprite is for
horizontal travel, and a distance-derived duration means a long drag takes longer to run home,
which is what makes it read as running rather than sliding. Linear on purpose: easing phase two
would show the character slowing down without its legs slowing with it.

- [x] **Step 4: Build, run, and check every interaction**

Run: `cd src-tauri && cargo tauri build --debug --bundles app`, then run the bundle.

Expected, and check each one:
- A click opens the popover. **If clicks do nothing at all, `acceptsFirstMouse:` is the first suspect.**
- A drag of more than 4pt starts the run animation and the window follows the cursor.
- The character faces the direction of travel and turns **in place**, not shunted sideways.
- Releasing glides to the nearest corner and the run continues until it lands.
- The cursor is an open hand over the pet and a closed hand while dragging.
- A drag of less than 4pt is treated as a click.

- [x] **Step 5: Commit**

```bash
git add src-tauri/src/sprite.rs src-tauri/src/pet.rs
git commit -m "Move the pet's click and drag into the sprite view"
```

---

## Task 5: The pet window stops being a webview window

The blast radius is bounded but wider than two lines, and two of the changes fail **silently**.

**Files:**
- Modify: `src-tauri/Cargo.toml` (`unstable`), `src-tauri/tauri.conf.json` (delete the `pet` entry), `src-tauri/capabilities/default.json`
- Modify: `src-tauri/src/pet.rs`, `src-tauri/src/app.rs`, `src-tauri/src/commands.rs`, `src-tauri/src/main.rs`
- Delete: `src/pet.html`, `src/pet.js`

- [x] **Step 1: Enable the `unstable` feature**

In `src-tauri/Cargo.toml`:

```toml
tauri = { version = "2", features = ["macos-private-api", "tray-icon", "image-png", "unstable"] }
```

Measured: without it, `tauri::window::WindowBuilder` is private and `Manager::get_window` does not
exist. With it, both work.

- [x] **Step 2: Build the window in Rust**

`app.windows` has no webview-less form, so **the `pet` entry in `tauri.conf.json` is deleted, not
converted.** Delete the whole object with `"label": "pet"`.

In `src-tauri/src/pet.rs`, replace the top of `setup`:

```rust
pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    // Built here rather than in tauri.conf.json because `app.windows` has no webview-less form.
    // Per spec 4.1, `WindowBuilder::transparent()` is gated on `macos-private-api` and this build
    // does not have it, so the window is opaque until `appkit::make_transparent` runs below.
    let win = tauri::window::WindowBuilder::new(app, PET)
        .inner_size(SIZE, SIZE)
        .resizable(false)
        .decorations(false)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .visible(false)
        .title("Momentum Mascot")
        .build()?;

    win.set_size(LogicalSize::new(SIZE, SIZE))?;
    place(&win, app)?;
```

- [x] **Step 3: Change the six signatures**

**Six, not two**, and the two lookups fail *silently* if missed: the old `pet.rs:50` early-returns
and `commands.rs:33` is a `?` on an `Option`, so the symptom is a pet that never appears with
nothing logged.

Change `&tauri::WebviewWindow` to `&tauri::window::Window` in: `usable_bounds`, `place`,
`nearest_corner`, `glide_to`. Change `get_webview_window(PET)` to `get_window(PET)` in
`pet::on_drag_end` and `pet::move_to`. The `ns_window()` call is identical on both types, so the
NSPanel reclass reaches the same window.

- [x] **Step 4: Delete the frontend and the three commands**

```sh
git rm src/pet.html src/pet.js
```

**The command surface shrinks by three, not two.** From `main.rs`'s `generate_handler!` and from
`commands.rs`, delete `toggle_popover`, `snap_pet` and `cancel_glide`. `toggle_popover` is the
non-obvious one: the *command*'s only caller was `pet.js:132`, while `tray.rs:50` calls the Rust
function `app::toggle_popover` directly, so deleting `pet.js` makes the command callerless too.

Update `commands.rs`'s module doc, which currently says "Eleven commands": it becomes **eight**.

- [x] **Step 5: Replace the two events with direct calls**

`app.rs:19 MOOD_EVENT` **stays**: `src/popover.js:185` is still a listener. What changes is that
the pet no longer listens, so it must be told directly.

In `src-tauri/src/app.rs`, at the end of `publish`, after the emit:

```rust
    let _ = app.emit(MOOD_EVENT, payload.clone());

    // The pet has no webview to listen any more, so it is told directly. Both direct callers
    // arrive off the main thread: the tick runs on `start_tick`'s thread and the watcher on its
    // own, and touching an NSView off the main thread crashes. `app.emit` marshalled for free;
    // a direct setter does not.
    crate::pet::set_mood(app, payload.mood.as_str(), &payload.character_id);
```

In `src-tauri/src/pet.rs`:

```rust
/// The pet's sprite view, which is main-thread-only. Held here rather than in `AppState` so the
/// main-thread rule lives next to the code that has to honour it.
#[cfg(target_os = "macos")]
static SPRITE: std::sync::OnceLock<crate::sprite::SpriteHandle> = std::sync::OnceLock::new();

/// Tell the pet what mood to show. Safe to call from any thread.
pub fn set_mood(app: &AppHandle, mood: &str, character_id: &str) {
    #[cfg(target_os = "macos")]
    {
        let mood = mood.to_string();
        let character_id = character_id.to_string();
        let _ = app.run_on_main_thread(move || {
            if let Some(handle) = SPRITE.get() {
                handle.set_mood(&mood, &character_id);
            }
        });
    }
}
```

`SpriteHandle` is the wrapper that makes the view storable in a `static`. `Retained<T>` is `Send`
only when `T` is `Send + Sync`, and a `MainThreadOnly` class is neither, so this is where the
`unsafe impl` lives and the main-thread guarantee comes from `run_on_main_thread` being the only
caller. Add to `sprite.rs`, inside `mod view`:

```rust
    /// A main-thread-only view, storable in a `static`.
    ///
    /// **The safety of this type rests entirely on one rule: every method may only be called from
    /// the main thread.** `pet::set_mood` and `pet::end_run` are the only callers and both go
    /// through `AppHandle::run_on_main_thread`. Touching an `NSView` off the main thread crashes,
    /// and the two direct callers this design introduces both arrive off it: the glide runs on its
    /// own `std::thread::spawn` and emits from there, and the mood publish runs on the tick thread
    /// and the watcher thread.
    pub struct SpriteHandle(Retained<SpriteView>);

    // SAFETY: see the type's own comment. The inner view is only ever touched on the main thread.
    unsafe impl Send for SpriteHandle {}
    unsafe impl Sync for SpriteHandle {}

    impl SpriteHandle {
        pub fn new(view: Retained<SpriteView>) -> Self {
            Self(view)
        }

        pub fn set_mood(&self, mood: &str, character_id: &str) {
            debug_assert!(MainThreadMarker::new().is_some(), "off the main thread");
            self.0.set_mood(mood, character_id);
        }

        pub fn end_run(&self) {
            debug_assert!(MainThreadMarker::new().is_some(), "off the main thread");
            self.0.end_run();
        }
    }
```

and re-export it beside `SpriteView`:

```rust
#[cfg(target_os = "macos")]
pub use view::{SpriteHandle, SpriteView};
```

`SpriteView::install` in Task 3 returns the `Retained<SpriteView>`, so `pet::setup` stores it:

```rust
        match crate::sprite::SpriteView::install(win.ns_window()?, app) {
            Some(view) => {
                let _ = SPRITE.set(crate::sprite::SpriteHandle::new(view));
            }
            None => eprintln!("the sprite view could not be installed"),
        }
```

`glide_to`'s completion replaces `GLIDE_DONE_EVENT` with the same hop. Delete the
`GLIDE_DONE_EVENT` constant and the `win.emit` at the end of the glide thread, and add:

```rust
/// The glide landed, so the pet stops running. Safe to call from any thread.
///
/// Only a glide that **completes** calls this: one cancelled by a newer drag stays silent, so a
/// re-grab does not get its run cut short. That was the contract of the `glide-done` event and it
/// is unchanged.
pub fn end_run(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    {
        let _ = app.run_on_main_thread(|| {
            if let Some(handle) = SPRITE.get() {
                handle.end_run();
            }
        });
    }
}
```

and call it from the glide thread's tail, where the emit used to be:

```rust
        // Runs on the glide thread, so it hops.
        end_run(&app);
```

`glide_to` takes `&Window` and needs an `AppHandle` to hop with, so give it one: change the
signature to `glide_to(app: &AppHandle, win: &Window, from: (f64, f64), to: (i32, i32))` and pass
`app` from `on_drag_end`. That is a seventh signature change on top of Task 5 step 3's six.

- [x] **Step 6: Narrow the capabilities**

In `src-tauri/capabilities/default.json`:

- `"windows": ["popover"]`, dropping `"pet"`.
- Keep `core:window:allow-set-size`, because `popover.js:128-131` calls `getCurrentWindow().setSize`.
- Delete `core:window:allow-set-position`, `core:window:allow-outer-position` and
  `core:window:allow-start-dragging`. **This is settled, not an experiment:** `getCurrentWindow`
  appears in `popover.js` only at `:128`, feeding that `setSize`, so those three existed only for
  the pet's drag.

**Corrections found while executing this task.**

- **The command surface goes from twelve to nine, not eleven to eight.** `commands.rs`'s module doc
  said "Eleven commands" and was already off by one: `grep -c '#\[tauri::command\]'` returned twelve
  before this task. Count it rather than trusting the doc. Removing `toggle_popover`, `snap_pet`
  and `cancel_glide` leaves **nine**, and the doc now says so.
- **`paint` needs to dedupe, and this is a real visible bug, not tidiness.** Startup asks for the
  same sprite four times: `install`, then `viewDidChangeBackingProperties`, then two mood publishes.
  Every `paint` reloaded the PNG and re-added the keyframe animation, which restarts the walk cycle
  at frame 0, so the pet visibly stuttered through its first second. It was also measurable: the
  Task 3 probe went from `out_of_order_transitions=0` to `4` on this task's build, purely from
  startup paints landing inside its sampling window. `SpriteState::painted` records the last
  applied `(mood, character, flipped)` and an identical request now returns early. Two of the four
  startup paints are skipped, and the probe is back to `0`.
- **Implicit animations have to be turned off on the sprite layer.** A `CALayer` that is not a
  view's backing layer animates its own property changes over 0.25s by default, so `setFrame`,
  `setTransform` and `setContentsScale` each attached an animation of their own: `animationKeys`
  measured **3**, where Task 3's build measured 1. Both consequences are wrong for pixel art. The
  horizontal flip would interpolate through a squash rather than snapping, and any `contentsRect`
  set outside the keyframe animation would crossfade. Fixed with an `NSNull` action per key for
  `bounds`, `position`, `transform`, `contents`, `contentsRect` and `contentsScale`;
  `animationKeys` is back to 1. Note `setActions` is typed as taking `CAAction` values, so the
  `NSNull` needs a cast to `ProtocolObject<dyn CAAction>`. That cast satisfies the Rust signature
  and claims nothing: Apple documents `NSNull` as the value meaning "no action", which the runtime
  special-cases before it would ever message it.
- **The `MASCOT_HIDE_WEBVIEW` escape hatch added in Task 3 is deleted here**, along with the
  webview it existed to hide.

**Verified on the webview-less build:** the pet appears, animates, drags, runs home and opens the
popover; the popover works end to end with the narrowed capabilities (add a project, cycle a
character, toggle operating, untrack, copy the share card, dismiss with Escape).

**The cursor limitation is confirmed pre-existing.** A/B against the previously installed build:
hovering its pet without clicking first does not change the cursor either. So the click-to-activate
requirement recorded in Task 4 is a property of a nonactivating panel in an accessory app, not
something this rewrite introduced.

- [x] **Step 7: Build and verify nothing regressed**

Run: `cargo test --manifest-path src-tauri/Cargo.toml` then build and run the bundle.

Expected: the pet appears, animates, drags, glides and opens the popover; the popover still works
end to end (add a project, cycle a character, toggle operating, untrack, copy the share card,
dismiss with Escape). **The pet appearing at all is a real test**, because step 3's two lookups
fail silently.

- [x] **Step 8: Commit**

```bash
git add -A
git commit -m "Make the pet window a plain window with no webview"
```

---

## Task 6: Drop the private API and close the gate

**Files:**
- Modify: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`

- [x] **Step 1: Drop the feature**

In `src-tauri/Cargo.toml`, remove `"macos-private-api"` and rewrite the stale comment above it,
which currently says the feature "makes the app ineligible for the Mac App Store":

```toml
# The pet's window transparency is applied by the app itself in `appkit.rs`, because
# `WindowBuilder::transparent()` and `window.transparent(...)` are both gated on
# `macos-private-api` and that feature is what made the app ineligible for the Mac App Store.
# `unstable` is required for `WindowBuilder`, which is how a window with no webview is made.
tauri = { version = "2", features = ["tray-icon", "image-png", "unstable"] }
```

In `src-tauri/tauri.conf.json`: `"macOSPrivateApi": false`.

The pet window's `transparent` flag is already gone with its config entry, and the popover's was
dropped in the parent plan's Task 11.

- [x] **Step 2: The gate**

```sh
cd src-tauri && cargo tauri build --bundles app && cd ..
B="src-tauri/target/release/bundle/macos/Momentum Mascot.app/Contents/MacOS/momentum-mascot"
strings -a "$B" | grep -cE 'drawsBackground|fullScreenEnabled'
```

Expected: **0**. That is the whole point of this plan.

**Measured: 0.** And the two that do not leave measured 1 each, as expected.

And confirm what does not leave, so nobody re-litigates it:

```sh
strings -a "$B" | grep -c 'allowsPictureInPictureMediaPlayback'
strings -a "$B" | grep -c '_wantsKeyDownForEvent'
```

Expected: non-zero for both. Neither is reachable from this codebase and neither is behind a
feature gate; removing them means forking wry and tao. Spec section 2.2.

- [ ] **Step 3: Confirm the parent plan's release gate now passes**

```sh
MASCOT_MAS_ALLOW_PRIVATE_API= tools/release-mas.sh
```

Expected: `private API check: clean`, where before it refused. If `release-mas.sh` does not exist
yet, this step waits for the parent plan's Task 15.

**Waiting.** `tools/release-mas.sh` does not exist yet, so this step is deferred to the parent
plan's Task 15. The check it wraps has been run by hand above and passes.

- [x] **Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "Drop the private API feature the store rejected"
```

---

## Task 7: Manual acceptance

The list from spec section 9 that this plan is responsible for. Run every one against the signed
sandboxed build, not a dev build.

- [x] **Step 1: Sign a sandboxed build**

```sh
cd src-tauri && cargo tauri build --bundles app && cd ..
APP="$PWD/src-tauri/target/release/bundle/macos/Momentum Mascot.app"
codesign --force --sign - --options runtime \
  --entitlements "$PWD/src-tauri/Entitlements.mas.plist" "$APP"
open "$APP"
```

- [x] **Step 2: The regression the NSPanel decision was won against**

Put a Chrome or Safari window into fullscreen. Expected: the pet is **visible over it**, and
clicking the pet neither switches Space nor steals focus.

This is the single most valuable test in the plan. `pet.rs`'s module comment records that ten
`collectionBehavior` values across four window levels were all invisible over fullscreen, and that
what works is changing the *kind* of window it is. Task 5 rebuilt the window; if the reclass no
longer reaches it, this is where it shows.

- [x] **Step 3: Everything else**

- The pet appears at all. Task 5 step 3's lookups fail silently, so this is a real test.
- Drag to all four corners; each glides and lands on the corner.
- A click opens the popover; a sub-4pt drag counts as a click.
- The cursor changes to a closed hand while dragging.
- The character turns in place when running leftward, not shunted sideways.
- Pixel art stays crisp when the pet is dragged to a display of a different density. This is
  `viewDidChangeBackingProperties` plus the manual `contentsScale`.
- The popover still works with the narrowed capabilities: add a project, cycle a character, toggle
  operating, untrack, copy the share card, dismiss with Escape.
- The character picker in the popover still shows three heads, which is the `frontendDist` copy of
  the sprites rather than the bundle resource.
- Sandbox persistence still passes: add a repository, quit, relaunch, still readable.

- [x] **Step 4: Record and commit**

Append the results to `spikes/app-store/RESULTS.md`, then:

```bash
git add spikes/app-store/RESULTS.md
git commit -m "Record the native pet acceptance results"
```

---

## Found while executing, and not in the plan

**The build architecture.** Every build made while executing this plan was **x86_64**, on an arm64
Mac, because an x86_64 Homebrew Rust at `/usr/local/bin` shadows rustup's aarch64 toolchain on
`PATH`. macOS surfaced it as a "Support Ending for Intel-based Apps" notification during Task 7's
acceptance run. Building with `PATH="$HOME/.cargo/bin:$PATH"` gives arm64 with no other change, and
the private API gate was re-run there: still 0. **This is a prerequisite for the parent plan's
Phase 5**, because an x86_64-only bundle cannot be what ships. `tools/release-mas.sh` should resolve
cargo explicitly rather than inheriting `PATH`, and assert `lipo -archs` on its own output. The
user has taken the toolchain itself as theirs to sort out.

**Debug hooks added by this plan needed `#[cfg]`, not `cfg!`.** The Task 3 probe is reachable from
the Objective-C runtime because `define_class!` registers it as a selector, so `cfg!` left seven of
its format strings in the stripped release binary. Detail in `spikes/app-store/RESULTS.md`.

# Self-review notes

**Spec coverage.** Section 4.0: done by the parent plan's Task 3, which is why this plan exists.
4.1, the corrected premise: Global Constraints plus Task 5 step 2 and Task 6. 4.2, why native:
this plan's existence. 4.3, where the sprite view goes and where it must not: Task 3 step 2, with
all four guards (subview not content view, layer-hosting, sprite on a sublayer, autoresizing mask
plus never overriding `hitTest:`). 4.4, the animation and the N+1 rule: Tasks 1 and 3, asserted as
pure unit tests. 4.5, interaction: Task 4, including all four obligations
(`acceptsFirstMouse`, main-thread hops, backing-scale changes, `menu(for:)` not being full
suppression) and all three smaller losses (cursor, tooltip, the load-time first publish, which
Task 5 step 5 covers by making `publish` tell the pet directly). 4.6, all six consequences: Task 5
steps 2 to 6, plus `bundle.resources` in Task 2 and the `frontendDist` duplication in Global
Constraints.

**One thing the spec did not know.** Tauri's `unstable` feature is required. Measured by
compiling both ways. Without it there is no `WindowBuilder` and no `Manager::get_window`, so
spec 4.6's whole approach does not build. This is the single most likely thing to stop an
implementer on day one, which is why it is in Global Constraints rather than buried in a task.

**Two deliberate departures from the spec.**

1. Task 3 installs the sprite view **on top of the still-present webview** and judges the renderer
   there, before Task 5 changes the window. The spec has no such intermediate step. It costs one
   throwaway call site and it means the largest change in the plan does not land on top of an
   unverified renderer.
2. Spec 4.5 says the `devicePixelRatio` scaling "disappears". It does not, and Task 4 step 1 says
   why: an `NSView`'s bounds is in points while everything downstream is in Tauri physical pixels.

**Known rough edge.** Task 3 step 2's `relayout` contains two lines that must be deleted, marked
in the note directly below the code block. They are there to make the point that the backing scale
comes from the window rather than the view. Expect the first compile of Task 3 to need small fixes
around `define_class!` and `Retained` conversions; every signature it uses is in the Verified API
surface table, which is what to work from.
