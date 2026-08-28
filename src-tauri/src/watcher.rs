//! Watching repositories two ways: reflog commits and working-tree file changes.
//!
//! No hooks are installed in the user's repositories (section 9.3): watching the reflog gets
//! near-instant commit updates with zero footprint inside them, and watching the working tree
//! catches edits before they are committed. Untracking is a line removed from JSON, and
//! uninstalling leaves nothing behind.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use notify::event::{CreateKind, RemoveKind};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as _};

/// A single git operation writes the reflog several times, and an editor save often writes a
/// file plus swap/backup siblings. Collapsing the burst means one reaction per operation.
const DEBOUNCE: Duration = Duration::from_millis(250);

/// Operating system metadata that nobody authored: these appear because a folder was looked at,
/// not because work happened in it.
const OS_METADATA: &[&str] = &[
    ".DS_Store",
    "._*",
    ".Spotlight-V100",
    ".Trashes",
    ".fseventsd",
    "Thumbs.db",
    "desktop.ini",
];

pub enum ChangeEvent {
    /// A relevant `.git/logs/HEAD` change was seen. Re-read every project's reflog.
    ReflogChanged,
    /// A non-ignored file inside a tracked working tree changed. Carries the project id.
    TreeChanged(String),
}

/// Watch state is shared between the `Watcher` struct and the `notify` callback so the callback
/// can classify events without re-deriving which project a path belongs to.
struct WatchState {
    git_dirs: HashSet<PathBuf>,
    work_trees: HashMap<PathBuf, String>,
    gitignore: HashMap<PathBuf, ignore::gitignore::Gitignore>,
}

pub struct Watcher {
    inner: RecommendedWatcher,
    state: Arc<Mutex<WatchState>>,
}

/// `KEEPGOING_WATCH_DEBUG` prints every filesystem event and every reading, **in debug builds
/// only**.
///
/// It earns its place: "the watcher is not firing" and "the reading is being discarded" look
/// identical from the outside, and telling them apart by reasoning cost more than this line
/// does. It was the thing that showed the watcher was working perfectly while the timestamps
/// were arriving on the wrong timeline.
///
/// The build gate was missing at first, and packaging is what found it: the variable's name was
/// still in the release binary while `KEEPGOING_CLOCK_SCALE` and `KEEPGOING_MASCOT_STATE` were
/// compiled out of it. What it prints is filesystem paths inside the user's own repositories, so
/// a shipped build that can be asked to write those into the system log is not what section 5.3
/// promises, however deliberately somebody has to ask.
pub fn debugging() -> bool {
    if !cfg!(debug_assertions) {
        return false;
    }
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("KEEPGOING_WATCH_DEBUG").is_some())
}

impl Watcher {
    /// `on_change` runs on a background thread, already debounced. The event tells the caller
    /// whether to re-read reflogs or to record activity for a single project.
    pub fn new(on_change: impl Fn(ChangeEvent) + Send + 'static) -> notify::Result<Self> {
        let (tx, rx) = mpsc::channel::<ChangeEvent>();
        let state = Arc::new(Mutex::new(WatchState {
            git_dirs: HashSet::new(),
            work_trees: HashMap::new(),
            gitignore: HashMap::new(),
        }));

        let state_for_callback = Arc::clone(&state);
        let inner = notify::recommended_watcher(move |res: notify::Result<Event>| {
            let event = match res {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("watch error: {e}");
                    return;
                }
            };
            if debugging() {
                eprintln!("event {:?} {:?}", event.kind, event.paths);
            }

            let guard = state_for_callback.lock().unwrap();
            let mut to_send: Option<ChangeEvent> = None;
            for path in &event.paths {
                if let Some(evt) = classify(&guard, &event.kind, path) {
                    // Prefer the more specific TreeChanged event; ReflogChanged is already a
                    // blanket refresh, so one is enough.
                    to_send = Some(evt);
                    break;
                }
            }
            drop(guard);

            if let Some(evt) = to_send {
                let _ = tx.send(evt);
            }
        })?;

        std::thread::spawn(move || {
            while let Ok(first) = rx.recv() {
                // Drain the burst: keep resetting the window until the writes stop.
                // If we saw any TreeChanged event, prefer that specificity; otherwise the
                // ReflogChanged blanket refresh is what we send.
                let mut event = first;
                while let Ok(next) = rx.recv_timeout(DEBOUNCE) {
                    if matches!(next, ChangeEvent::TreeChanged(_)) {
                        event = next;
                    }
                }
                on_change(event);
            }
        });

