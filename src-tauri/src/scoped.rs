//! Security-scoped bookmarks: how a sandboxed build keeps reading a folder the user picked on
//! an earlier launch.
//!
//! Under App Sandbox a folder chosen through the picker is granted for **that launch only**. The
//! app stores plain paths and re-resolves them at load, so without this module every tracked
//! project reports `RepoError::Missing` on the second launch and the mood is built from nothing.
//!
//! Three NSURL calls do the whole job, and the bookmark blob they hand back is opaque bytes, so
//! `state.json` carries it as base64. The base64 is hand-rolled for the same reason the RFC 3339
//! parsing in `store.rs` is: it is thirty lines against a dependency, in a project whose stated
//! failure mode is sprawl. It is also the only part of this module that can be tested honestly
//! without a sandbox, which is why it is tested hard.

use std::path::{Path, PathBuf};

// --------------------------------------------------------------------------------------
// base64, because a bookmark is bytes and state.json is text
// --------------------------------------------------------------------------------------

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);
        let n = ((chunk[0] as u32) << 16) | ((b1 as u32) << 8) | b2 as u32;
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// `None` for anything that is not base64. A hand-edited state file loses that project's
/// bookmark and falls back to the stored path, which is the same degradation as a project added
/// before bookmarks existed.
fn base64_decode(text: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    let mut acc: u32 = 0;
    let mut bits: u32 = 0;
    for c in text.bytes() {
        if matches!(c, b'=' | b'\n' | b'\r') {
            continue;
        }
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        } as u32;
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    Some(out)
}

// --------------------------------------------------------------------------------------
// The three NSURL calls
// --------------------------------------------------------------------------------------

/// A held security-scoped access grant. Access stops when this is dropped, so the guard has to
/// outlive every read: `momentum::read_commit_time` runs on every tick and every watcher event,
/// and `watcher.rs` registers watches later still. `Momentum` therefore holds these for the life
/// of the process.
pub struct ScopedAccess {
    /// `None` when `startAccessingSecurityScopedResource` said no, which is the unsandboxed
    /// channel's normal answer for a non-security-scoped URL. Nothing was started, so nothing is
    /// stopped, and an unbalanced `stop` never happens.
    #[cfg(target_os = "macos")]
    url: Option<objc2::rc::Retained<objc2_foundation::NSURL>>,
}

impl Drop for ScopedAccess {
    fn drop(&mut self) {
        #[cfg(target_os = "macos")]
        if let Some(url) = self.url.take() {
            unsafe { url.stopAccessingSecurityScopedResource() };
        }
    }
}

pub struct Resolved {
    /// Where the bookmark says the folder is **now**. Prefer this over the stored path: surviving
    /// a move is the entire point of a security-scoped bookmark.
    pub path: PathBuf,
    /// The folder moved since the bookmark was made. The caller re-creates the bookmark from the
    /// resolved URL while access is held, repairing the entry without another picker prompt.
    pub stale: bool,
    pub access: ScopedAccess,
}

#[cfg(target_os = "macos")]
pub fn create(path: &Path) -> Option<String> {
    use objc2_foundation::NSURLBookmarkCreationOptions;
    create_with(path, NSURLBookmarkCreationOptions::WithSecurityScope)
}

