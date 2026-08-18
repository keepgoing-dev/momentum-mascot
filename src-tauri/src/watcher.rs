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

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher as _};

/// A single git operation writes the reflog several times, and an editor save often writes a
/// file plus swap/backup siblings. Collapsing the burst means one reaction per operation.
const DEBOUNCE: Duration = Duration::from_millis(250);

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
    gitignore: HashMap<PathBuf, Option<ignore::gitignore::Gitignore>>,
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
                if let Some(evt) = classify(&guard, path) {
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

fn load_gitignore(work_tree: &Path) -> Option<ignore::gitignore::Gitignore> {
    let path = work_tree.join(".gitignore");
    let (gi, err) = ignore::gitignore::Gitignore::new(path);
    if let Some(e) = err {
        eprintln!("could not parse .gitignore: {e}");
    }
    Some(gi)
}

fn classify(state: &WatchState, path: &Path) -> Option<ChangeEvent> {
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
        if let Some(Some(gi)) = state.gitignore.get(work_tree) {
            if gi.matched(path, path.is_dir()).is_ignore() {
                return None;
            }
        }
        return Some(ChangeEvent::TreeChanged(project_id.clone()));
    }
    None
}
