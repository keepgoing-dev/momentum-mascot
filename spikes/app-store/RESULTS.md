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
