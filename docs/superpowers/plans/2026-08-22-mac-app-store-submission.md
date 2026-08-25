# Mac App Store submission Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Get Momentum Mascot onto the Mac App Store as a free listing, sandboxed, with no loss of product surface, while the notarized DMG channel keeps shipping from the same codebase.

**Architecture:** One universal binary, two signings. App Sandbox arrives through a second entitlements file applied at `codesign` time, so there is no cargo feature split and no conditional compilation for the sandbox itself. Persistent folder access comes from security-scoped bookmarks stored beside each project's path in `state.json`. A new `scoped.rs` wraps the three NSURL calls that make that work, and a new `appkit.rs` holds the public AppKit calls the app now has to make for itself. A sibling release script signs, packages and uploads the store build without touching the DMG path.

**Tech Stack:** Rust 2021, Tauri 2.11.5, objc2 0.6 / objc2-app-kit 0.3 / objc2-foundation 0.3 (already in the tree), plain POSIX `sh` for the release scripts, `codesign` / `productbuild` / `xcrun altool` from Xcode 26.6.

**Spec:** `docs/superpowers/specs/2026-08-22-mac-app-store-design.md`

## Scope: what this plan covers and what it does not

The spec's section 4 (the native AppKit pet) is **not in this plan**. It exists only if the
section 4.0 probe fails, and that probe is Task 3 here. When Task 3 reports a failure, write a
second plan for section 4 from the spec and execute it between this plan's Phase 3 and Phase 5.

Everything else in the spec is covered: sections 1, 2, 3, 5, 6, 7, 8, 9, 10, 11 and 12.

**The one hard gate.** Phase 6 cannot complete until `drawsBackground` and `fullScreenEnabled`
are out of the binary, which happens either in Task 3 (if the probe succeeds) or in the deferred
section 4 plan. Task 15 builds that gate into `release-mas.sh` so it cannot be forgotten, with a
rehearsal escape hatch that refuses to upload.

## Global Constraints

