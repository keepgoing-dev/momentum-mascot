# Mac App Store eligibility and first submission

**Date:** 2026-08-22
**Status:** reworked after adversarial audit. Safe to plan from.
**Reverses:** `docs/spec-v2.md` section 10.3, which accepted App Store ineligibility as a
permanent trade. That decision was correct when direct distribution was the only target. The
goal has changed: the Developer Program membership is already paid for, and the point of this
work is to complete one real submission end to end so that later apps land on the store with
the process already learned.

**Preserves:** the NSPanel conversion recorded as "Fullscreen gate passed". That fix is what
makes the pet visible and non-hostile over fullscreen apps, and no NSWindow level could do it.
Nothing here replaces the panel or its content view.

**Revision note.** An earlier draft of this document was wrong in three ways that mattered, and
the corrections are kept visible rather than tidied away, because each one is a trap a reader
would otherwise re-enter. In order of severity: section 4.1's premise, that a window can be
transparent while only a webview cannot, is **true of AppKit and false of Tauri** (section 4.1);
the private-API surface is **three strings, not one** (section 2.1 and 2.2); and the state file
migration this spec designed **could not have been written at all** (section 5.2). All three were
found by measurement, two of them after the design was already approved.

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

### 2.1 The removable private APIs: two strings, not one

`Cargo.toml:16` enables tauri's `macos-private-api`. `tauri-runtime-wry-2.11.4/Cargo.toml:49-53`
expands it to `["wry/fullscreen", "wry/transparent", ...]`, and `tauri-2.11.5/src/lib.rs:31`
states the scope upstream: "currently the `transparent` window functionality **and the
`fullScreenEnabled` preference setting** to `true`." So it buys two private KVC keys, not one:

- `wry-0.55.1/src/wkwebview/mod.rs:376`, `:382`, `:973` set `drawsBackground`.
- `wry-0.55.1/src/wkwebview/mod.rs:386-388` sets `fullScreenEnabled`.

Both are marked `// NOTE: Private API` in wry's own source. Measured on the shipped universal
binary:

```
drawsBackground                        2
fullScreenEnabled                      2
allowsPictureInPictureMediaPlayback    2      <- see 2.2, not removable
developerExtrasEnabled                 0      <- devtools-gated, correctly absent
_wantsKeyDownForEvent                  2      <- see 2.2, not removable
```

**The acceptance test must cover both**, or it goes green on an incomplete removal:

```sh
strings -a <binary> | grep -cE 'drawsBackground|fullScreenEnabled'   # must be 0
```

An earlier draft tested `drawsBackground` alone, which would have passed while
`fullScreenEnabled` was still in the binary.

### 2.2 Two private strings survive every change in this document

Neither is reachable from this codebase, and both ship regardless:

- `_wantsKeyDownForEvent:` — `tao-0.35.3/src/platform_impl/macos/view.rs:255-257` registers it on
  tao's view class unconditionally, no feature gate.
- `allowsPictureInPictureMediaPlayback` — `wry-0.55.1/src/wkwebview/mod.rs:343-347` sets this
  private KVC key on `WKPreferences` **behind no feature gate at all**.

The second one deserves emphasis, because it is the same *category* of thing that all of section
4 exists to remove: a private KVC key on a WebKit object. Removing it means forking wry. So the
honest statement of this plan's private-API position is: the work in section 4 removes the two
strings that are removable, and ships the two that are not.

The justification for shipping them is precedent alone: Tauri applications are on the Mac App
Store today, and they all carry both strings. That precedent is doing more work than the earlier
draft admitted, and section 11 records it as the risk it is. If a rejection ever cites private
API use, this section is where to start.

### 2.3 App Sandbox is absent

`src-tauri/Entitlements.plist` holds exactly one key, `com.apple.security.cs.allow-jit`. The
sandbox is mandatory for the store. Sections 5 and 6 cover what it breaks.

### 2.4 Tracked repositories stop being readable after relaunch

Under sandbox, a folder chosen through the picker grants access for that launch only. The app
stores plain paths (`store::Project::path`) and re-resolves them at load
(`momentum.rs:74 resolve_paths`), so on the second launch every tracked project would report
`RepoError::Missing` and the mood would be built from nothing. Section 6.

### 2.5 The certificate is the wrong type

`docs/notarization.md:32` already says it: Developer ID is for distribution outside the store.
`security find-identity -v -p codesigning` shows exactly one identity,
`Developer ID Application: Hoa Trinh (3LM6674AC2)`. Submitting with it yields App Store Connect
error 90034, "not signed using an Apple submission certificate". Section 8.1 has the correct
identities and their exact common names, which are **not** what the portal calls them.

## 3. Distribution shape: one binary, two signings

The same universal binary serves both channels. App Sandbox is applied at **signing** time
through a different entitlements file, not at compile time, so no cargo feature split and no
conditional compilation is needed for the sandbox itself.

**The DMG build stays unsandboxed.** Only the store build gets the sandbox entitlement. This is
not a compromise, it is the point of applying the sandbox at signing time: the two channels
differ by one entitlements file and share every line of code.

An earlier draft of this section sandboxed both channels for uniformity and accepted a state
file migration as the cost. That was wrong, and section 5.2 records the measurement that proves
it: a sandboxed process cannot read `~/.keepgoing/mascot/state.json` at all, so the migration it
depended on cannot be written. Leaving the DMG unsandboxed removes the need for one entirely.
Existing direct-download users keep their state file exactly where it is, the cross-tool design
intent at `store.rs:62-67` survives in the channel where it matters, and store users are new
users by definition so they have nothing to migrate.

