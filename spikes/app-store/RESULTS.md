# App Store probes

Throwaway code, kept findings. Same rule as `spikes/always-on-top/RESULTS.md`: a future
macOS release that breaks one of these should be re-diagnosed in minutes rather than
re-explored from scratch.

## Probe 1: does the sandboxed webview need com.apple.security.network.client?

**Answer: yes. It is required, and without it the app is silently broken.**

**Measured:** 2026-08-22, macOS 26 (Darwin 25.5.0), Xcode 26.6, on the debug bundle
ad-hoc signed three ways with `codesign --force --sign - --entitlements <file>`.

### Why an eye test was not used

The obvious method is "open the popover and look". Screen recording permission was not
available to the shell, and clicking a menu bar status item is not scriptable without
accessibility permission, so the observation had to come from the app's own behaviour.

**The instrument: distinct inodes of `state.json`.** `store::save` writes a temporary file
in the same directory and renames it, so **every publish produces a new inode**. Both
windows call `invoke("refresh")` as the last line of their script (`pet.js:149`,
`popover.js:191`), which runs `app::publish`, which saves. So:

- 1 distinct inode = only the startup publish from `main.rs`'s `app::refresh(&handle)` ran.
- 2 distinct inodes = a webview finished navigation and executed its JavaScript.

Polled every 200ms, so two saves in the same wall-clock second are still counted
separately. This matters: an earlier attempt compared mtimes at 3s and 11s and could not
tell the two publishes apart, because both land inside the first second. That attempt
reported a false pass.

### Results

| Signing | saves in 15s | Webview JS ran |
|---|---|---|
| ad-hoc, no entitlements (unsandboxed baseline) | 2 | yes |
| ad-hoc + `Entitlements.mas.plist` without `network.client` | 1 | **no** |
| ad-hoc + the same plus `network.client` | 2 | yes |

Six runs total. The first three in the order above; then three more with the order
reversed (with, without, with, without) to rule out an ordering artefact, and with the
window widened from 8s to 15s so that "no JS" could not merely mean "slow JS". The result
was identical every time: **1 save without the key, 2 with it.**

A second, weaker instrument agreed on the first pass: the count of
`com.apple.WebKit.WebContent` processes rose by 2 when unsandboxed and by 0 when sandboxed
without the key. It is only corroboration, because WebKit content processes linger for a
few seconds after their app dies, which confounds the delta on back-to-back runs.

### Why this is dangerous

The failure is **completely silent**. The process stays alive, stderr prints nothing, and
the kernel logs no sandbox denial (checked with
`log show --predicate 'eventMessage CONTAINS "deny"'`; the only denials in the window
belonged to unrelated system daemons). A blank popover with no error, discovered at the end
of a multi-week phase, is exactly what running this probe first was meant to avoid.

The spec had this as an open question, reasoning that Tauri serves the popover through a
custom scheme handler rather than a URL load and so might be exempt from the failure that
makes Electron's Mac App Store instructions mandate the key. **Tauri is not exempt.**

### What it does not mean

It is not a privacy claim and it does not change the App Store privacy answers. The app
makes no network requests. What the key buys is WebKit being allowed to talk to its own
networking process.

## Confirmed in passing: the container redirect is real

The sandboxed bundle wrote its state to
`~/Library/Containers/dev.keepgoing.momentum-mascot/Data/.keepgoing/mascot/state.json`
with **no code change and no cfg**, because App Sandbox redirects `$HOME` itself and
`store::default_path` reads `$HOME`. Spec section 5.3 measured this on a synthetic probe
bundle; this is the same result from the real app.

The real `~/.keepgoing/mascot/state.json` was untouched by the sandboxed build, which is
the other half of the point: there is no migration, and a sandboxed process could not read
that file to write one.

**One correction for anyone repeating this:** `rm -rf ~/Library/Containers/<bundle id>`
fails with `Operation not permitted` on `.com.apple.containermanagerd.metadata.plist`. The
container root is protected. Delete the state file inside `Data/` instead.

## Probe 2: can the pet keep its alpha without the private API? (spec 4.0)

**Answer: no. The native pet in spec section 4 is required.**

**Measured:** 2026-08-22, macOS 26 (Darwin 25.5.0), on a debug bundle built with
`macos-private-api` dropped from `src-tauri/Cargo.toml` and `macOSPrivateApi` set to false
in `tauri.conf.json`, plus, by hand and with public API only:

- `setOpaque: NO` and `setBackgroundColor: NSColor.clearColor` on the pet's NSPanel, which
  is what `tao/window.rs:544-561` does behind the private feature.
