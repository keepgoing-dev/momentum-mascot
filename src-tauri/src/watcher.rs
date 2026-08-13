//! Watching `.git/logs/HEAD`, debounced. Section 9.2.
//!
//! No hooks are installed in the user's repositories (section 9.3): watching the reflog gets
//! the same near-instant update with zero footprint inside them. Untracking is a line removed
//! from JSON, and uninstalling leaves nothing behind.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher as _};

/// A single git operation writes the reflog several times. Collapsing the burst means one
/// re-read per operation rather than one per write.
const DEBOUNCE: Duration = Duration::from_millis(250);

pub struct Watcher {
    inner: RecommendedWatcher,
    watched: HashSet<PathBuf>,
}

/// `KEEPGOING_WATCH_DEBUG` prints every filesystem event and every reading.
///
/// It earns its place: "the watcher is not firing" and "the reading is being discarded" look
/// identical from the outside, and telling them apart by reasoning cost more than this line
/// does. It was the thing that showed the watcher was working perfectly while the timestamps
/// were arriving on the wrong timeline.
pub fn debugging() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("KEEPGOING_WATCH_DEBUG").is_some())
}

impl Watcher {
    /// `on_change` runs on a background thread, already debounced, and says only "something
    /// happened". It deliberately carries no path: the reader re-reads every project anyway
    /// (see `Momentum::refresh_all`), so mapping an event back to a project would be extra
    /// code with no extra information in it.
    pub fn new(on_change: impl Fn() + Send + 'static) -> notify::Result<Self> {
        let (tx, rx) = mpsc::channel::<()>();

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
            // The watch is recursive over the whole git directory, because `logs/HEAD` does
            // not exist yet in a repository with no commits and a watch cannot be registered
            // on a path that is not there. Filtering here is what keeps the object churn of
            // a large fetch from waking anything up.
            if event.paths.iter().any(|p| p.ends_with("logs/HEAD")) {
                let _ = tx.send(());
            }
        })?;

        std::thread::spawn(move || {
            while rx.recv().is_ok() {
                // Drain the burst: keep resetting the window until the writes stop.
                while rx.recv_timeout(DEBOUNCE).is_ok() {}
                on_change();
            }
        });

        Ok(Watcher {
            inner,
            watched: HashSet::new(),
        })
    }

    /// Bring the watch set in line with the tracked projects, adding and removing only the
    /// difference. Called on startup and whenever a project is added or untracked.
    pub fn sync(&mut self, paths: &[PathBuf]) {
        let wanted: HashSet<PathBuf> = paths.iter().cloned().collect();

        for gone in self.watched.difference(&wanted).cloned().collect::<Vec<_>>() {
            let _ = self.inner.unwatch(&gone);
            self.watched.remove(&gone);
        }
        for added in wanted.difference(&self.watched).cloned().collect::<Vec<_>>() {
            if self.watch(&added) {
                self.watched.insert(added);
            }
        }
    }

    fn watch(&mut self, git_dir: &Path) -> bool {
        match self.inner.watch(git_dir, RecursiveMode::Recursive) {
            Ok(()) => true,
            Err(e) => {
                // A watch that cannot be registered is not fatal: the 60 second tick still
                // re-evaluates and startup re-reads every project, so the worst case is that
                // this repository updates within a minute instead of within a second.
                eprintln!("watch failed for {}: {e}", git_dir.display());
                false
            }
        }
    }
}