The alternative, a `mas` cargo feature producing two binaries, is still rejected: it doubles
what has to be tested for a difference no user perceives.

Bookmarks (section 6) are created and resolved in both channels. Measured unsandboxed:

```
create WithSecurityScope  : OK (764 bytes)   <- byte-identical to the sandboxed result
resolve WithSecurityScope : <repo path>  stale=0
startAccessing (scoped)   : TRUE
```

So one code path serves both without a `cfg`. **One guard on that**, because the observed `TRUE`
is undocumented: Apple documents `startAccessingSecurityScopedResource` returning `false` for a
non-security-scoped URL. `scoped.rs` must therefore treat `false` as "fall back to using the
stored path directly", never as "drop the project". A hard failure there would break the DMG
channel on some future macOS that follows its own documentation.

## 4. The pet

This is the largest piece of work, and section 10 records plainly that it is also the piece that
teaches nothing about the App Store. Read 4.0 before planning any of it.

### 4.0 The probe that could delete this entire section

The earlier draft assumed the native rewrite was the only route. It may not be. Before phase 3
is planned, spend one hour on this:

Keep the pet as a webview. Drop `macos-private-api`. Then, by hand and with public API only, set
`setOpaque: NO` and `setBackgroundColor: NSColor.clearColor` on the panel, and set
`WKWebView.underPageBackgroundColor` to clear (public since macOS 12, and wry already calls it
at `wkwebview/mod.rs:441`). If the pet's alpha survives that, **sections 4.1 through 4.6 are
unnecessary** and the store costs nothing but the sandbox work.

The honest expectation is that it fails: wry's own comment at `wkwebview/mod.rs:429-431` implies
`underPageBackgroundColor` covers only the overscroll region, not the page's own backdrop. But it
is one hour against a multi-week rewrite, and phase 1's afternoon is already budgeted. It is the
highest-leverage hour in this plan.

### 4.1 Corrected premise: Tauri cannot make any window transparent without the private API

The earlier draft's central claim was that "a window can be transparent with public API and only
a webview cannot." That is true of AppKit and **false of Tauri**:

- `tauri-runtime-wry-2.11.4/src/lib.rs:880-883` — `window = window.transparent(config.transparent)`
  sits inside `#[cfg(any(not(target_os = "macos"), feature = "macos-private-api"))]`.
- `:884-893` — with the feature off, the only feedback is an `eprintln!` gated on
  `debug_assertions`. **Silent in release builds.**
- `tauri-2.11.5/src/window/mod.rs:909` — `WindowBuilder::transparent()` carries the same cfg, so
  even a hand-built webview-less window cannot set it.
- Therefore `tao-0.35.3/src/platform_impl/macos/window.rs:544-545`
  (`if win_attribs.transparent { ns_window.setOpaque(false) }`) and `:547-561`
  (`setBackgroundColor`) are **never reached**.

So dropping the feature makes *both* windows opaque, silently, in release. An implementer who
follows this section, writes a correct sprite view, and sees a 64x64 opaque square would
reasonably conclude the private API was load-bearing after all, at the end of the largest phase.

**The app makes these calls itself.** `pet::macos::apply` already `msg_send`s
`setLevel`, `setCollectionBehavior` and `setHidesOnDeactivate` at `pet.rs:284-292`. Two more join
them, both public AppKit: `setOpaque: NO` and `setBackgroundColor: NSColor.clearColor`. The same
two are needed on the popover window for section 5.1's rounded corners to read against the
desktop.

### 4.2 Why native rather than opaque

A 64x64 opaque square in a screen corner is not a character sitting on a desktop, it is a tile.
The product's identity is the character, so the alpha is not decoration. Given 4.1, the route to
it is: the app sets window transparency itself with public API, and the pet's content stops being
a webview so that nothing private is needed to see through it.

### 4.3 Where the sprite view goes, and where it must not go

**It must not replace the panel's content view.** `tao/window.rs:535-536` calls
`setContentView` *and* `setInitialFirstResponder`, and `:539` builds the IME input context on
that view. The class it installs (`tao/view.rs:222-258`) is where `mouseDown:`, `mouseDragged:`,
`scrollWheel:`, `frameDidChange:`, `cancelOperation:` and `acceptsFirstMouse:` all live.
`setContentView:` would throw all of that away, including the `frameDidChange` plumbing the
pet's sizing depends on.

The design instead:

- The pet window is built with `WindowBuilder` (section 4.6), so there is **no webview** in it
  and nothing to remove. Tao's view is the content view and stays that way.
- A new `NSView` subclass is added as a **subview** of tao's content view, sized to fill it.
  Hit-testing prefers subviews, so it receives the mouse events.
