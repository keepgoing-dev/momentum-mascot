//! Reading "when did I last do work here" out of `.git/logs/HEAD`. Section 9.
//!
//! The filter is the correctness requirement, not an optimisation. `git checkout` and
//! `git pull` move `HEAD` without the user writing anything, so treating all `HEAD` movement
//! as momentum would let **a comeback celebration fire because someone checked out a branch
//! after three weeks away**, hollowing out the single most important moment in the product.

/// How far back to scan. A qualifying commit is normally the last line or close to it; this
/// bound exists so that a repository whose recent history is nothing but checkouts costs a
/// bounded read rather than a full one.
pub const SCAN_LIMIT: usize = 200;

/// A reflog line is `<old> <new> <name> <email> <unix-ts> <tz>\t<message>`.
///
/// Returns the entry's timestamp and its message. The split is on the **first** tab, because
/// a commit message can itself contain tabs and the header never does.
fn parse_line(line: &str) -> Option<(i64, &str)> {
    let (header, message) = line.split_once('\t')?;
    let mut fields = header.split_whitespace().rev();
    let _tz = fields.next()?;
    let ts = fields.next()?.parse::<i64>().ok()?;
    Some((ts, message))
}

/// Does this reflog entry represent new work by the user?
///
/// Counted: `commit:`, `commit (initial):`, `commit (amend):`, `commit (merge):`.
///
/// Ignored: `checkout:`, `pull:`, `merge <branch>:` (a fast-forward moves HEAD without
/// producing anything new), `reset:`, `clone:`, and `rebase (pick):` / `rebase (finish):`,
/// which replay commits that already existed.
pub fn qualifies(message: &str) -> bool {
    message.starts_with("commit")
}

/// Scan backwards from the end until the first qualifying entry.
///
/// This replaces the naive "read the last line", which returns the wrong answer any time the
/// most recent operation was a checkout or a pull, which is most of the time.
///
/// The timestamp taken is the **reflog entry's**, not the commit's committer time. An amend
/// or a rebase rewrites committer time, while the reflog records when the user actually did
/// the thing. "When did I last do work here" is a question about the user, not about the
/// commit object.
pub fn last_qualifying(contents: &str) -> Option<i64> {
    contents
        .lines()
        .rev()
        .take(SCAN_LIMIT)
        .filter_map(parse_line)
        .find(|(_, message)| qualifies(message))
        .map(|(ts, _)| ts)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ZERO: &str = "0000000000000000000000000000000000000000";
    const SHA: &str = "1111111111111111111111111111111111111111";

    fn line(ts: i64, message: &str) -> String {
        format!("{ZERO} {SHA} Someone <someone@example.com> {ts} +0700\t{message}")
    }

    #[test]
    fn the_filter_matches_the_spec_table() {
        for counted in [
            "commit: add the thing",
            "commit (initial): first",
            "commit (amend): fix the message",
            "commit (merge): merge branch 'x'",
        ] {
            assert!(qualifies(counted), "should count: {counted}");
        }
        for ignored in [
            "checkout: moving from main to feature",
            "pull: Fast-forward",
            "merge feature: Fast-forward",
            "reset: moving to HEAD~1",
            "clone: from https://example.com/repo.git",
            "rebase (pick): rework the thing",
            "rebase (finish): returning to refs/heads/main",
        ] {
            assert!(!qualifies(ignored), "should be ignored: {ignored}");
        }
    }

    #[test]
    fn a_checkout_after_a_commit_does_not_hide_the_commit() {
        // The exact sequence that a "read the last line" implementation gets wrong, and the
        // one that would fire a false comeback.
        let log = [
            line(1_000, "commit: real work"),
            line(2_000, "checkout: moving from main to feature"),
            line(3_000, "pull: Fast-forward"),
        ]
        .join("\n");
        assert_eq!(last_qualifying(&log), Some(1_000));
    }

    #[test]
    fn a_message_containing_tabs_still_parses() {
        let log = line(4_242, "commit: fix\tthe\ttabs");
        assert_eq!(last_qualifying(&log), Some(4_242));
    }

    #[test]
    fn a_name_containing_spaces_still_parses() {
        // The timestamp is found from the right-hand end precisely so that a multi-word
        // name cannot shift the field positions.
        let log =
            format!("{ZERO} {SHA} Ada Lovelace van der Berg <ada@example.com> 5555 +0000\tcommit: x");
        assert_eq!(last_qualifying(&log), Some(5555));
    }

    #[test]
    fn a_qualifying_entry_many_lines_back_is_found() {
        let mut lines = vec![line(1_000, "commit: the work")];
        for i in 0..150 {
            lines.push(line(2_000 + i, "checkout: moving from a to b"));
        }
        assert_eq!(last_qualifying(&lines.join("\n")), Some(1_000));
    }

    #[test]
    fn nothing_qualifying_within_the_bound_falls_through() {
        let lines: Vec<String> = (0..SCAN_LIMIT + 50)
            .map(|i| line(2_000 + i as i64, "checkout: moving from a to b"))
            .collect();
        // A commit sits below the bound, so the scan must give up rather than find it, and
        // the caller falls back to HEAD.
        let mut log = vec![line(1_000, "commit: too far back")];
        log.extend(lines);
        assert_eq!(last_qualifying(&log.join("\n")), None);
    }

    #[test]
    fn junk_survives_contact() {
        for junk in ["", "\n\n", "not a reflog line at all", "\t\t\t", "a b c d e\tcommit: x"] {
            let _ = last_qualifying(junk);
        }
        // The last one has no parseable timestamp, so it must not be mistaken for one.
        assert_eq!(last_qualifying("a b c d e\tcommit: x"), None);
    }

    #[test]
    fn a_trailing_newline_does_not_hide_the_last_entry() {
        // git always writes one, so this is the normal case rather than an edge case.
        let log = format!("{}\n", line(9_000, "commit: the newest work"));
        assert_eq!(last_qualifying(&log), Some(9_000));
    }
}
