# Mac App Store eligibility and first submission

**Date:** 2026-08-22
**Status:** approved design, not yet planned
**Reverses:** `docs/spec-v2.md` section 10.3, which accepted App Store ineligibility as a
permanent trade. That decision was correct when direct distribution was the only target. The
goal has changed: the Developer Program membership is already paid for, and the point of this
work is to complete one real submission end to end so that later apps land on the store with
the process already learned.

**Preserves:** the NSPanel conversion recorded as "Fullscreen gate passed". That fix is what
makes the pet visible and non-hostile over fullscreen apps, and no NSWindow level could do it.
The native pet keeps the panel and changes only what draws inside it.

---

## 1. Goal and non-goals

The goal is an approved Mac App Store listing for Momentum Mascot, free, with no loss of
product surface: the desktop pet stays a non-rectangular character sitting on the desktop, the
popover stays the animated room, and the share card still copies.

Non-goals, explicitly:

- Retiring the notarized DMG. Both channels ship from one codebase.
- Adding features to satisfy a guess about App Review's appetite. If review pushes on 4.2
  minimum functionality, the answer is the review-notes response in section 8, not new scope.
- iOS, Catalyst, or any second platform.
- A login item. It is a real quality gap, tracked separately, and not a submission blocker.

## 2. What actually blocks the store

Four blockers and one accepted risk. Each is stated with the evidence that established it,
because two of the five were wrong on first inspection.

### 2.1 The private API is one KVC key, not the transparency model

`Cargo.toml:16` enables tauri's `macos-private-api`, which `tauri-2.11.5/Cargo.toml:108-110`
forwards to `tauri-runtime-wry`. What that ultimately buys is one thing:
`wry-0.55.1/src/wkwebview/mod.rs:376`, `:382`, and `:973` call
`setValue_forKey(no, ns_string!("drawsBackground"))` on `WKWebViewConfiguration` and on the
`WKWebView` instance. All three sites carry the comment `// NOTE: Private API`.

Confirmed present in the shipped universal binary:

```sh
strings -a "src-tauri/target/universal-apple-darwin/release/bundle/macos/Momentum Mascot.app/Contents/MacOS/momentum-mascot" \
  | grep -c drawsBackground
# 2
```

Window-level transparency is **not** private. `tao-0.35.3/src/platform_impl/macos/window.rs:545`
calls `setOpaque(false)` and `:560` calls `setBackgroundColor`, both public AppKit. Only the
webview's see-through background is private. That distinction is the whole reason the design in
section 4 works: a window can be transparent legally, so anything that is not a webview can sit
in a transparent window and keep its alpha.

### 2.2 One private selector is unfixable and accepted

`tao-0.35.3/src/platform_impl/macos/view.rs:244` registers `sel!(_wantsKeyDownForEvent:)` on
tao's view class **unconditionally**, with no feature gate. It is present in the binary
(`grep -c _wantsKeyDownForEvent` returns 2) and will remain after every change in this document.

This is accepted rather than fixed. Removing it means forking tao. Tauri applications do ship
on the Mac App Store today, which is direct evidence that this symbol does not trip review.
Recorded here so that a future reader does not mistake it for an oversight, and so that if a
rejection ever cites private API use, this is the first thing to look at.

### 2.3 App Sandbox is absent

`src-tauri/Entitlements.plist` holds exactly one key, `com.apple.security.cs.allow-jit`. The
sandbox is mandatory for the store. Sections 5 and 6 cover what it breaks.

### 2.4 Tracked repositories stop being readable after relaunch

Under sandbox, a folder chosen through the picker grants access for that launch only. The app
stores plain paths (`store::Project::path`) and re-resolves them at load
(`momentum.rs:73 resolve_paths`), so on the second launch every tracked project would report
`RepoError::Missing` and the mood would be built from nothing. Section 6.

### 2.5 The certificate is the wrong type