- That subview is **layer-hosting**: we assign its layer and own it. This matters for two
  reasons. First, writing `transform` on a view's *backing* layer is documented undefined
  behavior (AppKit Release Notes for macOS 10.13: if an app modifies a backing layer's
  `bounds`, `position`, `anchorPoint`, `transform`, `frame`, `masksToBounds` or `opaque`, "the
  behavior of the application is undefined"), and `NSView` exposes no `transform` cover
  property, so the horizontal flip has nowhere legal to live on a backing layer. Second,
  `NSView.wantsLayer` says "do not add subviews to a layer-hosting view" — our sprite view adds
  none, so hosting is available to it while it would not be available to tao's view.

### 4.4 The sprite animation, and the rule that is not guessable

Each mood is a horizontal strip of 12 frames of 32x32 (`src/pet.html:31-33`), stepped by CSS
`animation: walk 2s steps(12) infinite` (`pet.html:27`).

`contentsRect` is the right mechanism: unit coordinate space, animatable, `(i/12, 0, 1/12, 1)`
selects frame `i`, and `contentsGravity` defaults to `resize`, which stretches the selected cell
to fill the layer. `magnificationFilter` must be set to `.nearest`, because
`CALayer.magnificationFilter` documents its "default value of this property is `linear`" and
pixel art must not be smoothed.

**The N+1 keyTimes rule.** `CAKeyframeAnimation.keyTimes` documents that with
`calculationMode = .discrete`, "the array should have one more entry than appears in the values
array," and `CAAnimationCalculationMode.discrete` says each value/keyTime pair "represents the
value from the specified time until the next keyframe." CSS `steps(12)` is `floor(p*12)/12`,
i.e. exactly 12 plateaus of `D/12`. So:

```
values:   12 entries, frame 0 through frame 11
keyTimes: 13 entries, (0...12).map { Double($0) / 12.0 }
```

12 values with `nil` keyTimes yields **11 visible frames** of `D/11` each, with the 12th holding
for zero time. These strips are walk cycles, so a dropped frame reads as a limp. Apple documents
no `nil`-keyTimes fallback for discrete mode, only that "the timing of your animation might not
be what you expect."

Durations carry over from `pet.html:47-77` unchanged: awake 4s, dozing 6s, asleep 6s, comeback
1.5s, run 0.75s.

Scaling. `pet.js:26-30` floors the cell to a whole multiple of 32 so a 1.5x character is never
drawn blurry and a mis-sized window shows a small pet rather than a cropped one. That rule
carries over, computed from the subview's own bounds.

### 4.5 Interaction: what ports cleanly, what does not

`pet.js:83-125` becomes `mouseDown` / `mouseDragged` / `mouseUp`, keeping the 4pt threshold, the
`cancel_glide` on mouse down, and the `busy` flag that stops a mood tick from walking back the
run sprite mid-glide.

**Two of the earlier draft's three "it gets simpler" claims hold.** `NSEvent.deltaX`/`deltaY` are
documented valid for mouse-drag events, so accumulate-deltas ports legitimately. AppKit's
implicit mouse capture is real (a `mouseDown:` responder receives the whole drag through
`mouseUp:`), so `pet.js:70-75`'s window-level listener and its deliberate avoidance of
`setPointerCapture` genuinely disappear.

**One claim was false.** "The `devicePixelRatio` scaling also disappears, because the view works
in backing coordinates" is wrong: an `NSView`'s `bounds` is in **points**, not backing pixels,
and everything downstream is in Tauri physical pixels (`commands.rs:32`, `pet.rs:117`,
`pet.rs:186`, `pet.rs:237`). The `* scale` at `pet.js:105-107` does not vanish; it becomes
`backingScaleFactor` in the new view.

**One claim is sound inference, not documentation, and should say so.** Apple documents neither
units nor sign nor pointer-acceleration for mouse-event deltas. The sign is settled only by
Apple's own sample (Handling Mouse Events, Listing 4-4: `windowOrigin.y - [theEvent deltaY]`
against a y-up `NSWindow.frame.origin`), which establishes that `deltaY` on a drag is already
y-down, and therefore feeds Tauri's y-down `PhysicalPosition` **unnegated**.

**Considered and rejected:** `NSWindow.performDrag(with:)` is Apple's recommended way to drag a
window from a view, because it hands off to the Window Server and participates in space
switching. Wrong here: Apple notes "a mouse-up event may not get sent", and both the 4pt
click/drag discrimination and the corner snap need `mouseUp`. Recorded so a reader does not
wonder why the harder path was taken.

**Four obligations the webview handled for free:**

1. **`acceptsFirstMouse` must return `YES`.** Without it the pet stops responding to clicks
   entirely. `NSView.acceptsFirstMouse(for:)` "ignores event and returns false" by default, and
   a mouse-down in a non-key window "isn't sent to the NSView object over which the mouse click
   occurs." The panel is nonactivating (`pet.rs:277`), `becomesKeyOnlyIfNeeded` is YES
   (`pet.rs:279`), and the app is `ActivationPolicy::Accessory` (`main.rs:55`), so the panel is
   never key and **every** click is structurally first-mouse. This works today only because tao
   already handles it (`tao/view.rs:255-257` registers the selector, `:1148` returns YES). A new
   subclass inherits `false`. This is the single most likely way the rewrite presents as "the
   panel regressed", which is exactly the risk section 11 plans to catch by manual test.
2. **Main-thread hops.** Both direct calls this design introduces arrive off the main thread.
   The glide runs on `std::thread::spawn` (`pet.rs:226`) and emits from there (`:242`); the mood
   publish runs on the tick thread (`app.rs:274`) and the watcher thread (`watcher.rs:112` into
   `app.rs:291`). `app.emit` marshals for free; a direct setter does not, and touching an
   `NSView` off the main thread crashes. Both entry points need an explicit hop, and the handle
   `AppState` holds must be treated as main-thread-only.
3. **Backing-scale changes.** `pet.js:30`'s resize listener exists for "moving between displays
   of different densities." Natively that is `viewDidChangeBackingProperties()`, **plus**
   manually updating `contentsScale`, because `CALayer.contentsScale` says "for layers you create
   and manage yourself, you must set the value of this property yourself." Without it the pixel
   art blurs on a second display.
