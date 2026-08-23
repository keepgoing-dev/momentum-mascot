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
