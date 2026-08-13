//! State derivation: section 8.1.
//!
//! A pure function of the tracked timestamps and an injected `now`, with no side effects and
//! no access to the system clock. It is the most testable piece of the system, so it is
//! written that way and kept independently replaceable.

use serde::{Deserialize, Serialize};

pub const DOZING_AFTER: i64 = 24 * 3600;
pub const ASLEEP_AFTER: i64 = 72 * 3600;

/// How long the comeback celebration survives if the popover is never opened (section 4.5).
pub const COMEBACK_CAP: i64 = 30 * 60;

/// What time alone says. These three are the only values that are ever written to disk as
/// `last_displayed_state`, which is why they are their own type: comeback is a *transition*
/// and cannot be a resting state, and a type that cannot represent it cannot persist it.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Rest {
    Awake,
    Dozing,
    Asleep,
}

/// What the user sees. Rest plus the one moment the whole product is for.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Mood {
    Rest(Rest),
    Comeback,
}

impl Rest {
    pub fn as_str(self) -> &'static str {
        match self {
            Rest::Awake => "awake",
            Rest::Dozing => "dozing",
            Rest::Asleep => "asleep",
        }
    }
}

impl Mood {
    pub fn as_str(self) -> &'static str {
        match self {
            Mood::Rest(r) => r.as_str(),
            Mood::Comeback => "comeback",
        }
    }
}

impl Serialize for Mood {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

/// The whole state machine, minus the comeback.
///
/// `latest` is the most recent qualifying commit across every tracked project, or `None`
/// when nothing is tracked or nothing has a commit yet.
///
/// The empty case resolves to awake rather than asleep: someone who has just installed the
/// app should meet a cheerful room, not a character who has already given up on them.
pub fn resting(latest: Option<i64>, now: i64) -> Rest {
    let Some(latest) = latest else {
        return Rest::Awake;
    };
    // A commit stamped in the future (clock skew, a rewritten history, a machine that came
    // back from sleep with a stale clock) counts as recent rather than wrapping round to
    // asleep. Being wrong in the cheerful direction is the correct way to be wrong here.
    let elapsed = (now - latest).max(0);
    if elapsed < DOZING_AFTER {
        Rest::Awake
    } else if elapsed < ASLEEP_AFTER {
        Rest::Dozing
    } else {
        Rest::Asleep
    }
}

/// Does this transition earn a celebration?
///
/// Only `asleep -> awake`. A `dozing -> awake` return does not qualify: the user has to have
/// been gone long enough for coming back to mean something (section 4.5).
pub fn is_comeback(previous: Option<Rest>, current: Rest) -> bool {
    previous == Some(Rest::Asleep) && current == Rest::Awake
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: i64 = 1_760_000_000;

    /// Table-driven across every boundary, per section 14. The boundaries are the whole
    /// point: "just under 24h" and "exactly 24h" are the two cases an implementation can get
    /// wrong without anyone noticing for a day.
    #[test]
    fn derivation_boundaries() {
        let cases: &[(&str, Option<i64>, i64, Rest)] = &[
            ("nothing tracked", None, T, Rest::Awake),
            ("all null", None, T, Rest::Awake),
            ("this second", Some(T), T, Rest::Awake),
            ("one second under 24h", Some(T - DOZING_AFTER + 1), T, Rest::Awake),
            ("exactly 24h", Some(T - DOZING_AFTER), T, Rest::Dozing),
            ("one second under 72h", Some(T - ASLEEP_AFTER + 1), T, Rest::Dozing),
            ("exactly 72h", Some(T - ASLEEP_AFTER), T, Rest::Asleep),
            ("a year", Some(T - 365 * 86400), T, Rest::Asleep),
            ("in the future", Some(T + 86400), T, Rest::Awake),
        ];
        for (name, latest, now, want) in cases {
            assert_eq!(resting(*latest, *now), *want, "case: {name}");
        }
    }

    /// The empty case is called out separately because it is a product decision rather than
    /// an arithmetic one, and a refactor that "simplified" it to asleep would pass every
    /// other test in this file.
    #[test]
    fn an_empty_list_is_cheerful() {
        assert_eq!(resting(None, T), Rest::Awake);
    }

    #[test]
    fn mixed_null_and_present_uses_the_newest() {
        // The caller reduces to a single `latest`; this documents the contract it must hold
        // to, which is max-over-non-null rather than first, last, or average.
        let stamps = [None, Some(T - 100 * 3600), None, Some(T - 3600)];
        let latest = stamps.iter().flatten().copied().max();
        assert_eq!(resting(latest, T), Rest::Awake);
    }

    #[test]
    fn only_asleep_to_awake_celebrates() {
        assert!(is_comeback(Some(Rest::Asleep), Rest::Awake));
        assert!(!is_comeback(Some(Rest::Dozing), Rest::Awake));
        assert!(!is_comeback(Some(Rest::Awake), Rest::Awake));
        assert!(!is_comeback(Some(Rest::Asleep), Rest::Dozing));
        // First run, with nothing on disk yet. A fresh install must not celebrate.
        assert!(!is_comeback(None, Rest::Awake));
    }

    #[test]
    fn rest_round_trips_through_json() {
        // `last_displayed_state` is written and read back across restarts, and comeback
        // detection depends on it surviving intact.
        for r in [Rest::Awake, Rest::Dozing, Rest::Asleep] {
            let json = serde_json::to_string(&r).unwrap();
            assert_eq!(json, format!("\"{}\"", r.as_str()));
            assert_eq!(serde_json::from_str::<Rest>(&json).unwrap(), r);
        }
    }
}
