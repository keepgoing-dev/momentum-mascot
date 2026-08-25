//! The app's own state: tracked projects, the mood they add up to, and the comeback.
//!
//! Everything here that can be pure is pure and takes `now` as an argument. The only IO is
//! `read_reflog`, which is deliberately a free function so that the comeback lifecycle (the
//! part with a real chance of being wrong) is testable without a git repository.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::clock::Clock;
use crate::copy;
use crate::mood::{self, Mood, Rest, COMEBACK_CAP};
use crate::reflog;
use crate::repo::{self, RepoError};
use crate::scoped::{self, ScopedAccess};
use crate::store::{self, Project, StateFile};

pub struct Momentum {
    pub state: StateFile,
    /// Resolved git directory per project id. Kept out of `state.json` because it is derived
    /// from the path and re-resolving it on load costs nothing, and a stored copy would go
    /// stale the first time someone moves a worktree.
    git_dirs: HashMap<String, PathBuf>,
    /// Working tree root per project id. The path the user picked is the root of the project,
    /// so this is just that path. Kept alongside `git_dirs` so the watcher can register both.
    work_trees: HashMap<String, PathBuf>,
    /// Why a project failed to resolve, per project id.
    ///
    /// `resolve_paths` used to discard this error, and `available` was derived from `git_dirs`
    /// alone, so the popover could only ever say "not reachable right now" for four different
    /// causes. Under sandbox one of those causes is specific enough to be worth naming.
    reasons: HashMap<String, &'static str>,
    /// Held security-scoped access, per project id.
    ///
    /// **These live for the whole process and that is not an accident.** `read_commit_time` runs
    /// on every tick (`app.rs`'s `start_tick`) and every watcher event, and the watcher registers
    /// watches later still, so a guard scoped to load would revoke access under a live
    /// `refresh_all`. Anyone adding a fourth map here: `resolve_paths` opens by clearing
    /// `git_dirs` and `work_trees`, and this map is deliberately **replaced** at the end instead,
    /// so the old grants drop only once the new ones are held.
    access: HashMap<String, ScopedAccess>,
    /// When the celebration started, on the **real** clock rather than the scaled one.
    ///
    /// The 30 minute cap is a dwell time on a piece of UI, not a quantity the state machine
    /// derives, and the two must not share a timeline. Scaling it was tested and was wrong in
    /// exactly the way that matters: at 3600x the comeback lasted half a real second, so the
    /// one moment the whole product exists for was over before it could be seen, in the very
    /// recording made to show it. Nothing else about the app speeds up at 3600x either; the
    /// animations do not, and this is the same kind of quantity.
    comeback_since: Option<i64>,
    quote_turn: usize,
    /// Held so that timestamps read out of git can be mapped onto the app's timeline. At the
    /// default scale that mapping is the identity and this does nothing at all.
    clock: Clock,
}

pub struct Snapshot {
    pub mood: Mood,
    pub quote: &'static str,
    pub character_id: String,
    pub projects: Vec<ProjectRow>,
}

pub struct ProjectRow {
    pub id: String,
    pub name: String,
    pub relative: String,
    /// A tracked project whose path has gone is kept and shown as unavailable rather than
    /// silently deleted: a disconnected external drive must not erase the user's list.
    pub available: bool,
    /// Display-only tag: opted out of mood evaluation.
    pub operating: bool,
    /// Why it is unavailable, when it is. `None` when it is available, so the popover falls back
    /// to its own generic line only when there really is nothing more specific to say.
    pub reason: Option<&'static str>,
}

impl Momentum {
    pub fn load(path: &Path, clock: Clock) -> Self {
        let state = store::load(path);
        let mut m = Momentum {
            state,
            git_dirs: HashMap::new(),
            work_trees: HashMap::new(),
            reasons: HashMap::new(),
            access: HashMap::new(),
            comeback_since: None,
            quote_turn: 0,
            clock,
        };
        m.resolve_paths();
        m
    }

