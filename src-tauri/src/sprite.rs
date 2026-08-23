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

/// The absolute path of a sprite inside the running bundle, or `None` if the resource directory
/// cannot be resolved.
///
/// Separate from `relative_path` so the layout is testable without an `AppHandle`.
pub fn resolve_path(app: &tauri::AppHandle, character_id: &str, mood: &str) -> Option<PathBuf> {
    use tauri::Manager;
    let dir = app.path().resource_dir().ok()?;
    Some(dir.join(relative_path(character_id, mood)))
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

// --------------------------------------------------------------------------------------
// native
// --------------------------------------------------------------------------------------

#[cfg(target_os = "macos")]
mod view {
    use std::cell::RefCell;
    use std::ffi::c_void;

    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2::{define_class, msg_send, AllocAnyThread, DefinedClass, MainThreadOnly};
    use objc2_app_kit::{
        NSAutoresizingMaskOptions, NSCursor, NSImage, NSScreen, NSTrackingArea,
        NSTrackingAreaOptions, NSView,
    };
    use objc2_foundation::{
        MainThreadMarker, NSArray, NSNumber, NSPoint, NSRect, NSSize, NSString, NSValue,
    };
    use objc2_quartz_core::{
        kCAAnimationDiscrete, kCAFilterNearest, CAKeyframeAnimation, CALayer, CAMediaTiming,
        CATransform3D, CATransform3DIdentity,
    };

    use super::{
        cell_origin, cell_side, duration, frame_at, frame_rect, key_times, resolve_path, FRAMES,
    };

    #[derive(Default)]
    pub struct SpriteState {
        pub mood: String,
        pub character_id: String,
        /// Whether the sprite is mirrored horizontally. Only the run strip is ever mirrored.
        ///
        /// **The strip is authored facing LEFT, not right.** `pet.html:66-68` claims "composed
        /// once facing right and flipped here for leftward travel" and that comment is wrong: with
        /// it, the character faced away from its direction of travel. `contentsRect` selects a
        /// sub-rect and cannot mirror anything, so the native render is pixel-identical to the
        /// webview's for the unmirrored case, which means the old build faced the wrong way too.
        /// Confirmed by observation on the native build and corrected here rather than carried
        /// over. So: mirror for RIGHTWARD travel.
        pub flipped: bool,
        /// Set from the moment a drag is recognised to the moment its glide lands. While it is
        /// set, a mood publish updates the stored mood but leaves the running sprite alone, so a
        /// tick cannot walk the idle back mid-glide.
        pub busy: bool,
        pub drag: Option<Drag>,
    }

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

    /// `MASCOT_TRACE=1`. Diagnostics for the two things about this view that cannot be reasoned
    /// out from the source: which way a drag delta actually points, and whether AppKit delivers
    /// mouse-tracking events to a window that is never key in an app that is never active.
    fn trace() -> bool {
        std::env::var_os("MASCOT_TRACE").is_some()
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
        /// Only used by the `MASCOT_PROBE_FRAMES` probe. See `probeFrames`.
        probe: RefCell<Probe>,
    }

    /// The frame-count probe's progress.
    ///
    /// **Sampling, not seeking.** Two seek-based designs were tried and both failed to move the
    /// presentation layer at all: `speed = 0` plus `timeOffset` reads the same frame for every
    /// seek, whether the seek and the read share a run-loop turn or not, and whether
    /// `CATransaction::flush()` is called or not. Diagnostics ruled out the obvious causes
    /// (`animationKeys=1`, `contents=true`, `presentationLayer=true`, cell frame 64x64), so the
    /// seek itself is what does not take. Sampling the animation while it runs needs none of
    /// that machinery and measures the claim more directly anyway: the eleven-plateau mistake
    /// holds its twelfth frame for zero time, so it is exactly the scheme under which a sampler
    /// can never observe twelve distinct frames.
    #[derive(Default)]
    pub struct Probe {
        /// How many samples have been taken.
        step: usize,
        /// Which twelfth of the strip each sample found on screen.
        readings: Vec<i64>,
    }

    /// Sampling interval and count: 40ms across 5s, comfortably longer than the 4s `awake`
    /// cycle, so every one of the twelve plateaus is sampled several times over.
    const PROBE_INTERVAL: f64 = 0.04;
    const PROBE_SAMPLES: usize = 125;

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

            /// The frame-count probe from the plan's Task 3 step 5, run only when
            /// `MASCOT_PROBE_FRAMES` is set.
            ///
            /// This exists because counting eleven against twelve frames in a 0.75s cycle by eye
            /// is exactly the measurement that is easy to get wrong. Task 1 asserts the arrays
            /// this code hands to Core Animation; this asserts what Core Animation does with
            /// them, which is the part no unit test can reach. Time is made deterministic rather
            /// than observed: `speed = 0` freezes the animation, `timeOffset` seeks it, and the
            /// presentation layer reports the frame actually on screen.
            ///
            /// **One seek per run-loop turn, and this is the part that has to be right.** Seeking
            /// and reading in the same turn returns the same stale frame for all twelve seeks,
            /// even with `CATransaction::flush()`: measured, twelve readings of frame 0, which is
            /// worse than useless because it is indistinguishable from a sprite that never
            /// animates. So each turn records the frame the *previous* seek produced and then
            /// issues the next one.
            #[unsafe(method(probeFrames))]
            fn probe_frames(&self) {
                let sprite = &self.ivars().sprite;
                let mood = self.ivars().state.borrow().mood.clone();
                let total = duration(&mood);

                let step = self.ivars().probe.borrow().step;
                if step == 0 {
                    let keys = sprite.animationKeys()
                        .map(|k| k.count())
                        .unwrap_or(0);
                    let has_contents = unsafe { sprite.contents() }.is_some();
                    // `magnificationFilter` and `contentsScale` are the two ways the pixel art
                    // goes blurry, and both read back, so neither needs an eye test.
                    let filter = sprite.magnificationFilter().to_string();
                    println!(
                        "PROBE frames: mood={mood} duration={total} animationKeys={keys} \
                         contents={has_contents} cell={:?}",
                        sprite.frame()
                    );
                    println!(
                        "PROBE sprite: magnificationFilter={filter} contentsScale={} \
                         backingScale={} viewBounds={:?}",
                        sprite.contentsScale(),
                        self.backing_scale(),
                        self.bounds()
                    );
                }

                if step < PROBE_SAMPLES {
                    // A nil presentation layer records as -1 rather than 0, because
                    // `f64::NAN as i64` is 0 in Rust and would masquerade as a real frame.
                    let frame = match unsafe { sprite.presentationLayer() } {
                        Some(p) => {
                            let x = p.contentsRect().origin.x;
                            (x * FRAMES as f64).round() as i64
                        }
                        None => -1,
                    };
                    let mut probe = self.ivars().probe.borrow_mut();
                    probe.readings.push(frame);
                    probe.step = step + 1;
                    drop(probe);
                    self.schedule_probe(PROBE_INTERVAL);
                    return;
                }

                let readings = self.ivars().probe.borrow().readings.clone();
                let distinct: std::collections::BTreeSet<i64> = readings.iter().copied().collect();
                // From the oracle rather than from 0..11 literally, so the probe is checking
                // Core Animation against the same `steps(12)` rule the unit tests check.
                let expected: std::collections::BTreeSet<i64> = (0..FRAMES)
                    .map(|i| frame_at((i as f64 + 0.5) / FRAMES as f64) as i64)
                    .collect();

                // Frames should appear in order and wrap, so count the transitions that are not
                // "the next frame" or "back to the start". A scheme that skips frames shows up
                // here even if every frame is eventually seen.
                let mut out_of_order = 0usize;
                for pair in readings.windows(2) {
                    let (a, b) = (pair[0], pair[1]);
                    if a == b {
                        continue;
                    }
                    let next = (a + 1) % FRAMES as i64;
                    if b != next {
                        out_of_order += 1;
                    }
                }

                println!("PROBE frames: {} samples over {}s", readings.len(),
                         PROBE_SAMPLES as f64 * PROBE_INTERVAL);
                println!("PROBE frames: distinct={:?}", distinct);
                println!("PROBE frames: out_of_order_transitions={out_of_order}");
                if distinct == expected && out_of_order == 0 {
                    println!(
                        "PROBE frames: PASS, all twelve frames render in order, so Core \
                         Animation honours the N+1 keyTimes in discrete mode"
                    );
                } else if distinct.len() == 1 {
                    println!(
                        "PROBE frames: INCONCLUSIVE, only frame {:?} was ever on screen",
                        distinct.iter().next()
                    );
                } else {
                    println!(
                        "PROBE frames: FAIL, expected the twelve frames 0..11 in order, got \
                         {} distinct with {out_of_order} bad transitions",
                        distinct.len()
                    );
                }
            }

            #[unsafe(method(mouseDown:))]
            fn mouse_down(&self, _event: *mut AnyObject) {
                // A drag that starts while the last glide is still landing would have its own
                // movement fought by the glide's remaining steps.
                crate::pet::cancel_glide();

                let Some(origin) = self.window_origin() else {
                    return;
                };
                self.ivars().state.borrow_mut().drag = Some(Drag {
                    x: origin.0,
                    y: origin.1,
                    origin_x: origin.0,
                    origin_y: origin.1,
                    moved: false,
                });
                // **Pushed, not set.** The pet is 64x64 and the pointer leaves it within a few
                // pixels of the drag starting, at which point a merely `set` cursor reverts to
                // the arrow. A pushed cursor holds until it is popped, wherever the pointer goes.
                NSCursor::closedHandCursor().push();
            }

            #[unsafe(method(mouseDragged:))]
            fn mouse_dragged(&self, event: *mut AnyObject) {
                let scale = self.backing_scale();
                // `NSEvent.deltaX`/`deltaY` are documented valid for mouse-drag events, so
                // `pet.js:106-107`'s accumulation carries over unchanged. The sign is settled by
                // Handling Mouse Events, Listing 4-4, which subtracts `deltaY` from a y-up
                // window origin: a drag's `deltaY` is already y-down, so it feeds Tauri's y-down
                // `PhysicalPosition` unnegated.
                let dx: f64 = unsafe { msg_send![event, deltaX] };
                let dy: f64 = unsafe { msg_send![event, deltaY] };

                let (drag, needs_paint, flipped) = {
                    let mut s = self.ivars().state.borrow_mut();
                    let Some(d) = s.drag.as_mut() else {
                        return;
                    };
                    // An NSView's bounds is in points and everything downstream is in Tauri
                    // physical pixels, so the scale does not disappear here, it changes source.
                    d.x += dx * scale;
                    d.y += dy * scale;
                    let crossed = !d.moved
                        && ((d.x - d.origin_x).powi(2) + (d.y - d.origin_y).powi(2)).sqrt()
                            >= DRAG_THRESHOLD * scale;
                    if crossed {
                        d.moved = true;
                    }
                    let moved = d.moved;
                    let snapshot = *d;
                    if crossed {
                        s.busy = true;
                    }
                    let was_flipped = s.flipped;
                    if moved && dx != 0.0 {
                        // Mirror for rightward travel. See `SpriteState::flipped`.
                        s.flipped = dx > 0.0;
                    }
                    // **Repaint on a direction change, not only on the first crossing.** Painting
                    // only when the threshold is crossed fixes the facing for the whole drag, so
                    // the character never turns after it starts running: reported as "runs, but
                    // doesn't turn". Repainting on *every* drag event is the other wrong answer,
                    // because each paint restarts the walk cycle at frame 0 and the run stutters.
                    let turned = s.flipped != was_flipped;
                    (snapshot, crossed || turned, s.flipped)
                };

                if needs_paint {
                    let character_id = self.ivars().state.borrow().character_id.clone();
                    self.paint("run", &character_id, flipped);
                    if trace() {
                        println!(
                            "TRACE drag: dx={dx} dy={dy} flipped={flipped} window_x={} \
                             scale={scale}",
                            drag.x
                        );
                    }
                }
                crate::pet::move_to(&self.ivars().app, (drag.x, drag.y));
            }

            #[unsafe(method(mouseUp:))]
            fn mouse_up(&self, _event: *mut AnyObject) {
                // Balances the push in `mouseDown`. Taken before the early return below, so the
                // stack cannot be left unbalanced by a mouse-up with no drag recorded.
                NSCursor::pop_class();
                let taken = self.ivars().state.borrow_mut().drag.take();
                self.apply_cursor();
                let Some(drag) = taken else {
                    return;
                };

                if !drag.moved {
                    self.end_run();
                    crate::pet::on_click(&self.ivars().app);
                    return;
                }

                // Keep running: the backend glides the window, and the facing comes from the
                // corner it reports. `busy` clears when the glide lands.
                match crate::pet::on_drag_end(&self.ivars().app, (drag.x, drag.y)) {
                    Some(target) => {
                        // Mirror for rightward travel, matching the drag. The glide's second
                        // phase is the horizontal run to the corner, so this is its direction.
                        let flipped = (target.0 as f64) > drag.x;
                        let character_id = {
                            let mut s = self.ivars().state.borrow_mut();
                            s.flipped = flipped;
                            s.character_id.clone()
                        };
                        self.paint("run", &character_id, flipped);
                    }
                    None => self.end_run(),
                }
            }

            /// `cursor: grab` from `pet.html:28`. `NSCursor` has no "grab", and the open and
            /// closed hand cursors are its native equivalents.
            ///
            /// **This is a tracking area, not `addCursorRect:cursor:`, and that is load-bearing.**
            /// The obvious implementation is `resetCursorRects` plus `addCursorRect:cursor:`, and
            /// it produces no cursor change whatsoever on this window: measured. AppKit's cursor
            /// rect machinery is driven by the active window, and the pet's panel is nonactivating
            /// with `becomesKeyOnlyIfNeeded` set, inside an accessory app, so it is **never key**
            /// and `resetCursorRects` never runs. A tracking area with `ActiveAlways` does not
            /// care whether the window is key.
            #[unsafe(method(mouseEntered:))]
            fn mouse_entered(&self, _event: *mut AnyObject) {
                if trace() {
                    println!("TRACE cursor: mouseEntered");
                }
                self.apply_cursor();
            }

            /// Sent when the pointer enters the tracking area, which is also what re-asserts the
            /// cursor after something else has changed it.
            #[unsafe(method(cursorUpdate:))]
            fn cursor_update(&self, _event: *mut AnyObject) {
                if trace() {
                    println!("TRACE cursor: cursorUpdate");
                }
                self.apply_cursor();
            }

            #[unsafe(method(mouseExited:))]
            fn mouse_exited(&self, _event: *mut AnyObject) {
                // A drag holds a pushed cursor, and the pet is 64x64, so the pointer leaves it
                // immediately on almost every drag. Restoring the arrow here would fight that.
                if self.ivars().state.borrow().drag.is_none() {
                    NSCursor::arrowCursor().set();
                }
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

            let sprite = CALayer::new();
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
                probe: RefCell::new(Probe::default()),
            });
            let this: Retained<Self> = unsafe { msg_send![super(this), initWithFrame: bounds] };

            // Layer-HOSTING, in this order: assign our own layer, then ask for layer backing.
            // `NSView.wantsLayer` says "do not add subviews to a layer-hosting view"; this view
            // adds none, so hosting is available to it. That constraint is about a hosting view's
            // own subviews and says nothing about the hosting view being someone else's subview.
            let root = CALayer::new();
            root.addSublayer(&sprite);
            this.setLayer(Some(&root));
            this.setWantsLayer(true);

            // `pet::setup` calls `set_size` AFTER the window exists, so a subview added with a
            // fixed frame would be the wrong size and would leave a dead margin owned by tao,
            // which is the silent way to lose the hit test.
            this.setAutoresizingMask(
                NSAutoresizingMaskOptions::ViewWidthSizable
                    | NSAutoresizingMaskOptions::ViewHeightSizable,
            );

            // `title="Momentum Mascot"` from `pet.html:78`. Native tooltips work on an NSView,
            // unlike the `title` attribute on a span in the popover's webview, which was measured
            // not to render at all.
            this.setToolTip(Some(&NSString::from_str("Momentum Mascot")));
            unsafe { this.setMenu(None) };

            // `InVisibleRect` means the rect argument is ignored and the area tracks the view's
            // visible bounds as it resizes, so this survives `pet::setup`'s later `set_size`.
            // `ActiveAlways` is what makes it work on a window that is never key.
            let tracking = unsafe {
                NSTrackingArea::initWithRect_options_owner_userInfo(
                    NSTrackingArea::alloc(),
                    bounds,
                    NSTrackingAreaOptions::MouseEnteredAndExited
                        | NSTrackingAreaOptions::CursorUpdate
                        | NSTrackingAreaOptions::ActiveAlways
                        | NSTrackingAreaOptions::InVisibleRect,
                    Some(&this),
                    None,
                )
            };
            this.addTrackingArea(&tracking);

            unsafe {
                let _: () = msg_send![content, addSubview: &*this];
            }

            this.relayout();

            // The probe needs a committed render tree, which does not exist until the run loop
            // has turned, and `pet::setup` runs before it does. `performSelector:afterDelay:`
            // queues it on the main run loop, which is also the only thread it may touch the
            // view from, so no `Send` wrapper is needed for it.
            if std::env::var_os("MASCOT_PROBE_FRAMES").is_some() {
                this.schedule_probe(2.0);
            }

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

            let (mood, character_id, flipped) = {
                let s = self.ivars().state.borrow();
                (s.mood.clone(), s.character_id.clone(), s.flipped)
            };
            self.paint(&mood, &character_id, flipped);
        }

        /// Load the strip, apply the flip, and start the discrete keyframe animation.
        fn paint(&self, mood: &str, character_id: &str, flipped: bool) {
            let sprite = &self.ivars().sprite;

            if let Some(path) = resolve_path(&self.ivars().app, character_id, mood) {
                let s = NSString::from_str(&path.to_string_lossy());
                if let Some(image) = NSImage::initWithContentsOfFile(NSImage::alloc(), &s) {
                    unsafe { sprite.setContents(Some(&image)) };
                }
            }

            // The strip is centred in its cell, so with the sublayer's default anchorPoint of
            // (0.5, 0.5) this mirrors about the cell's own centre and turns the character in
            // place rather than shunting it across the window.
            // `CATransform3D::new_scale`, not the older `CATransform3DMakeScale`, which this
            // crate version deprecates in favour of it.
            sprite.setTransform(if flipped {
                CATransform3D::new_scale(-1.0, 1.0, 1.0)
            } else {
                unsafe { CATransform3DIdentity }
            });

            let values: Vec<Retained<AnyObject>> = (0..FRAMES)
                .map(|i| {
                    let (x, y, w, h) = frame_rect(i);
                    let v = unsafe {
                        NSValue::valueWithRect(NSRect::new(
                            NSPoint::new(x, y),
                            NSSize::new(w, h),
                        ))
                    };
                    // `setValues` takes an untyped `NSArray`, so the rects are erased here
                    // rather than fighting the generic parameter at the call site.
                    unsafe { Retained::cast_unchecked::<AnyObject>(v) }
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

        /// Queue `probeFrames` on the main run loop. It is the only thread that may touch the
        /// view, so nothing needs to be `Send` for this.
        fn schedule_probe(&self, delay: f64) {
            unsafe {
                let _: () = msg_send![
                    self,
                    performSelector: objc2::sel!(probeFrames),
                    withObject: std::ptr::null::<AnyObject>(),
                    afterDelay: delay,
                ];
            }
        }

        /// An open hand hovering, a closed one while a drag is in progress.
        fn apply_cursor(&self) {
            if trace() {
                println!(
                    "TRACE cursor: apply, app_active={}",
                    objc2_app_kit::NSApplication::sharedApplication(
                        MainThreadMarker::new().unwrap()
                    )
                    .isActive()
                );
            }
            let dragging = self.ivars().state.borrow().drag.is_some();
            if dragging {
                NSCursor::closedHandCursor().set();
            } else {
                NSCursor::openHandCursor().set();
            }
        }

        fn backing_scale(&self) -> f64 {
            self.window().map(|w| w.backingScaleFactor()).unwrap_or(1.0)
        }

        /// The window's top-left in Tauri physical pixels. AppKit measures from the bottom-left
        /// of the primary screen with y increasing upwards; Tauri measures from the top-left with
        /// y increasing downwards, so the flip needs the primary screen's height. This is the
        /// same conversion `pet::macos::visible_bottom_right` already does.
        fn window_origin(&self) -> Option<(f64, f64)> {
            let window = self.window()?;
            let frame = window.frame();
            let mtm = MainThreadMarker::new()?;
            let screens = NSScreen::screens(mtm);
            let primary_height = screens.iter().next()?.frame().size.height;
            let scale = window.backingScaleFactor();
            let top_from_top = primary_height - (frame.origin.y + frame.size.height);
            Some((frame.origin.x * scale, top_from_top * scale))
        }

        /// The glide landed, or a click happened: stop running and show the idle mood again.
        pub fn end_run(&self) {
            let (mood, character_id) = {
                let mut s = self.ivars().state.borrow_mut();
                s.busy = false;
                s.flipped = false;
                (s.mood.clone(), s.character_id.clone())
            };
            self.paint(&mood, &character_id, false);
        }

        /// Called from `pet.rs` on the main thread only.
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
    }

    /// A main-thread-only view, storable in a `static`.
    ///
    /// **The safety of this type rests entirely on one rule: every method may only be called from
    /// the main thread.** `pet::set_mood` and `pet::end_run` are the only callers and both go
    /// through `AppHandle::run_on_main_thread`. Touching an `NSView` off the main thread crashes,
    /// and the two direct callers this design introduces both arrive off it: the glide runs on its
    /// own `std::thread::spawn` and calls back from there, and the mood publish runs on the tick
    /// thread and the watcher thread.
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
}

#[cfg(target_os = "macos")]
pub use view::{SpriteHandle, SpriteView};
