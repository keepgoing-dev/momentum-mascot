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