/// The options are a parameter for one reason: **`WithSecurityScope` cannot be exercised from a
/// `cargo test` binary at all.** It needs the sandbox entitlements, and without them
/// `bookmarkDataWithOptions:` fails with NSCocoaErrorDomain 256, "The file couldn't be opened."
/// So the tests drive this with empty options, which exercises every line of the FFI plumbing
/// (the selector names, the NSData bridge, the base64, the guard's Drop) and leaves only the
/// option flag itself uncovered. Without this split, `scoped.rs` would have no automated
/// coverage of its FFI at all, and a mistyped selector would surface only in a manual test.
#[cfg(target_os = "macos")]
fn create_with(
    path: &Path,
    options: objc2_foundation::NSURLBookmarkCreationOptions,
) -> Option<String> {
    use objc2_foundation::{NSString, NSURL};

    let url = NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()));
    match url.bookmarkDataWithOptions_includingResourceValuesForKeys_relativeToURL_error(
        options, None, None,
    ) {
        Ok(data) => Some(base64_encode(&data.to_vec())),
        Err(e) => {
            // Not fatal anywhere: the caller adds the project with `bookmark: None` and
            // degrades to working this launch and reporting unavailable on the next.
            eprintln!("could not create a bookmark for {}: {e}", path.display());
            None
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn create(_path: &Path) -> Option<String> {
    None
}

#[cfg(target_os = "macos")]
pub fn resolve(bookmark: &str) -> Option<Resolved> {
    use objc2_foundation::NSURLBookmarkResolutionOptions;
    resolve_with(bookmark, NSURLBookmarkResolutionOptions::WithSecurityScope)
}

/// Options as a parameter, for the reason given on `create_with`.
#[cfg(target_os = "macos")]
fn resolve_with(
    bookmark: &str,
    options: objc2_foundation::NSURLBookmarkResolutionOptions,
) -> Option<Resolved> {
    use objc2::runtime::Bool;
    use objc2_foundation::{NSData, NSURL};

    let bytes = base64_decode(bookmark)?;
    let data = NSData::with_bytes(&bytes);
    let mut stale = Bool::NO;
    let url = unsafe {
        NSURL::URLByResolvingBookmarkData_options_relativeToURL_bookmarkDataIsStale_error(
            &data,
            options,
            None,
            &mut stale,
        )
    }
    .ok()?;

    let path = PathBuf::from(url.path()?.to_string());

    // Apple documents `false` for a URL that is not security scoped, which is what the
    // unsandboxed DMG channel may legitimately get. Section 3 of the spec: a `false` means "use
    // the stored path directly", never "drop the project". A hard failure here would break the
    // DMG channel on some future macOS that follows its own documentation.
    let started = unsafe { url.startAccessingSecurityScopedResource() };

    Some(Resolved {
        path,
        stale: stale.as_bool(),
        access: ScopedAccess {
            url: started.then_some(url),
        },
    })
}

#[cfg(not(target_os = "macos"))]
pub fn resolve(_bookmark: &str) -> Option<Resolved> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_the_rfc_4648_test_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
        assert_eq!(base64_decode("Zm9vYmE=").as_deref(), Some(&b"fooba"[..]));
    }

    #[test]
    fn base64_round_trips_every_tail_length() {
        // A bookmark is a few hundred bytes of arbitrary binary, so the tail cases are the
        // ones that matter and every length modulo 3 is covered here.
        for len in 0..=64usize {
            let bytes: Vec<u8> = (0..len).map(|i| (i * 7 + 13) as u8).collect();
            let text = base64_encode(&bytes);
            assert_eq!(
                base64_decode(&text).as_deref(),
                Some(bytes.as_slice()),
                "round trip failed at length {len}"
            );
        }
    }

    #[test]
    fn base64_rejects_what_is_not_base64() {
        assert_eq!(base64_decode("not base64!"), None);
        assert_eq!(base64_decode("Zm9v\n"), Some(b"foo".to_vec()));
    }

    /// **A smoke test, and it must stay labelled one.**
    ///
    /// A cargo test binary is not an `.app`, is not sandboxed, and cannot be made so: a bare
    /// Mach-O signed with `app-sandbox` outside a bundle is killed with SIGTRAP, exit 133.
    /// Unsandboxed, creation and resolution succeed trivially and `startAccessing` returns
    /// whatever it returns. So a green result here proves exactly three things: it does not
    /// crash, it does not leak, and the guard drops. It proves **nothing** about persistence
    /// across a relaunch. That is the manual test in Task 13 of the plan, and this test is not
    /// evidence for it.
    #[cfg(target_os = "macos")]
    #[test]
    fn a_bookmark_round_trips_without_crashing_or_leaking() {
        let dir = std::env::temp_dir().join(format!("mascot-scoped-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // Empty options, NOT WithSecurityScope: see the comment on `create_with`. This is the
        // most the FFI can be driven to outside an entitled app bundle.
        let bookmark = create_with(&dir, objc2_foundation::NSURLBookmarkCreationOptions::empty())
            .expect("plain bookmark creation failed, so the FFI plumbing itself is wrong");
        assert!(!bookmark.is_empty());
        assert!(base64_decode(&bookmark).is_some(), "not storable as text");

        let resolved = resolve_with(
            &bookmark,
            objc2_foundation::NSURLBookmarkResolutionOptions::empty(),
        )
        .expect("resolution failed");
        assert_eq!(
            resolved.path.canonicalize().unwrap(),
            dir.canonicalize().unwrap()
        );
        assert!(!resolved.stale, "a folder that never moved read as stale");

        // Dropping the guard must be safe whether or not access was ever started.
        drop(resolved);

        assert!(resolve("not base64!").is_none(), "garbage must not resolve");
        assert!(resolve("").is_none(), "an empty bookmark must not resolve");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