- `underPageBackgroundColor = clearColor` on the WKWebView, public since macOS 12 and
  already called by wry at `wkwebview/mod.rs:441`.

**Result: a visible opaque square around the character.** The wallpaper did not show
through. So `underPageBackgroundColor` does not reach the page's own backdrop, which is what
wry's own comment at `wkwebview/mod.rs:429-431` implies: it covers the overscroll region
only. `_drawsBackground` is the only thing that makes a WKWebView see-through, and it is
private.

### Two things this probe proved on the way, both worth keeping

**The window really is transparent by public API. Only the webview is not.** Read back from
AppKit in the same run:

```
PROBE window: isOpaque=false backgroundColorAlpha=0
```

That is the manual `setOpaque:`/`setBackgroundColor:` pair taking effect on the panel. So
spec 4.2's route is sound: the app makes the window calls itself, and the pet's content
stops being a webview so nothing private is needed to see through it.

**Tauri's complaint is real and it is debug-only**, exactly as spec 4.1 says. With the
feature off, launching printed, once per window:

```
The window is set to be transparent but the `macos-private-api` is not enabled.
```

This came from a **debug** build. `tauri-runtime-wry/src/lib.rs:884-893` gates that
`eprintln!` on `debug_assertions`, so a release build says nothing at all. An implementer
who drops the feature, writes a correct sprite view, and sees a 64x64 opaque square would
have no message to go on.

### The private strings, confirmed by measurement

Same build, feature off, single-architecture debug binary:

| String | Count | Status |
|---|---|---|
| `drawsBackground` | 0 | removable, gone with the feature |
| `fullScreenEnabled` | 0 | removable, gone with the feature |
| `allowsPictureInPictureMediaPlayback` | 1 | **not removable**, wry sets it behind no gate |
| `_wantsKeyDownForEvent` | 1 | **not removable**, tao registers it unconditionally |

Counts are 1 rather than 2 because this is one architecture; the shipped universal binary
has two slices. On that binary the two removable keys sit inside the **same** string blob,
so `grep -cE 'drawsBackground|fullScreenEnabled'` reports 2 lines and not 4. The gate in
`tools/release-mas.sh` asserts 0, which is correct either way.

This is spec sections 2.1 and 2.2 confirmed: the work in section 4 removes the two strings
that are removable, and ships the two that are not.

### Consequence

A second plan is required for spec section 4, the native AppKit pet, to be written and
executed after Phase 3 of
`docs/superpowers/plans/2026-08-22-mac-app-store-submission.md`. Until it lands, the
private-API gate in `tools/release-mas.sh` refuses to upload.

## Probe 3: does native corner rounding replace the transparent popover? (spec 5.1)

**Answer: yes, on the content view. The simpler of the two options is the one that works.**