        Ok(Watcher { inner, state })
    }

    /// Bring the watch set in line with the tracked projects, adding and removing only the
    /// difference. Called on startup and whenever a project is added or untracked.
    pub fn sync(
        &mut self,
        git_dirs: &HashMap<String, PathBuf>,
        work_trees: &HashMap<String, PathBuf>,
    ) {
        let wanted_git: HashSet<PathBuf> = git_dirs.values().cloned().collect();
        let wanted_trees: HashMap<PathBuf, String> = work_trees
            .iter()
            .map(|(id, path)| (path.clone(), id.clone()))
            .collect();
        let wanted_tree_set: HashSet<PathBuf> = wanted_trees.keys().cloned().collect();

        // Determine the diff while holding the lock, then drop it before calling
        // `self.inner.watch`/`unwatch`, which need `&mut self`.
        let (remove_git, add_git, remove_trees, add_trees) = {
            let guard = self.state.lock().unwrap();
            let remove_git: Vec<PathBuf> =
                guard.git_dirs.difference(&wanted_git).cloned().collect();
            let add_git: Vec<PathBuf> = wanted_git.difference(&guard.git_dirs).cloned().collect();
            let current_trees: HashSet<PathBuf> = guard.work_trees.keys().cloned().collect();
            let remove_trees: Vec<PathBuf> =
                current_trees.difference(&wanted_tree_set).cloned().collect();
            let add_trees: Vec<PathBuf> =
                wanted_tree_set.difference(&current_trees).cloned().collect();
            (remove_git, add_git, remove_trees, add_trees)
        };

        // Perform all IO without holding the state lock.
        for path in &remove_git {
            let _ = self.inner.unwatch(path);
        }
        let added_git: Vec<PathBuf> = add_git
            .into_iter()
            .filter(|path| self.watch(path))
            .collect();

        for path in &remove_trees {
            let _ = self.inner.unwatch(path);
        }
        let added_trees: Vec<(PathBuf, String)> = add_trees
            .into_iter()
            .filter(|path| self.watch(path))
            .map(|path| {
                let id = wanted_trees.get(&path).cloned().unwrap_or_default();
                (path, id)
            })
            .collect();

        let mut guard = self.state.lock().unwrap();
        for path in remove_git {
            guard.git_dirs.remove(&path);
        }
        for path in added_git {
            guard.git_dirs.insert(path);
        }
        for path in remove_trees {
            guard.work_trees.remove(&path);
            guard.gitignore.remove(&path);
        }
        for (path, id) in added_trees {
            guard.work_trees.insert(path.clone(), id);
            guard.gitignore.insert(path.clone(), load_gitignore(&path));
        }
    }

    fn watch(&mut self, path: &Path) -> bool {
        match self.inner.watch(path, RecursiveMode::Recursive) {
            Ok(()) => true,
            Err(e) => {
                // A watch that cannot be registered is not fatal: the 60 second tick still
                // re-evaluates and startup re-reads every project, so the worst case is that
                // this repository updates within a minute instead of within a second.
                eprintln!("watch failed for {}: {e}", path.display());
                false
            }
        }
    }
}

/// The ignore matcher for one work tree: the project's own `.gitignore`, plus the operating
/// system's metadata files.
///
/// Section 9.4 step 2 names only the project's `.gitignore`, and on macOS that is not enough.
/// `.DS_Store` belongs in a *global* ignore file, and the shipped build cannot read one: `$HOME`
/// inside the App Sandbox container is not the home the user's git config lives in. Without
/// `OS_METADATA`, opening a dormant project in Finder wakes the mascot and spends the comeback
/// the product exists for.
fn load_gitignore(work_tree: &Path) -> ignore::gitignore::Gitignore {
    let mut builder = ignore::gitignore::GitignoreBuilder::new(work_tree);
    // First, so a project that genuinely tracks one of these can override it with a `!` line,
    // the same way a repository's own file beats a global one in git.
    for pattern in OS_METADATA {
        let _ = builder.add_line(None, pattern);
    }
    if let Some(e) = builder.add(work_tree.join(".gitignore")) {
        eprintln!("could not parse .gitignore: {e}");
    }
    builder.build().unwrap_or_else(|e| {
        eprintln!("could not build the ignore matcher: {e}");
        ignore::gitignore::Gitignore::empty()
    })
}

/// Section 9.4 step 3: only file changes count. `notify` labels the folder events it can, and
/// `is_dir` catches the rest; a folder that has already been removed cannot be told from a file
/// by the time we look at the path, so for those the label is the only evidence there is.
fn is_directory(kind: &EventKind, path: &Path) -> bool {
    matches!(
        kind,
        EventKind::Create(CreateKind::Folder) | EventKind::Remove(RemoveKind::Folder)
    ) || path.is_dir()
}

