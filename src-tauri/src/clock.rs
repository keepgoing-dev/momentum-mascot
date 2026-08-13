//! The one source of "now" in the app.
//!
//! Section 8.1 requires the current time to be *injected* into state derivation rather than
//! read from the system clock inside it. This is that injection point, and it is built here
//! rather than retrofitted because it is a few lines now and an invasive change later
//! (section 12, Phase 3).
//!
//! It carries a scale factor, and the scale factor pays for itself twice:
//!
//! 1. **The demo** (section 5.4). The real thresholds are 24 and 72 hours, so a recording
//!    that shows the full arc is impossible at 1x. A timelapse is not a nicety here.
//! 2. **The transition checks** on the definition of done (section 18), which require the
//!    awake to dozing to asleep transitions to be verified from time passing alone, without
//!    restarting the app. Waiting three days per test run is hoping, not verifying, and an
//!    untestable line on a definition of done quietly becomes an unchecked one.
//!
//! It is not a setting, it never appears in the UI, and it never touches `state.json`.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// How often state is re-evaluated in *simulated* seconds (section 8.2).
const TICK_SIMULATED_SECS: f64 = 60.0;

/// A floor on the real interval between ticks, so a large scale factor cannot turn the
/// re-evaluation loop into a spin.
const TICK_FLOOR: Duration = Duration::from_millis(250);

#[derive(Clone, Copy)]
pub struct Clock {
    /// Wall-clock unix seconds at the moment the app started.
    origin_unix: i64,
    /// A monotonic partner for `origin_unix`, so the scaling is immune to the system clock
    /// being adjusted underneath us.
    origin_instant: Instant,
    scale: f64,
}

impl Clock {
    /// The real clock, running at 1x.
    pub fn real() -> Self {
        Self::scaled(1.0)
    }

    pub fn scaled(scale: f64) -> Self {
        Self {
            origin_unix: real_unix_now(),
            origin_instant: Instant::now(),
            scale,
        }
    }

    /// Reads `KEEPGOING_CLOCK_SCALE`, but **only in a debug build**. A release binary
    /// ignores the variable entirely, so there is no way to ship an accelerated clock by
    /// accident and no way for a user to find one.
    pub fn from_env() -> Self {
        if !cfg!(debug_assertions) {
            return Self::real();
        }
        let scale = std::env::var("KEEPGOING_CLOCK_SCALE")
            .ok()
            .and_then(|v| v.trim().parse::<f64>().ok())
            .filter(|s| s.is_finite() && *s >= 1.0)
            .unwrap_or(1.0);
        Self::scaled(scale)
    }

    /// Simulated unix seconds. Equal to the wall clock when the scale is 1.
    pub fn now(&self) -> i64 {
        if self.scale == 1.0 {
            return real_unix_now();
        }
        let elapsed = self.origin_instant.elapsed().as_secs_f64();
        self.origin_unix + (elapsed * self.scale) as i64
    }

    /// Map a timestamp from the real world onto this clock's timeline.
    ///
    /// **Scaling `now` alone is not enough, and getting this wrong is silent.** The first
    /// version of this module scaled only the current time, which is what the spec described,
    /// and it was tested by driving the real state machine at 3600x: awake, dozing and asleep
    /// all arrived exactly on their thresholds, and then a real `git commit` did nothing at
    /// all. The reason is that git writes wall-clock timestamps while the app had moved on to
    /// a simulated one, so a commit made 80 real seconds after startup was born 72 simulated
    /// hours old and read as ancient history.
    ///
    /// The fix is to treat the scale as defining a whole timeline anchored at startup rather
    /// than a faster read of the current moment. Every timestamp entering the app from outside
    /// is mapped into it, so a commit made *now* is *now* at any scale, and the past is
    /// stretched by the same factor as the future.
    ///
    /// At scale 1 this is the identity, so nothing in a release build is affected by any of it.
    pub fn to_simulated(&self, real_unix: i64) -> i64 {
        if self.scale == 1.0 {
            return real_unix;
        }
        self.origin_unix + ((real_unix - self.origin_unix) as f64 * self.scale) as i64
    }

    pub fn scale(&self) -> f64 {
        self.scale
    }

    /// The real time to wait between re-evaluations, being 60 *simulated* seconds.
    ///
    /// This has to scale with the clock. At 360x an unscaled 60 second tick would be six
    /// simulated hours per re-evaluation, and the demo would step through the states in
    /// visible jumps instead of crossing them.
    pub fn tick_interval(&self) -> Duration {
        Duration::from_secs_f64(TICK_SIMULATED_SECS / self.scale).max(TICK_FLOOR)
    }
}

/// The wall clock, untouched by any scaling. Used for the two things that are about a person
/// watching a screen rather than about git: how long the comeback stays up, and nothing else.
pub fn real_unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_clock_tracks_the_wall_clock() {
        let c = Clock::real();
        assert!((c.now() - real_unix_now()).abs() <= 1);
    }

    #[test]
    fn a_scaled_clock_runs_ahead() {
        let c = Clock::scaled(3600.0);
        std::thread::sleep(Duration::from_millis(50));
        // Half a second of real time is at least half an hour of simulated time.
        assert!(c.now() - c.origin_unix >= 60, "scaled clock did not advance");
    }

    /// The regression test for the bug above. A commit landing *now* must read as landing
    /// now, whatever the clock is doing, or the demo cannot show the one moment it exists to
    /// show.
    #[test]
    fn a_commit_made_now_is_now_at_any_scale() {
        for scale in [1.0, 60.0, 3600.0] {
            let c = Clock::scaled(scale);
            let real_now = real_unix_now();
            assert!(
                (c.to_simulated(real_now) - c.now()).abs() <= 1,
                "a commit made now read as stale at {scale}x",
            );
        }
    }

    #[test]
    fn the_past_is_stretched_by_the_same_factor() {
        let c = Clock::scaled(3600.0);
        // An hour ago in the real world is 3600 hours ago on a 3600x timeline. That is the
        // point rather than a side effect: one timeline, one scale, no seam between them.
        let hour_ago = c.origin_unix - 3600;
        assert_eq!(c.to_simulated(hour_ago), c.origin_unix - 3600 * 3600);
    }

    #[test]
    fn a_real_clock_maps_timestamps_untouched() {
        let c = Clock::real();
        assert_eq!(c.to_simulated(1_760_000_000), 1_760_000_000);
    }

    #[test]
    fn tick_scales_down_but_never_to_zero() {
        assert_eq!(Clock::scaled(1.0).tick_interval(), Duration::from_secs(60));
        assert_eq!(Clock::scaled(60.0).tick_interval(), Duration::from_secs(1));
        assert_eq!(Clock::scaled(100_000.0).tick_interval(), TICK_FLOOR);
    }
}