- **No em dashes anywhere.** House rule, applies to code, comments, docs and commit messages. Use a hyphen or rewrite. Existing files predate the rule; do not add new ones.
- **No new crate dependencies.** `objc2`, `objc2-app-kit`, `objc2-foundation` are already in `src-tauri/Cargo.toml` under `[target.'cfg(target_os = "macos")'.dependencies]`. Everything this plan needs is in their default features (verified: `NSWorkspace` and `NSScreen` are both in objc2-app-kit 0.3.2's default feature list).
- **Bundle identifier:** `dev.keepgoing.momentum-mascot`. **Team ID:** `3LM6674AC2`. **Current version:** `0.3.1`.
- **Minimum system version:** 10.15, from `src-tauri/tauri.conf.json`. Do not use API newer than that in shipped code. (`underPageBackgroundColor` is macOS 12 and appears only in throwaway probe code.)
- **Test command:** `cargo test --manifest-path src-tauri/Cargo.toml`. Every existing test must keep passing untouched.
- **Commit messages:** imperative, sentence case, no conventional-commit prefixes. Match `git log`: "Close the two open decisions", "Sign what the smoke test builds".
- **Never commit `src/assets/`, `docs/mockups/`, `src-tauri/icons/bundle/` or `tools/.release-env`.** All gitignored; they hold licensed LimeZu art or real secrets.
- **Site URL:** `https://keepgoing.dev`, served from `site/` as a Cloudflare Pages project named `keepgoing` (`.wrangler/cache/pages.json`).
- The DMG channel must not change behaviour. Every task that touches shared code says what the DMG channel sees.

## Execution log

Phases 1, 2 and 3 are **done**, on branch `mac-app-store`, 73 tests passing (56 at baseline).
Every measurement is recorded in `spikes/app-store/RESULTS.md`. What follows is what the plan
got wrong, so a reader trusts the document rather than re-deriving it.

**Probe outcomes (Phase 2).**

- Task 2: `com.apple.security.network.client` is **required**, measured six times,
  order-independent. Without it no webview finishes navigation and the failure is completely
  silent. The entitlements table in Task 2 below still shows four keys; the shipped
  `Entitlements.mas.plist` has five.
- Task 3: the section 4.0 probe **failed**. The pet is an opaque square without the private
  API, so spec section 4 (the native AppKit pet) is required and needs its own plan. Both
  removable private strings are confirmed gone with the feature off, and both unremovable ones
  confirmed still present.
- Task 4: rounding the **content view** works, radius 12.0, and the drop shadow follows the
  rounded shape, so the documented fallback and any `invalidateShadow` work are not needed.

**Corrections to the tasks as written.**

1. **Task 5 and Task 9 could not be written as specified.** `NSURLBookmarkCreationWithSecurityScope`
   fails with NSCocoaErrorDomain 256 from a `cargo test` binary, because the option needs the
   sandbox entitlements. It works in a signed app bundle. So `create`/`resolve` delegate to
   private `create_with(path, options)` / `resolve_with(bookmark, options)`, and the tests drive
   those with empty options, covering the whole FFI except the flag itself. Task 9's two
   end-to-end tests were replaced with a pure `apply_resolved(project, resolved) -> bool` seam
   and three tests. `resolve_paths` re-creates the bookmark on `stale || moved`, not just
   `stale`.
2. **Task 10's tooltip does not render.** A `title` attribute on the project name shows nothing
   in this webview, though one on a `<button>` works. Spec 7.2's argument is that the affected
   user must get an explanation, and a tooltip that never appears is no explanation. The reason
   is now a visible 10px line under the name, present only on unavailable rows
   (`.projects li { flex-wrap: wrap }` plus a `.reason` element at `flex: 0 0 100%`).
3. **Task 13 step 2 could not delete the container.** macOS protects the container root;
   `rm -rf ~/Library/Containers/<id>` fails on `.com.apple.containermanagerd.metadata.plist`.
   Delete `Data/.keepgoing` instead. Fixed in the step below.
4. **Task 12's URL check was useless as written.** `keepgoing.dev` returns **200 with the
   homepage for every unknown path**, so a status-code check cannot tell a present page from a
   missing one. Verify by content: `curl -sS https://keepgoing.dev/privacy | grep -q "<title>Privacy Policy"`.
   The page is in the repo but **not deployed yet**, which is a Phase 6 prerequisite.
5. **Task 13's assertion was strengthened.** "The project is still listed" proves nothing,
   because the list and its timestamps persist in `state.json` either way. The test used instead:
   quit, commit while closed, relaunch, and assert `last_commit_id` advanced. It passed
   (10:55:10Z to 16:11:54Z), as did a live commit through the watcher inside the sandbox.

**Still open.** What the unsandboxed DMG channel gets from `WithSecurityScope` creation is
unmeasured. It is not a correctness risk, since that channel does not need bookmarks, but spec
section 3's wording assumes it succeeds there.

### Phase 4 and Phase 5

Phase 4 (spec section 4, the native AppKit pet) got its own plan,
`docs/superpowers/plans/2026-08-22-native-pet.md`, and is **done and merged to `master`**. The
gate it existed to pass reports 0 on a signed sandboxed arm64 build.

Phase 5 is **done**. Task 14 and Task 15 in full, Task 16 in full including the Apple-account
work, and the whole pipeline rehearsed to `VERIFY SUCCEEDED` against App Store Connect. 80 tests
passing.

**Corrections to the tasks as written.**

6. **Task 14's line reference had drifted.** `category` is at `src-tauri/tauri.conf.json:43`, not
   `:51`. The value and the verification were right: `LSApplicationCategoryType` now reads
   `public.app-category.developer-tools`.
7. **Task 15's `lipo -archs` was a print, and is now an assertion.** This is the departure the
   native pet plan asked for. Printing it is what let every build during that plan come out
   x86_64 on an arm64 Mac without anyone noticing, so a wrong architecture now fails the build.
   Both slices are required, since the build asks for `--target universal-apple-darwin` and
   anything else means it did not do what was asked. Counter-tested against seven `lipo` outputs:
   `arm64e` does not satisfy `arm64` and `x86_64h` does not satisfy `x86_64`, which is what the
   space-padded `case` globs are for. The `PATH` line also now honours `$CARGO_HOME`, and the
   script prints which `cargo` it resolved, so a log says what built the binary.
8. **Task 15 step 4's expected output is unreachable in the state it is written for.** It lists a
   universal build, `lipo`, the private-API gate, stamping, signing and a `.pkg`, and then says
   the script correctly exits at the signing preflight when the certificates are missing. Both
   are true but not at once: the preflight runs first, on purpose, so none of that output can
   appear. What was actually verified, by handing the preflight placeholder identities:

   ```
   architectures: x86_64 arm64
   private API check: clean
   CFBundleShortVersionString: 0.3.1
   CFBundleVersion:            1
   ```

   then `no identity found` at `codesign`, as designed. The real preflight run burned no build
   number, which is the ordering working. `codesign`, `productbuild` and `altool --validate-app`
   are the only steps left unrehearsed.
9. **Task 16 step 6 no longer needs `MASCOT_MAS_ALLOW_PRIVATE_API=1`.** That flag exists to
   rehearse before the pet work lands, and the pet work has landed: the gate reports
   `private API check: clean` on its own. Setting the flag now would only refuse the upload.
10. **`private API check: clean` through this script closes the native pet plan's Task 6 step 3**,
    which was the one step deferred there because `release-mas.sh` did not exist yet.

### Task 16, sections 1 to 3

Done 25 August 2026. The App ID is registered and both certificates exist, so
`security find-identity -v` reports three and the signing preflight passes for the first time.
Xcode's Settings > Accounts > Manage Certificates issued both without a CSR or a portal visit,
which is not the flow Task 16's document described.

11. **Both certificates installed correctly and still did not appear.** `security find-identity
    -v` kept reporting one. The `-v` flag filters to *valid* identities, so `security
    find-identity` without it is the diagnostic, and it named the fault directly:
    `CSSMERR_TP_NOT_TRUSTED` on both new certificates. The cause was a missing intermediate. Both
    are issued by **WWDR G3** and this machine had only the **G1**, expired 7 February 2023. The
    Developer ID certificate chains through a different CA, which is exactly why it kept working
    and made the failure look like it was about the new certificates. Installing
    `AppleWWDRCAG3.cer` into the login keychain fixed it with no restart and nothing reissued.
    Written up as `docs/app-store.md` section 3a, because the misleading part is that the `-v`
    output is indistinguishable from the certificates not existing.
12. **The installer certificate's common name includes the name.** It reads `3rd Party Mac
    Developer Installer: Hoa Trinh (3LM6674AC2)`, not the team-ID-only form Task 16's document
    predicted. No script change was needed: the discovery `sed` matches the prefix and takes
    whatever follows, verified against the real `find-identity` output. The document and
    `tools/.release-env.example` are corrected.

Sections 4 to 6 (API key, app record) are still open, so Task 16 steps 4 and 5 stay unticked.

### Task 16, sections 4 to 6

13. **The document's own API key verification command is impossible.** Section 4 said to run
    `altool --list-providers --api-key ... --api-issuer ...`, and altool refuses it:
    `AuthenticationFailure("list-providers does not support APIKey authentication.")`. That
    command is username-and-password only. The replacement is `altool --generate-jwt`, which
    proves the key id, issuer id and `.p8` are mutually consistent without a network call, and
    whose flags are camelCase `--apiKey`/`--apiIssuer` while every other altool command takes
    `--api-key`/`--api-issuer`. Task 16 step 5 is corrected in place.
14. **Section 6 is a prerequisite for validation, not a later step.** With the certificates and
    the API key in place, `--validate-app` on the real package failed with `Cannot determine the
    Apple ID from Bundle ID 'dev.keepgoing.momentum-mascot' and platform 'MAC_OS'. (19)`. Nothing
    can be validated before the app record exists. The error is also the useful half: it is App
    Store Connect answering a query rather than rejecting a credential, so it doubles as the
    server-side proof that the API key works.
15. **`codesign` and `productbuild` both passed on the first attempt with real certificates**,
    which retires two of the three stages correction 8 listed as unrehearsed. Verified on the
    build rather than assumed: `Authority=Apple Distribution: Hoa Trinh (3LM6674AC2)` over WWDR
    over Apple Root CA, `flags=0x10000(runtime)`, `TeamIdentifier=3LM6674AC2`, the five expected
    entitlements sealed in, and the `.pkg` chained through
    `3rd Party Mac Developer Installer: Hoa Trinh (3LM6674AC2)`. Build number 3 is spent.
16. **The review notes in Task 17 contradict the bundle.** They say "The app makes no network
    requests of any kind" while the sandboxed build ships `com.apple.security.network.client`,
    which Probe 1 measured as mandatory: without it the webview never finishes navigation and the
    popover is blank, silently. Both statements are true, and the distinction (WebKit talking to
    its own networking process, not the app making requests) has to be stated in the review notes
    themselves rather than only in the entitlements file's comment. To be folded into Task 17.

17. **The rehearsal is complete: `VERIFY SUCCEEDED with no errors, 1 warning`.** The warning is
    `90889`, the missing provisioning profile, which is a TestFlight eligibility statement and not
    a store one. It is exactly what section 5 of `docs/app-store.md` says to expect, so it
    confirms the TN3125 reading rather than contradicting it. Validation was run against the
    package the script had already produced rather than through a fresh script run, so no
    additional build number was spent; the script's own path to this point is unchanged and was
    exercised in full to produce that package. Nothing in the pipeline is now unproven except the
    upload itself.
18. **The app record was created at version 1.0 and the build is 0.3.1.** App Store Connect
    matches a build to a version by `CFBundleShortVersionString`, so these have to agree before a
    build can be attached. Open decision, deliberately not resolved here: editing the record down
    to 0.3.1 keeps the single version line that `release-mas.sh` has no version bumping *in order
    to* preserve, whereas moving the project to 1.0 means `tools/release.sh 1.0`, which also tags,
    dates the changelog, notarizes a DMG and publishes a GitHub release.

**Also noticed, and since fixed.** The release build emitted six dead-code warnings from
`src-tauri/src/sprite.rs`: the debug probe's `Probe` struct, its ivar, its two constants and
`frame_at`. The `#[cfg(debug_assertions)]` restructure at the end of the pet work gated the
probe's *methods* and left the state they operate on compiled but unreachable. Now gated to
match, with `frame_at` at `#[cfg(any(test, debug_assertions))]` because the tests call it too.
Release, debug and test all compile without warnings, 80 tests pass, and the release binary still
contains zero `PROBE` strings and zero private-API strings.

---

## File Structure

New files:

| File | Responsibility |
|---|---|
| `src-tauri/Entitlements.mas.plist` | The store channel's entitlements. Never used by the DMG build. |
| `src-tauri/src/scoped.rs` | Security-scoped bookmarks: create, resolve, hold. Plus the base64 the bookmark blob is stored as. |
| `src-tauri/src/appkit.rs` | The public AppKit calls the app makes for itself: window transparency, corner rounding, opening a URL. |
| `tools/release-mas.sh` | Build, sign, package, validate and upload the store build. |
| `docs/app-store.md` | The one-time Apple setup, the sibling of `docs/notarization.md`. |
| `docs/app-store-listing.md` | The exact App Store Connect metadata, review notes and screenshot list. |
| `site/privacy.html` | The hosted privacy policy that guideline 5.1.1(i) requires. |
| `spikes/app-store/RESULTS.md` | What the three Phase 2 probes measured. Throwaway code, kept findings. |

Modified files:

| File | Change |
|---|---|
| `src-tauri/src/store.rs` | `Project.bookmark`, `SCHEMA_VERSION` to `3.1`, `default_path` split for testability. |
| `src-tauri/src/momentum.rs` | `resolve_paths` resolves bookmarks and holds guards; `add` carries a bookmark; `ProjectRow.reason`. |
| `src-tauri/src/repo.rs` | `RepoError::GitDirOutside` and a `&'static str` message accessor. |
| `src-tauri/src/commands.rs` | `add_project` creates a bookmark; new `open_privacy_policy`. |
| `src-tauri/src/app.rs` | `ProjectDto.reason`; `setup_popover`. |
| `src-tauri/src/pet.rs` | Explicit `setOpaque:` / `setBackgroundColor:` on the panel. |
| `src-tauri/src/main.rs` | Two new modules, one new command, the popover setup call. |
| `src-tauri/tauri.conf.json` | Popover loses `transparent: true`; category becomes `DeveloperTool`. |
| `src/index.html`, `src/popover.css`, `src/popover.js` | The privacy link, the opaque popover background, the failure reason in the project row's tooltip. |
| `site/index.html` | Links to the privacy policy. |
| `.gitignore` | `tools/.mas-build`, `tools/embedded.provisionprofile`. |
| `tools/.release-env.example` | App Store Connect API key variables. |
| `docs/spec-v2.md` | Section 10.3 and the risk table point here instead of recording permanent ineligibility. |

---

# Phase 1: the licence check

## Task 1: Record the LimeZu licence verdict

Blocking by construction, reading only, and it can invalidate everything after it. The spec puts
it first for that reason.

**Note:** the licence text has already been read once while this plan was written, and it
passes. This task records that verdict as a durable artifact rather than a memory, and confirms
the credit obligation is met in both places the store cares about. Do not skip it: the finding is
what a future reader needs, and step 1 re-reads the file rather than trusting this note.

**Files:**
- Create: `docs/app-store-licence-check.md`

- [x] **Step 1: Read the licence text**

Run:

```sh
cat "${MASCOT_PACK:-$HOME/Workspace/OneQode/projects/repos/oneqode-pixel-assets/moderninteriors-win}/LICENSE.txt"
```

Expected: the "MODERN INTERIORS FULL VERSION LICENSE" text, with a YOU CAN list, a YOU CAN'T
list, and a credits requirement.

- [x] **Step 2: Confirm the credit is present in both places the listing needs it**

Run:

```sh
grep -n "limezu" src/index.html src-tauri/tauri.conf.json
```

Expected: `src/index.html` carries `art: limezu.itch.io` in the credit paragraph, and
`tauri.conf.json`'s `copyright` field names LimeZu. Both are required, because the in-app credit
satisfies the licence and the bundle `copyright` carries into the App Store listing.

- [x] **Step 3: Write the verdict down**

Create `docs/app-store-licence-check.md`:

```markdown
# LimeZu Modern Interiors: App Store distribution check

**Date:** 2026-08-22
**Verdict:** clear. A free Mac App Store listing is permitted.

Checked against the licence text shipped with the full pack, at
`$MASCOT_PACK/LICENSE.txt`. Quoted in full because it is four lines and the whole
question turns on them:

> MODERN INTERIORS FULL VERSION LICENSE
> -
> YOU CAN:
> -Edit and use the asset in any commercial  or non commercial project
> -Use the asset in any commercial  or non commercial project
> -
> YOU CAN'T:
> - Resell or distribute the asset to others
> - Edit and resell the asset to others
> -
> - Credits required (limezu.itch.io)

Why this permits the store build:

- Shipping the art composited into an application is **use in a project**, which the
  licence permits with no channel restriction. It does not distinguish App Store from
  direct download, and it does not distinguish free from paid.
- The store build **redistributes no assets**. `src/assets/` is composited at build time
  from a local licensed copy and is gitignored. Nothing standalone leaves the machine.
- The credit is present in the app (`src/index.html`, the credit line under the buttons)
  and in the bundle's `copyright` field, which carries into the listing.

The one thing to keep true: if a future release ever exposes the raw sprite sheets as
files a user can extract or export, that becomes distribution and this verdict no longer
covers it.
```

- [x] **Step 4: Commit**

```sh
git add docs/app-store-licence-check.md
git commit -m "Record the LimeZu licence check for store distribution"
```

---

# Phase 2: throwaway probes, one afternoon

Three measurements, in this order, because the order is the value. The code written here is
thrown away even when it works; only `spikes/app-store/RESULTS.md` survives.

## Task 2: Entitlements.mas.plist, and whether the popover needs the network entitlement

The failure mode this probe exists to catch is a popover that hangs blank under sandbox with
nothing logged, discovered at the end of Phase 3. Ad-hoc signing is enough, so it needs nothing
from Phase 5.

**Files:**
- Create: `src-tauri/Entitlements.mas.plist`
- Create: `spikes/app-store/RESULTS.md`

**Interfaces:**
- Produces: `src-tauri/Entitlements.mas.plist`, referenced by Task 13's manual test and by `tools/release-mas.sh` in Task 15.

- [x] **Step 1: Write the store entitlements file**

Create `src-tauri/Entitlements.mas.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<!-- Mandatory for the Mac App Store. This is also the key that redirects $HOME into the
	     app's container, which is how the store build gets its own state.json with no code
	     change and no cfg. See spec section 5.3. -->
	<key>com.apple.security.app-sandbox</key>
	<true/>

	<!-- The folder picker is the only way a repository enters the app. Read-only is
	     sufficient: nothing is ever written inside a tracked repository. -->
	<key>com.apple.security.files.user-selected.read-only</key>
	<true/>

	<!-- Required for PERSISTENT access. Bookmark creation and resolution appear to work
	     without it, and the only case that proves otherwise is a relaunch, which is the
	     only case scoped.rs exists for. Apple: "If you want to provide your sandboxed app
	     with persistent access to file-system resources, you must enable security-scoped
	     bookmark and URL access." The live documentation for this key is a 10.7.3-era page
	     filed under Professional Video Applications, which is the weakest documentary
	     ground in this whole design. It costs a line and review does not object. -->
	<key>com.apple.security.files.bookmarks.app-scope</key>
	<true/>

	<!-- Carried over from the DMG channel's hardened runtime so the two channels' runtime
	     behaviour is identical. NOT required for the store, and NOT required by the
	     popover's JavaScript: WKWebView runs JS out of process in
	     com.apple.WebKit.WebContent.xpc with its own entitlements. Measured: a sandboxed
	     hardened-runtime bundle without this key could not mmap(MAP_JIT|PROT_EXEC) itself,
	     and its WKWebView ran a three-million-iteration JS loop regardless. -->
	<key>com.apple.security.cs.allow-jit</key>
	<true/>
</dict>
</plist>
```

- [x] **Step 2: Build a debug bundle to sign**

Run:

```sh
cd src-tauri && cargo tauri build --debug --bundles app && cd ..
ls -d "src-tauri/target/debug/bundle/macos/Momentum Mascot.app"
```

Expected: the path exists.

- [x] **Step 3: Ad-hoc sign it sandboxed, without the network entitlement, and open the popover**

Run:

```sh
APP="src-tauri/target/debug/bundle/macos/Momentum Mascot.app"
codesign --force --sign - --entitlements src-tauri/Entitlements.mas.plist "$APP"
codesign -d --entitlements - "$APP"
open "$APP"
```

Then click the tray icon.

Expected, and this is the whole measurement: either the room renders (the entitlement is not
needed) or the popover comes up blank and stays blank (it is). Note that a blank popover under
sandbox logs **no** violation, which is why this is an eye test and not a log grep. Watch
`log stream --predicate 'sender == "sandboxd"'` in a second terminal anyway, so a real violation
is not missed.

- [x] **Step 4: If it was blank, repeat with the entitlement and confirm the difference**

Add to `Entitlements.mas.plist`, temporarily:

```xml
	<key>com.apple.security.network.client</key>
	<true/>
```

Re-sign and re-open exactly as in step 3. If the popover now renders, the entitlement is
required: keep the key, and delete the spec's sentence claiming there is no network entitlement.
If it was already rendering in step 3, revert this step so the key does not ship.

- [x] **Step 5: Record the finding**

Create `spikes/app-store/RESULTS.md`:

```markdown
# App Store probes

Throwaway code, kept findings. Same rule as `spikes/always-on-top/RESULTS.md`: a future
macOS release that breaks one of these should be re-diagnosed in minutes.

## Probe 1: does the sandboxed popover need com.apple.security.network.client?

**Measured on:** <date>, macOS <version>
**Method:** debug bundle, `codesign --force --sign - --entitlements Entitlements.mas.plist`,
opened the popover with and without the key.

**Result:** <renders without it | hangs blank without it>

Why the question existed: a sandboxed WKWebView calling `loadHTMLString:` was observed
elsewhere to never finish navigation without this key, with no sandbox violation logged,
which is why Electron's MAS instructions mandate it. Tauri serves the popover through a
custom scheme handler rather than a URL load, so it may not apply, and now we know.
```

- [x] **Step 6: Commit**

```sh
git add src-tauri/Entitlements.mas.plist spikes/app-store/RESULTS.md
git commit -m "Add store entitlements and measure the sandboxed popover's network need"
```

## Task 3: The section 4.0 probe, which can delete the whole native pet rewrite

One hour against a multi-week rewrite. The highest-leverage hour in this plan.

The question: can the pet stay a webview and keep its alpha with `macos-private-api` gone, using
only public API? The honest expectation is that it fails, because wry's own comment at
`wkwebview/mod.rs:429-431` implies `underPageBackgroundColor` covers only the overscroll region
and not the page's own backdrop. Measure it anyway.

**Files:**
- Modify (throwaway, on a branch): `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/pet.rs`
- Modify: `spikes/app-store/RESULTS.md`

**Interfaces:**
- Produces: the verdict that decides whether a second plan for spec section 4 is needed. Nothing else.

- [x] **Step 1: Branch, so the throwaway stays throwaway**

```sh
git switch -c probe/pet-transparency
```

- [x] **Step 2: Turn the private API off**

In `src-tauri/Cargo.toml`, line 16, drop the feature:

```toml
tauri = { version = "2", features = ["tray-icon", "image-png"] }
```

In `src-tauri/tauri.conf.json`, line 10:

```json
    "macOSPrivateApi": false,
```

Note what this alone does, per spec 4.1: `tauri-runtime-wry-2.11.4/src/lib.rs:880-893` gates
`window.transparent(...)` behind the feature and, with it off, its only feedback is an
`eprintln!` gated on `debug_assertions`. **Both** windows go opaque, silently, in release. That
silence is the trap this probe is measuring around.

- [x] **Step 3: Make the calls the app now has to make itself**

In `src-tauri/src/pet.rs`, inside `setup`, replace the `#[cfg(target_os = "macos")]` block with:

```rust
    #[cfg(target_os = "macos")]
    {
        let ns = win.ns_window()? as *mut objc2::runtime::AnyObject;
        if !macos::make_panel(ns) {
            eprintln!("NSPanel class not found; the pet will not show over fullscreen apps");
        }
        macos::apply(ns, LEVEL, BEHAVIOR);

        // Probe only. Public AppKit: what tao would have done for us behind the private
        // feature (tao/window.rs:544-561), done by hand.
        unsafe {
            let _: () = objc2::msg_send![ns, setOpaque: objc2::runtime::Bool::NO];
            let clear = objc2_app_kit::NSColor::clearColor();
            let _: () = objc2::msg_send![ns, setBackgroundColor: &*clear];
        }

        // Probe only. Public since macOS 12, and wry already calls it at
        // wkwebview/mod.rs:441. The question is whether it reaches the page's own backdrop
        // or only the overscroll region.
        let _ = win.with_webview(|wv| {
            let webview = wv.inner() as *mut objc2::runtime::AnyObject;
            unsafe {
                let clear = objc2_app_kit::NSColor::clearColor();
                let _: () = objc2::msg_send![webview, setUnderPageBackgroundColor: &*clear];
            }
        });
    }
```

- [x] **Step 4: Run it and look at the pet**

```sh
cd src-tauri && cargo tauri dev
```

Expected, and there are only two outcomes worth recording:

- **The pet is a character with transparent surroundings.** Sections 4.1 through 4.6 of the spec are unnecessary, the deferred plan is never written, and the store costs nothing but the sandbox work.
- **The pet is a 64x64 opaque square, or a square with a tinted backdrop.** The native rewrite is required.

Check both a plain desktop and a mid-tone wallpaper: an opaque backdrop that happens to be
near-black is easy to mistake for transparency against a dark wallpaper.

- [x] **Step 5: Confirm the private strings actually left, either way**

```sh
cd src-tauri && cargo build --release && cd ..
strings -a src-tauri/target/release/momentum-mascot | grep -cE 'drawsBackground|fullScreenEnabled'
```

Expected: `0`.

For contrast, the same command on the currently shipped universal binary returns `2`, being one
line per architecture slice, with both keys inside the same string blob:

```sh
strings -a "src-tauri/target/universal-apple-darwin/release/bundle/macos/Momentum Mascot.app/Contents/MacOS/momentum-mascot" \
  | grep -cE 'drawsBackground|fullScreenEnabled'
```

Also confirm what does **not** leave, so nobody re-litigates it later:

```sh
strings -a src-tauri/target/release/momentum-mascot | grep -c 'allowsPictureInPictureMediaPlayback'
strings -a src-tauri/target/release/momentum-mascot | grep -c '_wantsKeyDownForEvent'
```

Expected: non-zero for both. Neither is reachable from this codebase and neither is behind a
feature gate. Removing them means forking wry and tao. Spec section 2.2.

- [x] **Step 6: Record the finding and throw the code away**

Append to `spikes/app-store/RESULTS.md`:

```markdown
## Probe 2: can the pet keep its alpha with the private API gone? (spec 4.0)

**Measured on:** <date>
**Method:** dropped `macos-private-api` from Cargo.toml and `macOSPrivateApi` from
tauri.conf.json, then set `setOpaque: NO` and `setBackgroundColor: clearColor` on the
panel by hand plus `underPageBackgroundColor = clearColor` on the WKWebView.

**Result:** <the pet kept its alpha | the pet was an opaque square>

**Consequence:** <spec sections 4.1-4.6 are dead; no native pet plan is needed>
                / <a second plan for spec section 4 is required before Phase 5>

Private strings after the feature was dropped:
- `drawsBackground` / `fullScreenEnabled`: 0 (was 2 on the shipped universal binary, one
  line per arch slice, both keys in the same blob)
- `allowsPictureInPictureMediaPlayback`: <n>, unremovable, wry sets it behind no gate
- `_wantsKeyDownForEvent`: <n>, unremovable, tao registers it unconditionally
```

```sh
git switch master
git branch -D probe/pet-transparency
```

Then re-apply only the RESULTS.md edit on master (copy it out before switching, or redo the few
lines) and commit:

```sh
git add spikes/app-store/RESULTS.md
git commit -m "Measure whether the pet can keep its alpha without the private API"
```

- [x] **Step 7: Report the verdict before continuing**

Stop and say which of the two outcomes happened. If the probe failed, a second plan for spec
section 4 has to be written and executed between Phase 3 and Phase 5 of this plan, and the
person running this plan needs to know that now rather than at Phase 5.

## Task 4: The popover corner-rounding probe

Task 11 implements the popover's native rounding. This probe answers, cheaply and before that
task is written, whether `masksToBounds` on the content view actually clips the WKWebView's
layer. If it does not, Task 11 takes its documented fallback instead.

**Files:**
- Modify (throwaway, on a branch): `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`, `src/popover.css`
- Modify: `spikes/app-store/RESULTS.md`

- [x] **Step 1: Branch**

```sh
git switch -c probe/popover-corners
```

- [x] **Step 2: Make the popover's webview opaque and its page opaque**

In `src-tauri/tauri.conf.json`, delete line 21 (`"transparent": true,`) from the **popover**
entry only. Leave the pet entry alone.

In `src/popover.css`, add at the top, under the opening comment:

```css
/* The window is opaque now (spec 5.1): the popover never needed a see-through webview, it
   needed rounded corners. style.css keeps `background: transparent` because the pet shares
   it, so the popover states its own ground here. */
html,
body {
  background: var(--panel);
}
```

- [x] **Step 3: Round the content view by hand**

In `src-tauri/src/main.rs`, inside `.setup(...)` after `pet::setup(&handle)?;`:

```rust
            // Probe only.
            #[cfg(target_os = "macos")]
            if let Some(popover) = handle.get_webview_window(app::POPOVER) {
                if let Ok(ns) = popover.ns_window() {
                    let ns = ns as *mut objc2::runtime::AnyObject;
                    unsafe {
                        let _: () = objc2::msg_send![ns, setOpaque: objc2::runtime::Bool::NO];
                        let clear = objc2_app_kit::NSColor::clearColor();
                        let _: () = objc2::msg_send![ns, setBackgroundColor: &*clear];

                        let view: *mut objc2::runtime::AnyObject =
                            objc2::msg_send![ns, contentView];
                        let _: () =
                            objc2::msg_send![view, setWantsLayer: objc2::runtime::Bool::YES];
                        let layer: *mut objc2::runtime::AnyObject =
                            objc2::msg_send![view, layer];
                        let _: () = objc2::msg_send![layer, setCornerRadius: 12.0f64];
                        let _: () =
                            objc2::msg_send![layer, setMasksToBounds: objc2::runtime::Bool::YES];
                    }
                }
            }
```

- [x] **Step 4: Look at the corners on two wallpapers**

```sh
cd src-tauri && cargo tauri dev
```

Open the popover over a light wallpaper and a dark one. Expected: the corners are rounded to
12pt and the wallpaper shows through outside the radius, matching what `.panel`'s
`border-radius: 12px` looks like today. Look specifically for a square opaque backdrop peeking
out behind the rounded corner, which is the failure this probe is for. Check the drop shadow
too: it should follow the rounded shape rather than a square.

- [x] **Step 5: If the corners are square, try the fallback**

Round the webview's own layer instead of the content view's. Replace the `view`/`layer` block
with:

```rust
                let _ = popover.with_webview(|wv| {
                    let webview = wv.inner() as *mut objc2::runtime::AnyObject;
                    unsafe {
                        let _: () =
                            objc2::msg_send![webview, setWantsLayer: objc2::runtime::Bool::YES];
                        let layer: *mut objc2::runtime::AnyObject =
                            objc2::msg_send![webview, layer];
                        let _: () = objc2::msg_send![layer, setCornerRadius: 12.0f64];
                        let _: () =
                            objc2::msg_send![layer, setMasksToBounds: objc2::runtime::Bool::YES];
                    }
                });
```

Record which of the two worked. Task 11 implements the one that did.

- [x] **Step 6: Record and discard**

Append to `spikes/app-store/RESULTS.md`:

```markdown
## Probe 3: does native corner rounding replace the transparent popover? (spec 5.1)

**Measured on:** <date>
**Method:** dropped `transparent: true` from the popover window, gave the page an opaque
background in popover.css, then set cornerRadius 12 + masksToBounds on <the content view
| the WKWebView's own layer>.

**Result:** <rounded correctly on light and dark wallpapers | content view masking did not
clip the webview, the webview's own layer did>

**Consequence for Task 11:** round <the content view | the webview's layer>.
Shadow followed the rounded shape: <yes | no>.
```

```sh
git switch master
git branch -D probe/popover-corners
```

Re-apply the RESULTS.md edit on master and commit:

```sh
git add spikes/app-store/RESULTS.md
git commit -m "Measure native corner rounding for an opaque popover"
```

---

# Phase 3: sandbox and bookmarks

This is the work the store actually requires and where the submission-relevant learning is. It
comes before the pet deliberately, and it is unaffected by how Task 3 turned out.

## Task 5: scoped.rs, the bookmark wrappers and the base64 they are stored as

**Files:**
- Create: `src-tauri/src/scoped.rs`
- Modify: `src-tauri/src/main.rs:14-25` (module list)
- Test: inline `mod tests` in `src-tauri/src/scoped.rs`

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `scoped::create(path: &Path) -> Option<String>`
  - `scoped::resolve(bookmark: &str) -> Option<scoped::Resolved>`
  - `struct scoped::Resolved { pub path: PathBuf, pub stale: bool, pub access: scoped::ScopedAccess }`
  - `struct scoped::ScopedAccess` with a `Drop` that stops access. `Send + Sync`, because `objc2` declares `unsafe impl Send for NSURL {}` and `Retained<T>` is `Send` when `T: Send + Sync`. That matters: `AppState` is handed to `tauri::Builder::manage`, which requires `Send + Sync`, and `Momentum` will hold these guards.

**Expected transient warning:** `create` and `resolve` are unused until Task 8. That is fine and
Task 8 removes it. Do not add `#[allow(dead_code)]` to paper over it.

- [x] **Step 1: Write the failing base64 tests**

Create `src-tauri/src/scoped.rs`:

```rust
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
}
```

- [x] **Step 2: Declare the module and run the tests to see them fail**

In `src-tauri/src/main.rs`, add `mod scoped;` to the module list in alphabetical position
(between `mod repo;` and `mod store;`).

Run: `cargo test --manifest-path src-tauri/Cargo.toml scoped::`
Expected: FAIL. The module does not compile yet if anything was mistyped; otherwise the three
tests run. If they all pass on the first try, that is legitimate here (base64 is written in full
above), so confirm by breaking one deliberately: change `b'+' => 62` to `b'+' => 61`, watch
`base64_round_trips_every_tail_length` fail, then change it back.

- [x] **Step 3: Run the tests to verify they pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml scoped::`
Expected: PASS, 3 tests.

- [x] **Step 4: Write the NSURL wrappers**

Append to `src-tauri/src/scoped.rs`:

```rust
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
    use objc2_foundation::{NSString, NSURL, NSURLBookmarkCreationOptions};

    let url = NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()));
    let data = url
        .bookmarkDataWithOptions_includingResourceValuesForKeys_relativeToURL_error(
            NSURLBookmarkCreationOptions::WithSecurityScope,
            None,
            None,
        )
        .ok()?;
    Some(base64_encode(&data.to_vec()))
}

