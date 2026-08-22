//! What counts as a repository, and where its reflog actually lives. Sections 7 and 9.2.

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
pub enum RepoError {
    Missing,
    NotARepo,
    Unreadable,
    /// The `.git` file held a valid `gitdir:` pointer and the target is not reachable.
    ///
    /// A linked worktree or a submodule under App Sandbox: the pointer leads outside the folder
    /// the user picked, so it is outside the picker's grant and outside any bookmark, on the
    /// launch the picker ran as well as every later one. Bookmarks do not fix this, because the
    /// grant never covered that path.
    ///
    /// Distinct from `NotARepo`, which is what this path returned before, because the DMG channel
    /// handles worktrees fine and a user whose project reads as unavailable in one channel and
    /// not the other deserves to know which case they are in.
    GitDirOutside,
}

impl RepoError {
    /// The line the popover shows. Split out from `Display` so a `ProjectRow` can carry it
    /// without allocating a `String` per project per publish.
    pub fn message(&self) -> &'static str {
        // These strings appear inline in the popover, so they follow section 4.6's voice:
        // factual, short, and never implying the user did something stupid.
        match self {
            RepoError::Missing => "That folder isn't there any more.",
            RepoError::NotARepo => "That folder isn't a git repository.",
            RepoError::Unreadable => "That repository can't be read.",
            // Deliberately "isn't reachable from here" and not "is outside the folder you
            // picked": the same code path also fires for a worktree whose git folder was simply
            // deleted, where the other wording would be false.
            RepoError::GitDirOutside => "That worktree's git folder isn't reachable from here.",
        }
    }
}

impl fmt::Display for RepoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

/// Resolve a user-chosen folder to the git directory whose `logs/HEAD` we watch.
///
/// `.git` is a **directory** in an ordinary clone and a **file** holding a `gitdir:` pointer
/// in a linked worktree or a submodule. Both are accepted and both are resolved, because a
/// developer working in a worktree is exactly the kind of person this product is for.
///
/// A repository with zero commits is valid. Its reflog does not exist yet, so it contributes
/// nothing until it has a commit, which is a state the rest of the app already handles.
pub fn resolve(path: &Path) -> Result<PathBuf, RepoError> {
    if !path.is_dir() {
        return Err(RepoError::Missing);
    }
    let dot_git = path.join(".git");
    let git_dir = if dot_git.is_dir() {
        dot_git
    } else if dot_git.is_file() {
        let contents = std::fs::read_to_string(&dot_git).map_err(|_| RepoError::Unreadable)?;
        let pointer = contents
            .lines()
            .find_map(|l| l.strip_prefix("gitdir:"))
            .ok_or(RepoError::NotARepo)?
            .trim();
        let resolved = if Path::new(pointer).is_absolute() {
            PathBuf::from(pointer)
        } else {
            path.join(pointer)
        };
        if !resolved.is_dir() {
            return Err(RepoError::GitDirOutside);
        }
        resolved
    } else {
        return Err(RepoError::NotARepo);
    };

    if !git_dir.join("HEAD").is_file() {
        return Err(RepoError::Unreadable);
    }
    Ok(git_dir)
}

/// The reflog this project's momentum is read from.
pub fn reflog_path(git_dir: &Path) -> PathBuf {
    git_dir.join("logs").join("HEAD")
}