4. **`menu(for:)` returning nil is not full suppression.** `NSView` "passes the event up the
   responder chain" if unhandled. Also set `view.menu = nil`, or override `rightMouseDown(with:)`
   without calling super.

**Smaller losses to restore**, against section 1's "no loss of product surface":
`cursor: grab`/`grabbing` (`pet.html:28`, `pet.js:96`, `:123`) needs `resetCursorRects` and
`NSCursor`; `title="Momentum Mascot"` (`pet.html:78`) needs `toolTip`; and `pet.js:149`'s
`invoke("refresh")` was the webview's load-time first publish, which has no native equivalent, so
the initial mood must be pushed explicitly at setup.

### 4.6 The pet window stops being a webview window

Tauri 2 separates windows from webviews: `WindowBuilder::build()` returns a plain `Window<R>`
with no webview (`tauri-2.11.5/src/window/mod.rs:352`, builder at `:147`, `Manager::get_window`
at `manager/mod.rs:640`). Tauri still owns creation, label and geometry, so `pet.rs`'s position
and glide code survives and the NSPanel reclass still reaches the same `ns_window()`.

Six consequences, all of them real work:

1. **The `pet` entry in `tauri.conf.json:28` is deleted, not converted.** `app.windows` has no
   webview-less form, so the window is built in Rust at startup. Note per 4.1 that
   `WindowBuilder::transparent()` is cfg-gated too, so the replacement window is opaque until the
   app sets `setOpaque:`/`setBackgroundColor:` itself.
2. **Six signatures change**, not two: `pet.rs:50` (`get_webview_window`), `pet.rs:95`
   (`usable_bounds(&WebviewWindow)`), `:169` (`place`), `:197` (`nearest_corner`), `:223`
   (`glide_to`), and `commands.rs:33`. The two lookups fail *silently* if missed — `pet.rs:50`
   early-returns and `commands.rs:33` is a `?` on an `Option` — so the symptom is a pet that
   never appears with nothing logged.
3. **`capabilities/default.json`**: `:5` drops `"pet"` from `windows`, and `:9-12`
   (`allow-set-position`, `allow-outer-position`, `allow-set-size`, `allow-start-dragging`) exist
   only for the pet and can go. The popover's only direct window call is `setSize`
   (`popover.js:131`), so verify by removal which of those it still needs.
4. **The command surface shrinks.** `main.rs:38-40` drops `snap_pet` and `cancel_glide` from
   `generate_handler!`, and `commands.rs:31-56` deletes both. The earlier draft said they become
   direct calls but never said the API surface changes, and `commands.rs:1-2` describes itself as
   "the whole API surface".
5. **`bundle.resources` does not exist yet.** `grep resources src-tauri/tauri.conf.json` returns
   nothing. The key must be added, and the native view reads through
   `app.path().resource_dir()`.
6. **Sprites must stay in `frontendDist` as well.** The earlier draft said "nothing serves them
   over the custom protocol any more." Something does: `src/popover.js:49` sets
   `backgroundImage = url("assets/pet/${id}/dozing.png")` for the three character-picker buttons.
   So sprites live in both places.

Nothing else depends on the pet being a webview. `app.rs:19 MOOD_EVENT` is a global emit the
popover keeps (`src/popover.js:185` is its only remaining listener), and `pet.rs:242`'s
`GLIDE_DONE_EVENT` becomes a direct call. The blast radius is bounded, just wider than two lines.

## 5. Popover, sandbox, and state

### 5.1 The popover keeps its webview

Drop `transparent: true` from the popover window. The room art fills the whole 352x540 surface,
so the popover never needed a see-through webview; it needed rounded corners, which come from
`layer.cornerRadius` plus `masksToBounds` on a container view, both public. Per 4.1, the window's
`setOpaque: NO` and clear `backgroundColor` are calls **the app makes itself** — nothing in tao
will make them for us once the feature is off.

### 5.2 Entitlements

New `src-tauri/Entitlements.mas.plist`:

| Key | Why |
|---|---|
| `com.apple.security.app-sandbox` | Mandatory for the store |
| `com.apple.security.files.user-selected.read-only` | The folder picker is the only way repos enter |
| `com.apple.security.files.bookmarks.app-scope` | Required for *persistent* access; section 6 exists for this |
| `com.apple.security.cs.allow-jit` | Carried over from the DMG channel's hardened runtime. **Not required for the store.** |

Three notes on that table, because two of its rows were wrong in the earlier draft.

**`bookmarks.app-scope` was missing.** Apple: "If you want to provide your sandboxed app with
persistent access to file-system resources, you must enable security-scoped bookmark and URL
access." Bookmark creation and resolution appear to work without it, but that cannot be tested
except across a relaunch, which is the only case section 6 exists for. This is the weakest
documentary ground in the plan: both `bookmarks.*` keys 404 in the modern Entitlements reference,
and their only live documentation is a macOS 10.7.3-era page filed under *Professional Video
Applications*. Add the key; it costs a line and review does not object.

