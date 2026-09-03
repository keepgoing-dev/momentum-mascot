//! `state.json`: sections 8.3 and 13.
//!
//! Two contracts here, and both are load-bearing.
//!
//! **Writes are atomic.** Temporary file in the same directory, then rename. A crash
//! mid-write must never leave a truncated state file.
//!
//! **Reads are resilient by contract.** A missing file, an empty file, `{}`, an empty array,
//! missing optional fields, unknown extra fields, and outright invalid JSON all resolve to
//! sane defaults rather than an error. Losing the tracked list is a mild annoyance; refusing
//! to start is a dead product. This is why nothing in this module returns a parse error: it
//! is not that errors are being swallowed, it is that there is no failure here worth
//! reporting to a user who wanted to look at a cartoon character.

use std::path::{Path, PathBuf};

use serde_json::{json, Map, Value};

use crate::mood::Rest;

pub const SCHEMA_VERSION: &str = "3.2";
pub const CHARACTERS: [&str; 3] = ["07", "12", "20"];

/// The id a built mascot is selected by. Deliberately not a member of `CHARACTERS`, which
/// means "the shipped premades" at every use.
pub const CUSTOM_ID: &str = "custom";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
    /// Unix seconds. Written as RFC 3339 so the file stays readable by a human who opens it.
    pub added_at: i64,
    /// `None` for a repository with no commits yet. Subject to the monotonicity rule in
    /// section 9.2: this never decreases for a given project.
    pub last_commit_at: Option<i64>,
    /// Last time a non-ignored working-tree file changed. Factual signal independent of commits.
    pub last_active_at: Option<i64>,
    /// Display-only tag: the user says this project is being worked on outside of commits.
    /// Operating projects are excluded from mood evaluation entirely.
    pub operating: bool,
    /// Base64 of an NSURL security-scoped bookmark for `path`, or `None` for a project added
    /// before bookmarks existed or on a launch where creating one failed. Only the sandboxed
    /// store build needs it; the DMG build creates and resolves it too, because one code path
    /// with no `cfg` is worth more than the bytes it costs. See `scoped.rs`.
    pub bookmark: Option<String>,
}

/// The five generator layers a built mascot is composited from, in the pack's stacking order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomCharacter {
    pub body: String,
    pub eyes: String,
    pub outfit: String,
    pub hair: String,
    pub accessory: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StateFile {
    pub last_displayed_state: Option<Rest>,
    pub character_id: String,
    pub pet_position: Option<(i32, i32)>,
    pub custom_character: Option<CustomCharacter>,
    pub projects: Vec<Project>,
}

impl Default for StateFile {
    fn default() -> Self {
        Self {
            last_displayed_state: None,
            character_id: CHARACTERS[0].to_string(),
            pet_position: None,
            custom_character: None,
            projects: Vec::new(),
        }
    }
}

