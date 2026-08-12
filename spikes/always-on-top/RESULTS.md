# Spike results: always-on-top over macOS fullscreen

Gate for `docs/spec-v2.md` section 11, wall 1 and section 12, Phase 3.

**Question being answered:** can a 64x64 always-on-top window be visible over a fullscreen
application on macOS, without being hostile to the user?

"Visible" is not the whole test. A pet that is visible but drags you out of your fullscreen
Space when you click it is worse than no pet. So the gate has four verdicts, not one.

## Findings log

Append to this as the spike runs. It is the record that goes back into the spec.

**1. The wall is real, and it is not a typo.** `AOT_COMBO=1` (level 25
`NSStatusWindowLevel`, collectionBehavior 273 = `canJoinAllSpaces|stationary|fullScreenAuxiliary`)
puts the dot on the desktop fine, and a fullscreen Chrome window hides it completely.

**2. AppKit accepted the values.** The read-back reports `level=25 behavior=273
visible=true onActiveSpace=true`, so this is not a silently rejected `collectionBehavior`
from mixing two options in one group. The configuration sticks and is simply insufficient.
That rules out the cheapest possible explanation and means the fix, if there is one, is a
different mechanism rather than a corrected constant.

**3. `fullScreenAuxiliary` is probably the wrong tool.** Apple documents it as letting a
window show alongside a fullscreen window *of the same application*. Chrome's fullscreen
Space belongs to Chrome, so nothing about the pet's own app is auxiliary to it. If anything
works here it will be level plus `canJoinAllSpaces`, or the NSPanel route below.

**4. No NSWindow configuration works.** Ten `collectionBehavior` values across four window
levels, up to `kCGMaximumWindowLevel`, all invisible over a fullscreen Chrome window while
`isVisible` and `isOnActiveSpace` both reported true the whole time.

**5. The NSPanel class swap is necessary.** `object_setClass` to `NSPanel` plus
`NSWindowStyleMaskNonactivatingPanel`, and the dot appears over fullscreen Chrome. It does
not crash, and the webview keeps rendering. This is the load-bearing finding.

**6. Placement must respect the Dock, and `work_area` will not do it for you.** Tauri's
`Monitor::work_area()` returned `pos=(0, 62) size=(6720, 2774)` on a 6720x2836 display: the
62px is the menu bar, and the bottom edge is the full screen height. The Dock band is not
excluded. A pet placed 24px from the bottom of either rect sits inside the Dock, which draws
at window level 20, so it is invisible at Tauri's default floating level 3. The real pet
needs an explicit Dock inset, not just the work area.

**7. `occlusionState` cannot be trusted, and neither can the other three.** Through an
entire run where nothing was on screen, `isVisible`, `isOnActiveSpace`, and `occlusionState`
all reported healthy. What actually found the problem was logging the window's position and
comparing it against the monitor rect. When the real pet misbehaves, measure geometry first.

**8. Applying configurations to a live window is contaminated by history.** This invalidated
two rounds of testing and is the most important process lesson here.

Cycling combos on one running window seemed efficient: one launch, one lap, many answers.
It is not sound. Level 25 with behavior 17 was **invisible** over fullscreen when the lap
never went above level 25, and **visible** over fullscreen in a later lap that passed
through level 1000 and `fullScreenAuxiliary` first. Same window, same config, opposite
result, decided by what had been applied minutes earlier.

So the panel appears to acquire fullscreen-Space presence once a strong enough configuration
has been applied, and to keep it after the configuration is dialled back. Whatever the
mechanism, the consequence for method is absolute: **every candidate must be measured in a
fresh process, as a single static configuration.** Cycling is only good for the first
question, "does anything work at all". It cannot answer "what is the minimum", which is the
question that decides what goes into production.

Corollary for the product: whatever recipe wins must be applied **once at window creation**
and never adjusted at runtime, because runtime adjustment is exactly the regime where
behaviour turned out to be history-dependent and therefore unpredictable.

## Run it

```sh
cd spikes/always-on-top
cargo run
```

A circle appears 24px in from the bottom-right corner, showing a digit: the index of the
active combo. Ctrl-C to quit.