**`allow-jit`'s stated reason was wrong.** The earlier draft justified it as "the popover webview
still runs JavaScriptCore." WKWebView runs JS out of process in `com.apple.WebKit.WebContent.xpc`
with its own entitlements. Verified: a sandboxed hardened-runtime bundle *without* `allow-jit`
could not `mmap(MAP_JIT|PROT_EXEC)` itself, yet its WKWebView ran a three-million-iteration JS
loop and `evaluateJavaScript` fine. Separately, Hardened Runtime is not needed for the store at
all — Apple: "Add the Hardened Runtime capability, which isn't necessary for App Store apps"; it
is required only "to upload a macOS app to be notarized." The key is permitted alongside
`app-sandbox` and costs nothing, so keep it, but the reason a future reader acts on must be the
true one.

**No network entitlement, pending one check.** There is no network layer and asserting nothing is
the honest label. But a sandboxed WKWebView calling `loadHTMLString:` with no network involved
was observed to never finish navigation without `com.apple.security.network.client` — a silent
hang with no sandbox violation logged, which is also why Electron's MAS instructions mandate it.
**This is not verified for Tauri**, which serves the popover through a custom scheme handler
rather than a URL load, so it may not apply. Phase 1 checks it first (section 10), because the
failure mode is a blank popover with no error message discovered at the *end* of phase 4. If the
entitlement turns out to be needed, it is not a privacy claim and section 8.3's answers do not
change, but this paragraph's last sentence has to go.

### 5.3 State path

**Measured, not assumed.** A minimal bundle signed with nothing but
`com.apple.security.app-sandbox` reports:

```
getenv(HOME)             = /Users/kyle/Library/Containers/dev.keepgoing.homeprobe/Data
NSHomeDirectory()        = /Users/kyle/Library/Containers/dev.keepgoing.homeprobe/Data
APP_SANDBOX_CONTAINER_ID = dev.keepgoing.homeprobe
```

The redirection applies to the raw environment variable, not only to `NSHomeDirectory()`, so
Rust's `std::env::var_os("HOME")` at `store.rs:88` sees it too. `store::default_path()`
(`store.rs:72`) resolves to
`~/Library/Containers/dev.keepgoing.momentum-mascot/Data/.keepgoing/mascot/state.json` in the
store build and stays at `~/.keepgoing/mascot/state.json` in the DMG build, with **no code change
and no `cfg`**. The path follows the entitlement.

`APP_SANDBOX_CONTAINER_ID` is also set, which is the cheapest runtime sandbox detection available
if anything ever needs to branch on it. Nothing in this design does.

No migration, and it could not be written if one were wanted:

```
getpwuid->pw_dir     = /Users/kyle          <- real home is still discoverable
read real state.json -> denied ("you don't have permission to view it")
list ~/.keepgoing    -> DENIED
```

The path is discoverable through `getpwuid` but unreadable, so a sandboxed build has no way to
import an existing user's project list. This is precisely why section 3 leaves the DMG build
unsandboxed: the channel that has existing users is the channel that keeps its state file where
it always was.

## 6. Security-scoped bookmarks

### 6.1 Schema

`store::Project` gains `bookmark: Option<String>`, the base64 of NSURL bookmark data. The JSON
reader is tolerant of unknown and missing fields by contract (`store.rs:8-13`), so old state
files load unchanged and no reader needs a version bump. `SCHEMA_VERSION` moves to `3.1` for
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
the bookmark from the resolved URL while access is held, repairing the entry without re-prompting.

Per section 3, a `false` from `startAccessingSecurityScopedResource` means "use the stored path
directly", never "drop the project".

### 6.3 Integration points

Two, and only two. This was audited exhaustively and nothing was missed: `resolve_paths` is
called from exactly one place (`momentum.rs:70`, inside `Momentum::load`), which is called from
exactly one place (`app.rs:62`, inside `AppState::new`); the only other `repo::resolve` in
production code is `momentum.rs:227` in `add()`, which is the picker path; and no other module
reads `Project.path`.

`commands.rs:70 add_project` creates the bookmark immediately after the picker returns, while
access is granted. If creation fails the project is still added with `bookmark: None`, degrading
to today's behavior: it works this launch and reports unavailable on the next.

`momentum.rs:74 resolve_paths()` starts access for each project before calling `repo::resolve`,
holding the guards in a `HashMap<String, ScopedAccess>` beside `git_dirs` and `work_trees`.

**The guards must live for the whole process, and the reason must be written down** so nobody
scopes them to load: `momentum.rs:278 read_commit_time` runs on every tick (`app.rs:274`) and
every watcher event (`watcher.rs:112`), and `watcher.rs:187` registers watches later still.
Also, `resolve_paths` opens by clearing `git_dirs` and `work_trees` (`momentum.rs:75-76`), so
whoever adds a third map must ensure guards are not cleared out from under a live `refresh_all`.

### 6.4 What the entitlement covers, verified

`files.user-selected.read-only` is sufficient. Both halves of the doubt check out:

- The recursive grant is documented: "When the URL your app receives from a standard user
  interface interaction represents a folder, the operating system extends your app's sandbox to
  items within that folder, and recursively in nested folders."
- Recursive FSEvents works inside a held scope. Verified in a sandboxed bundle:
  `FSEventStreamCreate` on a repo root with `kFSEventStreamCreateFlagFileEvents|NoDefer` started
  and delivered nested paths in both `.git/` and `src/`.
- Dot-directories are not specially blocked: `.git/logs/HEAD` read fine, and 679 entries
  enumerated recursively under `.git` from inside the sandbox.