`docs/notarization.md:32` already says it: Developer ID is for distribution outside the store.
`security find-identity -v -p codesigning` shows exactly one identity,
`Developer ID Application: Hoa Trinh (3LM6674AC2)`. The store needs an **Apple Distribution**
certificate for the app and a **Mac Installer Distribution** certificate for the package, plus
a Mac App Store provisioning profile embedded in the bundle. Notarization becomes irrelevant:
store builds are not notarized. None of `tools/release.sh` carries over unchanged.

## 3. Distribution shape: one binary, two signings

The same universal binary serves both channels. App Sandbox is applied at **signing** time
through a different entitlements file, not at compile time, so no cargo feature split and no
conditional compilation is needed for the sandbox itself.

**The DMG build stays unsandboxed.** Only the store build gets the sandbox entitlement. This is
not a compromise, it is the point of applying the sandbox at signing time: the two channels
differ by one entitlements file and share every line of code.

An earlier draft of this section sandboxed both channels for uniformity and accepted a state
file migration as the cost. That was wrong, and section 5.2 records the measurement that proves
it: a sandboxed process cannot read `~/.keepgoing/mascot/state.json` at all, so the migration
it depended on cannot be written. Leaving the DMG unsandboxed removes the need for one
entirely. Existing direct-download users keep their state file exactly where it is, the
cross-tool design intent at `store.rs:62-67` survives in the channel where it matters, and
store users are new users by definition so they have nothing to migrate.

The alternative, a `mas` cargo feature producing two binaries, is still rejected: it doubles
what has to be tested for a difference no user perceives.

Bookmarks (section 6) are created and resolved in both channels. Unsandboxed,
`startAccessingSecurityScopedResource` succeeds and is a no-op, so one code path serves both
without a `cfg`.

## 4. The pet becomes native

This is the change that removes `drawsBackground`, and it is the largest piece of work.

### 4.1 Why native rather than opaque

A 64x64 opaque square in a screen corner is not a character sitting on a desktop, it is a
tile. The product's identity is the character, so the alpha is not decoration. Since a *window*
can be transparent with public API and only a *webview* cannot, the fix is to stop drawing the
pet in a webview.

### 4.2 What is built

New module `src-tauri/src/pet_view.rs`. An `NSView` subclass, hosted as the content view of the
**existing** NSPanel that `pet.rs:254-290` already produces. The panel, its style mask, its
`setFloatingPanel`, `setBecomesKeyOnlyIfNeeded`, its collection behavior and its level are all
untouched. That code is the fullscreen fix and it is load-bearing.

Sprite animation. Each mood is a horizontal strip of 12 frames of 32x32
(`src/pet.html:31-33`). Native equivalent: set the layer's `contents` to the strip image and
run a `CAKeyframeAnimation` on `contentsRect` with `calculationMode = .discrete` and 12 values,
repeating forever. Per-mood durations are carried over exactly from `src/pet.html:47-77`:

| Mood | Duration |
|---|---|
| awake | 4s |
| dozing | 6s |
| asleep | 6s |
| comeback | 1.5s |
| run | 0.75s |

Facing. `src/pet.html:79-81` flips the run strip with `transform: scaleX(-1)`. Native
equivalent: set the layer's `transform` to `CATransform3DMakeScale(-1, 1, 1)`. The strip is
centred in its cell, so the flip turns the character in place, exactly as the CSS comment says.

Scaling. `src/pet.js:26-30` measures the window and floors the cell to a whole multiple of 32
so a 1.5x character is never drawn blurry, and so a mis-sized window shows a small pet rather
than a cropped one. That rule carries over: the view computes the same floor from its own
bounds, and the layer gets `magnificationFilter = .nearest` so pixel art stays crisp.

### 4.3 Interaction

`src/pet.js:83-125` becomes `mouseDown` / `mouseDragged` / `mouseUp` on the view, keeping the
4pt threshold that separates a click from a drag, keeping `cancel_glide` on mouse down, and
keeping the `busy` flag that stops a mood tick from walking back the run sprite mid-glide.