### The two sweeps

The terminal is invisible while another app owns the screen, so the dot reports its own
configuration. Each combo has a distinct colour and digit. Go fullscreen, watch the corner
for one full 40-second lap, and note which colour appears.

```sh
AOT_CYCLE=1 cargo run              # sweep 1: eight level/behavior combos, as an NSWindow
AOT_PANEL=1 AOT_CYCLE=1 cargo run  # sweep 2: the same eight, as a non-activating NSPanel
```

| # | Colour | Level | Behavior |
|---|---|---|---|
| 1 | magenta | 25 status | `canJoinAllSpaces\|stationary\|fullScreenAuxiliary` |
| 2 | cyan | 1000 screenSaver | same |
| 3 | yellow | 1000 screenSaver | `canJoinAllSpaces\|stationary` |
| 4 | lime | 1000 screenSaver | `canJoinAllSpaces` only |
| 5 | orange | 1000 screenSaver | `canJoinAllSpaces\|fullScreenAuxiliary` |
| 6 | red | 1000 screenSaver | `+ignoresCycle` |
| 7 | white | kCGMaximumWindowLevel | as #1 |
| 8 | blue | 3 floating | as #1. **Negative control**: what Tauri gives you unaided. |
| 9 | purple | 1000 screenSaver | `canJoinAllSpaces\|stationary\|fullScreenNone` |
| 10 | spring green | 1000 screenSaver | `moveToActiveSpace\|stationary` |

If sweep 1 finds a winner, confirm it with a fresh single-shot launch, because a combo
applied to a live window is not identical to one applied at creation:

```sh
AOT_COMBO=3 cargo run
```

`AOT_LEVEL` and `AOT_BEHAVIOR` still override any combo, for values not in the table.

### Sweep 2: the NSPanel route

`AOT_PANEL=1` swaps the NSWindow's class for `NSPanel` via `object_setClass` and adds
`NSWindowStyleMaskNonactivatingPanel`. This is what the `tauri-nspanel` community plugin
does, and it is how Spotlight-style HUDs manage to float over fullscreen apps: an NSPanel
can be shown without activating its application, which is the property a plain NSWindow
does not have. It is the most likely thing to work, and also the most invasive, which is
why it is sweep 2 rather than sweep 1.

If only the NSPanel route works, that is a real cost to record: the pet needs a class swap
on a Tauri-owned window, either by hand or by taking on `tauri-nspanel` as a dependency.
Section 10's "one codebase, no platform-specific escape hatches" gets a documented exception.

### Reading the log after the fact

The reporter thread prints `level`, `behavior`, `visible`, and `onActiveSpace` every two
seconds in every mode. So the sequence can be reconstructed afterwards: if `onActiveSpace`
goes `false` the moment Chrome enters fullscreen, macOS is excluding the window from that
Space, which is a different failure from the window being present but drawn underneath.

## Checklist

Fullscreen means the green-button fullscreen Space, not a maximised window. Use a second app
(Safari, Terminal, an editor) rather than the spike's own window.

### 1. Visibility (the wall itself)

- [ ] Dot visible on the normal desktop
- [ ] Dot visible over a fullscreen app
- [ ] Dot visible over fullscreen video (Safari or YouTube fullscreen, a different code path)
- [ ] Dot visible over a Split View pair
- [ ] Dot stays put when swiping between Spaces (does not slide with the Space)

### 2. Not hostile (the part that fails quietly)

- [ ] Clicking the dot while a fullscreen app is frontmost does **not** switch Spaces
- [ ] Clicking the dot does not steal keyboard focus from the fullscreen app
- [ ] The dot does not cover the menu bar when it reveals on hover
- [ ] Cmd-Tab still behaves normally, and the spike is not in the Cmd-Tab list
- [ ] No Dock icon (default run, `AOT_DOCK` unset)

### 3. Transparency

- [ ] A circle, not a square. A square means `transparent: true` did not take effect.
- [ ] No drop shadow rectangle around it