    fn resolve_paths(&mut self) {
        self.git_dirs.clear();
        self.work_trees.clear();
        self.reasons.clear();

        // Built locally and assigned at the end rather than cleared first. See the comment on
        // the `access` field: dropping a grant we are about to re-take would close it under a
        // reader.
        let mut access: HashMap<String, ScopedAccess> = HashMap::new();

        // By index, because what the bookmark resolves to is written back into the project it
        // came from.
        for i in 0..self.state.projects.len() {
            let id = self.state.projects[i].id.clone();

            if let Some(resolved) = self.state.projects[i]
                .bookmark
                .as_deref()
                .and_then(scoped::resolve)
            {
                let moved = apply_resolved(&mut self.state.projects[i], &resolved.path);

                // Re-created while access is held, so a moved folder repairs its own entry
                // without asking the user for it again.
                if resolved.stale || moved {
                    if let Some(fresh) = scoped::create(&resolved.path) {
                        self.state.projects[i].bookmark = Some(fresh);
                    }
                }

                access.insert(id.clone(), resolved.access);
            }

            let path = self.state.projects[i].path.clone();
            match repo::resolve(&path) {
                Ok(git_dir) => {
                    self.git_dirs.insert(id.clone(), git_dir);
                }
                Err(e) => {
                    self.reasons.insert(id.clone(), e.message());
                }
            }
            self.work_trees.insert(id, path);
        }

        self.access = access;
    }

    /// Every path that is currently watchable. A project on an unplugged drive simply is not
    /// in this list, and reappears when the drive does.
    pub fn watch_paths(&self) -> (HashMap<String, PathBuf>, HashMap<String, PathBuf>) {
        (self.git_dirs.clone(), self.work_trees.clone())
    }

    /// The single number the whole product runs on: the most recent real activity across every
    /// non-operating tracked project. Not per project, not averaged, not weighted (section 4.4).
    /// Operating projects are excluded; if none are left to evaluate, this returns `None`,
    /// which resolves to Awake.
    pub fn latest(&self) -> Option<i64> {
        let best = self
            .state
            .projects
            .iter()
            .filter(|p| !p.operating)
            .filter_map(|p| {
                let commit = p.last_commit_at.unwrap_or(0);
                let active = p.last_active_at.unwrap_or(0);
                let best = commit.max(active);
                if best > 0 { Some(best) } else { None }
            })
            .max();
        best
    }

    /// Re-read every tracked project's reflog. Returns true if anything moved.
    ///
    /// Re-reading all of them on any event is deliberate: at a handful of projects each read
    /// is a few kilobytes, and the alternative is mapping filesystem events back to projects,
    /// which is more code and more ways to be wrong for no measurable gain.
    pub fn refresh_all(&mut self) -> bool {
        let readings: Vec<(String, Option<i64>)> = self
            .state
            .projects
            .iter()
            .map(|p| {
                let reading = self
                    .git_dirs
                    .get(&p.id)
                    .and_then(|g| read_commit_time(g, &p.path))
                    .map(|ts| self.clock.to_simulated(ts));
                (p.id.clone(), reading)
            })
            .collect();

        let mut changed = false;
        for (id, reading) in readings {
            if let Some(p) = self.state.projects.iter_mut().find(|p| p.id == id) {
                changed |= apply_reading(p, reading);
            }
        }
        changed
    }

    /// The state machine, run forward to `now`.
    ///
    /// This is the only place `last_displayed_state` is written, and it is written with the
    /// **resting** state rather than the mood, because comeback is a transition. Storing a
    /// transition would mean the celebration either re-fires on every restart or never fires
    /// again, depending on which way the bug went.
    /// Two times, deliberately. `now` is the app's timeline, which the demo clock scales and
    /// which every threshold in section 8.1 is measured on. `real_now` is the wall clock, and
    /// the only thing measured on it is how long a piece of UI stays on screen. At the default
    /// scale they are the same number.
    pub fn evaluate(&mut self, now: i64, real_now: i64) -> Mood {
        let rest = mood::resting(self.latest(), now);

        if let Some(since) = self.comeback_since {
            // A celebration nobody attended is not a debt: the cap expires silently, with no
            // badge and no queued notification (section 4.5).
            if real_now - since >= COMEBACK_CAP || rest != Rest::Awake {
                self.comeback_since = None;
            } else {
                self.state.last_displayed_state = Some(rest);
                return Mood::Comeback;
            }
        } else if self.latest().is_some()
            && mood::is_comeback(self.state.last_displayed_state, rest)
        {
            self.comeback_since = Some(real_now);
            self.state.last_displayed_state = Some(rest);
            return Mood::Comeback;
        }

        self.state.last_displayed_state = Some(rest);
        Mood::Rest(rest)
    }

