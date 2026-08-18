//! The character's voice, verbatim from section 4.6, kept as data.
//!
//! Data rather than code on purpose (section 16): a fifth state should be a room sheet and a
//! quote pool, not a new branch. That is an architectural seam worth leaving and costs
//! nothing today.
//!
//! Every line here passes one test: **would this make a tired person feel worse about
//! themselves?** Nothing may be added that fails it. Banned outright: elapsed-time shaming,
//! comparative framing, pleading, and second-person accusation.

use crate::mood::{Mood, Rest};

const AWAKE: [&str; 4] = [
    "Look at you go.",
    "Something moved today. That counts.",
    "I saw that commit. I'm telling everyone.",
    "Certified in motion.",
];

const DOZING: [&str; 4] = [
    "Still warm. I've got the seat.",
    "Taking five. Same here.",
    "Day off? Good. Rest is part of it.",
    "No rush. It'll keep.",
];

const ASLEEP: [&str; 4] = [
    "Dreaming about that thing you're building.",
    "Sleeping, not gone. Wake me whenever.",
    "I'll hold your place. However long it takes.",
    "Zzz. The project's still there. So am I.",
];

const COMEBACK: [&str; 4] = [
    "YOU CAME BACK.",
    "I KNEW IT.",
    "Woke up for this. Worth it.",
    "Best day. Objectively.",
];

pub fn pool(mood: Mood) -> &'static [&'static str] {
    match mood {
        Mood::Rest(Rest::Awake) => &AWAKE,
        Mood::Rest(Rest::Dozing) => &DOZING,
        Mood::Rest(Rest::Asleep) => &ASLEEP,
        Mood::Comeback => &COMEBACK,
    }
}

/// Rotated rather than random, so two consecutive views are never the same line and the
/// sequence is reproducible when recording the demo.
pub fn quote(mood: Mood, turn: usize) -> &'static str {
    let p = pool(mood);
    p[turn % p.len()]
}

/// Relative time for the project list (section 6.3). Factual, and the character never
/// comments on it.
///
/// "a while back" past 30 days is the one that matters: it is where an honest relative time
/// would start to read as an accusation, so the scale simply stops there.
pub fn relative_time(last_at: Option<i64>, now: i64) -> String {
    let Some(then) = last_at else {
        return "no activity yet".into();
    };
    let secs = (now - then).max(0);
    let (mins, hours, days) = (secs / 60, secs / 3600, secs / 86400);
    match (secs, mins, hours, days) {
        (s, _, _, _) if s < 60 => "just now".into(),
        (_, 1, _, _) => "a minute ago".into(),
        (_, m, 0, _) => format!("{m} minutes ago"),
        (_, _, 1, _) => "an hour ago".into(),
        (_, _, h, 0) => format!("{h} hours ago"),
        (_, _, _, 1) => "yesterday".into(),
        (_, _, _, d) if d <= 30 => format!("{d} days ago"),
        _ => "a while back".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_state_has_a_pool_and_rotation_wraps() {
        for mood in [
            Mood::Rest(Rest::Awake),
            Mood::Rest(Rest::Dozing),
            Mood::Rest(Rest::Asleep),
            Mood::Comeback,
        ] {
            assert_eq!(pool(mood).len(), 4);
            assert_eq!(quote(mood, 0), quote(mood, 4));
            assert_ne!(quote(mood, 0), quote(mood, 1));
        }
    }

    #[test]
    fn no_line_shames_the_user() {
        // A crude guard, but it fails loudly if someone later drops in a line built out of
        // an elapsed time, which is the one pattern section 4.6 bans outright.
        for mood in [
            Mood::Rest(Rest::Awake),
            Mood::Rest(Rest::Dozing),
            Mood::Rest(Rest::Asleep),
            Mood::Comeback,
        ] {
            for line in pool(mood) {
                assert!(
                    !line.chars().any(|c| c.is_ascii_digit()),
                    "a quote carrying a number is almost certainly elapsed-time shaming: {line}"
                );
            }
        }
    }

    #[test]
    fn the_relative_scale_reads_the_way_a_person_would_say_it() {
        let now = 1_760_000_000;
        let ago = |s: i64| relative_time(Some(now - s), now);
        assert_eq!(ago(0), "just now");
        assert_eq!(ago(59), "just now");
        assert_eq!(ago(60), "a minute ago");
        assert_eq!(ago(120), "2 minutes ago");
        assert_eq!(ago(3600), "an hour ago");
        assert_eq!(ago(2 * 3600), "2 hours ago");
        assert_eq!(ago(23 * 3600), "23 hours ago");
        assert_eq!(ago(25 * 3600), "yesterday");
        assert_eq!(ago(3 * 86400), "3 days ago");
        assert_eq!(ago(30 * 86400), "30 days ago");
        assert_eq!(ago(31 * 86400), "a while back");
        assert_eq!(ago(365 * 86400), "a while back");
        assert_eq!(relative_time(None, now), "no activity yet");
    }
}