/// `~/.keepgoing/mascot/state.json`, or `%APPDATA%\KeepGoing\Mascot\state.json` on Windows.
///
/// Section 13 wrote this as `~/.keepgoing/state.json`, and the extra folder is a deliberate
/// correction rather than drift. That directory already exists on the author's machine and is
/// full of other KeepGoing tooling: databases, a socket, logs, `current-tasks.json`. A file
/// called `state.json` sitting among those is exactly the name a sibling tool would also
/// reach for, and the collision would be silent and mutual. Sharing the family directory is
/// the part worth keeping; sharing the namespace is not.
///
/// The `KEEPGOING_MASCOT_STATE` override is debug-only, for the same reason as the clock: the
/// demo needs a throwaway project list, and pointing it somewhere else beats recording
/// against the author's real one and editing it back afterwards.
///
/// **Under App Sandbox this moves and the code does not.** The sandbox redirects `$HOME` itself,
/// so the store build resolves to
/// `~/Library/Containers/dev.keepgoing.momentum-mascot/Data/.keepgoing/mascot/state.json` and the
/// DMG build stays where it always was. Measured on the real app, not inferred. There is
/// deliberately no migration between them: a sandboxed process can discover the real home
/// through `getpwuid` but cannot read it, so a migration could not be written even if one were
/// wanted. This is why the DMG channel is not sandboxed: the channel with existing users keeps
/// its file.
pub fn default_path() -> PathBuf {
    if cfg!(debug_assertions) {
        if let Some(path) = std::env::var_os("KEEPGOING_MASCOT_STATE") {
            return PathBuf::from(path);
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            return PathBuf::from(appdata)
                .join("KeepGoing")
                .join("Mascot")
                .join("state.json");
        }
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    path_in_home(&home)
}

/// Split out from `default_path` so the shape of the path is testable without mutating the
/// process environment, which a parallel test binary cannot do safely.
fn path_in_home(home: &Path) -> PathBuf {
    home.join(".keepgoing").join("mascot").join("state.json")
}

pub fn load(path: &Path) -> StateFile {
    let Ok(text) = std::fs::read_to_string(path) else {
        return StateFile::default();
    };
    from_json(&text)
}

/// Split out from `load` so every resilience case is testable without touching a filesystem.
pub fn from_json(text: &str) -> StateFile {
    let root = serde_json::from_str::<Value>(text)
        .ok()
        .and_then(|v| match v {
            Value::Object(map) => Some(map),
            _ => None,
        })
        .unwrap_or_default();

    // Parsed before character_id, which is only allowed to be CUSTOM_ID when this is Some.
    // All four required layers or nothing: a half-written build must show the picker its "+"
    // again rather than a mascot missing its face.
    let custom = root.get("custom_character").and_then(|v| {
        let f = |k: &str| v.get(k).and_then(Value::as_str).map(str::to_string);
        Some(CustomCharacter {
            body: f("body")?,
            eyes: f("eyes")?,
            outfit: f("outfit")?,
            hair: f("hair")?,
            accessory: f("accessory"),
        })
    });

    StateFile {
        last_displayed_state: root
            .get("last_displayed_state")
            .and_then(|v| serde_json::from_value::<Rest>(v.clone()).ok()),
        // An unknown character id is not an error. A future release that ships more
        // characters must not break the state file of a user who downgrades.
        character_id: root
            .get("character_id")
            .and_then(Value::as_str)
            .filter(|id| CHARACTERS.contains(id) || (*id == CUSTOM_ID && custom.is_some()))
            .unwrap_or(CHARACTERS[0])
            .to_string(),
        custom_character: custom.clone(),
        pet_position: root.get("pet_position").and_then(|v| {
            let x = v.get("x")?.as_i64()? as i32;
            let y = v.get("y")?.as_i64()? as i32;
            Some((x, y))
        }),
        // Per element, so one malformed entry costs that entry rather than the whole list.
        projects: root
            .get("tracked_projects")
            .and_then(Value::as_array)
            .map(|items| items.iter().filter_map(project_from_value).collect())
            .unwrap_or_default(),
    }
}

fn project_from_value(v: &Value) -> Option<Project> {
    let path = PathBuf::from(v.get("path")?.as_str()?);
    if path.as_os_str().is_empty() {
        return None;
    }
    let name = v
        .get("name")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| display_name(&path));
    Some(Project {
        id: v
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(new_id),
        path,
        name,
        added_at: v.get("added_at").and_then(parse_time).unwrap_or(0),
        last_commit_at: v.get("last_commit_at").and_then(parse_time),
        last_active_at: v.get("last_active_at").and_then(parse_time),
        operating: v.get("operating").and_then(Value::as_bool).unwrap_or(false),
        bookmark: v
            .get("bookmark")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

pub fn save(path: &Path, state: &StateFile) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let text = serde_json::to_string_pretty(&to_json(state))?;

    // The temporary file must be in the same directory as the target, because `rename` is
    // only atomic within a filesystem and `/tmp` is not guaranteed to be one.
    let tmp = path.with_extension(format!("json.tmp-{}", std::process::id()));
    {
        use std::io::Write;
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(text.as_bytes())?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path)
}

fn to_json(state: &StateFile) -> Value {
    let mut root = Map::new();
    root.insert("version".into(), json!(SCHEMA_VERSION));
    root.insert(
        "last_displayed_state".into(),
        state
            .last_displayed_state
            .map(|r| json!(r.as_str()))
            .unwrap_or(Value::Null),
    );
    root.insert("character_id".into(), json!(state.character_id));
    root.insert(
        "custom_character".into(),
        state
            .custom_character
            .as_ref()
            .map(|c| {
                json!({
                    "body": c.body, "eyes": c.eyes, "outfit": c.outfit,
                    "hair": c.hair, "accessory": c.accessory,
                })
            })
            .unwrap_or(Value::Null),
    );
    root.insert(
        "pet_position".into(),
        state
            .pet_position
            .map(|(x, y)| json!({ "x": x, "y": y }))
            .unwrap_or(Value::Null),
    );
    root.insert(
        "tracked_projects".into(),
        Value::Array(
            state
                .projects
                .iter()
                .map(|p| {
                    json!({
                        "id": p.id,
                        "path": p.path,
                        "name": p.name,
                        "added_at": format_time(p.added_at),
                        "last_commit_at": p.last_commit_at.map(format_time),
                        "last_active_at": p.last_active_at.map(format_time),
                        "operating": p.operating,
                        "bookmark": p.bookmark,
                    })
                })
                .collect(),
        ),
    );
    Value::Object(root)
}

pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

// --------------------------------------------------------------------------------------
// Time, in the file only
// --------------------------------------------------------------------------------------
// The app works in unix seconds everywhere, because that is what the reflog gives us and
// what state derivation compares. The *file* uses RFC 3339, because section 13's schema does
// and because `state.json` is a file a curious user will open. Both directions are tolerant:
// a plain integer is accepted on read, so a hand-edited file does not lose its projects.

fn parse_time(v: &Value) -> Option<i64> {
    if let Some(n) = v.as_i64() {
        return Some(n);
    }
    parse_rfc3339(v.as_str()?)
}

/// Only the shape this app writes: `YYYY-MM-DDTHH:MM:SSZ`. Anything else falls back to the
/// caller's default, which costs a timestamp rather than the whole entry.
fn parse_rfc3339(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    if b.len() < 20 || b[4] != b'-' || b[7] != b'-' || b[10] != b'T' {
        return None;
    }
    let num = |r: std::ops::Range<usize>| s.get(r)?.parse::<i64>().ok();
    let (y, mo, d) = (num(0..4)?, num(5..7)?, num(8..10)?);
    let (h, mi, sec) = (num(11..13)?, num(14..16)?, num(17..19)?);
    Some(days_from_civil(y, mo, d) * 86400 + h * 3600 + mi * 60 + sec)
}

fn format_time(unix: i64) -> String {
    let (days, rem) = (unix.div_euclid(86400), unix.rem_euclid(86400));
    let (y, mo, d) = civil_from_days(days);
    format!(
        "{y:04}-{mo:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem / 60) % 60,
        rem % 60
    )
}

/// Howard Hinnant's `days_from_civil`, which is the standard way to do this without pulling
/// in a calendar library for two functions.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unknown_character_id_still_falls_back() {
        assert_eq!(from_json(r#"{"version":"3.2","character_id":"custom"}"#).character_id, "07");
        assert_eq!(from_json(r#"{"version":"3.2","character_id":"nonsense"}"#).character_id, "07");
    }

    #[test]
    fn custom_is_kept_when_the_build_is_present() {
        let s = from_json(
            r#"{"version":"3.2","character_id":"custom","custom_character":
                {"body":"Body_01","eyes":"Eyes_01","outfit":"Outfit_01_01","hair":"Hairstyle_05_02"}}"#,
        );
        assert_eq!(s.character_id, "custom");
    }

    #[test]
    fn custom_character_round_trips() {
        let mut state = StateFile::default();
        state.custom_character = Some(CustomCharacter {
            body: "Body_03".into(),
            eyes: "Eyes_02".into(),
            outfit: "Outfit_11_04".into(),
            hair: "Hairstyle_11_03".into(),
            accessory: Some("Accessory_15_Glasses_05".into()),
        });
        let text = serde_json::to_string(&to_json(&state)).unwrap();
        assert_eq!(from_json(&text).custom_character, state.custom_character);
    }

    #[test]
    fn custom_character_accessory_is_optional() {
        let mut state = StateFile::default();
        state.custom_character = Some(CustomCharacter {
            body: "Body_01".into(),
            eyes: "Eyes_01".into(),
            outfit: "Outfit_01_01".into(),
            hair: "Hairstyle_05_02".into(),
            accessory: None,
        });
        let text = serde_json::to_string(&to_json(&state)).unwrap();
        assert_eq!(from_json(&text).custom_character.unwrap().accessory, None);
    }

    #[test]
    fn a_partial_custom_character_is_dropped_not_defaulted() {
        let s = from_json(r#"{"version":"3.2","custom_character":{"body":"Body_01"}}"#);
        assert_eq!(s.custom_character, None);
    }

    #[test]
    fn every_broken_file_shape_loads() {
        // Section 14's list, verbatim. None of these may panic and none may lose the app.
        for text in [
            "",
            "   ",
            "{}",
            "[]",
            "null",
            "not json at all",
            r#"{"tracked_projects": []}"#,
            r#"{"tracked_projects": {}}"#,
            r#"{"version": "9.9", "unknown_field": 42}"#,
            r#"{"character_id": 17}"#,
            r#"{"last_displayed_state": "exploded"}"#,
            r#"{"pet_position": "bottom right"}"#,
            r#"{"tracked_projects": [{"path": 3}, {"no": "path"}]}"#,
            r#"{"tracked_projects"#,
        ] {
            let s = from_json(text);
            assert_eq!(s.character_id, "07", "input: {text}");
            assert!(s.projects.is_empty(), "input: {text}");
            assert_eq!(s.last_displayed_state, None, "input: {text}");
        }
    }

    #[test]
    fn a_v2_file_loads_into_v3_with_sane_defaults() {
        let s = from_json(
            r#"{
                "version": "2.0",
                "tracked_projects": [
                    {
                        "id": "p1",
                        "path": "/a/b",
                        "name": "b",
                        "added_at": "2025-08-01T08:00:00Z",
                        "last_commit_at": "2025-08-02T08:00:00Z"
                    }
                ]
            }"#,
        );
        assert_eq!(s.projects.len(), 1);
        assert_eq!(s.projects[0].id, "p1");
        assert_eq!(s.projects[0].last_active_at, None);
        assert!(!s.projects[0].operating);
    }

    #[test]
    fn one_bad_entry_does_not_take_the_good_ones_with_it() {
        let s = from_json(
            r#"{"tracked_projects": [
                 {"path": "/a/one", "name": "one"},
                 {"broken": true},
                 {"path": "/a/two", "name": "two"}
               ]}"#,
        );
        assert_eq!(s.projects.len(), 2);
        assert_eq!(s.projects[0].name, "one");
        assert_eq!(s.projects[1].name, "two");
    }

    #[test]
    fn a_missing_name_is_derived_from_the_path() {
        let s = from_json(r#"{"tracked_projects": [{"path": "/a/b/my-side-project"}]}"#);
        assert_eq!(s.projects[0].name, "my-side-project");
        assert!(!s.projects[0].id.is_empty(), "a missing id must be generated");
    }

    #[test]
    fn an_unknown_character_falls_back_rather_than_erroring() {
        assert_eq!(from_json(r#"{"character_id": "99"}"#).character_id, "07");
        assert_eq!(from_json(r#"{"character_id": "12"}"#).character_id, "12");
    }

    #[test]
    fn a_round_trip_keeps_everything() {
        let state = StateFile {
            last_displayed_state: Some(Rest::Asleep),
            character_id: "20".into(),
            pet_position: Some((1780, 940)),
            custom_character: Some(CustomCharacter {
                body: "Body_06".into(),
                eyes: "Eyes_04".into(),
                outfit: "Outfit_25_01".into(),
                hair: "Hairstyle_29_03".into(),
                accessory: Some("Accessory_11_Beanie_02".into()),
            }),
            projects: vec![Project {
                id: "a1b2".into(),
                path: PathBuf::from("/Users/someone/Projects/thing"),
                name: "thing".into(),
                added_at: 1_754_035_200,
                last_commit_at: Some(1_755_000_000),
                last_active_at: Some(1_755_000_100),
                operating: true,
                bookmark: None,
            }],
        };
        let back = from_json(&serde_json::to_string(&to_json(&state)).unwrap());
        assert_eq!(back.last_displayed_state, Some(Rest::Asleep));
        assert_eq!(back.character_id, "20");
        assert_eq!(back.pet_position, Some((1780, 940)));
        assert_eq!(back.projects, state.projects);
    }

    #[test]
    fn a_null_last_commit_survives_the_round_trip() {
        // A repository with no commits yet. This must come back as null, not as epoch zero,
        // because zero would read as "committed in 1970" and put the character to sleep.
        let state = StateFile {
            projects: vec![Project {
                id: "x".into(),
                path: PathBuf::from("/a/b"),
                name: "b".into(),
                added_at: 1_754_035_200,
                last_commit_at: None,
                last_active_at: None,
                operating: false,
                bookmark: None,
            }],
            ..Default::default()
        };
        let back = from_json(&serde_json::to_string(&to_json(&state)).unwrap());
        assert_eq!(back.projects[0].last_commit_at, None);
        assert_eq!(back.projects[0].last_active_at, None);
    }

    #[test]
    fn the_written_file_is_readable_by_a_human() {
        let json = to_json(&StateFile {
            projects: vec![Project {
                id: "x".into(),
                path: PathBuf::from("/a/b"),
                name: "b".into(),
                added_at: 1_754_035_200,
                last_commit_at: Some(1_754_035_200),
                last_active_at: None,
                operating: false,
                bookmark: None,
            }],
            ..Default::default()
        });
        let text = serde_json::to_string(&json).unwrap();
        assert!(text.contains("2025-08-01T08:00:00Z"), "got: {text}");
    }

    #[test]
    fn timestamps_round_trip_through_the_file_format() {
        for unix in [0, 1, 1_754_035_200, 1_760_000_000, 2_000_000_000] {
            assert_eq!(parse_rfc3339(&format_time(unix)), Some(unix), "at {unix}");
        }
        // A hand-edited file holding a plain integer keeps its project.
        assert_eq!(parse_time(&json!(1_754_035_200)), Some(1_754_035_200));
        assert_eq!(parse_time(&json!("not a date")), None);
    }

    #[test]
    fn writing_is_atomic_and_leaves_no_litter() {
        let dir = std::env::temp_dir().join(format!("mascot-store-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let path = dir.join("state.json");

        let mut state = StateFile::default();
        state.character_id = "12".into();
        save(&path, &state).unwrap();
        assert_eq!(load(&path).character_id, "12");

        // Overwrite, then confirm the directory holds exactly the state file: a leftover
        // `.tmp` would mean the rename did not happen and the write was not atomic.
        state.character_id = "20".into();
        save(&path, &state).unwrap();
        assert_eq!(load(&path).character_id, "20");
        let entries: Vec<_> = std::fs::read_dir(&dir).unwrap().flatten().collect();
        assert_eq!(entries.len(), 1, "temporary file was left behind");

        // A file that is present but corrupt still starts the app.
        std::fs::write(&path, "{ this is not json").unwrap();
        assert_eq!(load(&path).character_id, "07");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_bookmark_survives_the_round_trip() {
        let state = StateFile {
            projects: vec![Project {
                id: "x".into(),
                path: PathBuf::from("/a/b"),
                name: "b".into(),
                added_at: 1_754_035_200,
                last_commit_at: None,
                last_active_at: None,
                operating: false,
                bookmark: Some("Ym9va21hcms=".into()),
            }],
            ..Default::default()
        };
        let back = from_json(&serde_json::to_string(&to_json(&state)).unwrap());
        assert_eq!(back.projects[0].bookmark.as_deref(), Some("Ym9va21hcms="));
    }

    #[test]
    fn a_file_written_before_bookmarks_existed_still_loads() {
        // The reader is tolerant of missing optional fields by contract, so a 3.0 file loads
        // with no bookmark and degrades to today's behaviour: it works this launch and reports
        // unavailable on the next one under sandbox.
        let s = from_json(
            r#"{"version": "3.0", "tracked_projects": [{"path": "/a/b", "name": "b"}]}"#,
        );
        assert_eq!(s.projects.len(), 1);
        assert_eq!(s.projects[0].bookmark, None);
    }

    #[test]
    fn a_bookmark_of_the_wrong_type_costs_the_bookmark_and_not_the_project() {
        let s = from_json(r#"{"tracked_projects": [{"path": "/a/b", "bookmark": 17}]}"#);
        assert_eq!(s.projects.len(), 1, "the project was dropped");
        assert_eq!(s.projects[0].bookmark, None);
    }

    #[test]
    fn writers_declare_schema_3_2() {
        // A file written with bookmarks is meaningfully different from one without, and the
        // reader has to keep accepting both.
        let text = serde_json::to_string(&to_json(&StateFile::default())).unwrap();
        assert!(text.contains(r#""version":"3.2""#), "got: {text}");
    }

    #[test]
    fn the_state_path_is_home_relative_and_nothing_else() {
        // The whole sandbox story for state.json is that this function does not change. In the
        // DMG build $HOME is /Users/<someone>; in the store build the sandbox redirects it to
        // ~/Library/Containers/dev.keepgoing.momentum-mascot/Data, for the raw environment
        // variable and not only for NSHomeDirectory(). Measured on the real app, spec section
        // 5.3 and spikes/app-store/RESULTS.md. So there is no branch to test, only the shape of
        // the path.
        assert_eq!(
            path_in_home(Path::new("/Users/someone")),
            PathBuf::from("/Users/someone/.keepgoing/mascot/state.json")
        );
        assert_eq!(
            path_in_home(Path::new(
                "/Users/someone/Library/Containers/dev.keepgoing.momentum-mascot/Data"
            )),
            PathBuf::from(
                "/Users/someone/Library/Containers/dev.keepgoing.momentum-mascot/Data/.keepgoing/mascot/state.json"
            )
        );
    }
}