So "the watcher needs no change" is correct and `watcher.rs:187`'s
`watch(path, RecursiveMode::Recursive)` needs nothing extra. One footnote, not a blocker:
FSEvents *historical* events via `sinceWhen` need the volume's `/.fseventsd`, which is unreadable
in a sandbox. `notify` uses `kFSEventStreamEventIdSinceNow`, so it does not bite.

## 7. Deliberately accepted degradations

### 7.1 The git shellout stops firing, but not for the stated reason

`repo.rs:86 head_commit_time` shells out to `git`. The earlier draft said App Sandbox blocks
`exec`. **It does not.** Verified from a sandboxed bundle with only `app-sandbox` and
`user-selected.read-only`: `posix_spawn` of a real git binary succeeded and printed its version,
and the child inherits the sandbox. (Setuid binaries genuinely *are* blocked: `/bin/ps` returns
"Operation not permitted" while `/bin/cat` in the same directory spawns fine.)

What actually breaks it is that `/usr/bin/git` is the **xcrun shim**, which self-refuses with
"xcrun: error: cannot be used within an App Sandbox", plus `git` looking for `~/.gitconfig` at a
redirected `$HOME`. Also relevant, from Apple's sandbox file-access page: "Your app can't run
programs in locations outside its app bundle, sandbox container, or app group containers using
the entitlements to access user-selected files."

The outcome is identical (`output()` → `Err` → `ok()?` → `None`), so the decision stands and the
graceful-degradation argument at `repo.rs:74-84` is untouched: "the worst case is a slightly
stale timestamp, never a false comeback." But the wrong reason produces wrong decisions
elsewhere, so the right one is recorded here.

### 7.2 Linked worktrees and submodules break under sandbox, on the first launch

**This needs a decision before phase 4**, because it changes what the sandbox-persistence test
asserts.

`repo.rs:35-66 resolve()` follows a `.git` *file*'s `gitdir:` pointer, and `:49-52` accepts an
absolute pointer. For a `git worktree` checkout or a submodule, the resolved git dir lies
**outside** the folder the user picked, so it is outside the NSOpenPanel grant and outside any
bookmark section 6 creates. `:54` and `:62` both fail, giving `Err(NotARepo)`, which
`momentum.rs:78` skips, which surfaces as `available == false` (`momentum.rs:204`).

Unlike section 2.4 this is **not** fixed by bookmarks: the grant never covered that path, even on
the launch where the picker ran. And it is pointed, because `repo.rs:30-31` says "a developer
working in a worktree is exactly the kind of person this product is for." Under sandbox, that
person's project silently reads as unavailable.

Two options: record it as a second accepted degradation, or bookmark the resolved git dir
separately, which needs its own picker prompt. Decide before phase 4.

## 8. Signing and submission

### 8.1 Certificates and identifiers, one time

1. Register App ID `dev.keepgoing.momentum-mascot` in the developer portal.
2. Create an **Apple Distribution** certificate, which signs the app. Its common name is
   `Apple Distribution: <Team Name> (<Team ID>)`.
3. Create a **Mac Installer Distribution** certificate, which signs the package. **The portal
   label is not the certificate's common name**: the CN reads
   `3rd Party Mac Developer Installer: <Team ID>`. No certificate's CN reads "Mac Installer
   Distribution".
4. Create an App Store Connect **API key** (`--api-key`/`--api-issuer` with `AuthKey_<id>.p8`).
   `tools/.release-env` currently holds an app-specific password, which works, but an API key is
   the better auth for uploads.
5. Optionally create a Mac App Store provisioning profile and place it at
   `Contents/embedded.provisionprofile`. **It is not required.** TN3125: "A Mac app that uses no
   restricted entitlements doesn't need a provisioning profile. This is true even if the app is
   distributed on the App Store. The only exception to this rule is TestFlight, which always
   requires a profile." Apple's unrestricted list explicitly includes "entitlements that enable
   and configure App Sandbox" and "entitlements that configure the Hardened Runtime" — every key
   in section 5.2's table. Keep the step anyway: it is one `cp`, it is the only route to macOS
   TestFlight later, and for a submission whose purpose is learning the process, having walked it
   is arguably the point. But `release-mas.sh` must not hard-fail on its absence. If you do embed
   one, copy it in **before** signing: "the profile is sealed by the code signature."
6. Create the app record in App Store Connect.

### 8.2 `tools/release-mas.sh`

A sibling to `release.sh`, not a modification of it. The DMG path works and must not be
destabilised.

1. Verify the certificates with **bare `security find-identity -v`**, not `-p codesigning`.
   Apple: "Don't use the `-p codesigning` option... Installer-signing identities are different
   from code-signing identities, so the `-p codesigning` option filters out installer-signing
   identities." That pattern is already at `release.sh:118` and
   `tools/.release-env.example:12`, so copying the sibling script would make this step fail on a
   correctly-configured machine. Treat a missing provisioning profile as a warning, not an error.
2. Build universal, as `release.sh` does.
3. **Assign a unique build number.** Tauri writes `tauri.conf.json:4`'s version into both
   `CFBundleShortVersionString` and `CFBundleVersion`, and App Store Connect rejects a re-upload
   that reuses a build number. A first submission is very likely re-uploaded at least once, so
   this will stall phase 6 otherwise. Version bumping stays in `release.sh` so the channels
   cannot disagree about what a version *is*; the build number is this script's own counter.
4. Copy `embedded.provisionprofile` in, if one exists.
5. Sign the app with the Apple Distribution identity and `Entitlements.mas.plist`, inside out.
   Apple, both current: "Sign code from the inside out" and "Don't pass the `--deep` option to
   codesign when you sign code." Two adjacent rules worth honouring: do not apply entitlements
   to library code, and never run `codesign` under `sudo`.