The port makes this code **simpler**, and the reason is worth recording. The long comment at
`src/pet.js:66-78` explains that the drag has to listen on `window` rather than the element,
accumulate `movementX`/`movementY` rather than read window-relative coordinates, and avoid
`setPointerCapture` entirely, all because the window moves out from under a webview's cursor
and WebKit drops captured pointer events. None of that applies to AppKit: a view that received
a `mouseDown` receives the whole drag, and `NSEvent.deltaX` is screen-space by construction.
The `devicePixelRatio` scaling also disappears, because the view works in backing coordinates.

`toggle_popover`, `cancel_glide` and `snap_pet` are called directly as Rust functions instead
of through `invoke`. The glide itself stays in the backend where it already lives, and
`glide-done` stops being an event and becomes a direct call back into the view.

Right-click and native drag suppression (`src/pet.js:139-140`) become a `menu(for:)` override
returning nil. There is no HTML5 drag to suppress.

### 4.4 What is deleted

`src/pet.html` and `src/pet.js`, and the `pet` entry in `tauri.conf.json` (section 4.6 covers
what replaces it). Pet sprites move from webview-served assets under `src/assets/pet/` to bundle
`resources`, since nothing serves them over the custom protocol any more. The popover keeps
serving its room art the way it does today.

### 4.5 Mood delivery

`app.rs:124` emits `MOOD_EVENT` with `app.emit`, which broadcasts to every window. The popover
keeps receiving it unchanged. The native pet gets a direct setter call instead of a listener.

`pet.rs:242` emits `GLIDE_DONE_EVENT` to the pet window specifically. That becomes a direct call
into the native view, and the event constant disappears.

### 4.6 The pet window stops being a webview window

An earlier draft said only that the pet "stops being a webview window" without saying what
replaces it. This is that answer, and it is larger than a config edit.

Tauri 2 separates windows from webviews: `WindowBuilder::build()` returns a plain `Window<R>`
with no webview at all (`tauri-2.11.5/src/window/mod.rs:352`). That is what the pet becomes.
Tauri still owns the window's creation, label, and geometry, so everything in `pet.rs` about
position, `outer_position`, and the glide keeps working, and the NSPanel reclass at
`pet.rs:254-290` still reaches the same `ns_window()`. Only the content view changes.

Four consequences the spec previously missed:

1. **The window must be built in Rust, not declared in config.** The `app.windows` array in
   `tauri.conf.json` creates *webview* windows, so the `pet` entry at `tauri.conf.json:28` is
   deleted and the window is constructed with `WindowBuilder` at startup instead. The popover
   entry stays as it is.

2. **`get_webview_window` becomes `get_window`** at `pet.rs:50` and `commands.rs:33`. Both
   currently look the pet up as a webview window and would return `None` forever otherwise,
   which would fail silently: `pet.rs:50` early-returns and `commands.rs:33` is a `?` on an
   `Option`. A native pet that simply never appears, with nothing logged, is the likely first
   symptom of getting this wrong.

3. **`capabilities/default.json:5` drops `"pet"`** from its `windows` list, since capabilities
   gate webview IPC and there is no longer a webview to gate. Two of its permissions exist only
   for the pet's drag and become dead: `core:window:allow-set-position` (`pet.js:117`) and
   `core:window:allow-outer-position` (`pet.js:88`).

4. **Several permissions in that file look already dead today**, and the audit of point 3 is the
   moment to check. The popover's only direct window API call is `setSize`
   (`popover.js:131`), so `core:window:allow-set-size` stays. But `core:window:allow-hide` is
   unused (the popover calls the Rust command `hide_popover`), `core:window:allow-start-dragging`
   appears in neither JS file, `dialog:allow-open` is unused because the picker is opened from
   Rust at `commands.rs:76`, and `clipboard-manager:allow-write-image` is unused because the
   clipboard write happens in Rust in `copy.rs`. Capabilities gate webview IPC and not Rust-side
   plugin calls, so all four are probably removable. **Verify by removing them and confirming
   the popover still works**, rather than reasoning about it: the capability file's own
   description calls itself "deliberately short", and it should be true.

The net effect is that the pet contributes zero IPC surface, and the capability file shrinks to
what the popover actually uses.

## 5. Popover, sandbox, and state