fn classify(state: &WatchState, kind: &EventKind, path: &Path) -> Option<ChangeEvent> {
    // Git directories first: a normal repo's work tree contains its `.git` folder, so
    // reflog events must win the classification.
    if state.git_dirs.iter().any(|git_dir| path.starts_with(git_dir)) && path.ends_with("logs/HEAD") {
        return Some(ChangeEvent::ReflogChanged);
    }

    for (work_tree, project_id) in &state.work_trees {
        if !path.starts_with(work_tree) {
            continue;
        }
        // `.git` inside the work tree is handled above via `git_dirs`.
        if path.starts_with(work_tree.join(".git")) {
            return None;
        }
        if is_directory(kind, path) {
            return None;
        }
        if let Some(gi) = state.gitignore.get(work_tree) {
            // `matched` alone tests the path and nothing above it, so `target/debug/app` read as
            // unignored while `target/` sat in the .gitignore right there. Almost every line in a
            // real ignore file names a directory, which made step 2's filter mostly decorative.
            if gi.matched_path_or_any_parents(path, false).is_ignore() {
                return None;
            }
        }
        return Some(ChangeEvent::TreeChanged(project_id.clone()));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Temp(PathBuf);
    impl Temp {
        fn new(name: &str) -> Self {
            let p = std::env::temp_dir().join(format!("mascot-watch-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&p);
            std::fs::create_dir_all(&p).unwrap();
            Temp(p)
        }
        fn path(&self) -> &Path {
            &self.0
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn state_for(work_tree: &Path) -> WatchState {
        let mut work_trees = HashMap::new();
        work_trees.insert(work_tree.to_path_buf(), "p1".to_string());
        let mut gitignore = HashMap::new();
        gitignore.insert(work_tree.to_path_buf(), load_gitignore(work_tree));
        WatchState {
            git_dirs: HashSet::new(),
            work_trees,
            gitignore,
        }
    }

    fn is_work(state: &WatchState, kind: EventKind, path: &Path) -> bool {
        matches!(
            classify(state, &kind, path),
            Some(ChangeEvent::TreeChanged(_))
        )
    }

    #[test]
    fn a_finder_metadata_file_is_not_work() {
        // A folder looked at in Finder is not a folder worked in. Before this filter a single
        // `.DS_Store` took a project from asleep to awake, which spends the comeback.
        let t = Temp::new("dsstore");
        let s = state_for(t.path());
        for name in [".DS_Store", "src/.DS_Store", "._notes.md"] {
            let f = t.path().join(name);
            std::fs::create_dir_all(f.parent().unwrap()).unwrap();
            std::fs::write(&f, "x").unwrap();
            assert!(
                !is_work(&s, EventKind::Create(CreateKind::File), &f),
                "{name}"
            );
        }
    }

    #[test]
    fn a_new_directory_is_not_work() {
        let t = Temp::new("mkdir");
        let s = state_for(t.path());
        let dir = t.path().join("a-new-folder");
        std::fs::create_dir(&dir).unwrap();
        assert!(!is_work(&s, EventKind::Create(CreateKind::Folder), &dir));
        assert!(!is_work(&s, EventKind::Any, &dir));
    }

    #[test]
    fn a_removed_directory_is_not_work_either() {
        // The path is gone, so `is_dir` says nothing. The event's own label is the only
        // evidence left that this was a folder.
        let t = Temp::new("rmdir");
        let s = state_for(t.path());
        let gone = t.path().join("was-a-folder");
        assert!(!is_work(&s, EventKind::Remove(RemoveKind::Folder), &gone));
    }

    #[test]
    fn an_ordinary_file_change_is_still_work() {
        let t = Temp::new("real");
        let s = state_for(t.path());
        std::fs::create_dir(t.path().join("src")).unwrap();
        let f = t.path().join("src/main.rs");
        std::fs::write(&f, "fn main() {}").unwrap();
        assert!(is_work(&s, EventKind::Create(CreateKind::File), &f));
    }

    #[test]
    fn the_projects_own_gitignore_has_the_last_word() {
        let t = Temp::new("ignore");
        std::fs::write(t.path().join(".gitignore"), "build/\n!.DS_Store\n").unwrap();
        let s = state_for(t.path());

        // A file *inside* an ignored directory, which is how ignore files are actually written.
        let built = t.path().join("build/nested/out.o");
        std::fs::create_dir_all(built.parent().unwrap()).unwrap();
        std::fs::write(&built, "x").unwrap();
        assert!(!is_work(&s, EventKind::Create(CreateKind::File), &built));

        // A project that deliberately tracks `.DS_Store` is believed, because `OS_METADATA` is
        // added before the project's own file rather than after it.
        let ds = t.path().join(".DS_Store");
        std::fs::write(&ds, "x").unwrap();
        assert!(is_work(&s, EventKind::Create(CreateKind::File), &ds));
    }

    #[test]
    fn a_reflog_write_still_wins_over_the_work_tree() {
        let t = Temp::new("reflog");
        let mut s = state_for(t.path());
        s.git_dirs.insert(t.path().join(".git"));
        let reflog = t.path().join(".git/logs/HEAD");
        assert!(matches!(
            classify(&s, &EventKind::Modify(notify::event::ModifyKind::Any), &reflog),
            Some(ChangeEvent::ReflogChanged)
        ));
    }
}