/// Step 3 of section 9.2: when the reflog is missing, empty, unparseable, or holds no
/// qualifying entry within the bound, fall back to the committer time of the commit `HEAD`
/// points at.
///
/// **This is the one place the app shells out**, and it is worth being explicit about why.
/// Section 9.2 prefers reading the reflog directly because it is a small read with no process
/// spawn, and that argument is about the *per-event* path, which runs on every commit. This
/// path runs when the cheap read has already failed, which in practice means a repository
/// written by a GUI client with non-standard reflog messages. Reaching the commit object
/// without `git` would mean inflating loose objects *and* parsing packfiles, which is a large
/// amount of code for a rare fallback. The degradation is graceful in both directions: if
/// `git` is not on `PATH` the worst case is a slightly stale timestamp, never a false
/// comeback.
pub fn head_commit_time(work_tree: &Path) -> Option<i64> {
    let out = Command::new("git")
        .arg("-C")
        .arg(work_tree)
        .args(["log", "-1", "--format=%ct", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8(out.stdout).ok()?.trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Temp(PathBuf);
    impl Temp {
        fn new(name: &str) -> Self {
            let p = std::env::temp_dir().join(format!("mascot-repo-{name}-{}", std::process::id()));
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

    #[test]
    fn a_plain_repository_resolves_to_its_dot_git() {
        let t = Temp::new("plain");
        std::fs::create_dir_all(t.path().join(".git")).unwrap();
        std::fs::write(t.path().join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
        assert_eq!(resolve(t.path()), Ok(t.path().join(".git")));
    }

    #[test]
    fn a_worktree_pointer_file_is_followed() {
        let t = Temp::new("worktree");
        let real = t.path().join("real-git-dir");
        std::fs::create_dir_all(&real).unwrap();
        std::fs::write(real.join("HEAD"), "ref: refs/heads/main\n").unwrap();
        let wt = t.path().join("checkout");
        std::fs::create_dir_all(&wt).unwrap();
        std::fs::write(wt.join(".git"), format!("gitdir: {}\n", real.display())).unwrap();
        assert_eq!(resolve(&wt), Ok(real));
    }

    #[test]
    fn a_relative_worktree_pointer_is_resolved_against_the_folder() {
        let t = Temp::new("relative");
        let wt = t.path().join("checkout");
        std::fs::create_dir_all(wt.join("elsewhere")).unwrap();
        std::fs::write(wt.join("elsewhere/HEAD"), "ref: refs/heads/main\n").unwrap();
        std::fs::write(wt.join(".git"), "gitdir: elsewhere\n").unwrap();
        assert_eq!(resolve(&wt), Ok(wt.join("elsewhere")));
    }

    #[test]
    fn a_repository_with_no_commits_is_accepted() {
        // `git init` and nothing else: HEAD exists, logs/HEAD does not.
        let t = Temp::new("empty");
        std::fs::create_dir_all(t.path().join(".git")).unwrap();
        std::fs::write(t.path().join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
        assert!(resolve(t.path()).is_ok());
        assert!(!reflog_path(&t.path().join(".git")).exists());
    }

    #[test]
    fn a_plain_folder_and_a_missing_path_are_told_apart() {
        let t = Temp::new("plain-folder");
        assert_eq!(resolve(t.path()), Err(RepoError::NotARepo));
        assert_eq!(resolve(&t.path().join("nope")), Err(RepoError::Missing));
    }

    #[test]
    fn a_dot_git_directory_without_a_head_is_unreadable() {
        let t = Temp::new("headless");
        std::fs::create_dir_all(t.path().join(".git")).unwrap();
        assert_eq!(resolve(t.path()), Err(RepoError::Unreadable));
    }

    #[test]
    fn a_worktree_pointing_outside_the_picked_folder_is_told_apart_from_a_plain_folder() {
        // Under sandbox this is the shape of every linked worktree and every submodule: the
        // `.git` file holds a perfectly good `gitdir:` pointer and the target is not reachable,
        // because the grant never covered it. That is a different thing from "this is not a
        // repository", and the popover says so.
        let t = Temp::new("outside");
        let wt = t.path().join("checkout");
        std::fs::create_dir_all(&wt).unwrap();
        std::fs::write(
            wt.join(".git"),
            "gitdir: /nowhere/that/exists/.git/worktrees/x\n",
        )
        .unwrap();
        assert_eq!(resolve(&wt), Err(RepoError::GitDirOutside));

        // A `.git` file with no pointer in it at all is still just not a repository.
        let bad = t.path().join("bad");
        std::fs::create_dir_all(&bad).unwrap();
        std::fs::write(bad.join(".git"), "this is not a pointer\n").unwrap();
        assert_eq!(resolve(&bad), Err(RepoError::NotARepo));
    }

    #[test]
    fn every_error_has_a_message_that_blames_nobody() {
        for e in [
            RepoError::Missing,
            RepoError::NotARepo,
            RepoError::Unreadable,
            RepoError::GitDirOutside,
        ] {
            let m = e.message();
            assert!(!m.is_empty());
            assert_eq!(m, e.to_string(), "Display and message disagree");
        }
    }
}