    /// Opening the popover is the resolution (section 4.5): the user sees the full-room
    /// celebration, and it settles back to awake.
    pub fn resolve_comeback(&mut self) {
        self.comeback_since = None;
    }

    pub fn snapshot(&mut self, now: i64, real_now: i64) -> Snapshot {
        let mood = self.evaluate(now, real_now);
        Snapshot {
            mood,
            quote: copy::quote(mood, self.quote_turn),
            character_id: self.state.character_id.clone(),
            projects: self
                .state
                .projects
                .iter()
                .map(|p| ProjectRow {
                    id: p.id.clone(),
                    name: p.name.clone(),
                    relative: if p.operating {
                        "operating".into()
                    } else {
                        let last = match (p.last_commit_at, p.last_active_at) {
                            (Some(c), Some(a)) => Some(c.max(a)),
                            (Some(c), None) => Some(c),
                            (None, Some(a)) => Some(a),
                            (None, None) => None,
                        };
                        copy::relative_time(last, now)
                    },
                    available: self.git_dirs.contains_key(&p.id),
                    operating: p.operating,
                    reason: self.reasons.get(&p.id).copied(),
                })
                .collect(),
        }
    }

    /// Advanced when the popover opens, so repeat views are not identical.
    pub fn next_quote(&mut self) {
        self.quote_turn = self.quote_turn.wrapping_add(1);
    }

    pub fn cycle_character(&mut self) -> String {
        let i = store::CHARACTERS
            .iter()
            .position(|c| *c == self.state.character_id)
            .unwrap_or(0);
        self.state.character_id = store::CHARACTERS[(i + 1) % store::CHARACTERS.len()].to_string();
        self.state.character_id.clone()
    }

    /// Re-adding an existing project is a friendly no-op, not an error (section 7).
    ///
    /// `bookmark` comes from the caller rather than being made here, because it must be created
    /// while the picker's grant is live and only `commands::add_project` knows when that is.
    pub fn add(
        &mut self,
        path: &Path,
        now: i64,
        bookmark: Option<String>,
    ) -> Result<bool, RepoError> {
        let git_dir = repo::resolve(path)?;
        if self.state.projects.iter().any(|p| p.path == path) {
            return Ok(false);
        }
        let mut project = Project {
            id: store::new_id(),
            path: path.to_path_buf(),
            name: store::display_name(path),
            added_at: now,
            last_commit_at: None,
            last_active_at: None,
            operating: false,
            bookmark,
        };
        let reading = read_commit_time(&git_dir, path).map(|ts| self.clock.to_simulated(ts));
        apply_reading(&mut project, reading);
        self.git_dirs.insert(project.id.clone(), git_dir);
        self.work_trees.insert(project.id.clone(), path.to_path_buf());
        self.state.projects.push(project);
        Ok(true)
    }

    pub fn remove(&mut self, id: &str) {
        self.state.projects.retain(|p| p.id != id);
        self.git_dirs.remove(id);
        self.work_trees.remove(id);
    }

    /// Record that a non-ignored file changed in a project's working tree.
    /// The monotonicity rule applies: the timestamp never moves backwards.
    pub fn touch_activity(&mut self, id: &str, now: i64) -> bool {
        let Some(project) = self.state.projects.iter_mut().find(|p| p.id == id) else {
            return false;
        };
        if project.last_active_at.is_some_and(|stored| stored >= now) {
            return false;
        }
        project.last_active_at = Some(now);
        true
    }