### 5.1 The popover keeps its webview

Drop `transparent: true` from the popover window in `tauri.conf.json`. The room art fills the
whole 352x540 surface, so the popover never actually needed a see-through webview. What it
needed was rounded corners, and those come from `layer.cornerRadius` plus
`masksToBounds = true` on the container view, both public. The window keeps tao's public
`setOpaque(false)` so the rounded corners are not boxed by an opaque rectangle.

Then `macos-private-api` comes out of `Cargo.toml:16` and `macOSPrivateApi` out of
`tauri.conf.json`. Acceptance test:

```sh
strings -a <binary> | grep -c drawsBackground   # must be 0
```

### 5.2 Entitlements and the state file

New `src-tauri/Entitlements.mas.plist`:

| Key | Why |
|---|---|
| `com.apple.security.app-sandbox` | Mandatory for the store |
| `com.apple.security.files.user-selected.read-only` | The folder picker is the only way repos enter |
| `com.apple.security.cs.allow-jit` | The popover webview still runs JavaScriptCore |

No network entitlement of any kind. There is no network layer, and asserting nothing is the
honest label.

State path. **Measured, not assumed.** A minimal bundle signed with nothing but
`com.apple.security.app-sandbox` reports:

```
getenv(HOME)             = /Users/kyle/Library/Containers/dev.keepgoing.homeprobe/Data
NSHomeDirectory()        = /Users/kyle/Library/Containers/dev.keepgoing.homeprobe/Data
APP_SANDBOX_CONTAINER_ID = dev.keepgoing.homeprobe
```

So the redirection applies to the raw environment variable, not only to `NSHomeDirectory()`,
which means Rust's `std::env::var_os("HOME")` at `store.rs:87` sees it too.
`store::default_path()` resolves to
`~/Library/Containers/dev.keepgoing.momentum-mascot/Data/.keepgoing/mascot/state.json` in the
store build and stays at `~/.keepgoing/mascot/state.json` in the DMG build, with **no code
change and no `cfg`**. The path simply follows the entitlement.

`APP_SANDBOX_CONTAINER_ID` is also set, which is the cheapest available runtime sandbox
detection if anything ever needs to branch on it. Nothing in this design does.

No migration, and it could not be written if one were wanted. The same probe, signed the same
way, cannot reach the old file:

```
getpwuid->pw_dir     = /Users/kyle          <- real home is still discoverable
read real state.json -> denied ("you don't have permission to view it")
list ~/.keepgoing    -> DENIED
```

The path is discoverable through `getpwuid` but unreadable, so a sandboxed build has no way to
import an existing user's project list. This is precisely why section 3 leaves the DMG build
unsandboxed: there is nothing to migrate, because the channel that has existing users is the
channel that keeps its state file where it always was.

## 6. Security-scoped bookmarks

### 6.1 Schema

`store::Project` gains `bookmark: Option<String>`, the base64 of NSURL bookmark data. The JSON
reader is tolerant of unknown and missing fields by contract, so old state files load unchanged
and nothing needs a schema version bump for readers. `SCHEMA_VERSION` still moves to `3.1` for
writers, because a file written with bookmarks is meaningfully different from one without.

### 6.2 New module

`src-tauri/src/scoped.rs`, wrapping three NSURL calls:

- `create(path) -> Option<String>` via `bookmarkDataWithOptions:` with
  `NSURLBookmarkCreationWithSecurityScope`.
- `resolve(bookmark) -> Option<(PathBuf, ScopedAccess)>` via
  `URLByResolvingBookmarkData:options:` with `NSURLBookmarkResolutionWithSecurityScope`, then
  `startAccessingSecurityScopedResource`.
- `ScopedAccess`, a guard calling `stopAccessingSecurityScopedResource` on `Drop`.

Bookmarks go stale when a folder moves. `resolve` reports staleness so the caller can re-create
the bookmark from the resolved URL while access is held, which repairs the entry without
re-prompting the user.

### 6.3 Integration points

Two, and only two.