**Measured:** 2026-08-22, macOS 26 (Darwin 25.5.0). Dropped `"transparent": true` from the
popover window in `tauri.conf.json` (the pet's entry left alone), gave the page an opaque
`background: var(--panel)` in `popover.css` because `style.css` is shared with the pet and
says `background: transparent`, then on the popover's NSWindow:

- `setOpaque: NO` and `setBackgroundColor: NSColor.clearColor` on the window,
- `wantsLayer = YES`, `layer.cornerRadius = 12`, `layer.masksToBounds = YES` on the
  **content view**.

Read back in the same run:

```
PROBE popover: cornerRadius=12 masksToBounds=true
```

**Result: rounded corners with the desktop showing through outside the curve**, on both a
light and a dark backdrop, matching what `.panel`'s `border-radius: 12px` looks like with
the transparent webview. The drop shadow followed the rounded shape, so no
`invalidateShadow` call is needed.

So masking on an ancestor view **does** clip the WKWebView's remote-hosted layer. The
documented fallback, rounding the webview's own layer via `with_webview`, is not needed.

**Consequence for Task 11:** round the content view, radius 12.0 to match
`popover.css`'s `.panel { border-radius: 12px }`. No shadow work.

### One incidental finding worth keeping

The popover hides itself on focus loss (`main.rs`'s `WindowEvent::Focused(false)` handler),
which closed it the instant focus moved and made the observation unreliable. The probe
disabled that handler to hold the window on screen. Anyone doing visual work on the popover
should expect to do the same: it is not a bug, it is the click-outside rule doing its job.

## Incidental: `WithSecurityScope` cannot be exercised from a cargo test binary at all

Found while writing `scoped.rs`'s tests, and it corrects two sentences in the spec.

Spec section 9 says of the `scoped.rs` round trip: "Unsandboxed, creation, resolution and
`startAccessing` all return success trivially, so a green result proves only 'doesn't crash,
doesn't leak, guard drops'." The premise is wrong. From a `cargo test` binary,
`bookmarkDataWithOptions:` with `NSURLBookmarkCreationWithSecurityScope` does not succeed
trivially. It **fails**:

```
NSError { code: 256, localizedDescription: "The file couldn't be opened.",
          domain: "NSCocoaErrorDomain" }
```

The same call with **empty** options succeeds in the same binary, on the same directory, in
the same run. So the cause is the security-scope flag needing the sandbox entitlements, not
the path and not the FFI.

**What this changed in the code.** `create` and `resolve` now delegate to private
`create_with(path, options)` and `resolve_with(bookmark, options)`. The tests drive those
with empty options, which covers every line of the FFI plumbing: the selector names, the
`NSData` bridge, the base64 in both directions, and `ScopedAccess`'s `Drop`. Only the option
flag itself is left uncovered. Without the split, `scoped.rs` would have had **no** automated
coverage of its FFI, and a mistyped selector would have surfaced for the first time in a
manual test after signing.

**Still open, for the sandbox persistence test to settle.** Spec section 3 says
`WithSecurityScope` creation was measured working in an *unsandboxed* context, byte-identical
to the sandboxed result, and uses that to justify one code path with no `cfg` across both
channels. This finding does not contradict that (a signed app bundle is not a bare test
binary) but it does weaken it. What the DMG channel actually gets is worth measuring when a
real bundle is next signed: if `create` always fails there, the behaviour is still correct,
because that channel does not need bookmarks, but the claim in section 3 needs rewording.

## Trap: keepgoing.dev returns 200 with the homepage for every unknown path

Found while wiring the privacy policy link, 2026-08-22.

```
curl -o /dev/null -w '%{http_code}'  https://keepgoing.dev/privacy                  -> 200
curl -o /dev/null -w '%{http_code}'  https://keepgoing.dev/definitely-not-a-page    -> 200
```

Both serve the **homepage**, title "Momentum Mascot - a tiny desktop companion for side
projects". There is a catch-all fallback, so a missing page is indistinguishable from a
present one by status code.

Two consequences:

1. **Verify the policy page by content, never by status.** The check is
   `curl -sS https://keepgoing.dev/privacy | grep -q "<title>Privacy Policy"`, not a 200.
   An earlier version of the plan's Task 12 checked the status code and would have passed
   against a homepage.
2. **App Review would see the homepage, not a policy**, if the site is not redeployed before
   submission. Guideline 5.1.1(i) wants a real policy at the URL given in App Store Connect.
   `site/privacy.html` exists in the repo now; it has to be deployed, and the content check
   above is what proves it.

## The sandbox persistence test: PASS

**Measured:** 2026-08-22, on the release build, ad-hoc signed with
`codesign --force --sign - --options runtime --entitlements src-tauri/Entitlements.mas.plist`,
container state cleared first. Two folders added through the app's own picker: an ordinary
git repository and a linked `git worktree` whose parent repo lives in `/private/tmp`, outside
anything the picker granted.

### Bookmarks are created for real inside a sandboxed bundle

`state.json` in the container, immediately after adding:

```
schema version: 3.1
  mascot-mas-test           bookmark=YES 952 chars   last_commit_at=2026-08-22T10:55:10Z
  mascot-mas-test-worktree  bookmark=YES 960 chars   last_commit_at=None
```

This settles the question left open above: `NSURLBookmarkCreationWithSecurityScope` **works**
in a signed, entitled app bundle. It fails only in a bare `cargo test` binary. So spec
section 3's "one code path serves both channels" claim stands for the sandboxed channel, and
what the unsandboxed DMG channel gets is still worth a look when one is next signed.

### The test itself

"The project is still listed" proves nothing: the list and its timestamps persist in
`state.json` regardless of whether the folder can be read. So the measurement was: quit, make
a **new commit while the app was closed**, relaunch, and see whether `last_commit_at`
advances. It can only advance if `resolve_paths` resolved the bookmark, took the grant, and
`repo::resolve` then succeeded at that path.

```
before quit      last_commit_at=2026-08-22T10:55:10Z
new commit       2026-08-22T16:11:54Z
after relaunch   last_commit_at=2026-08-22T16:11:54Z     PASS
```

Without section 6 this would have stayed at 10:55:10Z, because the picker's grant expired
with the first launch.

### The watcher, inside the sandbox, on a live commit

A third commit made while the relaunched app was running:

```
after live commit  last_commit_at=2026-08-22T16:12:00Z    PASS
```

That exercises recursive FSEvents inside a held security scope and a read of
`.git/logs/HEAD`, a dot-directory, both of which spec 6.4 predicted would work.
`watcher.rs` needed no change.

### Section 7.2, confirmed as designed

The linked worktree's `last_commit_at` stayed `None` through all of it: its git dir is in
`/private/tmp`, outside the grant, so `repo::resolve` returns the new `GitDirOutside`. The
accepted degradation behaves exactly as the spec said it would.

## Defect found by the manual test: a `title` tooltip on a span never renders here

The plan surfaced the unavailable-reason as a `title` attribute on the project name, per spec
7.2. **It does not render.** Hovering a row's name shows nothing; hovering the operating
toggle, which is a `<button>` with a `title`, does show its tooltip. So the attribute works on
buttons in this webview and not on the span.

No CSS explains it: `.projects .name` sets only flex, colour and ellipsis, and nothing in the
row disables pointer events.

**Fixed by not using a tooltip.** The reason is now a visible line of its own under the name,
present only on unavailable rows, at 10px in `var(--muted)`. `.projects li` gained
`flex-wrap: wrap` and the new `.reason` element takes `flex: 0 0 100%`. The `title` is kept as
well, since it costs nothing and does work in some contexts.

This is worth more than the layout change: spec 7.2's whole argument is that silent acceptance
was rejected because "the affected user gets no explanation". An explanation in a tooltip that
never appears is the same thing as no explanation. Verified afterwards with a fixture holding
one healthy repo, one unreachable worktree and one deleted folder: both reasons render, the
healthy row gets no extra line.

## Native pet, Task 3: does Core Animation honour N+1 discrete keyTimes?

The native pet plan's Task 1 asserts the arrays handed to Core Animation. It cannot assert what
Core Animation does with them, and that is the plan's central claim, so it was measured on a real
build. Instrument: `MASCOT_PROBE_FRAMES=1`, which runs `sprite::view`'s `probeFrames`.

**Result: PASS.**

```
PROBE frames: mood=awake duration=4 animationKeys=1 contents=true cell=64x64 at (0,0)
PROBE sprite: magnificationFilter=nearest contentsScale=2 backingScale=2 viewBounds=64x64 at (0,0)
PROBE frames: 125 samples over 5s
PROBE frames: distinct={0,1,2,3,4,5,6,7,8,9,10,11}
PROBE frames: out_of_order_transitions=0
PROBE frames: PASS
```

**The counter-test is what makes that mean anything.** With `key_times` changed to the
eleven-plateau mistake, `(0..12).map(|i| i / 11)`, the same probe on the same build reports:

```
PROBE frames: distinct={0,1,2,3,4,5,6,7,8,9,10}
PROBE frames: out_of_order_transitions=1
PROBE frames: FAIL
```

Frame 11 never reaches the screen and the cycle transitions 10 to 0 instead of 10 to 11. That is
the limp the N+1 rule exists to prevent, observed rather than argued.

### Two seek-based probe designs that do not work

Worth recording, because both look correct and both produce a **false** reading of "frame 0 for
every seek", which is indistinguishable from a sprite that never animates at all:

1. `sprite.speed = 0`, then `timeOffset` across the twelve plateaus with `CATransaction::flush()`
   between reads, all within one run-loop callback. Twelve readings of frame 0.
2. The same, but one seek per run-loop turn via `performSelector:afterDelay:`, recording the frame
   the previous seek produced. Also twelve readings of frame 0.

Diagnostics ruled out every obvious cause: `animationKeys=1`, `contents=true`,
`presentationLayer=true`, cell frame 64x64. The seek itself does not take. Sampling the running
animation needs none of that machinery and measures the claim more directly, because the
eleven-plateau scheme holds its twelfth frame for zero time and therefore cannot ever be sampled
showing twelve distinct frames.

Also note `f64::NAN as i64` is 0 in Rust. The first probe used `unwrap_or(f64::NAN)` for a nil
presentation layer, so a nil layer would have been reported as a real reading of frame 0. The
probe now records -1 for nil.

### What this settled without an eye test

- `magnificationFilter` reads back as `nearest`, so the pixel art is not being smoothed.
- `contentsScale` is 2 and equals the window's `backingScaleFactor`, so it is not half-resolution.
- The cell is 64x64 at (0,0) in a 64x64 view, which is `cell_side`/`cell_origin` agreeing with
  the unit tests on the size that actually ships.