    /// Toggle whether a project is in operating mode. Returns the new value.
    pub fn toggle_operating(&mut self, id: &str) -> Option<bool> {
        let project = self.state.projects.iter_mut().find(|p| p.id == id)?;
        project.operating = !project.operating;
        Some(project.operating)
    }
}

/// Section 9.2, steps 1 to 3, in order: scan the reflog backwards for a qualifying commit,
/// and fall back to `HEAD` if that finds nothing.
pub fn read_commit_time(git_dir: &Path, work_tree: &Path) -> Option<i64> {
    std::fs::read_to_string(repo::reflog_path(git_dir))
        .ok()
        .and_then(|text| reflog::last_qualifying(&text))
        .or_else(|| repo::head_commit_time(work_tree))
}

/// Take what a bookmark resolved to and write it back into the project.
///
/// Returns whether the folder moved, which is the caller's cue to re-create the bookmark.
///
/// Pure, and separated from the NSURL call on purpose: a `WithSecurityScope` bookmark can be
/// neither created nor resolved from a `cargo test` binary, because the option needs the sandbox
/// entitlements (`scoped::create_with` documents the measurement). So this function holds the
/// part that can be wrong and can be tested, and the FFI around it is covered by the sandbox
/// persistence test after signing.
///
/// The display name is refreshed alongside the path because it is derived from it and drifts for
/// the same reason. An unchanged path is left completely alone, so a name the user's file already
/// carried is not clobbered on every launch.
fn apply_resolved(project: &mut Project, resolved: &Path) -> bool {
    if resolved == project.path {
        return false;
    }
    project.name = store::display_name(resolved);
    project.path = resolved.to_path_buf();
    true
}