Note: `transparent: true` on macOS needs the `macos-private-api` Cargo feature and
`macOSPrivateApi: true` in `tauri.conf.json`, both already set here. That flag makes the app
ineligible for the Mac App Store. Direct distribution is unaffected. If the App Store is ever
a target, the pet has to be an opaque square instead, which is a design problem, not a bug.

### 4. Environment interactions

- [ ] Mission Control: dot behaves sanely (hidden or floating, not duplicated per Space)
- [ ] Stage Manager on: dot still visible
- [ ] Second monitor: dot stays on the monitor it was placed on
- [ ] Screenshot (`Cmd-Shift-4`) captures the dot, which matters for Share Status
- [ ] Screen sharing / recording shows the dot

Out of scope: games that take an exclusive display via `CGDisplayCapture` will cover
everything, and nothing can be done about that. Note it in the README, do not chase it.

## Objective check, instead of squinting

Eyeballing a small dot over a busy screen is unreliable. This counts pure-magenta pixels in a
full-screen capture, so the answer is a number:

```sh
shot=$(mktemp -t aot).png
screencapture -x "$shot"
magick "$shot" -fuzz 6% -fill white -opaque '#ff00ff' -fill black +opaque white \
  -format '%[fx:int(mean*w*h)] magenta px\n' info:
```

Zero means not visible. A few thousand means visible. Run it once on the desktop to get a
baseline, then again from a fullscreen Space.

`screencapture` needs Screen Recording permission for the terminal, and will prompt once on
first use. Granting it to a terminal is a real permission; skip this section and check by eye
if that trade is not worth it.

## Verdict: PASS

| | |
|---|---|
| Date | 2026-08-12 |
| Display | 6720x2836 at scale 2, plus a second 3200x2000 |
| Window kind | **`NSPanel`**, via `object_setClass` plus `NSWindowStyleMaskNonactivatingPanel` |
| Level | **25** (`NSStatusWindowLevel`) |
| `collectionBehavior` | **273** (`canJoinAllSpaces \| stationary \| fullScreenAuxiliary`) |
| Visible over fullscreen Chrome | **yes** |
| Hostile on click | **no**: stays in the Space, does not steal focus, Chrome stays interactive |
| Transparency works | **yes**, a circle with transparent corners |

Measured as a single static configuration in a cold process, which after finding 8 is the
only measurement that means anything.

**The pet is the primary surface as specified.** Tray icons stay monochrome template images
(section 6.2). Section 6.1's "clickable, not click-through" stands.

## What goes into production

Port these, do not rediscover them.

1. **The `NSPanel` conversion.** `make_panel()` in `src/main.rs`, hand-rolled rather than
   taking `tauri-nspanel`, for the reasons in spec section 10.3.
2. **Level 25 and `collectionBehavior` 273**, applied **once at window creation** and never
   adjusted at runtime. Finding 8 is why.
3. **A Dock-aware inset.** `work_area()` alone puts the pet under the Dock. Finding 6.
4. **`-webkit-user-select: none`** on the pet's webview, plus no context menu and no drag.
   Plain `user-select` is not honoured in WKWebView, so the character highlights blue on
   click, which reads as broken. Found by clicking the spike.
5. **`macos-private-api` and `macOSPrivateApi: true`** for `transparent: true`. Mac App Store
   ineligibility is accepted; distribution is direct.

## Not tested, and deliberately so

These were on the original checklist and are not worth another round now that both gate
verdicts have passed. They are cheap to check once the real pet exists, and none of them can
reverse a design decision:

- Mission Control and Stage Manager behaviour
- Second monitor, and what happens when the display configuration changes
- Whether `Cmd-Shift-4` and screen recording capture the pet
- Fullscreen video as a separate code path from a fullscreen Space
- Games that take an exclusive display via `CGDisplayCapture`, which nothing can float over.
  Note it in the README, do not chase it.

## Disposal

The spike has served its purpose. Delete `spikes/always-on-top/` once the AppKit block is
ported, and **keep this file**, moving it to `docs/` if that reads better. The dead ends are
the valuable part: if the pet stops appearing over fullscreen on some future macOS release,
findings 4 through 8 turn a day of re-exploration into an afternoon.