#[cfg(not(target_os = "macos"))]
pub fn create(_path: &Path) -> Option<String> {
    None
}

#[cfg(target_os = "macos")]
pub fn resolve(bookmark: &str) -> Option<Resolved> {
    use objc2::runtime::Bool;
    use objc2_foundation::{NSData, NSURL, NSURLBookmarkResolutionOptions};

    let bytes = base64_decode(bookmark)?;
    let data = NSData::with_bytes(&bytes);
    let mut stale = Bool::NO;
    let url = unsafe {
        NSURL::URLByResolvingBookmarkData_options_relativeToURL_bookmarkDataIsStale_error(
            &data,
            NSURLBookmarkResolutionOptions::WithSecurityScope,
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
```

- [x] **Step 5: Write the round-trip smoke test, labelled as one**

Append inside `mod tests` in `src-tauri/src/scoped.rs`:

```rust
    /// **A smoke test, and it must stay labelled one.**
    ///
    /// A cargo test binary is not an `.app`, is not sandboxed, and cannot be made so: a bare
    /// Mach-O signed with `app-sandbox` outside a bundle is killed with SIGTRAP, exit 133.
    /// Unsandboxed, creation and resolution succeed trivially and `startAccessing` returns
    /// whatever it returns. So a green result here proves exactly three things: it does not
    /// crash, it does not leak, and the guard drops. It proves **nothing** about persistence
    /// across a relaunch. That is the manual test in Task 13, and this test is not evidence for
    /// it.
    #[cfg(target_os = "macos")]
    #[test]
    fn a_bookmark_round_trips_without_crashing_or_leaking() {
        let dir = std::env::temp_dir().join(format!("mascot-scoped-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let bookmark = create(&dir).expect("bookmark creation failed outside the sandbox too");
        assert!(!bookmark.is_empty());
        assert!(base64_decode(&bookmark).is_some(), "not storable as text");

        let resolved = resolve(&bookmark).expect("resolution failed");
        assert_eq!(
            resolved.path.canonicalize().unwrap(),
            dir.canonicalize().unwrap()
        );
        assert!(!resolved.stale, "a folder that never moved read as stale");

        // Dropping the guard must be safe whether or not access was ever started.
        drop(resolved);

        assert!(resolve("not base64!").is_none());
        assert!(resolve("").is_none());

        let _ = std::fs::remove_dir_all(&dir);
    }
```

- [x] **Step 6: Run the tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml scoped::`
Expected: PASS, 4 tests. If `create` returns `None`, the NSURL call failed for a real reason:
print the `NSError` before assuming the binding is wrong.

- [x] **Step 7: Run the whole suite**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: PASS, everything, with warnings that `create` and `resolve` are never used.

- [x] **Step 8: Commit**

```sh
git add src-tauri/src/scoped.rs src-tauri/src/main.rs
git commit -m "Add security-scoped bookmark wrappers"
```

## Task 6: state.json carries the bookmark

**Files:**
- Modify: `src-tauri/src/store.rs:21` (`SCHEMA_VERSION`), `:24-39` (`Project`), `:137-160` (`project_from_value`), `:198-217` (`to_json`), and the test module's `Project` literals at `:380-388`, `:403-410`, `:422-429`
- Modify: `src-tauri/src/momentum.rs:231-239` (`add`'s `Project` literal), `:305-315` (the test helper)
- Test: inline `mod tests` in `src-tauri/src/store.rs`

**Interfaces:**
- Consumes: nothing from Task 5.
- Produces: `store::Project.bookmark: Option<String>`, read and written by `store::from_json` / `store::to_json`. `store::SCHEMA_VERSION == "3.1"`.

- [x] **Step 1: Write the failing tests**

Append inside `mod tests` in `src-tauri/src/store.rs`:

```rust
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
    fn writers_declare_schema_3_1() {
        // A file written with bookmarks is meaningfully different from one without, and the
        // reader has to keep accepting both.
        let text = serde_json::to_string(&to_json(&StateFile::default())).unwrap();
        assert!(text.contains(r#""version":"3.1""#), "got: {text}");
    }
```

- [x] **Step 2: Run them to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml store::`
Expected: FAIL to compile, `struct Project has no field named bookmark`.

- [x] **Step 3: Add the field**

In `src-tauri/src/store.rs`, change line 21:

```rust
pub const SCHEMA_VERSION: &str = "3.1";
```

Add to `Project`, after `operating`:

```rust
    /// Base64 of an NSURL security-scoped bookmark for `path`, or `None` for a project added
    /// before bookmarks existed or on a launch where creating one failed. Only the sandboxed
    /// store build needs it; the DMG build creates and resolves it too, because one code path
    /// with no `cfg` is worth more than the bytes it costs. See `scoped.rs`.
    pub bookmark: Option<String>,
```

In `project_from_value`, add to the returned `Project`:

```rust
        bookmark: v
            .get("bookmark")
            .and_then(Value::as_str)
            .map(str::to_string),
```

In `to_json`'s per-project `json!`, add:

```rust
                        "bookmark": p.bookmark,
```

- [x] **Step 4: Fix the existing literals**

Add `bookmark: None,` to every `Project { ... }` literal that now fails to compile:
`src-tauri/src/store.rs` at roughly `:380`, `:403`, `:422`, and `src-tauri/src/momentum.rs` at
roughly `:231` (inside `add`) and `:305` (the `project` test helper).

Run: `cargo build --manifest-path src-tauri/Cargo.toml`
Expected: compiles.

- [x] **Step 5: Run the tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: PASS, everything, including the four new tests and the untouched resilience tests.

- [x] **Step 6: Commit**

```sh
git add src-tauri/src/store.rs src-tauri/src/momentum.rs
git commit -m "Carry a security-scoped bookmark per project in state.json"
```

## Task 7: Make the state path testable, and prove it needs no sandbox branch

Spec 5.3 measured that `$HOME` is redirected for `getenv` as well as `NSHomeDirectory()`, so
`store::default_path()` follows the entitlement with no code change and no `cfg`. That claim
deserves a test, and it cannot be tested by mutating `HOME` in a parallel test binary. Split the
pure part out, which is the same move the module already made for `from_json`.

**Files:**
- Modify: `src-tauri/src/store.rs:72-92` (`default_path`)
- Test: inline `mod tests` in `src-tauri/src/store.rs`

**Interfaces:**
- Produces: `store::path_in_home(home: &Path) -> PathBuf`. Used only by `default_path` and its test.

- [x] **Step 1: Write the failing test**

Append inside `mod tests` in `src-tauri/src/store.rs`:

```rust
    #[test]
    fn the_state_path_is_home_relative_and_nothing_else() {
        // The whole sandbox story for state.json is that this function does not change. In the
        // DMG build $HOME is /Users/<someone>; in the store build the sandbox redirects it to
        // ~/Library/Containers/dev.keepgoing.momentum-mascot/Data, for the raw environment
        // variable and not only for NSHomeDirectory(). Measured, spec section 5.3. So there is
        // no branch to test, only the shape of the path.
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
```

- [x] **Step 2: Run it to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml the_state_path_is_home_relative`
Expected: FAIL to compile, `cannot find function path_in_home`.

- [x] **Step 3: Split the function**

In `src-tauri/src/store.rs`, replace the tail of `default_path` (the `HOME` lookup and the
`join` chain) with:

```rust
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
```

Also extend `default_path`'s doc comment with:

```rust
/// **Under App Sandbox this moves and the code does not.** The sandbox redirects `$HOME` itself,
/// so the store build resolves to
/// `~/Library/Containers/dev.keepgoing.momentum-mascot/Data/.keepgoing/mascot/state.json` and the
/// DMG build stays where it always was. There is deliberately no migration between them: a
/// sandboxed process can discover the real home through `getpwuid` but cannot read it, so a
/// migration could not be written even if one were wanted. This is why the DMG channel is not
/// sandboxed (spec section 3): the channel with existing users keeps its file.
```

- [x] **Step 4: Run the tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: PASS, everything.

- [x] **Step 5: Commit**

```sh
git add src-tauri/src/store.rs
git commit -m "Split the state path so the sandbox claim is testable"
```

## Task 8: Adding a project creates its bookmark

**Files:**
- Modify: `src-tauri/src/momentum.rs:226-246` (`add`)
- Modify: `src-tauri/src/commands.rs:69-108` (`add_project`)
- Test: inline `mod tests` in `src-tauri/src/momentum.rs`

**Interfaces:**
- Consumes: `scoped::create` from Task 5, `store::Project.bookmark` from Task 6.
- Produces: `Momentum::add(&mut self, path: &Path, now: i64, bookmark: Option<String>) -> Result<bool, RepoError>`. The third parameter is new and every caller has to pass it.

- [x] **Step 1: Write the failing test**

Append inside `mod tests` in `src-tauri/src/momentum.rs`:

```rust
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
```

- [x] **Step 2: Run it to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml adding_a_project_keeps_the_bookmark`
Expected: FAIL to compile, `this method takes 2 arguments but 3 arguments were supplied`.

- [x] **Step 3: Widen `add`**

In `src-tauri/src/momentum.rs`, change the signature and the literal:

```rust
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
```

- [x] **Step 4: Create the bookmark at the picker**

In `src-tauri/src/commands.rs`, add `use crate::scoped;` to the imports, and replace the add call
in `add_project`:

```rust
    // Created here and nowhere else: the bookmark has to be made while the picker's grant is
    // live, and this is the only moment the app knows it is. A failure costs the bookmark, not
    // the project.
    let bookmark = scoped::create(&path);

    let state = app.state::<AppState>();
    let now = state.clock.now();
    state
        .momentum
        .lock()
        .unwrap()
        .add(&path, now, bookmark)
        .map_err(|e| e.to_string())?;
```

- [x] **Step 5: Run the tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: PASS, everything. The dead-code warnings for `scoped::create` are gone; `scoped::resolve` still warns until Task 9.

- [x] **Step 6: Commit**

```sh
git add src-tauri/src/momentum.rs src-tauri/src/commands.rs
git commit -m "Create a bookmark when a project is picked"
```

## Task 9: resolve_paths resolves the bookmark, prefers what it says, and holds the grant

The task the whole sandbox effort turns on. Three things have to be true at once: access is
started before `repo::resolve` runs, the **resolved** path is used rather than the stored one,
and the guards live for the whole process.

**Files:**
- Modify: `src-tauri/src/momentum.rs:17-39` (struct fields), `:59-83` (`load` and `resolve_paths`)
- Test: inline `mod tests` in `src-tauri/src/momentum.rs`, plus the `with` helper at `:317-330`

**Interfaces:**
- Consumes: `scoped::resolve`, `scoped::create`, `scoped::ScopedAccess` from Task 5.
- Produces: `Momentum.access: HashMap<String, ScopedAccess>` (private). No public signature change.

- [x] **Step 1: Write the failing test**

Append inside `mod tests` in `src-tauri/src/momentum.rs`:

```rust
    /// The rule this test exists for: for a folder the user moved, the bookmark resolves
    /// correctly, access is granted at the new location, and then `repo::resolve` fails on the
    /// stale stored path unless `resolve_paths` prefers the resolved one. Surviving a move is
    /// the entire point of a security-scoped bookmark.
    #[cfg(target_os = "macos")]
    #[test]
    fn a_moved_folder_is_followed_and_written_back() {
        let root = std::env::temp_dir().join(format!("mascot-moved-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let before = root.join("old-name");
        let after = root.join("new-name");
        std::fs::create_dir_all(before.join(".git")).unwrap();
        std::fs::write(before.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();

        let bookmark = crate::scoped::create(&before).expect("bookmark creation failed");
        std::fs::rename(&before, &after).unwrap();

        let mut m = with(
            vec![Project {
                id: "p1".into(),
                path: before.clone(),
                name: "old-name".into(),
                bookmark: Some(bookmark),
                ..project(None)
            }],
            None,
        );
        m.resolve_paths();

        assert_eq!(m.state.projects[0].path, after, "the move was not followed");
        assert_eq!(
            m.state.projects[0].name, "new-name",
            "the display name drifted with the path and was not refreshed"
        );
        assert!(
            m.git_dirs.contains_key("p1"),
            "the git dir was not resolved at the new location"
        );

        let _ = std::fs::remove_dir_all(&root);
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
```

The `with` helper needs the new field. In `src-tauri/src/momentum.rs`, add to the `Momentum`
literal inside `with`:

```rust
            access: HashMap::new(),
```

- [x] **Step 2: Run them to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml momentum::`
Expected: FAIL to compile, `struct Momentum has no field named access`. Once that is added,
`a_moved_folder_is_followed_and_written_back` fails on the first assertion, because the stored
path is still `old-name`.

- [x] **Step 3: Add the field**

In `src-tauri/src/momentum.rs`, add to `struct Momentum` after `work_trees`:

```rust
    /// Held security-scoped access, per project id.
    ///
    /// **These live for the whole process and that is not an accident.** `read_commit_time` runs
    /// on every tick (`app.rs:274`) and every watcher event (`watcher.rs:112`), and
    /// `watcher.rs:187` registers watches later still, so a guard scoped to load would revoke
    /// access under a live `refresh_all`. Anyone adding a fourth map here: `resolve_paths` opens
    /// by clearing `git_dirs` and `work_trees`, and this map is deliberately **replaced** at the
    /// end instead, so the old grants drop only once the new ones are held.
    access: HashMap<String, ScopedAccess>,
```

Add the import: `use crate::scoped::{self, ScopedAccess};`

Add `access: HashMap::new(),` to the `Momentum` literal in `load`.

- [x] **Step 4: Rewrite resolve_paths**

Replace `resolve_paths` in `src-tauri/src/momentum.rs`:

```rust
    fn resolve_paths(&mut self) {
        self.git_dirs.clear();
        self.work_trees.clear();

        // Built locally and assigned at the end rather than cleared first. See the comment on
        // `access`.
        let mut access: HashMap<String, ScopedAccess> = HashMap::new();

        // By index, because the bookmark's answer is written back into the project it came from.
        for i in 0..self.state.projects.len() {
            let id = self.state.projects[i].id.clone();

            if let Some(resolved) = self.state.projects[i]
                .bookmark
                .as_deref()
                .and_then(scoped::resolve)
            {
                // The resolved path wins over the stored one. The display name drifts for the
                // same reason, so it is refreshed alongside. This persists for free: `load` runs
                // before the first publish, and publish saves (`app.rs:107`).
                if resolved.path != self.state.projects[i].path {
                    self.state.projects[i].name = store::display_name(&resolved.path);
                    self.state.projects[i].path = resolved.path.clone();
                }

                // Re-created while access is held, so a moved folder repairs its own entry
                // without asking the user for it again.
                if resolved.stale {
                    if let Some(fresh) = scoped::create(&resolved.path) {
                        self.state.projects[i].bookmark = Some(fresh);
                    }
                }

                access.insert(id.clone(), resolved.access);
            }

            let path = self.state.projects[i].path.clone();
            if let Ok(git_dir) = repo::resolve(&path) {
                self.git_dirs.insert(id.clone(), git_dir);
            }
            self.work_trees.insert(id, path);
        }

        self.access = access;
    }
```

- [x] **Step 5: Run the tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: PASS, everything. All `scoped` dead-code warnings are gone.

- [x] **Step 6: Commit**

```sh
git add src-tauri/src/momentum.rs
git commit -m "Resolve bookmarks at load and follow a folder that moved"
```

## Task 10: Say why a worktree is unavailable instead of showing the generic line

Spec 7.2, decided: the degradation is accepted, and the affected user gets an explanation rather
than silence. Under sandbox a linked worktree's or submodule's git dir lies outside the folder
the user picked, so it is outside the grant and outside any bookmark, and bookmarks do not fix
it. `repo.rs:30-31` says "a developer working in a worktree is exactly the kind of person this
product is for", which is why this cannot be silent.

Four files, and the spec says plainly that it is not one line.

**Files:**
- Modify: `src-tauri/src/repo.rs:7-25` (the enum and its `Display`), `:40-60` (`resolve`)
- Modify: `src-tauri/src/momentum.rs` (`reasons` map, `resolve_paths`, `ProjectRow`, `snapshot`)
- Modify: `src-tauri/src/app.rs:49-56` (`ProjectDto`), `:91-101` (the mapping)
- Modify: `src/popover.js:100`
- Test: inline `mod tests` in `src-tauri/src/repo.rs` and `src-tauri/src/momentum.rs`

**Interfaces:**
- Produces:
  - `RepoError::GitDirOutside`
  - `RepoError::message(&self) -> &'static str`, which `Display` now delegates to
  - `momentum::ProjectRow.reason: Option<&'static str>`
  - `app::ProjectDto.reason: Option<&'static str>`, serialized to the popover as `reason`

- [x] **Step 1: Write the failing repo test**

Append inside `mod tests` in `src-tauri/src/repo.rs`:

```rust
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
```

- [x] **Step 2: Run it to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml repo::`
Expected: FAIL to compile, `no variant named GitDirOutside`.

- [x] **Step 3: Add the variant and the message accessor**

In `src-tauri/src/repo.rs`:

```rust
#[derive(Debug, PartialEq, Eq)]
pub enum RepoError {
    Missing,
    NotARepo,
    Unreadable,
    /// The `.git` file held a valid `gitdir:` pointer and the target is not reachable. A linked
    /// worktree or a submodule, under App Sandbox: the pointer leads outside the folder the user
    /// picked, so it is outside the picker's grant and outside any bookmark, on the launch the
    /// picker ran as well as every later one. Distinct from `NotARepo`, which is what this path
    /// returned before, because the DMG channel handles worktrees fine and a user whose project
    /// reads as unavailable in one channel and not the other deserves to know which case they
    /// are in. Spec section 7.2.
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
            RepoError::GitDirOutside => "That worktree's git folder isn't reachable from here.",
        }
    }
}

impl fmt::Display for RepoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}
```

The `GitDirOutside` wording deliberately differs from the spec's suggested "outside the folder
you picked": the same code path also covers a genuinely deleted worktree git dir, and "isn't
reachable from here" is true in both cases while the other is not.

In `resolve`, change the pointer-target check at what is currently line 54:

```rust
        if !resolved.is_dir() {
            return Err(RepoError::GitDirOutside);
        }
```

- [x] **Step 4: Run the repo tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml repo::`
Expected: PASS, including the untouched `a_worktree_pointer_file_is_followed` and
`a_relative_worktree_pointer_is_resolved_against_the_folder`.

- [x] **Step 5: Write the failing momentum test**

Append inside `mod tests` in `src-tauri/src/momentum.rs`:

```rust
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
```

- [x] **Step 6: Run them to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml momentum::`
Expected: FAIL to compile, `struct ProjectRow has no field named reason`.

- [x] **Step 7: Plumb the reason**

In `src-tauri/src/momentum.rs`, add to `struct Momentum`:

```rust
    /// Why a project failed to resolve, per project id. `resolve_paths` discarded this before,
    /// and `available` was derived from `git_dirs` alone, so the popover could only ever say
    /// "not reachable right now" for four different causes.
    reasons: HashMap<String, &'static str>,
```

Add `reasons: HashMap::new(),` to the `Momentum` literal in `load` and in the `with` test helper.

In `resolve_paths`, clear it alongside the others and record the failure:

```rust
        self.git_dirs.clear();
        self.work_trees.clear();
        self.reasons.clear();
```

```rust
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
```

Add to `struct ProjectRow`:

```rust
    /// Why it is unavailable, when it is. `None` when it is available, and also `None` for a
    /// project that resolved fine, so the popover falls back to its own generic line only when
    /// there really is nothing more specific to say.
    pub reason: Option<&'static str>,
```

In `snapshot`'s `ProjectRow` literal:

```rust
                    reason: self.reasons.get(&p.id).copied(),
```

- [x] **Step 8: Carry it to the frontend**

In `src-tauri/src/app.rs`, add to `struct ProjectDto`:

```rust
    pub reason: Option<&'static str>,
```

and to the mapping inside `publish`:

```rust
                    reason: p.reason,
```

In `src/popover.js`, replace line 100:

```js
  name.title = project.available
    ? project.name
    : `${project.name} (${project.reason ?? "not reachable right now"})`;
```

The CSS class at `:94` and the `"unavailable"` text at `:113` stay exactly as they are: the row
still reads as away, and only the tooltip gets more specific.

- [x] **Step 9: Run the tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: PASS, everything.

- [x] **Step 10: Commit**

```sh
git add src-tauri/src/repo.rs src-tauri/src/momentum.rs src-tauri/src/app.rs src/popover.js
git commit -m "Say why a worktree reads as unavailable"
```

## Task 11: appkit.rs, and the popover stops needing a transparent webview

Per spec 4.1, with `macos-private-api` off nothing in tao will call `setOpaque:` for us, and its
only complaint is an `eprintln!` gated on `debug_assertions`. So the app makes these calls
itself, now, while the feature is still on and the change is harmless. Doing it now means the
day the feature is dropped is not also the day this is discovered.

Task 4's probe decided whether the content view or the webview's own layer carries the radius.
Implement the one that worked.

**Files:**
- Create: `src-tauri/src/appkit.rs`
- Modify: `src-tauri/src/main.rs` (module list, and the setup call)
- Modify: `src-tauri/src/app.rs` (`setup_popover`)
- Modify: `src-tauri/src/pet.rs:57-64` (the macOS block in `setup`)
- Modify: `src-tauri/tauri.conf.json:21` (drop the popover's `transparent`)
- Modify: `src/popover.css` (the page states its own background)

**Interfaces:**
- Produces:
  - `appkit::make_transparent(ns: *mut std::ffi::c_void)`
  - `appkit::round_corners(ns: *mut std::ffi::c_void, radius: f64)`
  - `appkit::open_url(url: &str) -> bool` (used by Task 12)
  - `app::setup_popover(app: &AppHandle)`

- [x] **Step 1: Write the module**

Create `src-tauri/src/appkit.rs`:

```rust
//! The public AppKit calls this app makes for itself.
//!
//! Every one of these is something a framework used to do for us and will not once
//! `macos-private-api` is off. Spec section 4.1: `tauri-runtime-wry`'s
//! `window.transparent(...)` is behind that feature gate, `WindowBuilder::transparent()` carries
//! the same gate, and with the feature off the only feedback is an `eprintln!` gated on
//! `debug_assertions`, which is **silent in release builds**. So the premise that "a window can
//! be transparent with public API and only a webview cannot" is true of AppKit and false of
//! Tauri, and the way out is to make the AppKit calls here.
//!
//! `pet::macos` keeps the NSPanel reclass, which is a different kind of thing: that one changes
//! what the window *is*, and it is the fix the fullscreen behaviour was won with. This module is
//! only public API, only cosmetic, and safe to call on either window.
//!
//! Non-macOS builds get no-ops rather than a `cfg` at every call site, which is the same shape
//! `store::default_path` already uses for Windows.

/// `setOpaque: NO` plus a clear `backgroundColor`, which is exactly what
/// `tao/window.rs:544-561` does behind the private feature.
///
/// Takes tauri's own `ns_window()` return type, so no cast at the call site.
#[cfg(target_os = "macos")]
pub fn make_transparent(ns: *mut std::ffi::c_void) {
    use objc2::runtime::{AnyObject, Bool};

    let ns = ns as *mut AnyObject;
    if ns.is_null() {
        return;
    }
    unsafe {
        let _: () = objc2::msg_send![ns, setOpaque: Bool::NO];
        let clear = objc2_app_kit::NSColor::clearColor();
        let _: () = objc2::msg_send![ns, setBackgroundColor: &*clear];
    }
}

#[cfg(not(target_os = "macos"))]
pub fn make_transparent(_ns: *mut std::ffi::c_void) {}

/// Round the window's content view, so the popover reads as a panel against the desktop rather
/// than a rectangle with a drawn-on radius. `layer.cornerRadius` plus `masksToBounds`, both
/// public, which is what replaces the transparent webview (spec 5.1).
#[cfg(target_os = "macos")]
pub fn round_corners(ns: *mut std::ffi::c_void, radius: f64) {
    use objc2::runtime::{AnyObject, Bool};

    let ns = ns as *mut AnyObject;
    if ns.is_null() {
        return;
    }
    unsafe {
        let view: *mut AnyObject = objc2::msg_send![ns, contentView];
        if view.is_null() {
            return;
        }
        let _: () = objc2::msg_send![view, setWantsLayer: Bool::YES];
        let layer: *mut AnyObject = objc2::msg_send![view, layer];
        if layer.is_null() {
            return;
        }
        let _: () = objc2::msg_send![layer, setCornerRadius: radius];
        let _: () = objc2::msg_send![layer, setMasksToBounds: Bool::YES];
    }
}

#[cfg(not(target_os = "macos"))]
pub fn round_corners(_ns: *mut std::ffi::c_void, _radius: f64) {}

/// Open a URL in the user's browser. `NSWorkspace`, not a shellout: `/usr/bin/open` would be an
/// `exec` of a program outside the bundle, which Apple's sandbox documentation puts out of reach
/// of the file-access entitlements this app has.
///
/// Returns whether AppKit accepted it. The caller does nothing with a `false` except not lie
/// about it: a link that will not open is a cosmetic failure, and the same page is also linked
/// from the App Store listing.
#[cfg(target_os = "macos")]
pub fn open_url(url: &str) -> bool {
    use objc2_app_kit::NSWorkspace;
    use objc2_foundation::{NSString, NSURL};

    match NSURL::URLWithString(&NSString::from_str(url)) {
        Some(url) => NSWorkspace::sharedWorkspace().openURL(&url),
        None => false,
    }
}

#[cfg(not(target_os = "macos"))]
pub fn open_url(_url: &str) -> bool {
    false
}
```

- [x] **Step 2: Add the popover setup**

In `src-tauri/src/app.rs`, add after `sync_watcher`:

```rust
/// The popover's window chrome, which the app owns now.
///
/// Two calls, both public AppKit, both previously done for us by the private-API feature or not
/// needed at all. `transparent: true` is gone from the window's config (spec 5.1): the room art
/// fills the whole surface, so the popover never needed a see-through webview. What it needed was
/// rounded corners, and those come from the layer.
pub fn setup_popover(app: &AppHandle) {
    let Some(win) = app.get_webview_window(POPOVER) else {
        return;
    };
    #[cfg(target_os = "macos")]
    if let Ok(ns) = win.ns_window() {
        crate::appkit::make_transparent(ns);
        // 12pt, matching `.panel`'s `border-radius: 12px` in popover.css. If these ever
        // disagree the border draws a different curve than the mask cuts.
        crate::appkit::round_corners(ns, 12.0);
    }
}
```

- [x] **Step 3: Wire it up, and give the pet the same call**

In `src-tauri/src/main.rs`, add `mod appkit;` to the module list (first, alphabetically) and add
after `pet::setup(&handle)?;`:

```rust
            app::setup_popover(&handle);
```

In `src-tauri/src/pet.rs`, inside `setup`'s macOS block, after `macos::apply(...)`:

```rust
        // Redundant while `macos-private-api` is on, because tao does it. Load-bearing the day
        // it is off, and silent if it is missing then. Spec section 4.1.
        crate::appkit::make_transparent(win.ns_window()?);
```

- [x] **Step 4: Drop the popover's transparency and give the page a background**

In `src-tauri/tauri.conf.json`, delete `"transparent": true,` from the **popover** window entry
(line 21). Leave the pet entry untouched: whether that one keeps the flag is decided by the
deferred section 4 work, not here.

In `src/popover.css`, add under the opening comment:

```css
/* The window is opaque now (spec 5.1). style.css says `background: transparent` because the pet
   shares it, so the popover states its own ground here rather than changing that for both. */
html,
body {
  background: var(--panel);
}
```

- [x] **Step 5: Build and look at it**

Run: `cd src-tauri && cargo tauri dev`

Expected: the popover opens with 12pt rounded corners over both a light and a dark wallpaper,
with no square opaque backdrop showing behind the radius, and the drop shadow following the
rounded shape. The room, the quote, the project list and both buttons are unchanged.

If the corners are square, apply the fallback Task 4 recorded: round the webview's own layer
instead of the content view's, via `win.with_webview(...)` in `app::setup_popover`.

- [x] **Step 6: Run the tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: PASS, everything. No test covers window chrome; this step is checking nothing was
broken on the way.

- [x] **Step 7: Commit**

```sh
git add src-tauri/src/appkit.rs src-tauri/src/main.rs src-tauri/src/app.rs \
        src-tauri/src/pet.rs src-tauri/tauri.conf.json src/popover.css
git commit -m "Make the window chrome calls the app has to make itself"
```

## Task 12: The privacy policy, hosted and linked from the popover

Guideline 5.1.1(i) requires a policy link in App Store Connect **and** "within the app in an
easily accessible manner". `site/index.html:135` is a privacy *section* on a one-page site, not a
policy, so a real page is needed.

Decided in the spec: the in-app link goes in the **popover**, not the tray. `tray.rs:22-23`'s
"exactly two items, and adding a third is a spec change" is a deliberate design position, and the
popover is the app's one real surface with interactive chrome already in it, so a reviewer
scanning for the link finds it there.

**Files:**
- Create: `site/privacy.html`
- Modify: `site/index.html` (the privacy section and the footer)
- Modify: `src-tauri/src/commands.rs` (a new command), `src-tauri/src/main.rs` (register it)
- Modify: `src/index.html:42` (the credit line), `src/popover.css` (the link style), `src/popover.js` (the handler)

**Interfaces:**
- Consumes: `appkit::open_url` from Task 11.
- Produces: the `open_privacy_policy` Tauri command, invoked from `popover.js`. Custom app commands need no capability entry; only plugin and `core:` commands do, which is why `capabilities/default.json` does not change.

- [x] **Step 1: Write the policy page**

Create `site/privacy.html`, reusing the site's own stylesheet and voice:

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Privacy Policy | Momentum Mascot</title>
    <meta name="description" content="Momentum Mascot collects nothing, sends nothing, and stores everything on your own machine." />
    <link rel="stylesheet" href="/style.css" />
  </head>
  <body>
    <main>
      <section class="privacy wrap">
        <h1>Privacy Policy</h1>
        <p><strong>Last updated: 22 August 2026</strong></p>

        <p>
          Momentum Mascot does not collect, transmit, sell, or share any personal data. There is
          no analytics, no telemetry, no crash reporting, no accounts, and no network layer at
          all. This is not a policy choice layered on top of the app; it is what the app is made
          of.
        </p>

        <h2>What the app reads</h2>
        <p>
          For each git repository you explicitly add through the folder picker, the app reads the
          repository's reflog and the modification times of files in the working tree. From that
          it derives one thing: when you last did work there. It does not read commit messages,
          diffs, branch names, file contents, or anything that identifies your code.
        </p>

        <h2>What the app stores, and where</h2>
        <p>
          A single JSON file on your machine, holding the folders you added, the timestamps
          derived from them, which character you picked, and where you dragged the pet. You can
          read it or delete it at any time.
        </p>
        <ul class="checks">
          <li>Mac App Store version: <code>~/Library/Containers/dev.keepgoing.momentum-mascot/Data/.keepgoing/mascot/state.json</code></li>
          <li>Direct download version: <code>~/.keepgoing/mascot/state.json</code></li>
        </ul>
        <p>
          Nothing in that file leaves your machine. There is no server to send it to.
        </p>

        <h2>The share card</h2>
        <p>
          Share Status draws a 1200x630 image and puts it on your clipboard. The image carries the
          mood and the room and deliberately nothing that identifies a project: no names, paths,
          commit messages, hashes, or timestamps. It goes to your clipboard, not to us. What you
          then paste it into is between you and that app.
        </p>

        <h2>Children</h2>
        <p>
          The app collects no data from anyone, of any age.
        </p>

        <h2>Changes</h2>
        <p>
          If this ever changes, this page changes with it, and the date at the top moves. Any
          release that starts collecting anything would say so here first.
        </p>

        <h2>Contact</h2>
        <p>
          Questions go to <a href="https://github.com/keepgoing-dev/momentum-mascot/issues">the issue tracker</a>.
        </p>

        <p><a href="/">Back to Momentum Mascot</a></p>
      </section>
    </main>
  </body>
</html>
```

- [x] **Step 2: Link it from the site**

In `site/index.html`, add to the privacy section's `<ul class="checks">` (after the
`~/.keepgoing/mascot/state.json` bullet):

```html
          <li>The full <a href="/privacy">privacy policy</a>, which says the same thing at greater length.</li>
```

And in the footer paragraph, after the GitHub link sentence:

```html
        <a href="/privacy">Privacy</a>.
```

- [x] **Step 3: Add the command**

In `src-tauri/src/commands.rs`, add `use crate::appkit;` to the imports and append:

```rust
/// The privacy policy, opened in the user's browser.
///
/// Narrow on purpose: it takes no URL. Guideline 5.1.1(i) wants the policy reachable from inside
/// the app, and an `open_url(url)` command would hand the webview the ability to open anything,
/// which is a larger API than the requirement. One constant, one destination.
///
/// This is the eleventh command, and `commands.rs`'s own rule is that there is no eleventh
/// without a reason. The reason is a review guideline.
#[tauri::command]
pub fn open_privacy_policy() {
    if !appkit::open_url(PRIVACY_POLICY_URL) {
        eprintln!("could not open {PRIVACY_POLICY_URL}");
    }
}

/// Kept next to the command that opens it, and it must stay in step with the URL in App Store
/// Connect: guideline 5.1.1(i) asks for the policy in both places.
pub const PRIVACY_POLICY_URL: &str = "https://keepgoing.dev/privacy";
```

Also update the module doc comment at `src-tauri/src/commands.rs:1-2`, which says "Ten commands":

```rust
//! Everything the webview is allowed to ask for. Eleven commands, and no twelfth without a
//! reason: this list is the whole API surface between the art and the machinery.
```

In `src-tauri/src/main.rs`, add to `generate_handler!`:

```rust
            commands::open_privacy_policy,
```

- [x] **Step 4: Add the link to the popover**

In `src/index.html`, replace line 42:

```html
      <!-- The credit is a licence requirement, not a courtesy (section 4.2). With no about
           window and no settings screen, this is the only place in the app it can live. The
           privacy link shares the line because App Review needs to find it and because the
           352x540 budget has no room for a line of its own. -->
      <p class="credit">
        art: limezu.itch.io
        <button id="privacy" class="linkish" type="button">privacy</button>
      </p>
```

In `src/popover.css`, add after the `.credit, .clock` rule:

```css
.credit .linkish {
  margin-left: 6px;
  padding: 0;
  border: 0;
  background: none;
  color: var(--muted);
  font: inherit;
  text-decoration: underline;
  cursor: pointer;
}

.credit .linkish:hover {
  color: var(--cream);
}
```

In `src/popover.js`, add after the `charHit` listener at line 176:

```js
document
  .getElementById("privacy")
  .addEventListener("click", () => invoke("open_privacy_policy"));
```

- [x] **Step 5: Run it and click the link**

Run: `cd src-tauri && cargo tauri dev`

Expected: the credit line reads `art: limezu.itch.io privacy`, the popover height still fits
(`fitWindow` measures it, so a taller credit line is absorbed), and clicking `privacy` opens
`https://keepgoing.dev/privacy` in the default browser without the popover closing on the focus
loss. If the popover does close, that is the click-outside rule in `main.rs:75-80` firing on a
focus loss caused by the browser, which is acceptable and expected: the page is open behind it.

- [x] **Step 6: Confirm the page is live**

The site deploys from `site/` as the Cloudflare Pages project named `keepgoing`. Push, wait for
the deploy, then:

```sh
curl -sS -o /dev/null -w '%{http_code}\n' https://keepgoing.dev/privacy
```

Expected: `200`. If it is `404`, the extensionless route is not being served and the constant in
`commands.rs` plus the two `site/` links must all move to `https://keepgoing.dev/privacy.html`.
The URL in the App Store Connect listing has to match whichever one is live.

- [x] **Step 7: Run the tests**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: PASS, everything.

- [x] **Step 8: Commit**

```sh
git add site/privacy.html site/index.html src-tauri/src/commands.rs src-tauri/src/main.rs \
        src/index.html src/popover.css src/popover.js
git commit -m "Add a privacy policy page and link it from the popover"
```

## Task 13: The manual sandbox persistence test, which is what the whole effort is for

Everything in Phase 3 is untested until this passes. The automated `scoped.rs` test is a smoke
test by construction and proves nothing about a relaunch.

**Files:**
- Modify: `docs/app-store.md` is not written yet, so record the result in the commit message and in `spikes/app-store/RESULTS.md`.

- [x] **Step 1: Build and ad-hoc sign sandboxed**

```sh
cd src-tauri && cargo tauri build --bundles app && cd ..
APP="src-tauri/target/release/bundle/macos/Momentum Mascot.app"
codesign --force --sign - --options runtime \
  --entitlements src-tauri/Entitlements.mas.plist "$APP"
codesign -d --entitlements - "$APP"
```

Expected: the printed entitlements list `com.apple.security.app-sandbox`,
`files.user-selected.read-only`, `files.bookmarks.app-scope` and `cs.allow-jit`, plus
`network.client` if Task 2 found it necessary.

- [x] **Step 2: Confirm the container, and that no migration happened**

```sh
# NOT `rm -rf ~/Library/Containers/<bundle id>`: macOS protects the container root and that
# fails with "Operation not permitted" on .com.apple.containermanagerd.metadata.plist. Delete
# what is inside Data/ instead.
rm -rf "$HOME/Library/Containers/dev.keepgoing.momentum-mascot/Data/.keepgoing"
open "$APP"
```

Add a repository through the popover's Add Project, then:

```sh
find "$HOME/Library/Containers/dev.keepgoing.momentum-mascot/Data/.keepgoing" -type f
python3 -c "import json,os,sys; p=os.path.expanduser('~/Library/Containers/dev.keepgoing.momentum-mascot/Data/.keepgoing/mascot/state.json'); d=json.load(open(p)); print(d['version']); [print(x['name'], x['path'], 'bookmark' if x.get('bookmark') else 'NO BOOKMARK') for x in d['tracked_projects']]"
```

Expected: `state.json` is inside the container, `version` is `3.1`, and the project has a
bookmark. Expected too: the real `~/.keepgoing/mascot/state.json` is untouched and the store
build has no projects it did not add itself, because there is no migration and could not be one.

- [x] **Step 3: The test itself**

Quit through the tray menu. Then:

```sh
open "$APP"
```

Expected, and this is the assertion the whole phase rests on: the repository is still listed,
still shows a relative time rather than `unavailable`, and the mood is built from it. Before this
change it would have read `unavailable` on this second launch, because the picker's grant expired
with the first one.

- [x] **Step 4: Assert the section 7.2 decision, whichever way it went**

Create a linked worktree and track it:

```sh
cd /tmp && rm -rf wt-probe && git init wt-probe && cd wt-probe \
  && git commit --allow-empty -m "first" \
  && git worktree add ../wt-probe-linked && cd -
```

Add `/tmp/wt-probe-linked` through the popover.

Expected: the row reads `unavailable`, and hovering the name shows "That worktree's git folder
isn't reachable from here." rather than "(not reachable right now)". An ordinary clone added
alongside it is unaffected. This is the accepted degradation, made legible.

- [x] **Step 5: Confirm the watcher still works inside the sandbox**

With the app running and an ordinary tracked repository:

```sh
cd <the tracked repo> && git commit --allow-empty -m "watcher check"
```

Expected: the popover's relative time updates within a second or two, not a minute. This
exercises the recursive FSEvents grant inside a held scope, and the dot-directory read of
`.git/logs/HEAD`, both of which the spec verified separately and neither of which needs a
`watcher.rs` change.

- [x] **Step 6: Record and commit**

Append to `spikes/app-store/RESULTS.md`:

```markdown
## Sandbox persistence, the test the whole effort is for

**Measured on:** <date>
**Method:** release build, `codesign --force --sign - --options runtime --entitlements
Entitlements.mas.plist`, added a repository, quit through the tray, relaunched.

**Result:** <the repository was still readable and the mood was still built from it | ...>
**Container state file:** version 3.1, bookmark present: <yes | no>
**Worktree row:** <showed the specific message | ...>
**Watcher inside the sandbox:** <a commit updated the popover within a second | ...>
```

```sh
git add spikes/app-store/RESULTS.md
git commit -m "Record the sandbox persistence result"
```

- [x] **Step 7: Stop here if it failed**

If the repository read as `unavailable` on the second launch, do not continue to Phase 5. The
things to check, in order: is `files.bookmarks.app-scope` actually in the signed entitlements
(`codesign -d --entitlements -`); did `scoped::create` return `Some` at add time (log it); does
`scoped::resolve` return `Some` at load (log it); and did `startAccessingSecurityScopedResource`
return true (log it). Spec section 6 exists for exactly this case and its four steps are the four
places it can break.

---

**If Task 3's probe failed:** write and execute the deferred plan for spec section 4 now, before
Phase 5. The gate in Task 15 will not let a submission through until `drawsBackground` and
`fullScreenEnabled` are out of the binary.

---

# Phase 5: certificates and the release script

## Task 14: Point the bundle at the category the review argument needs

The 4.2 minimum-functionality argument in the spec rests on this being a developer tool, and the
bundle currently says `Utility`. When the category is doing review work, the listing and the
bundle have to agree.

**Files:**
- Modify: `src-tauri/tauri.conf.json:51`

- [x] **Step 1: Change the category**

In `src-tauri/tauri.conf.json`, in the `bundle` object:

```json
    "category": "DeveloperTool",
```

- [x] **Step 2: Verify what lands in the built Info.plist**

```sh
cd src-tauri && cargo tauri build --bundles app && cd ..
/usr/libexec/PlistBuddy -c "Print :LSApplicationCategoryType" \
  "src-tauri/target/release/bundle/macos/Momentum Mascot.app/Contents/Info.plist"
```

Expected: `public.app-category.developer-tools`. It read `public.app-category.utilities` before.

- [x] **Step 3: Commit**

```sh
git add src-tauri/tauri.conf.json
git commit -m "Declare the app a developer tool in the bundle"
```

## Task 15: tools/release-mas.sh

A sibling of `release.sh`, not a modification of it. The DMG path works and must not be
destabilised. This script never tags, never touches `CHANGELOG.md`, never creates a GitHub
release, and never notarizes, because "you aren't required to notarize software that you
distribute through the Mac App Store".

**Files:**
- Create: `tools/release-mas.sh`
- Modify: `tools/.release-env.example`, `.gitignore`

**Interfaces:**
- Consumes: `src-tauri/Entitlements.mas.plist` from Task 2.
- Produces: `tools/.mas-build` (a gitignored monotonic build counter), and a signed `.pkg` under `src-tauri/target/universal-apple-darwin/release/bundle/`.

- [x] **Step 1: Write the script**

Create `tools/release-mas.sh`:

```sh
#!/bin/sh
#
# Builds, signs, packages and uploads the Mac App Store build.
#
# A sibling of release.sh, deliberately not a modification of it: the DMG path works and must not
# be destabilised. Compared to that script, this one never tags, never touches CHANGELOG.md,
# never creates a GitHub release, and never notarizes. Apple: "you aren't required to notarize
# software that you distribute through the Mac App Store because the App Store submission process
# already includes equivalent security checks."
#
# Version bumping stays in release.sh, so the two channels cannot disagree about what a version
# is. The BUILD NUMBER is this script's own counter, because App Store Connect rejects a re-upload
# that reuses one and a first submission is very likely re-uploaded at least once.
#
# Usage:
#
#   tools/release-mas.sh              # build, sign, package, validate. Uploads nothing.
#   tools/release-mas.sh --upload     # ... and upload to App Store Connect
#
# Credentials come from tools/.release-env, which is gitignored. The one-time Apple setup is in
# docs/app-store.md.
#
# MASCOT_MAS_ALLOW_PRIVATE_API=1 downgrades the private-API gate to a warning so the pipeline can
# be rehearsed before the pet work lands. It also refuses to upload, which is the point: every
# step can be practised except the irreversible one.
set -eu

ROOT=$(cd "$(dirname "$0")/.." && pwd)
APP_NAME="Momentum Mascot"
BUNDLE_ID="dev.keepgoing.momentum-mascot"
BIN_NAME="momentum-mascot"

export PATH="$HOME/.cargo/bin:$PATH"
cd "$ROOT"

UPLOAD=""
if [ "${1:-}" = "--upload" ]; then
  UPLOAD=1
elif [ $# -gt 0 ]; then
  echo "usage: tools/release-mas.sh [--upload]" >&2
  exit 1
fi

if [ -f "$ROOT/tools/.release-env" ]; then
  . "$ROOT/tools/.release-env"
fi

# Tauri signs with APPLE_SIGNING_IDENTITY and notarizes with APPLE_ID/APPLE_PASSWORD during the
# build when they are set, and both are wrong here: this build is signed by this script with a
# different identity and a different entitlements file, and a store submission is not notarized.
unset APPLE_SIGNING_IDENTITY APPLE_ID APPLE_PASSWORD

VERSION=$(grep -m1 '"version"' src-tauri/tauri.conf.json | sed 's/.*"version": "\([^"]*\)".*/\1/')
echo "version: $VERSION"

# ---------------------------------------------------------------- signing preflight
#
# Runs before the build on purpose. A missing certificate should cost a second, not a ten minute
# build.
#
# Bare `-v`, never `-p codesigning`. Apple: "Don't use the -p codesigning option... Installer-
# signing identities are different from code-signing identities, so the -p codesigning option
# filters out installer-signing identities." release.sh:118 uses `-p codesigning` correctly for
# its own purpose, and copying that line here would make this step fail on a correctly configured
# machine.

IDENTITIES=$(security find-identity -v 2>/dev/null || true)

if [ -z "${MAS_APP_IDENTITY:-}" ]; then
  MAS_APP_IDENTITY=$(echo "$IDENTITIES" \
    | sed -n 's/.*"\(Apple Distribution: [^"]*\)".*/\1/p' | head -n 1)
fi
if [ -z "${MAS_INSTALLER_IDENTITY:-}" ]; then
  MAS_INSTALLER_IDENTITY=$(echo "$IDENTITIES" \
    | sed -n 's/.*"\(3rd Party Mac Developer Installer: [^"]*\)".*/\1/p' | head -n 1)
fi

if [ -z "$MAS_APP_IDENTITY" ]; then
  echo "error: no 'Apple Distribution' certificate in the keychain" >&2
  echo "" >&2
  echo "  This is the certificate that signs the .app. Create it at" >&2
  echo "  https://developer.apple.com/account/resources/certificates" >&2
  echo "  Its common name reads 'Apple Distribution: <name> (<team id>)'." >&2
  echo "  See docs/app-store.md." >&2
  exit 1
fi

if [ -z "$MAS_INSTALLER_IDENTITY" ]; then
  echo "error: no '3rd Party Mac Developer Installer' certificate in the keychain" >&2
  echo "" >&2
  echo "  This is the certificate that signs the .pkg. The developer portal calls it" >&2
  echo "  'Mac Installer Distribution', which is NOT its common name: no certificate's" >&2
  echo "  common name reads that. Same certificate, different label." >&2
  echo "  See docs/app-store.md." >&2
  exit 1
fi

echo "app identity:       $MAS_APP_IDENTITY"
echo "installer identity: $MAS_INSTALLER_IDENTITY"

if [ -n "$UPLOAD" ]; then
  MISSING=""
  [ -z "${ASC_API_KEY_ID:-}" ] && MISSING="$MISSING ASC_API_KEY_ID"
  [ -z "${ASC_API_ISSUER_ID:-}" ] && MISSING="$MISSING ASC_API_ISSUER_ID"
  if [ -n "$MISSING" ]; then
    echo "error: missing App Store Connect credentials:$MISSING" >&2
    echo "  Copy tools/.release-env.example to tools/.release-env and fill it in." >&2
    exit 1
  fi
  KEYFILE="$HOME/.appstoreconnect/private_keys/AuthKey_$ASC_API_KEY_ID.p8"
  if [ ! -f "$KEYFILE" ]; then
    echo "error: $KEYFILE not found" >&2
    echo "  altool searches ./private_keys, ~/private_keys, ~/.private_keys and" >&2
    echo "  ~/.appstoreconnect/private_keys for AuthKey_<key id>.p8." >&2
    exit 1
  fi
fi

# ---------------------------------------------------------------- assets

if [ -n "${MASCOT_PACK:-}" ]; then
  echo "recompositing assets from \$MASCOT_PACK..."
  "$ROOT/tools/build-app-assets.sh"
elif [ ! -d "$ROOT/src/assets/rooms" ]; then
  echo "error: src/assets is missing" >&2
  echo "set \$MASCOT_PACK to recomposite, or run tools/build-app-assets.sh first" >&2
  exit 1
else
  echo "using existing composed assets in src/assets/"
fi

# ---------------------------------------------------------------- build number
#
# Burned before the build, not after the upload. A wasted number costs nothing; a reused one
# costs a rejected upload and a rebuild.

BUILD_FILE="$ROOT/tools/.mas-build"
if [ -f "$BUILD_FILE" ]; then
  PREV=$(cat "$BUILD_FILE")
else
  PREV=0
fi
BUILD=$((PREV + 1))
echo "$BUILD" > "$BUILD_FILE"
echo "build number: $BUILD"

# ---------------------------------------------------------------- build

echo "building universal macOS app..."

# Start from an empty bundle directory, for the same reason release.sh:223 does: Tauri reuses
# what is there, so a leftover ad-hoc signed .app can survive into a real submission.
rm -rf "$ROOT/src-tauri/target/universal-apple-darwin/release/bundle"

# --bundles app: no .dmg. The store channel does not want one and building it is a minute of
# nothing.
(cd src-tauri && cargo tauri build --target universal-apple-darwin --bundles app)

APP="$ROOT/src-tauri/target/universal-apple-darwin/release/bundle/macos/$APP_NAME.app"
if [ ! -d "$APP" ]; then
  echo "error: $APP not found after the build" >&2
  exit 1
fi

lipo -archs "$APP/Contents/MacOS/$BIN_NAME"

# ---------------------------------------------------------------- private API gate
#
# The two removable private KVC keys, from tauri's macos-private-api feature: drawsBackground
# (wry wkwebview/mod.rs:376, :382, :973) and fullScreenEnabled (:386-388). Both must be gone.
#
# Two OTHER private strings stay, and that is expected, not an oversight:
# allowsPictureInPictureMediaPlayback (wry, behind no feature gate) and _wantsKeyDownForEvent
# (tao, registered unconditionally). Removing those means forking wry and tao. Spec section 2.2.
# So this grep names exactly the two that are ours to remove, and nothing else.

PRIVATE=$(strings -a "$APP/Contents/MacOS/$BIN_NAME" \
  | grep -cE 'drawsBackground|fullScreenEnabled' || true)

if [ "$PRIVATE" -ne 0 ]; then
  if [ -n "${MASCOT_MAS_ALLOW_PRIVATE_API:-}" ]; then
    echo "" >&2
    echo "WARNING: $PRIVATE line(s) still carry drawsBackground / fullScreenEnabled." >&2
    echo "WARNING: rehearsal only. --upload is refused while this is set." >&2
    echo "" >&2
    UPLOAD=""
  else
    echo "error: the binary still carries the two removable private KVC keys" >&2
    echo "" >&2
    echo "  drawsBackground and fullScreenEnabled come from tauri's macos-private-api" >&2
    echo "  feature in src-tauri/Cargo.toml. Dropping it is the pet work in spec" >&2
    echo "  section 4, or the section 4.0 probe if that came out well." >&2
    echo "" >&2
    echo "  To rehearse this script without uploading:" >&2
    echo "    MASCOT_MAS_ALLOW_PRIVATE_API=1 tools/release-mas.sh" >&2
    exit 1
  fi
else
  echo "private API check: clean"
fi

# ---------------------------------------------------------------- build number and profile
#
# Tauri writes tauri.conf.json's version into BOTH CFBundleShortVersionString and CFBundleVersion,
# so without this every upload of 0.3.1 would claim build 0.3.1 and the second one would be
# rejected.

/usr/libexec/PlistBuddy -c "Set :CFBundleVersion $BUILD" "$APP/Contents/Info.plist"
echo "CFBundleShortVersionString: $(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$APP/Contents/Info.plist")"
echo "CFBundleVersion:            $(/usr/libexec/PlistBuddy -c 'Print :CFBundleVersion' "$APP/Contents/Info.plist")"

# A provisioning profile is NOT required. TN3125: "A Mac app that uses no restricted entitlements
# doesn't need a provisioning profile. This is true even if the app is distributed on the App
# Store. The only exception to this rule is TestFlight, which always requires a profile." App
# Sandbox and Hardened Runtime entitlements are both on Apple's unrestricted list. So this is a
# warning, never an error. If one IS present it is copied in BEFORE signing, because "the profile
# is sealed by the code signature".
PROFILE="$ROOT/tools/embedded.provisionprofile"
if [ -f "$PROFILE" ]; then
  cp "$PROFILE" "$APP/Contents/embedded.provisionprofile"
  echo "embedded the provisioning profile"
else
  echo "note: no tools/embedded.provisionprofile. Not required (TN3125); TestFlight would need one."
fi

# ---------------------------------------------------------------- sign
#
# Apple, both current: "Sign code from the inside out" and "Don't pass the --deep option to
# codesign when you sign code."
#
# Measured: this bundle has no nested code at all. Contents holds Info.plist, MacOS/, Resources/
# and the signature. So "inside out" is one call. If nested code ever appears, this refuses rather
# than signing an outer bundle over unsigned inner code.

if [ -d "$APP/Contents/Frameworks" ] || [ -d "$APP/Contents/PlugIns" ] \
  || [ -d "$APP/Contents/XPCServices" ] || [ -d "$APP/Contents/Library" ]; then
  echo "error: nested code appeared in the bundle" >&2
  echo "  Sign the nested code first, without entitlements, then the app. Do not use --deep." >&2
  exit 1
fi

codesign --force --timestamp --options runtime \
  --sign "$MAS_APP_IDENTITY" \
  --entitlements "$ROOT/src-tauri/Entitlements.mas.plist" \
  "$APP"

codesign --verify --strict --verbose=2 "$APP"
echo "sealed entitlements:"
codesign -d --entitlements - "$APP" 2>/dev/null || true

# ---------------------------------------------------------------- package
#
# Verbatim Apple's own recipe: "The following is the simplest use of productbuild, sufficient for
# submitting your app to the Mac App Store: productbuild --sign <Identity> --component
# <PathToApp> /Applications <PathToPackage>".

PKG="$ROOT/src-tauri/target/universal-apple-darwin/release/bundle/$APP_NAME-$VERSION-$BUILD.pkg"
rm -f "$PKG"
productbuild --sign "$MAS_INSTALLER_IDENTITY" --component "$APP" /Applications "$PKG"
echo "packaged: $PKG"

# ---------------------------------------------------------------- validate and upload
#
# altool, not notarytool. TN3147: "Apple has deprecated altool for the purposes of notarization...
# However, altool is still a good way to perform other tasks, like submitting an app to the App
# Store." notarytool is not a store-upload tool. Measured against altool 26.40.1: --upload-package
# and --validate-app both take a path plus authentication and nothing else, and the API key flags
# are --api-key / --api-issuer.

if [ -z "${ASC_API_KEY_ID:-}" ] || [ -z "${ASC_API_ISSUER_ID:-}" ]; then
  echo ""
  echo "no App Store Connect API key configured, so skipping validation."
  echo "package is at: $PKG"
  exit 0
fi

echo "validating the package (this catches signing and entitlement errors before an upload)..."
xcrun altool --validate-app "$PKG" \
  --api-key "$ASC_API_KEY_ID" \
  --api-issuer "$ASC_API_ISSUER_ID"

if [ -n "$UPLOAD" ]; then
  echo "uploading..."
  xcrun altool --upload-package "$PKG" \
    --api-key "$ASC_API_KEY_ID" \
    --api-issuer "$ASC_API_ISSUER_ID" \
    --show-progress
  echo ""
  echo "uploaded $VERSION build $BUILD"
  echo "processing takes a few minutes. Then set the build on the version in App Store Connect."
else
  echo ""
  echo "validated but NOT uploaded. Re-run with --upload when the validation is clean:"
  echo "  tools/release-mas.sh --upload"
  echo ""
  echo "note: that rebuilds and burns build number $((BUILD + 1)), which is fine and expected."
fi
```

- [x] **Step 2: Make it executable and gitignore its state**

```sh
chmod +x tools/release-mas.sh
```

Append to `.gitignore`:

```gitignore
# The Mac App Store build counter. App Store Connect rejects a reused build number, so this only
# ever goes up. Local because the store build is local, same as tools/.release-env.
tools/.mas-build

# A Mac App Store provisioning profile, if one is ever embedded. Not required (TN3125), and it
# carries the team's identifiers, so it is not committed.
tools/embedded.provisionprofile
```

- [x] **Step 3: Document the new credentials**

Append to `tools/.release-env.example`:

```sh
# ---------------------------------------------------------------- Mac App Store only
#
# tools/release-mas.sh reads these. The DMG path does not use them.
#
# Both certificates are found automatically in the keychain by their common names, so these two
# are only needed to disambiguate when more than one is installed. Note the second one's name:
# the developer portal calls it "Mac Installer Distribution", and no certificate's common name
# reads that.
# MAS_APP_IDENTITY="Apple Distribution: Your Name (TEAMID1234)"
# MAS_INSTALLER_IDENTITY="3rd Party Mac Developer Installer: TEAMID1234"

# App Store Connect API key, from Users and Access > Integrations > App Store Connect API.
# An API key is better auth for uploads than the app-specific password above, and it is what
# altool's --api-key / --api-issuer want.
#
# The private key file must be named AuthKey_<key id>.p8 and live in one of the directories
# altool searches:  ./private_keys  ~/private_keys  ~/.private_keys
#                   ~/.appstoreconnect/private_keys  $API_PRIVATE_KEYS_DIR
ASC_API_KEY_ID="ABC123DEF4"
ASC_API_ISSUER_ID="00000000-0000-0000-0000-000000000000"
```

- [x] **Step 4: Rehearse the script without uploading**

```sh
MASCOT_MAS_ALLOW_PRIVATE_API=1 tools/release-mas.sh
```

Expected, in order: version and both identities printed (or a clear error naming the missing
certificate, which is the correct outcome before Task 16 is done), assets found, a build number,
a universal build, `lipo -archs` reporting `x86_64 arm64`, the private-API warning with upload
refused, `CFBundleVersion` stamped to the build number, no nested code, a clean
`codesign --verify --strict`, the sealed entitlements printed with `app-sandbox` among them, and
a `.pkg` produced.

If the certificates are not installed yet, the script exits at the preflight with the message
that names which one is missing. That is the script working. Come back after Task 16.

- [x] **Step 5: Confirm the DMG path is untouched**

```sh
git diff --stat HEAD -- tools/release.sh
```

Expected: no output. `release.sh` was not modified.

- [x] **Step 6: Commit**

```sh
git add tools/release-mas.sh tools/.release-env.example .gitignore
git commit -m "Add the Mac App Store release script"
```

## Task 16: docs/app-store.md, the one-time Apple setup

The sibling of `docs/notarization.md`, which already says the thing this document exists to
correct: Developer ID is for distribution outside the store, and submitting with it yields App
Store Connect error 90034.

**Files:**
- Create: `docs/app-store.md`
- Modify: `docs/notarization.md` (a pointer at the top)
- Modify: `README.md` (the release section)

- [x] **Step 1: Write the document**

Create `docs/app-store.md`:

````markdown
# Mac App Store: the one-time setup

`docs/notarization.md` covers the DMG channel. This covers the store, and the two use
**different certificates**. The machine currently holds exactly one identity:

```sh
security find-identity -v
#  1) ... "Developer ID Application: Hoa Trinh (3LM6674AC2)"
```

That one signs the DMG and cannot sign a store submission. Submitting with it yields App
Store Connect error 90034, "not signed using an Apple submission certificate".

Design and rationale: `docs/superpowers/specs/2026-08-22-mac-app-store-design.md`.
Implementation plan: `docs/superpowers/plans/2026-08-22-mac-app-store-submission.md`.

## 1. Register the App ID

<https://developer.apple.com/account/resources/identifiers> > Identifiers > App IDs >
App. Bundle ID `dev.keepgoing.momentum-mascot`, explicit, description "Momentum Mascot".
Enable no capabilities: App Sandbox and Hardened Runtime are not capabilities that need
registering, and this app uses no restricted entitlements.

## 2. Create the Apple Distribution certificate

This signs the `.app`.

<https://developer.apple.com/account/resources/certificates> > Certificates > + >
**Apple Distribution**. Generate a CSR from Keychain Access
(Certificate Assistant > Request a Certificate From a Certificate Authority, saved to
disk, 2048-bit RSA), upload it, download the `.cer`, double-click to install.

Its common name reads `Apple Distribution: Hoa Trinh (3LM6674AC2)`.

## 3. Create the Mac Installer Distribution certificate

This signs the `.pkg`, and this is the step with the trap in it.

Same page, + > **Mac Installer Distribution**. Same CSR flow.

**The portal label is not the certificate's common name.** The installed certificate reads
`3rd Party Mac Developer Installer: 3LM6674AC2`. No certificate's common name reads "Mac
Installer Distribution". `tools/release-mas.sh` looks for the real name.

Verify both, and note the flag:

```sh
security find-identity -v
```

Bare `-v`, not `-p codesigning`. Apple: "Don't use the `-p codesigning` option...
Installer-signing identities are different from code-signing identities, so the
`-p codesigning` option filters out installer-signing identities." `release.sh:118` uses
`-p codesigning` correctly for its own purpose; copying that line into a store script
makes it fail on a correctly configured machine.

Expected after this step: three identities, the Developer ID Application one plus these
two.

## 4. Create an App Store Connect API key

<https://appstoreconnect.apple.com/access/integrations/api> > Team Keys > +. Role
**App Manager** is enough for uploads. Download the `.p8` once; it cannot be downloaded
again.

```sh
mkdir -p ~/.appstoreconnect/private_keys
mv ~/Downloads/AuthKey_ABC123DEF4.p8 ~/.appstoreconnect/private_keys/
chmod 600 ~/.appstoreconnect/private_keys/AuthKey_ABC123DEF4.p8
```

Then put the key id and the issuer id into `tools/.release-env`, per
`tools/.release-env.example`. `tools/.release-env` currently holds an app-specific
password, which works, but an API key is the better auth for uploads.

Check it works before a build depends on it:

```sh
xcrun altool --list-providers --api-key "$ASC_API_KEY_ID" --api-issuer "$ASC_API_ISSUER_ID"
```

## 5. Optionally, a provisioning profile

**Not required.** TN3125: "A Mac app that uses no restricted entitlements doesn't need a
provisioning profile. This is true even if the app is distributed on the App Store. The
only exception to this rule is TestFlight, which always requires a profile." Apple's
unrestricted list explicitly includes "entitlements that enable and configure App Sandbox"
and "entitlements that configure the Hardened Runtime", which is every key in
`src-tauri/Entitlements.mas.plist`.

Do it anyway if you want to have walked it: Profiles > + > Mac App Store, pick the App ID
and the Apple Distribution certificate, download, and save it as
`tools/embedded.provisionprofile`. `release-mas.sh` copies it in **before** signing,
because "the profile is sealed by the code signature", and warns rather than failing when
it is absent.

## 6. Create the app record

<https://appstoreconnect.apple.com> > Apps > + > New App. Platform macOS, name
"Momentum Mascot", primary language English (UK or US), bundle ID from step 1, SKU
`momentum-mascot-1`.

The listing content itself is in `docs/app-store-listing.md`.

## Then

```sh
tools/release-mas.sh              # build, sign, package, validate
tools/release-mas.sh --upload     # and upload
```

The script never tags, never touches `CHANGELOG.md`, never creates a GitHub release, and
never notarizes. Version bumps stay in `tools/release.sh` so the two channels cannot
disagree about what a version is; the build number is `release-mas.sh`'s own counter in
`tools/.mas-build`, because App Store Connect rejects a re-upload that reuses one.
````

- [x] **Step 2: Cross-reference from the DMG doc**

Add near the top of `docs/notarization.md`, after its first paragraph:

```markdown
> This document is the **direct download** channel: a Developer ID certificate, notarized,
> shipped as a `.dmg`. The Mac App Store channel uses different certificates and a
> different script, and is documented in `docs/app-store.md`.
```

- [x] **Step 3: Mention it in the README**

In `README.md`, in the release section around line 80, add:

```markdown
The Mac App Store build is a separate script with separate certificates:
`tools/release-mas.sh`, set up per `docs/app-store.md`. Both channels ship from this one
codebase and differ by a single entitlements file at signing time.
```

- [x] **Step 4: Do the setup**

Work through sections 1 to 6 of the document just written. This is the part of the whole project
that the project exists to learn, so read what each screen actually says rather than clicking
through.

- [x] **Step 5: Verify**

```sh
security find-identity -v
xcrun altool --generate-jwt --apiKey "$ASC_API_KEY_ID" --apiIssuer "$ASC_API_ISSUER_ID"
```

Expected: three identities including `Apple Distribution:` and
`3rd Party Mac Developer Installer:`, and exit 0 from the second.

`--list-providers`, which this step originally specified, cannot verify an API key at all:
altool answers `list-providers does not support APIKey authentication`. See correction 13.

- [x] **Step 6: Rehearse the script again, now that the certificates exist**

```sh
MASCOT_MAS_ALLOW_PRIVATE_API=1 tools/release-mas.sh
```

Expected: all the way through to a signed `.pkg` and a clean `altool --validate-app`. A
validation failure here is worth more than anything else in this phase: it names exactly what App
Review's automated checks will object to, for free, before a build number is spent on it.

- [x] **Step 7: Commit**

```sh
git add docs/app-store.md docs/notarization.md README.md
git commit -m "Document the one-time App Store setup"
```

---

# Phase 6: the listing and the submission

## Task 17: The App Store Connect metadata, written down before it is typed in

Every field, with the actual copy, so the listing is reviewable in git rather than only in a web
form. The 4.2 argument lives here.

**Files:**
- Create: `docs/app-store-listing.md`

- [ ] **Step 1: Write it**

Create `docs/app-store-listing.md`:

````markdown
# App Store Connect listing: Momentum Mascot

Every field, so the listing is reviewable here rather than only in a web form. Update this
file when the listing changes.

## Basics

| Field | Value |
|---|---|
| Name | Momentum Mascot |
| Subtitle | A pixel pet for side projects |
| Price | Free |
| Primary category | Developer Tools |
| Secondary category | none |
| Bundle ID | dev.keepgoing.momentum-mascot |
| SKU | momentum-mascot-1 |
| Copyright | 2026 Hoa Trinh |
| Support URL | https://keepgoing.dev |
| Marketing URL | https://keepgoing.dev |
| Privacy Policy URL | https://keepgoing.dev/privacy |

**Developer Tools is doing review work, not decoration.** Guideline 4.2 says "If your app
is not particularly useful, unique, or 'app-like,' it doesn't belong on the App Store", and
an ambient desktop pet reads as a toy in Utilities and as a tool in Developer Tools. The
bundle agrees: `tauri.conf.json` sets `"category": "DeveloperTool"`, which lands as
`LSApplicationCategoryType = public.app-category.developer-tools`. If one of the two ever
changes, change both.

## Keywords

```
git,commit,pixel,pet,mascot,desktop,menubar,side project,momentum,reflog
```

## Description

```
Momentum Mascot is a retro pixel character who lives in a tiny room on your desktop and
reflects how your side projects are going.

Point it at the git repositories you care about. It reads exactly one thing from each: when
you last actually committed. Not messages, not diffs, not branch names. Commit something
and the character is at their desk. Take a few days off and they doze, then sleep, still
holding your place. Come back after a long silence and they leap out of bed.

The mascot never dies. It waits.

WHAT YOU GET

- A 64x64 desktop pet in the corner of your screen, visible over fullscreen apps, draggable
  to any corner.
- A full animated room in a popover from the menu bar, with the character and a line of
  copy that never scolds you.
- Three characters to choose from.
- A 1200x630 share card copied to your clipboard, carrying the room and the mood and
  nothing that identifies a project.
- Operating mode, for projects that run without commits: they keep their place in your list
  and the mascot ignores them.

WHAT IT IS NOT

It is not a productivity tool. There are no streaks, no scores, no notifications, no
leaderboards, and nothing in it will ever tell you how long it has been since you last
committed. It is a small companion for people with demanding day jobs who go through long
stretches where nothing gets committed because life is happening.

PRIVACY

No network requests. No accounts, no sign-in, no telemetry, no cloud, no sync. Everything
lives in one JSON file inside the app's own container, which you can read or delete.

Art by LimeZu (limezu.itch.io). Type: Departure Mono, OFL 1.1.
```

## Privacy answers: Data Not Collected, every category

Apple: "'Collect' refers to transmitting data off the device", and "data that is processed
only on device is not 'collected' and does not need to be disclosed." Reading the user's
filesystem is emphatically not collection.

The one caveat to keep in view: "if you derive anything from that data and send it off
device, the resulting data should be considered separately." The share card puts derived
data on the **clipboard**, not off device, so it stays clear. If a future release ever
posts a card anywhere, this answer changes.

## Review notes

Paste verbatim into App Review Notes. Without it the app looks broken to a reviewer who
never adds a folder, which is a 2.1 rejection: "We will reject incomplete app bundles."

```
This app has no Dock icon and no main window by design. It is a menu bar app (LSUIElement),
and it shows nothing until you add a repository. To review it:

1. Look for the small pixel character in the bottom-right corner of the screen. That is the
   desktop pet. It appears on launch. You can drag it to any corner.
2. Click the pixel icon in the menu bar, at the top right of the screen, or click the
   character itself. Either opens the popover: an animated room with the character in it.
3. Click "Add Project" and choose any folder that contains a git repository. If you need
   one, any checkout of any public repository works, and so does a folder where you have run
   "git init" followed by one commit.
4. A repository committed to today shows the "awake" state immediately: the character is at
   their desk and the project row shows how long ago that commit was.
5. The character's state is derived from time since the newest commit across the projects
   you added: awake under a day, dozing after a day, asleep after three days. A commit made
   after a long silence triggers a one-off "comeback" celebration.
6. "Share Status" copies a 1200x630 image to the clipboard. Paste it anywhere to see it.

The app makes no network requests of any kind and has no accounts. It reads only the reflog
and file modification times of the folders you add through the picker, and stores its state
in one JSON file in its own container.

Category: Developer Tools. The app's audience is developers with side projects, and the
signal it reads is a git reflog.
```

## Screenshots

2560x1600, five of them, in this order:

1. The pet on a desktop, in the bottom-right corner, over a real wallpaper. The product's face.
2. The popover open with the room in the **awake** state, showing two or three tracked projects.
3. The popover in **dozing**.
4. The popover in **comeback**, which is the moment the whole product exists for.
5. The share card at full size.

`KEEPGOING_CLOCK_SCALE` and `KEEPGOING_MASCOT_STATE` (debug builds only) drive the states
for shots 2 to 4 without waiting three days. `tools/drive-states.sh` already exists for
this.

Screenshots are derived LimeZu art, so they are covered by the licence check in
`docs/app-store-licence-check.md`: uploading them to the listing is presentation of the
app, not redistribution of the asset pack.

## If review pushes back on 4.2

The answer is the review notes and the category, not new features. Appeal explaining that
an ambient status indicator for developers is the whole product and that its restraint is
deliberate. Do not add scope to satisfy a guess about App Review's appetite.
````

- [ ] **Step 2: Take the screenshots**

```sh
tools/drive-states.sh
```

Capture the five shots listed above at 2560x1600. Check each one at 100%: pixel art that has been
resampled by a screenshot tool reads as blurry and undermines the one thing the listing is
selling.

- [ ] **Step 3: Fill in App Store Connect**

Type every field from the document into the listing. Upload the screenshots. Answer the privacy
questionnaire as "Data Not Collected" in every category.

- [ ] **Step 4: Commit**

```sh
git add docs/app-store-listing.md
git commit -m "Write the App Store listing metadata down"
```

## Task 18: Submit

- [ ] **Step 1: Confirm the gate is actually clean**

```sh
tools/release-mas.sh
```

Expected: `private API check: clean`. If it is not, the deferred section 4 work is not done and
this task cannot proceed. Do not set `MASCOT_MAS_ALLOW_PRIVATE_API` here; it refuses to upload
anyway, and the refusal is the feature.

Expected also: a clean `altool --validate-app`.

- [ ] **Step 2: Run the full manual test list one more time on the exact build being submitted**

From spec section 9, against the signed sandboxed build:

- Sandbox persistence: add a repository, quit, relaunch, still readable. (Task 13.)
- `strings -a <binary> | grep -cE 'drawsBackground|fullScreenEnabled'` returns 0.
- The pet is visible and non-hostile over a fullscreen app. This is the regression the NSPanel decision was won against, and it is the one this whole project could most easily have broken.
- The pet appears at all, drags to all four corners, glides, and a click opens the popover.
- The popover works: add a project, cycle a character, toggle operating, untrack, copy the share card, dismiss with Escape.
- A tracked `git worktree` checkout shows the specific message, and an ordinary clone is unaffected.
- The privacy link is present in the popover and opens the hosted page.
- The popover's rounded corners read correctly on a light and a dark desktop.
- Pixel art stays crisp when the pet is dragged to a display of a different density.

- [ ] **Step 3: Upload**

```sh
tools/release-mas.sh --upload
```

Expected: the upload completes and names a delivery id. Then, in App Store Connect, wait for
processing, attach the build to the version, and Submit for Review.

- [ ] **Step 4: Record what happened**

Append to `docs/app-store-listing.md`:

```markdown
## Submission log

| Date | Version | Build | Result |
|---|---|---|---|
| <date> | 0.3.1 | <n> | submitted |
```

Add a row for every outcome, including rejections and what they cited. That log is the actual
deliverable of this project: the point was to learn the process end to end, and a rejection
reason is worth more than a clean pass.

- [ ] **Step 5: Commit**

```sh
git add docs/app-store-listing.md
git commit -m "Log the first App Store submission"
```

## Task 19: Update spec-v2.md, which currently records this as impossible

Spec section 12: both places get updated to point here rather than being deleted, so the
reasoning trail stays intact.

**Files:**
- Modify: `docs/spec-v2.md` section 10.3 and the risk table entry at `docs/spec-v2.md:705`

- [ ] **Step 1: Find the two places**

```sh
grep -n "App Store\|opaque square" docs/spec-v2.md
```

- [ ] **Step 2: Amend section 10.3**

Leave the original reasoning and append:

```markdown
> **Superseded, 2026-08-22.** This trade was correct while direct distribution was the only
> target. It is no longer the plan: see
> `docs/superpowers/specs/2026-08-22-mac-app-store-design.md`.
>
> The prediction above was closer to right than the first draft of that document was. The
> pet does **not** have to be an opaque square. But the reason the first draft gave, that
> window transparency is public API, does not hold through Tauri:
> `tauri-runtime-wry`'s `window.transparent(...)` and `WindowBuilder::transparent()` are
> both gated on the `macos-private-api` feature, and with it off the only complaint is an
> `eprintln!` gated on `debug_assertions`. So the pet keeps its alpha only because the app
> makes the `setOpaque:` and `setBackgroundColor:` calls itself, in `src-tauri/src/appkit.rs`.
>
> Two private API strings also remain in the shipped binary and are not removable without
> forking wry and tao: `allowsPictureInPictureMediaPlayback` and `_wantsKeyDownForEvent`.
> Neither is reachable from this codebase. Precedent is the whole justification for shipping
> them.
```

- [ ] **Step 3: Amend the risk table entry**

Replace the App Store ineligibility row's response cell with:

```markdown
Reversed 2026-08-22. Eligible, via App Sandbox at signing time plus security-scoped bookmarks. See `docs/superpowers/specs/2026-08-22-mac-app-store-design.md` and `docs/app-store.md`.
```

- [ ] **Step 4: Commit**

```sh
git add docs/spec-v2.md
git commit -m "Point spec-v2's App Store trade at the design that reversed it"
```

---

# Self-review notes

**Spec coverage.** Section 1 goals: Task 17 (listing), Tasks 5 to 12 (no loss of surface).
Section 2.1 and 2.2, the private strings: Task 3 measures them, Task 15 gates on the two
removable ones and names the two that stay. Section 2.3, the absent sandbox: Task 2. Section 2.4,
repositories unreadable after relaunch: Tasks 5 to 9, proven in Task 13. Section 2.5, the wrong
certificate: Task 16. Section 3, one binary two signings: Task 15's single `codesign` call with a
different entitlements file, and `release.sh` untouched (verified in Task 15 step 5). Section 4:
deferred by decision, gated in Task 15. Section 4.0: Task 3. Section 5.1: Tasks 4 and 11. Section
5.2: Task 2. Section 5.3: Task 7. Section 6: Tasks 5, 6, 8, 9, 13. Section 6.4, what the
entitlement covers: Task 13 step 5 exercises it. Section 7.1, the git shellout: no task, and that
is correct; the behaviour is unchanged and the corrected reason is recorded in the spec, not in
code. Section 7.2: Task 10. Section 8.1: Task 16. Section 8.2: Task 15. Section 8.3: Task 17.
Section 8.4: Task 1. Section 9: the automated tests live in Tasks 5 to 10 and the manual list in
Tasks 13 and 18. Section 10, the order of work: this plan's phase order, minus the deferred pet.
Section 11's risks: each one is either a task or a gate, except the two about the native pet,
which travel with the deferred plan. Section 12: Task 19.

**Deliberately not covered.** Spec section 9's sprite-cycle test and everything else about the
12-frame animation belongs to the deferred section 4 plan, because it tests code that does not
exist unless the native pet is built. Spec 4.6's `bundle.resources` and the `frontendDist` sprite
duplication are the same: pet-only.

**One thing the spec asks for that this plan changes.** Spec 7.2 suggests the new error message
"That worktree's git folder is outside the folder you picked." Task 10 uses "That worktree's git
folder isn't reachable from here." because the same code path also fires for a genuinely deleted
worktree git dir, where the spec's wording would be false. The spec said "Something like", so
this is within its intent rather than against it.
