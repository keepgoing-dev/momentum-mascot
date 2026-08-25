# App Store Connect listing: Momentum Mascot

Every field, so the listing is reviewable here rather than only in a web form. Update this
file when the listing changes.

The one-time account setup is `docs/app-store.md`. The build and upload script is
`tools/release-mas.sh`.

## Basics

| Field | Value |
|---|---|
| Name | Momentum Mascot |
| Subtitle | A pixel pet for side projects |
| Version | 0.3.1 |
| Price | Free |
| Primary category | Developer Tools |
| Secondary category | none |
| Bundle ID | dev.keepgoing.momentum-mascot |
| SKU | momentum-mascot-1 |
| Copyright | 2026 Hoa Trinh |
| Support URL | https://keepgoing.dev |
| Marketing URL | https://keepgoing.dev |
| Privacy Policy URL | https://keepgoing.dev/privacy |

Name is 15 characters and subtitle 29, against Apple's limit of 30 for each.

**The version has to match the build.** App Store Connect attaches a build to a version by
`CFBundleShortVersionString`, so the record was created at 1.0 and edited down to 0.3.1 to
match what ships. Version numbers live in `src-tauri/tauri.conf.json` and are bumped only
by `tools/release.sh`; `tools/release-mas.sh` deliberately has no version bumping in it, so
that the two channels cannot disagree about what a version is.

**Developer Tools is doing review work, not decoration.** Guideline 4.2 says "If your app
is not particularly useful, unique, or 'app-like,' it doesn't belong on the App Store", and
an ambient desktop pet reads as a toy in Utilities and as a tool in Developer Tools. The
bundle agrees: `tauri.conf.json` sets `"category": "DeveloperTool"`, which lands as
`LSApplicationCategoryType = public.app-category.developer-tools`. If one of the two ever
changes, change both.

## Keywords

72 characters against a limit of 100.

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

"Three characters" is checked against `store::CHARACTERS`, which is `07`, `12` and `20`.

## Privacy answers: Data Not Collected, every category

Apple: "'Collect' refers to transmitting data off the device", and "data that is processed
only on device is not 'collected' and does not need to be disclosed." Reading the user's
filesystem is emphatically not collection.

The one caveat to keep in view: "if you derive anything from that data and send it off
device, the resulting data should be considered separately." The share card puts derived
data on the **clipboard**, not off device, so it stays clear. If a future release ever
posts a card anywhere, this answer changes.

**Answered in App Store Connect on 25 August 2026.** The App Privacy section reads "Data Not
Collected", Data Types reads "Data is not collected from this app", and the Privacy Policy URL
is set to `https://keepgoing.dev/privacy`.

The Privacy Policy URL must resolve before the version can be submitted. Note that
keepgoing.dev serves a page for unknown paths, so check it by content and not by status
code:

```sh
curl -sS https://keepgoing.dev/privacy | grep -q "<title>Privacy Policy" && echo live
```

Verified live on 25 August 2026: HTTP 200 and the right title. `/privacy.html` returns a
308 to the clean path, which is Cloudflare Pages normalising and not a problem.

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

The app makes no network requests. It has no accounts, no telemetry and no server of any
kind. It reads only the reflog and file modification times of the folders you add through
the picker, and stores its state in one JSON file in its own container.

The bundle does carry com.apple.security.network.client, and that is not a contradiction.
The entitlement is required for WKWebView to reach its own networking process: without it,
a sandboxed webview never finishes navigation, so the popover renders blank and the app
appears broken, with no sandbox violation logged. It grants WebKit that access; the app
itself issues no requests.