`commands.rs:71 add_project` creates the bookmark immediately after the picker returns, while
access is granted. If creation fails the project is still added with `bookmark: None`, which
degrades to today's behavior: it works this launch and reports unavailable on the next.

`momentum.rs:73 resolve_paths()` is the single choke point where every project's path is
re-resolved on load. It starts access for each project before calling `repo::resolve`, and
holds the guards in a `HashMap<String, ScopedAccess>` beside `git_dirs` and `work_trees` for
the process lifetime. A project whose bookmark is missing or unresolvable keeps the existing
"unavailable" presentation from `ProjectRow::available`, which already exists precisely so that
a disconnected drive does not erase the list.

The watcher needs no change. FSEvents registration inside a held scope works, and
`watcher.rs` receives already-resolved paths.

## 7. Deliberately left broken

`repo.rs:87 head_commit_time` shells out to `git` and cannot do so under sandbox. It stays.

Its own doc comment at `repo.rs:74-84` already commits to the degradation: this is the fallback
that runs only after the cheap reflog read has already failed, and "if `git` is not on `PATH`
the worst case is a slightly stale timestamp, never a false comeback." Under sandbox the spawn
fails, `output()` returns `Err`, and `ok()?` yields `None`, which is a path the caller already
handles. The sandbox turns a rare fallback into a never-fires fallback, and the failure mode is
the one the module was designed to tolerate.

## 8. Signing and submission

### 8.1 Certificates and identifiers, one time

1. Register App ID `dev.keepgoing.momentum-mascot` in the developer portal.
2. Create an **Apple Distribution** certificate (signs the .app).
3. Create a **Mac Installer Distribution** certificate (signs the .pkg).
4. Create a **Mac App Store** provisioning profile for the App ID, download it, and place it in
   the bundle as `Contents/embedded.provisionprofile`.
5. Create the app record in App Store Connect.

`tools/.release-env` gains the two new identity names alongside `APPLE_SIGNING_IDENTITY`. The
existing four keys stay, because the DMG channel still needs them.

### 8.2 `tools/release-mas.sh`

A sibling to `release.sh`, not a modification of it. The DMG path is working and automated and
must not be destabilised by this work. The MAS script:

1. Verifies both new certificates and the provisioning profile before doing anything.
2. Builds universal, the same as `release.sh`.
3. Copies `embedded.provisionprofile` into the bundle.
4. Signs the app with the Apple Distribution identity and `Entitlements.mas.plist`, inside out:
   nested frameworks and helpers first, bundle last.
5. Packages with `productbuild --component`, signed with the Mac Installer Distribution
   identity.
6. Uploads with `xcrun altool --upload-app` (or Transporter).
7. Does **not** notarize, and does **not** create a git tag or a GitHub release. Version
   bumping stays in `release.sh` so the two channels cannot disagree about what a version is.

### 8.3 App Store Connect metadata

- Price: free.
- Category: **Developer Tools**. Narrower reach than Utilities, but a better answer if review
  pushes on guideline 4.2, because the audience for a git-reflog mascot is developers.
- Privacy: "Data Not Collected", every category. This is literally true; there is no network
  layer.
- Privacy policy URL: required for all listings. Host it under the existing `site/`.
- Screenshots at 2560x1600: the pet on a desktop, the popover room in each of the four moods,
  and the share card.
- Review notes: the app shows nothing until a repository is added, so the notes must tell the
  reviewer to click the tray icon, add any folder containing a git repository, and say that a
  freshly committed repository shows the awake state immediately. Without this the app looks
  broken to a reviewer who never adds a folder, which is a 2.1 rejection.
- Copyright and attribution: the LimeZu Modern Interiors and Departure Mono credits already in
  `tauri.conf.json`'s `copyright` field carry into the listing.

### 8.4 Asset licence check, blocking

Before the first upload, confirm the LimeZu Modern Interiors licence permits distribution of
the compiled art through the Mac App Store. The README's claim, that the licence permits
shipping compiled into an application and forbids redistributing the assets, is very likely
sufficient for a free listing, and the store build redistributes no assets. Confirm it in the
licence text rather than by inference. If it fails, the whole submission stops and the design
in section 4 is wasted, so this is checked first.