/// Step 4, the monotonicity rule: **`last_commit_at` never decreases for a given project.**
///
/// Without this, checking out an older branch drags the timestamp backwards and puts the
/// character to sleep despite recent work. "When did I last do work here" does not move
/// backwards. An implementer would not infer this rule, which is why it is stated explicitly
/// in the spec and lives in one obvious function here.
fn apply_reading(project: &mut Project, reading: Option<i64>) -> bool {
    let Some(ts) = reading else { return false };
    if project.last_commit_at.is_some_and(|stored| stored >= ts) {
        return false;
    }
    project.last_commit_at = Some(ts);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: i64 = 1_760_000_000;

    fn project(last: Option<i64>) -> Project {
        Project {
            id: "p1".into(),
            path: PathBuf::from("/a/b"),
            name: "b".into(),
            added_at: T,
            last_commit_at: last,
            last_active_at: None,
            operating: false,
            bookmark: None,
        }
    }

    fn with(projects: Vec<Project>, last_displayed: Option<Rest>) -> Momentum {
        Momentum {
            state: StateFile {
                last_displayed_state: last_displayed,
                projects,
                ..Default::default()
            },
            git_dirs: HashMap::new(),
            work_trees: HashMap::new(),
            reasons: HashMap::new(),
            access: HashMap::new(),
            comeback_since: None,
            quote_turn: 0,
            clock: Clock::real(),
        }
    }

    #[test]
    fn an_older_reading_never_wins() {
        let mut p = project(Some(T));
        assert!(!apply_reading(&mut p, Some(T - 86400)), "an older reading moved it");
        assert_eq!(p.last_commit_at, Some(T));

        assert!(!apply_reading(&mut p, Some(T)), "an identical reading counted as a change");
        assert!(apply_reading(&mut p, Some(T + 60)));
        assert_eq!(p.last_commit_at, Some(T + 60));
    }

    #[test]
    fn a_first_reading_lands_on_a_repository_with_no_commits() {
        let mut p = project(None);
        assert!(!apply_reading(&mut p, None));
        assert_eq!(p.last_commit_at, None);
        assert!(apply_reading(&mut p, Some(T)));
    }

    #[test]
    fn the_newest_commit_anywhere_is_what_counts() {
        let m = with(
            vec![
                Project { id: "a".into(), ..project(Some(T - 100 * 3600)) },
                Project { id: "b".into(), ..project(None) },
                Project { id: "c".into(), ..project(Some(T - 3600)) },
            ],
            None,
        );
        assert_eq!(m.latest(), Some(T - 3600));
    }

    #[test]
    fn operating_projects_are_excluded_from_mood() {
        let mut m = with(
            vec![
                Project {
                    id: "a".into(),
                    operating: true,
                    ..project(Some(T))
                },
                Project {
                    id: "b".into(),
                    last_commit_at: Some(T - 100 * 3600),
                    ..project(None)
                },
            ],
            None,
        );
        // Only the non-operating project matters for the mascot.
        assert_eq!(m.latest(), Some(T - 100 * 3600));
        assert_eq!(m.evaluate(T, T), Mood::Rest(Rest::Asleep));

        // Mark the only evaluated project as operating too: nothing left to evaluate.
        m.toggle_operating("b");
        assert_eq!(m.latest(), None);
        assert_eq!(m.evaluate(T, T), Mood::Rest(Rest::Awake));
    }

    #[test]
    fn file_activity_counts_as_much_as_a_commit() {
        let mut m = with(
            vec![Project {
                id: "a".into(),
                last_commit_at: Some(T - 100 * 3600),
                last_active_at: Some(T - 30),
                ..project(None)
            }],
            None,
        );
        assert_eq!(m.latest(), Some(T - 30));
        assert_eq!(m.evaluate(T, T), Mood::Rest(Rest::Awake));
    }

    #[test]
    fn activity_never_moves_backwards() {
        let mut m = with(
            vec![Project {
                id: "p1".into(),
                last_active_at: Some(T - 100 * 3600),
                ..project(Some(T - 100 * 3600))
            }],
            None,
        );
        assert!(!m.touch_activity("p1", T - 200 * 3600));
        assert_eq!(m.state.projects[0].last_active_at, Some(T - 100 * 3600));

        assert!(m.touch_activity("p1", T));
        assert_eq!(m.state.projects[0].last_active_at, Some(T));

        assert!(!m.touch_activity("p1", T - 1));
        assert_eq!(m.state.projects[0].last_active_at, Some(T));
    }

    #[test]
    fn a_commit_after_three_days_asleep_celebrates() {
        let mut m = with(vec![project(Some(T - 100 * 3600))], None);
        assert_eq!(m.evaluate(T, T), Mood::Rest(Rest::Asleep));

        // The commit lands.
        m.state.projects[0].last_commit_at = Some(T);
        assert_eq!(m.evaluate(T, T), Mood::Comeback);
        // It holds, rather than flashing for one tick.
        assert_eq!(m.evaluate(T + 60, T + 60), Mood::Comeback);
        // And it settles at the cap, silently.
        assert_eq!(m.evaluate(T + COMEBACK_CAP, T + COMEBACK_CAP), Mood::Rest(Rest::Awake));
        // Once settled, it does not fire again on the next tick.
        assert_eq!(m.evaluate(T + COMEBACK_CAP + 60, T + COMEBACK_CAP + 60), Mood::Rest(Rest::Awake));
    }

    /// The regression test for the second thing the accelerated clock got wrong.
    ///
    /// At 3600x the app's timeline covers 30 minutes in half a real second, so a cap measured
    /// on that timeline ends the celebration before it can be seen. This was found by
    /// recording it: the log showed comeback and awake half a second apart.
    #[test]
    fn the_comeback_cap_is_wall_clock_and_not_scaled() {
        let mut m = with(vec![project(Some(T - 100 * 3600))], None);
        m.evaluate(T, T);
        m.state.projects[0].last_commit_at = Some(T);
        assert_eq!(m.evaluate(T, T), Mood::Comeback);

        // The app's timeline races an hour ahead while the wall clock moves one second.
        assert_eq!(m.evaluate(T + 3600, T + 1), Mood::Comeback);

        // It ends on the wall clock's terms, and only on those.
        assert_eq!(
            m.evaluate(T + 7200, T + COMEBACK_CAP),
            Mood::Rest(Rest::Awake)
        );
    }

    /// Section 4.5's two halves, in the order `app.rs` performs them. The middle assertion is
    /// the one with history: `show_popover` used to resolve before it published, so the room
    /// evaluated as `awake` in the very call that was meant to show the celebration, and the
    /// popover could never display a comeback at all.
    #[test]
    fn closing_the_popover_resolves_it_early() {
        let mut m = with(vec![project(Some(T - 100 * 3600))], None);
        m.evaluate(T, T);
        m.state.projects[0].last_commit_at = Some(T);
        assert_eq!(m.evaluate(T, T), Mood::Comeback);

        // Open: the room is published again and is still the celebration.
        assert_eq!(m.evaluate(T + 1, T + 1), Mood::Comeback);

        // Closed.
        m.resolve_comeback();
        assert_eq!(m.evaluate(T + 2, T + 2), Mood::Rest(Rest::Awake));
    }

    #[test]
    fn a_comeback_survives_a_restart() {
        // The real sequence this protects: quit while asleep, commit, relaunch. A freshly
        // started app must not miss the transition by having no memory of yesterday.
        let mut m = with(vec![project(Some(T))], Some(Rest::Asleep));
        assert_eq!(m.evaluate(T, T), Mood::Comeback);
    }

    #[test]
    fn dozing_to_awake_is_not_a_comeback() {
        let mut m = with(vec![project(Some(T))], Some(Rest::Dozing));
        assert_eq!(m.evaluate(T, T), Mood::Rest(Rest::Awake));
    }

    #[test]
    fn a_fresh_install_does_not_celebrate() {
        let mut m = with(vec![], None);
        assert_eq!(m.evaluate(T, T), Mood::Rest(Rest::Awake));
    }

    #[test]
    fn only_the_resting_state_is_ever_persisted() {
        let mut m = with(vec![project(Some(T - 100 * 3600))], None);
        m.evaluate(T, T);
        m.state.projects[0].last_commit_at = Some(T);
        assert_eq!(m.evaluate(T, T), Mood::Comeback);
        // If this ever became `Comeback`, the type would not allow it, which is the point of
        // `Rest` being a separate type. This asserts the value as well as the type.
        assert_eq!(m.state.last_displayed_state, Some(Rest::Awake));
    }

    #[test]
    fn the_states_are_crossed_by_time_alone() {
        // No events at all, only `now` moving: the transition that is easiest to forget and
        // the more common one in practice (section 8.2).
        let mut m = with(vec![project(Some(T))], None);
        assert_eq!(m.evaluate(T + 3600, T + 3600), Mood::Rest(Rest::Awake));
        assert_eq!(m.evaluate(T + 25 * 3600, T + 25 * 3600), Mood::Rest(Rest::Dozing));
        assert_eq!(m.evaluate(T + 80 * 3600, T + 80 * 3600), Mood::Rest(Rest::Asleep));
    }

    #[test]
    fn characters_cycle_and_wrap() {
        let mut m = with(vec![], None);
        assert_eq!(m.state.character_id, "07");
        assert_eq!(m.cycle_character(), "12");
        assert_eq!(m.cycle_character(), "20");
        assert_eq!(m.cycle_character(), "07");
    }

    #[test]
    fn adding_a_project_keeps_the_bookmark_it_was_handed() {
        let t = std::env::temp_dir().join(format!("mascot-add-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&t);
        std::fs::create_dir_all(t.join(".git")).unwrap();
        std::fs::write(t.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();

        let mut m = with(vec![], None);
        assert_eq!(m.add(&t, T, Some("Ym9va21hcms=".into())), Ok(true));
        assert_eq!(
            m.state.projects[0].bookmark.as_deref(),
            Some("Ym9va21hcms=")
        );

        // A bookmark that could not be created is not an error: the project is added anyway and
        // degrades to today's behaviour, which is "works this launch, unavailable on the next".
        let t2 = t.join("nested");
        std::fs::create_dir_all(t2.join(".git")).unwrap();
        std::fs::write(t2.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
        assert_eq!(m.add(&t2, T, None), Ok(true));
        assert_eq!(m.state.projects[1].bookmark, None);

        let _ = std::fs::remove_dir_all(&t);
    }

    #[test]
    fn a_moved_folder_is_followed_and_its_display_name_refreshed() {
        // The rule this exists for: for a folder the user moved, the bookmark resolves to the
        // NEW location, and `repo::resolve` must then be given that path rather than the stale
        // stored one. Surviving a move is the entire point of a security-scoped bookmark, and an
        // earlier draft of this design resolved the bookmark and then used `p.path` anyway.
        //
        // Pure on purpose. The end-to-end path cannot be tested here at all: a
        // `WithSecurityScope` bookmark can neither be created nor resolved from a cargo test
        // binary, because the option needs the sandbox entitlements (see `scoped::create_with`).
        // So the policy is tested here and the mechanism is covered by the sandbox persistence
        // test after signing.
        let mut p = Project {
            path: PathBuf::from("/a/old-name"),
            name: "old-name".into(),
            ..project(None)
        };
        assert!(apply_resolved(&mut p, Path::new("/a/new-name")));
        assert_eq!(p.path, PathBuf::from("/a/new-name"));
        assert_eq!(p.name, "new-name", "the display name drifted with the path");
    }

    #[test]
    fn a_folder_that_did_not_move_is_left_exactly_alone() {
        let mut p = Project {
            path: PathBuf::from("/a/b"),
            name: "renamed by hand".into(),
            ..project(None)
        };
        assert!(!apply_resolved(&mut p, Path::new("/a/b")));
        assert_eq!(
            p.name, "renamed by hand",
            "an unchanged path must not clobber the stored name"
        );
    }

    #[test]
    fn a_project_with_no_bookmark_still_resolves_from_its_stored_path() {
        let t = std::env::temp_dir().join(format!("mascot-nobm-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&t);
        std::fs::create_dir_all(t.join(".git")).unwrap();
        std::fs::write(t.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();

        let mut m = with(
            vec![Project {
                id: "p1".into(),
                path: t.clone(),
                bookmark: None,
                ..project(None)
            }],
            None,
        );
        m.resolve_paths();
        assert!(m.git_dirs.contains_key("p1"));
        assert_eq!(m.state.projects[0].path, t);

        let _ = std::fs::remove_dir_all(&t);
    }

    #[test]
    fn an_unavailable_project_carries_the_reason_it_is_unavailable() {
        let t = std::env::temp_dir().join(format!("mascot-reason-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&t);
        let wt = t.join("checkout");
        std::fs::create_dir_all(&wt).unwrap();
        std::fs::write(wt.join(".git"), "gitdir: /nowhere/that/exists\n").unwrap();

        let mut m = with(
            vec![Project {
                id: "p1".into(),
                path: wt,
                bookmark: None,
                ..project(None)
            }],
            None,
        );
        m.resolve_paths();

        let snap = m.snapshot(T, T);
        assert!(!snap.projects[0].available);
        assert_eq!(
            snap.projects[0].reason,
            Some("That worktree's git folder isn't reachable from here.")
        );

        let _ = std::fs::remove_dir_all(&t);
    }

    #[test]
    fn an_available_project_carries_no_reason() {
        let t = std::env::temp_dir().join(format!("mascot-noreason-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&t);
        std::fs::create_dir_all(t.join(".git")).unwrap();
        std::fs::write(t.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();

        let mut m = with(
            vec![Project {
                id: "p1".into(),
                path: t.clone(),
                bookmark: None,
                ..project(None)
            }],
            None,
        );
        m.resolve_paths();
        let snap = m.snapshot(T, T);
        assert!(snap.projects[0].available);
        assert_eq!(snap.projects[0].reason, None);

        let _ = std::fs::remove_dir_all(&t);
    }
}