Category: Developer Tools. The app's audience is developers with side projects, and the
signal it reads is a git reflog.
```

**The network entitlement paragraph is load-bearing.** An earlier draft of these notes said
the app "makes no network requests of any kind" while the signed bundle shipped
`com.apple.security.network.client`, which is the kind of mismatch a reviewer is entitled to
read as a false statement. The entitlement is mandatory and measured: Probe 1 found that
without it the sandboxed webview silently never finishes navigation. So the answer is to
explain it, not to remove it and not to leave it unmentioned.

## Screenshots

2560x1600, five of them, in this order:

1. The pet on a desktop, in the bottom-right corner, over a real wallpaper. The product's face.
2. The popover open with the room in the **awake** state, showing two or three tracked projects.
3. The popover in **dozing**.
4. The popover in **comeback**, which is the moment the whole product exists for.
5. The share card at full size.

### The two scripts, and why not `drive-states.sh`

An earlier draft of this section pointed at `tools/drive-states.sh`. That is the wrong tool.
It runs the whole arc past you on a 3600x clock, which is exactly right for watching the
transitions and exactly wrong for a photograph: the state you want is on screen for seconds,
and the clock keeps moving while you frame the shot.

**`tools/hold-state.sh awake|dozing|asleep|comeback`** holds one state still instead. It
leaves the clock at 1x and backdates the commits, so `dozing` staged at 30 hours has 42 hours
of headroom before it falls asleep and `awake` staged at 12 minutes has most of a day. The
timestamp it writes is the **reflog entry's**, through `GIT_COMMITTER_DATE`, because that is
the one the app reads; `--date` would change the author date and nothing else. It builds
throwaway repositories with no working-tree files at all, which matters more than it looks:
the mood is the max of `last_commit_at` and `last_active_at`, and a file modified five minutes
ago would pin every state to awake.

The comeback cannot be staged by a timestamp, because it is a transition rather than a
resting state. `hold-state.sh` seeds `last_displayed_state: "asleep"` in the throwaway state
file and lets a fresh commit complete the pair, which is the same path as the restart case in
spec section 4.5.

**`tools/store-shots.sh`** produces the files, and it exists for one reason: a screenshot of
pixel art that has been through a resize is not a smaller problem than a wrong screenshot.
Soft edges are the most visible way for this listing to look amateur. So nothing in it ever
resizes a screenshot. A 2x display's native capture is already twice the logical size,
2560x1600 is exactly 1280x800 of logical space, and a **crop is not a resize**, so every shot
is a full-display capture cropped at an integer offset.

The one thing it does scale is the share card, which is 1200x630 and has to reach 2560x1600.
It goes up by exactly 2 with a point filter and is matted on the card's own background
colour. Measured rather than asserted: the source card has 147 distinct colours and the 2x
result has 147, so no pixel was blended.

### The popover had to be pinned to be photographed at all

The popover closes when it loses focus, which is right, and every way of triggering a screen
capture takes the focus first: the shift-cmd-5 panel is an app, and a capture invoked from a
terminal makes the terminal frontmost. So shots 2, 3 and 4 were not awkward to take, they were
unobtainable, and 4 doubly so because closing the popover is also what resolves the
celebration.

`KEEPGOING_PIN_POPOVER` skips the hide-on-focus-loss, and `hold-state.sh` sets it. Escape and
the tray icon still close the popover, so there is always a way out. Debug builds only, for
the same reason as the clock and the state path, and measured rather than asserted: the release
binary contains the string zero times, exactly like `KEEPGOING_CLOCK_SCALE` and
`KEEPGOING_MASCOT_STATE`.

### The five commands

Shoot against a plain desktop. A full-screen browser or editor behind the popover makes a busy
screenshot and puts someone else's interface in the listing.

```sh
tools/hold-state.sh awake                              # then drag the pet to the corner
tools/store-shots.sh grab 1 pet br

tools/hold-state.sh awake                              # open the popover from the menu bar
tools/store-shots.sh grab 2 awake tr

tools/hold-state.sh dozing
tools/store-shots.sh grab 3 dozing tr

tools/hold-state.sh comeback                           # 30 real minutes, see below
tools/store-shots.sh grab 4 comeback tr

# Click Share Status, which puts the card on the clipboard, then:
tools/store-shots.sh card --clip 5 card

tools/store-shots.sh check                             # every file exactly 2560x1600
```

`grab` needs Screen & System Audio Recording permission for the terminal. Without it
`screencapture` fails with "could not create image from display", which reads like a bug and
is a permission. The way round it needs no permission change: take the shot with shift-cmd-5,
choosing the whole display rather than a region, and hand the file over.

```sh
tools/store-shots.sh crop ~/Desktop/'Screenshot ....png' 3 dozing tr
```

Output lands in `docs/store-shots/`, gitignored for the same reason `docs/mockups/` is.

### The comeback shot needed a fix to the app

Shot 4 was not obtainable when this was first attempted, and the reason was a real defect
rather than a tooling problem. `app.rs::show_popover` resolved the comeback and *then* called
`publish`, so the room evaluated as `awake` in the very call that was meant to show the
celebration. The pet celebrated correctly; the popover, which spec section 4.5 calls "the
screenshottable version", could never display a comeback at all.

Section 4.5 puts the resolution on close: "the user sees the full-room celebration, and on
close it settles into `awake`." The resolution moved to `note_popover_hidden`, which every
close path already goes through, and the regression test is
`momentum::tests::closing_the_popover_resolves_it_early`.

Worth noting how it stayed hidden: the unit test at the momentum layer passed before and
after, because the defect was the ordering of two calls in `app.rs` and not the behaviour of
either one. It took needing the screenshot to find it.

### Check each shot at 100%

The scripts guarantee the pixel dimensions and that nothing was resampled by them. They cannot
tell you that the popover is fully inside the crop, that the pet landed in the corner, or that
the wallpaper behind shot 1 is not distracting. Look at all five.

Screenshots are derived LimeZu art, so they are covered by the licence check in
`docs/app-store-licence-check.md`: uploading them to the listing is presentation of the
app, not redistribution of the asset pack.

## If review pushes back on 4.2

The answer is the review notes and the category, not new features. Appeal explaining that
an ambient status indicator for developers is the whole product and that its restraint is
deliberate. Do not add scope to satisfy a guess about App Review's appetite.

## Submission log

A row for every outcome, including rejections and what they cited. This log is the actual
deliverable of the project: the point was to learn the process end to end, and a rejection
reason is worth more than a clean pass.

| Date | Version | Build | Result |
|---|---|---|---|
| 2026-08-25 | 0.3.1 | 3 | validated only, not uploaded: `VERIFY SUCCEEDED with no errors, 1 warning` (90889, TestFlight profile) |