6. Package with `productbuild --component`, signed with the installer identity. This is verbatim
   Apple's own MAS recipe: "The following is the simplest use of productbuild, sufficient for
   submitting your app to the Mac App Store: `productbuild --sign <Identity> --component
   <PathToApp> /Applications <PathToPackage>`."
7. Upload with `xcrun altool`. **Not retired**, which was the highest-risk claim in this document
   and it survived: TN3147 says "Apple has deprecated altool for the purposes of notarization...
   However, altool is still a good way to perform other tasks, like submitting an app to the App
   Store," and App Store Connect Help confirms "Upload for all target types is supported for
   Transporter and altool." The 2023 cutoff was notarization-only; `notarytool` is not a
   store-upload tool. Two spelling corrections: use `--upload-package` (`--upload-app -f <file>`
   is an accepted alias), and Xcode 26 shipped a rewritten altool with renamed flags —
   `--type` → `-t`/`--platform`, `-p` → `--app-password`, `--apiKey`/`--apiIssuer` →
   `--api-key`/`--api-issuer`, `--asc-provider` → `--provider-public-id`, with a
   `--use-old-altool` escape hatch. Write the new names.
8. Do **not** notarize: "you aren't required to notarize software that you distribute through the
   Mac App Store because the App Store submission process already includes equivalent security
   checks." Do not tag or create a GitHub release either.

### 8.3 App Store Connect metadata

- Price: free.
- Category: **Developer Tools**, for the 4.2 reason below. **This contradicts the bundle today:**
  `tauri.conf.json:51` says `"category": "Utility"`, which becomes
  `LSApplicationCategoryType = public.app-category.utilities`. When the category is doing review
  work, the two must agree. Tauri accepts `"DeveloperTool"`.
- Privacy: "Data Not Collected", every category, and the reasoning is sound.
  Apple: "'Collect' refers to transmitting data off the device", and "data that is processed only
  on device is not 'collected' and does not need to be disclosed." Reading the user's filesystem
  is emphatically not collection. One caveat to keep in view: "if you derive anything from that
  data and send it off device, the resulting data should be considered separately." The share card
  puts derived data on the *clipboard*, not off-device, so it stays clear.
- **Privacy policy, and a decision to make.** Guideline 5.1.1(i) requires a policy link in App
  Store Connect **and "within the app in an easily accessible manner."** There is nowhere obvious
  to put the in-app link: `tray.rs:22-23` says "Exactly two items, and adding a third is a spec
  change", and `site/index.html:135` is a privacy *section* on a one-page site, not a policy.
  Small work, real 5.1.1 and 2.1 exposure, and it collides with an existing design constraint, so
  it needs a decision rather than a line item.
- Screenshots at 2560x1600: the pet on a desktop, the popover room in each of the four moods, and
  the share card.
- Review notes: the app shows nothing until a repository is added, so the notes must tell the
  reviewer to click the tray icon, add any folder containing a git repository, and say that a
  freshly committed repository shows the awake state immediately. Without this the app looks
  broken to a reviewer who never adds a folder, which is a 2.1 rejection. Guideline text
  confirmed current as of 2026-08-22: 4.2 "If your app is not particularly useful, unique, or
  'app-like,' it doesn't belong on the App Store"; 2.1 "We will reject incomplete app bundles."
- Copyright and attribution: the LimeZu and Departure Mono credits already in
  `tauri.conf.json`'s `copyright` field carry into the listing.

### 8.4 Asset licence check, blocking, and first

Confirm the LimeZu Modern Interiors licence permits distributing the compiled art through the Mac
App Store. The README's claim, that the licence permits shipping compiled into an application and
forbids redistributing the assets, is very likely sufficient for a free listing, and the store
build redistributes no assets. Confirm it in the licence text rather than by inference. If it
fails, the whole submission stops, so it is checked before anything else (section 10).

## 9. Testing

Existing Rust tests are pure or tempdir-based and must keep passing untouched.

New automated coverage:

- `store.rs`: a state file with a `bookmark` field loads, and one without it loads with
  `bookmark: None`, in the module's existing resilience style.
- `scoped.rs` round trip. **This is a smoke test and must be labelled one.** A cargo test binary
  is not an `.app`, is not sandboxed, and cannot be made so: a bare Mach-O signed with
  `app-sandbox` outside a bundle is killed with SIGTRAP, exit 133. Unsandboxed, creation,
  resolution and `startAccessing` all return success trivially, so a green result proves only
  "doesn't crash, doesn't leak, guard drops". The risk is phase 4 taking false confidence from a
  vacuous pass.
- `store::default_path()` returns the `$HOME`-relative path unchanged. There is no migration and
  no sandbox-aware branch to test, because section 5.3 measured that the environment does the
  work.

Manual, and the first of these is the test that proves the whole effort:

- **Sandbox persistence.** Sign locally with `Entitlements.mas.plist`, launch, add a repository,
  quit, relaunch, confirm the repository is still readable and the mood is still built from it.
  If this passes, section 6 is done. Assert the section 7.2 decision here too, whichever way it
  went.
- `strings -a <binary> | grep -cE 'drawsBackground|fullScreenEnabled'` returns 0.
- The **sprite cycle** shows all 12 frames, not 11. `drive-states.sh` compares the four-state
  arc, not the cycle, so it would not catch a dropped frame; this needs its own look.
