//! Where a built mascot's art lives, and what counts as a usable one.
//!
//! The strips sit beside `state.json` rather than in the bundle, so they inherit the sandbox
//! behaviour `store::default_path` already works out: the store build lands in the container,
//! the DMG build in `~/.keepgoing/mascot/`, and no code branches on which.

use std::path::{Path, PathBuf};

pub const ROOM_MOODS: [&str; 4] = ["awake", "dozing", "asleep", "comeback"];
pub const PET_MOODS: [&str; 5] = ["awake", "dozing", "asleep", "comeback", "run"];

/// The art directory beside a given `state.json`.
pub fn dir(state_path: &Path) -> PathBuf {
    state_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("custom")
}

/// The path for one art name, or `None` if the name is not one of the nine.
///
/// An allowlist rather than sanitising: the set of legal names is nine strings, so the check
/// is membership. There is no escaping to get wrong.
pub fn relative_art_path(name: &str) -> Option<PathBuf> {
    let (kind, mood) = name.split_once('/')?;
    let ok = match kind {
        "rooms" => ROOM_MOODS.contains(&mood),
        "pet" => PET_MOODS.contains(&mood),
        _ => false,
    };
    ok.then(|| PathBuf::from(kind).join(format!("{mood}.png")))
}

/// Every name the art directory must hold for a built mascot to be renderable.
pub fn art_names() -> Vec<String> {
    ROOM_MOODS
        .iter()
        .map(|m| format!("rooms/{m}"))
        .chain(PET_MOODS.iter().map(|m| format!("pet/{m}")))
        .collect()
}

/// Whether all nine strips are present and non-empty.
///
/// Distinct from `store`'s load-time id filter, which validates the id and cannot see whether
/// the art behind a valid id is on disk.
pub fn has_art(art_dir: &Path) -> bool {
    art_names().iter().all(|n| {
        relative_art_path(n)
            .map(|rel| art_dir.join(rel))
            .and_then(|p| std::fs::metadata(p).ok())
            .is_some_and(|m| m.is_file() && m.len() > 0)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn art_names_are_allowlisted_not_sanitised() {
        for bad in [
            "../state.json",
            "rooms/../../x",
            "rooms/awake/../y",
            "",
            "rooms/AWAKE",
            "rooms",
            "shared/coffee",
            "pet/../rooms/awake",
        ] {
            assert_eq!(relative_art_path(bad), None, "{bad} should be rejected");
        }
        for good in ["rooms/awake", "rooms/comeback", "pet/run", "pet/asleep"] {
            assert!(relative_art_path(good).is_some(), "{good} should be accepted");
        }
    }

    #[test]
    fn there_are_exactly_nine_art_names() {
        assert_eq!(art_names().len(), 9);
    }

    #[test]
    fn a_half_written_cache_is_not_usable_art() {
        let d = std::env::temp_dir().join(format!("mascot-custom-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("rooms")).unwrap();
        std::fs::write(d.join("rooms/awake.png"), b"x").unwrap();
        assert!(!has_art(&d));

        for n in art_names() {
            let p = d.join(relative_art_path(&n).unwrap());
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, b"x").unwrap();
        }
        assert!(has_art(&d));

        std::fs::write(d.join("pet/run.png"), b"").unwrap();
        assert!(!has_art(&d), "an empty strip is not art");

        let _ = std::fs::remove_dir_all(&d);
    }
}