## 9. Testing

Existing Rust tests are pure or tempdir-based and must keep passing untouched.

New automated coverage:

- `scoped.rs` round trip: create a bookmark for a temp directory, resolve it, assert the
  resolved path matches and that the guard drops cleanly.
- `store.rs`: a state file with a `bookmark` field loads, and a state file without one loads
  with `bookmark: None`, both already covered by the module's resilience test style.
- `store::default_path()` returns the `$HOME`-relative path unchanged. There is no migration
  path to test, and no sandbox-aware branch to test, because section 5.2 measured that the
  environment does the work.

Manual, and one of these is the test that proves the whole effort:

- **Sandbox persistence.** Sign locally with `Entitlements.mas.plist`, launch, add a repository,
  quit, relaunch, and confirm the repository is still readable and the mood is still built from
  it. If this passes, section 6 is done. If it fails, nothing else matters.
- `strings -a <binary> | grep -c drawsBackground` returns 0.
- `tools/drive-states.sh` still walks the four-state arc, re-verified visually because the pet
  is now native.
- The pet is still visible and non-hostile over a fullscreen app. This is the regression the
  NSPanel decision was won against, and the native rewrite touches the panel's content.
- The pet still drags to all four corners and glides, and a click still opens the popover. The
  `get_window` change in section 4.6 point 2 fails silently if missed, so "the pet appears at
  all" is itself a test.
- The popover still works with the narrowed `capabilities/default.json` from section 4.6 point
  4: add a project, cycle a character, toggle operating, untrack, copy the share card, and
  dismiss with Escape.
- The popover's rounded corners read correctly on a light and a dark desktop.

## 10. Order of work

**Phase 1, throwaway probe.** On a branch, remove `macos-private-api` and `transparent: true`
from both windows and look at what actually breaks. This costs an afternoon and it validates
section 5.1's claim that the popover only needs corner rounding. Its code is thrown away even
if it appears to work.

**Phase 2, asset licence check** (section 8.4). Blocking, cheap, and it can invalidate
everything after it.

**Phase 3, native pet** (section 4). The largest piece. Ends when the fullscreen, drag, glide
and click behaviours all match the webview pet.

**Phase 4, sandbox and bookmarks** (sections 5 and 6). Ends at the sandbox persistence test.

**Phase 5, certificates and `release-mas.sh`** (section 8.1, 8.2).

**Phase 6, listing and submission** (section 8.3). Then wait, and learn what review says.

## 11. Risks

| Risk | Likelihood | Response |
|---|---|---|
| Guideline 4.2, minimum functionality | Real | Developer Tools category, review notes, and a listing that describes an ambient desktop pet rather than a productivity tool. If rejected, the response is an appeal explaining the category, not new features. |
| `_wantsKeyDownForEvent:` cited as private API | Low | Precedent: Tauri apps ship on the store. If cited, the options are a tao fork or abandoning the store. |
| ~~`$HOME` is not redirected under sandbox~~ | **Closed** | Measured in section 5.2. It is redirected, for `getenv` as well as `NSHomeDirectory()`. No code change needed. |
| Native pet regresses the fullscreen fix | Medium | The panel code is untouched by construction, and the fullscreen behaviour is an explicit manual test. |
| LimeZu licence forbids store distribution | Low | Checked in phase 2, before any code is written. |
| Sprite animation reads differently native than in CSS | Medium | Durations and the whole-multiple scaling rule are carried over as explicit constants, and `drive-states.sh` compares the arc visually. |

## 12. Consequences for `docs/spec-v2.md`

Section 10.3 and the risk table entry at `spec-v2.md:705` record App Store ineligibility as an
accepted trade with the note that "if that ever becomes a target, the pet has to be an opaque
square, which is a design decision rather than a bug." That prediction turned out to be wrong
in a useful way: the pet does not have to be an opaque square, because window transparency is
public and only webview transparency is private. Both places get updated to point here rather
than being deleted, so the reasoning trail stays intact.