- The pet is still visible and non-hostile over a fullscreen app. This is the regression the
  NSPanel decision was won against.
- The pet appears at all. Section 4.6 point 2 fails silently if missed, so this is a real test.
- The pet drags to all four corners and glides, a click still opens the popover, and the cursor
  still changes on grab.
- Pixel art stays crisp when the pet is dragged to a display of a different density.
- The popover works with the narrowed `capabilities/default.json`: add a project, cycle a
  character, toggle operating, untrack, copy the share card, dismiss with Escape.
- The popover's rounded corners read correctly on a light and a dark desktop.

## 10. Order of work

**Phase 1, the asset licence check** (section 8.4). Blocking, cheap, reading only, and it can
invalidate everything after it. The earlier draft had this second, contradicting its own
sentence that "this is checked first."

**Phase 2, throwaway probes, one afternoon.** In this order, because the order is the value:

1. Does the popover need `com.apple.security.network.client` (section 5.2)? Sign a sandboxed
   build with and without it and open the popover. Decisive, and it changes the entitlements
   table.
2. **The section 4.0 probe.** Can the pet keep its alpha with the private key gone, using manual
   `setOpaque:`/`setBackgroundColor:` plus public `underPageBackgroundColor`? If yes, phase 4
   disappears. One hour against a multi-week rewrite.
3. Does the popover's corner rounding work as section 5.1 claims?

Its code is thrown away even if it works.

**Phase 3, sandbox and bookmarks** (sections 5, 6, 7.2). Moved ahead of the pet deliberately:
it is the work the store actually requires, it is where the submission-relevant learning is, and
it is unaffected by how phase 2's second probe turns out.

**Phase 4, the native pet** (section 4), only if the 4.0 probe failed. The largest piece. Ends
when fullscreen, drag, glide, click, cursor and the 12-frame cycle all match the webview pet.

**Phase 5, certificates and `release-mas.sh`** (sections 8.1, 8.2).

**Phase 6, listing and submission** (section 8.3). Then wait, and learn what review says.

**Where the schedule risk is.** Phases 5 and 6 are the learning this project exists for. Phase 3
is the price of admission. Phase 4 is most of the cost and teaches nothing about the App Store:
it is a full AppKit rewrite of the pet's rendering and interaction, carrying the traps in 4.1,
4.3, 4.4 and 4.5, in order to remove one KVC key — after which, per section 2.2, two private-API
strings remain in the binary anyway. That trade is still worth taking, because section 4.2's
argument that the character *is* the product is sound and shipping a knowingly-degraded pet to
learn a process would be the worse deal. But the entire schedule risk of this plan sits in the
one phase that is not about the store, which is why the 4.0 probe runs before it and why phase 3
comes first.

## 11. Risks

| Risk | Likelihood | Response |
|---|---|---|
| Guideline 4.2, minimum functionality | Real | Developer Tools category, review notes, and a listing describing an ambient desktop pet rather than a productivity tool. If rejected, appeal explaining the category, not new features. |
| The two unremovable private strings are cited (section 2.2) | Low | Precedent: Tauri apps ship on the store carrying both. If cited, the options are forking tao and wry, or abandoning the store. This precedent carries more weight than is comfortable. |
| Phase 4 runs long | **High** | The 4.0 probe may delete it. If not, the traps are enumerated in 4.1, 4.3, 4.4 and 4.5 precisely so they cost hours rather than days. |
| Native pet regresses the fullscreen fix | Medium | The panel and its content view are untouched by construction (4.3), and fullscreen is an explicit manual test. |
| Dropped sprite frame reads as a limp | Medium | The N+1 keyTimes rule in 4.4, plus a test that counts frames rather than watching the state arc. |
| Pet stops responding to clicks | Medium | `acceptsFirstMouse` in 4.5. Structurally guaranteed to bite, since the panel is never key. |
| Crash from touching NSView off-thread | Medium | Main-thread hops in 4.5, at both new entry points. |
| Popover hangs blank under sandbox | Unknown | Phase 2 probe 1, before any other work. |
| Worktree and submodule users see projects as unavailable | Confirmed | Section 7.2. Needs a decision before phase 3. |
| `bookmarks.app-scope` documentation is ancient | Low | The key is added; the caveat is recorded in 5.2. |
| Build number collision on re-upload | **High** | Section 8.2 step 3. Certain to bite on a first submission otherwise. |
| LimeZu licence forbids store distribution | Low | Phase 1, before any code. |
| ~~`$HOME` is not redirected under sandbox~~ | **Closed** | Measured in 5.3. It is redirected, for `getenv` as well as `NSHomeDirectory()`. |
| ~~`altool` retired for store uploads~~ | **Closed** | TN3147 and ASC Help both confirm it is current for uploads. |

## 12. Consequences for `docs/spec-v2.md`

Section 10.3 and the risk table entry at `spec-v2.md:705` record App Store ineligibility as an
accepted trade, with the note that "if that ever becomes a target, the pet has to be an opaque
square, which is a design decision rather than a bug."

That prediction was closer to right than this document's first draft was. The pet does not have
to be an opaque square, but the reason the first draft gave — that window transparency is public
— does not hold through Tauri (section 4.1). The pet keeps its alpha only because the app makes
the AppKit calls itself, and possibly, pending the 4.0 probe, only after a full native rewrite.
Both places get updated to point here rather than being deleted, so the reasoning trail stays
intact.
