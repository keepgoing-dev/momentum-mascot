# App Store Connect listing: Momentum Mascot

Every field, so the listing is reviewable here rather than only in a web form. Update this
file when the listing changes.

The one-time account setup is `docs/app-store.md`. The build and upload script is
`tools/release-mas.sh`.

**Every fenced block in this file is dropped verbatim into an App Store Connect field, so a
line break inside one is a line break the reader sees.** Prose is therefore left unwrapped
here, against this repository's habit everywhere else, and the lines run long. It is not a
style lapse: the 0.3.1 description shipped with the markdown source's 90-column wraps still
in it, which broke every paragraph at an arbitrary word and then let the App Store window
wrap it again on top.

## Basics

| Field | Value |
|---|---|
| Name | Momentum Mascot |
| Subtitle | A pixel pet for side projects |
| Version | 0.4.0 |
| Price | Free |
| Primary category | Developer Tools |
| Secondary category | none |
| Bundle ID | dev.keepgoing.momentum-mascot |
| Apple ID (app) | 6804925509 |
| App Store URL | https://apps.apple.com/app/momentum-mascot/id6804925509 |
| SKU | momentum-mascot-1 |
| Copyright | 2026 Hoa Trinh |
| Support URL | https://keepgoing.dev |
| Marketing URL | https://keepgoing.dev |
| Privacy Policy URL | https://keepgoing.dev/privacy |

Name is 15 characters and subtitle 29, against Apple's limit of 30 for each.

**Support URL takes http or https and nothing else.** Both `hello@keepgoing.dev` and
`mailto:hello@keepgoing.dev` are refused with "The URL is formatted incorrectly. URLs must be
formatted as: http://example.com", tried on 31 August 2026. So the support email lives on the
page rather than in the field, which is the better answer anyway: the App Store renders this
as a "Support" link on the product page, and a `mailto:` there opens an empty message with no
context, or nothing at all on a Mac with no mail client configured.

Guideline 1.5 wants contact information reachable **at** that URL, and until 31 August 2026 it
was not: the landing page carried credits and a privacy link and no way to contact anyone, and
the only address anywhere was an issue tracker link on the privacy page, which needs a GitHub
account. `hello@keepgoing.dev` now sits in the footer of both pages.

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

## Promotional Text

162 characters against a limit of 170.

```
For the side projects you keep coming back to. A pixel character who waits instead of nagging: no streaks, no scores, and never a word about how long it has been.
```

**It does not carry across to a new version, so every draft starts empty.** Measured on 31 August
2026: the live 0.3.1 localization held all 162 characters and the 0.3.2 draft held zero. App
Store Connect copies the description and the keywords into a new version and does not copy this,
so shipping a version as it stands deletes the line from the listing rather than changing it.
Nothing warns about that at submission, because an empty promotional text is a legal listing.
Re-enter it on every version, 0.4.0 included, and read it back rather than trusting the panel.

**This is the only field in the listing that can change without a new build.** Apple's own
description of it: it informs visitors of current features "without requiring an updated
submission". Everything else here is frozen until the next upload, so this is the one place to
say something that might need saying later.

It sits directly above the Description, which is what ruled out the obvious drafts. Three of
four candidates opened by restating "a retro pixel character who lives in a tiny room on your
desktop and reflects how your side projects are going", which is the Description's own first
sentence, two lines below. Spending the one changeable field on a line the reader is about to
read anyway is the waste worth avoiding. This one leads with the audience and the
anti-features, which the Description does not reach until "WHAT IT IS NOT".

## App Previews: one, shipped with 0.3.2

Optional, up to three, and video rather than stills. Skipped for 0.3.1, because the point of
that phase was to walk the process end to end and a preview delays that without changing
whether the app is approved. One `DESKTOP` set went up with 0.3.2 on 31 August 2026.

**The argument for making one is in the spec**, written about the share card and
truer of the listing: "A still frame cannot show a transition, and this product *is* a
transition." The listing is where a stranger decides, and no screenshot at any count can show
a character getting out of bed.

One measured constraint for whoever does it. `tools/drive-states.sh` is the obvious recording
tool and **cannot produce a full-arc take inside an app preview's length at any clock scale.**
Its comeback commit lands at `AT_ASLEEP + 30`, and the 30 is fixed wall-clock seconds on
purpose, because it is how long a person gets to look at the blanket rather than a quantity
derived from git. So the commit never arrives before the 30 second mark however fast the clock
runs:

```
scale       3600:  dozing   24.0s  asleep   72.0s  commit  102.0s
scale      10000:  dozing    8.6s  asleep   25.9s  commit   55.9s
scale     100000:  dozing    0.9s  asleep    2.6s  commit   32.6s
```

Two ways round it: edit that constant for the recording, or record only asleep to comeback,
which is the payload anyway.

**Two encoding requirements a QuickTime screen recording does not meet.** Both come from
Apple's stated App Preview specification rather than from a rejection, because the file was fixed
before it was ever offered to Media Manager. Apple asks for an **exact** frame rate, and a screen
recording comes out at `30000/1001`, which is 29.97 and not 30. It also asks for an audio track,
and a recording of a silent app has none. So the take needs a re-encode:

```sh
ffmpeg -i take.mov -r 30 -c:v libx264 -pix_fmt yuv420p \
  -f lavfi -i anullsrc=channel_layout=stereo:sample_rate=44100 \
  -c:a aac -shortest preview.mp4
```

What went up: 1920x1080, `30/1`, an AAC track, 17.07 seconds, 434795 bytes. 1920x1080 is
accepted for a macOS preview and is not the same constraint as the screenshots' exact 2560x1600.
Confirm the re-encode did not damage the pixel art rather than assuming, because a preview of a
pixel-art app that has been resampled is worse than no preview: `ffmpeg` with the `ssim` filter
against the original measured 0.999016 mean, with 6 frames of 512 under 0.99 and both clusters
sitting on hard cuts, which is where SSIM is expected to dip. Use `LC_ALL=C` when reading that
number through `awk`; a comma decimal separator parses as zero and reports a perfect failure.

**`tools/preview-take.sh` is the second way, built.** You start the screen recording and it acts
out the whole thing: a real asleep, the pointer travelling to the pet and clicking it, the night
room, a real commit into a throwaway repository, the room turning into the comeback, Escape, the
pet awake alone. Twenty-two seconds on camera, inside the thirty.

`hold-state.sh comeback` is the wrong tool for it, despite staging a comeback instantly: what it
stages has already fired and is pinned open, which is right for a photographer and useless for a
video, where the transition *is* the subject. So the take stages asleep and fires the comeback
mid-recording instead. Verified end to end before the first take: `120.00h -> asleep` at launch,
`0.00h -> comeback` about a third of a second after the commit, which is `watcher.rs`'s 250 ms
debounce and not a poll.

**Building it found three shipped bugs**, which is the argument for building tools that drive
the real app rather than mocking it, and all three were silent by construction.

The popover was opening in the centre of the screen whenever the mascot, rather than the menu
bar icon, was the first thing clicked: the icon's position was only ever learned from the icon's
own click event. Anchoring it to the icon on demand fixed that and exposed the second one, which
only exists on a desktop with two displays: macOS moves the menu bar's status items to whichever
display is active, so a click on the mascot in the corner of one screen opened the popover on
the other one. The panel now hangs off whatever was clicked, which is the only rule that is
right for both ways in. A take recorded on the laptop while the terminal was on the second
monitor is what surfaced it, and it looked perfectly fine from the terminal.

Third, a project tracked through a symbolic link was never watched at all - FSEvents reports
resolved paths, the watcher compared them against the path as given, and the project read as
healthy while silently never recording another commit. `mktemp -d` hands back a path under
`/var`, which is a symlink, so the take staged itself into exactly the blind spot.

Nothing about it is faked. The only debug overrides are the state file and the popover pin; the
commit is a real commit and `src/popover.js:211` redraws an already-open popover on the `mood`
event, so the room changes with nobody touching the machine. It needs Accessibility permission
for the terminal that runs it, which it checks before staging anything rather than dying halfway
through a take, and it rehearses the pointer's path to the pet while there is still a terminal to
read the warning on.

## Keywords

72 characters against a limit of 100.

```
git,commit,pixel,pet,mascot,desktop,menubar,side project,momentum,reflog
```

## Description

```
Momentum Mascot is a retro pixel character who lives in a tiny room on your desktop and reflects how your side projects are going.

Point it at the git repositories you care about. It reads exactly one thing from each: when you last actually committed. Not messages, not diffs, not branch names. Commit something and the character is at their desk. Take a few days off, and they doze, then sleep, still holding your place. Come back after a long silence, and they leap out of bed.

The mascot never dies. It waits.

WHAT YOU GET

- A 64x64 desktop pet in the corner of your screen, visible over fullscreen apps, draggable to any corner.
- A full animated room in a popover, opened from the mascot or the menu bar icon, with the character and a line of copy that never scolds you.
- Three characters to choose from, or build your own: a skin tone, eyes, a hairstyle and its colour, an outfit and its colour, and one accessory. The mascot you build lives everywhere a premade does, including the desktop pet and the share card.
- A 1200x630 share card copied to your clipboard, carrying the room and the mood and nothing that identifies a project.
- Operating mode, for projects that run without commits: they keep their place in your list, and the mascot ignores them.

WHAT IT IS NOT

It is not a productivity tool. There are no streaks, no scores, no notifications, no leaderboards, and nothing in it will ever tell you how long it has been since you last committed. It is a small companion for people with demanding day jobs who go through long stretches where nothing gets committed because life is happening.

PRIVACY

No network requests. No accounts, no sign-in, no telemetry, no cloud, no sync. Everything lives in one JSON file inside the app's own container, which you can read or delete.

Art by LimeZu (limezu.itch.io). Type: Departure Mono, OFL 1.1.
```

"Three characters" is checked against `store::CHARACTERS`, which is `07`, `12` and `20`. The
builder's own counts are checked against `src/assets/layers/index.json`, which is the shipped
palette rather than the pack's full library: 9 skin tones, 7 eyes, 14 hairstyles in 7 colours,
13 outfits in 4 colours and 42 accessories.

**The build-your-own clause is the 0.4.0 correction, and it goes in this bullet rather than in
a new one.** It is the reason for the version and it is what the seventh screenshot shows, so
the pull is to give it its own line near the top of WHAT YOU GET. That would separate it from
the sentence it modifies: a reader who meets "build your own" before "three characters to
choose from" has to hold an open question through the rest of the list. One bullet says the
whole thing once.

**This block is the live 0.3.1 text, not the draft it was written from.** Three commas were
typed straight into App Store Connect at submission (`off,`, `silence,`, `list,`) and never
came back here, so for four days the file that calls itself every field was wrong about three
of them. Read the shipping text out of the API before editing this block, rather than assuming
the file is ahead of the store:

```sh
set -a; . tools/.release-env; set +a
JWT=$(xcrun altool --generate-jwt --apiKey "$ASC_API_KEY_ID" --apiIssuer "$ASC_API_ISSUER_ID" \
  2>&1 | grep -o 'eyJ[A-Za-z0-9._-]*' | head -1)
VID=$(curl -s -H "Authorization: Bearer $JWT" \
  'https://api.appstoreconnect.apple.com/v1/apps/6804925509/appStoreVersions?limit=5' \
  | python3 -c 'import json,sys; [print(v["id"], v["attributes"]["versionString"], v["attributes"]["appStoreState"]) for v in json.load(sys.stdin)["data"]]')
curl -s -H "Authorization: Bearer $JWT" \
  "https://api.appstoreconnect.apple.com/v1/appStoreVersions/<id>/appStoreVersionLocalizations"
```

**"opened from the mascot or the menu bar icon" is a 0.3.2 correction.** The line read "in a
popover from the menu bar", which was true when it was written and is now half the story: the
mascot is the primary way in (spec 6.1), and since 0.3.2 the panel hangs off whichever of the
two was clicked rather than always off the icon. A description that names only the menu bar
sends the reader to the wrong corner of their own screen.

## What's New in This Version, for 0.4.0

**Required on every version after the first**, per-version, and frozen at upload. Unlike
Promotional Text, which is the one field above that can change at any time.

```
Build your own mascot.

The character picker has a fourth slot, marked +. Open it and you get a builder inside the popover: a skin tone, eyes, a hairstyle and its colour, an outfit and its colour, and one accessory. Nine skin tones, seven pairs of eyes, fourteen hairstyles in seven colours, thirteen outfits in four colours, and forty-two accessories. There is a Shuffle button for when nothing in particular comes to mind.

The room is the preview, so you are looking at your mascot in the place they are going to live rather than at a form. Once you are done they are a character like any of the three that ship with the app: they are in the room, they are the pet on your desktop, they are on the share card, and clicking the character cycles through them along with the rest.

Also in this version:

- The mascot no longer disappears when you drag it off the edge of a screen. It used to keep moving after the pointer had stopped against the edge, and once it was clear of every display it stayed there invisibly, with reopening the app the only way to get it back. It now runs home to the nearest corner of the display it left.
- On more than one display, the mascot's corners are measured from the display it is actually on, so a mascot on a second screen no longer runs to a corner of the first one.
- The menu bar icon's right-click menu has a Support item. Nothing inside the app said where to report a bug or ask for help.
```

1433 characters against a limit of 4000.

**The builder gets three paragraphs and the fixes get three bullets, which is the opposite of
the 0.3.2 split.** That version led with three watcher fixes sharing one paragraph, because
they were one fault to a user and the feature work in it was small. Here the proportion
inverts: the drag fix matters to anyone who hit it, but it is a repair, and a repair does not
need the top of a field that a stranger reads to decide whether to update.

**The counts are spelled out rather than written as digits**, against this file's habit
elsewhere, because the field renders as one prose block on the product page and a run of
numerals in the middle of it reads as a spec sheet. The digits stay in `CHANGELOG.md`, where
the reader is looking for exactly that.

**"They" for the mascot, throughout.** The app has three premades and a builder with nine skin
tones in it, and the whole point of the feature is that the character is whoever the user
decided they are. Every other field in this listing already avoids gendering them, and this is
the first one where a person is choosing.

## What's New, for 0.3.2, kept as the record

**Required, and new.** 0.3.1 was the first submission, so this field did not exist yet; every
version after the first refuses to submit without it. It is per-version and frozen at upload,
unlike Promotional Text, which is the one field above that can change at any time.

```
The mascot now only wakes up for work you actually did.

Three things were quietly counting as activity when they should not have: the .DS_Store that Finder writes just by looking at a folder, creating or deleting a folder, and anything inside a directory your .gitignore excludes, which meant build output and dependency folders were waking the character up. Any of the three could spend a comeback you had not earned yet, which is the one moment this app exists for.

Also in this version:

- The popover opens next to whatever you clicked: above the mascot, or under the menu bar icon. It used to open in the middle of the screen, and on a two-display desk it could open on the screen you were not looking at.
- The popover shows over fullscreen apps. The mascot was already visible there, so clicking it opened a panel you could not see.
- The app version is on the credit line at the bottom of the popover. There was nowhere in the app to read it, which is the moment you want it.
- A project whose folder path goes through a symbolic link is watched properly. It looked perfectly healthy and silently never recorded another commit.
- A tracked git worktree whose git folder sits outside the folder you picked now says so, instead of looking healthy and never recording a commit.
```

1284 characters against a limit of 4000.

**Written from the CHANGELOG rather than copied from it.** The changelog entry for the
gitignore fix says "files inside a directory your `.gitignore` excludes no longer count as
working on a project", which is precise and means nothing to a reader who has not been told
that a directory watch exists. What reaches them here is the consequence: build output was
waking the character up.

**The three watcher fixes lead, and share one paragraph, because they are one fault to a
user**: the mascot woke up when it should not have. Splitting them into three bullets would
spend the top of the field on three ways of saying the same thing. What earns the space is the
last sentence, which is the only line here that says why any of it matters.

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

## Export compliance: no encryption

App Store Connect asks this on every submission, and for this app the answer is factual rather
than a judgement. The app makes no network requests at all, links no cryptographic library, and
does not use the system's own HTTPS stack either, because it never opens a connection. So it
does not use encryption, exempt or otherwise, and there is no documentation to upload.

`com.apple.security.network.client` in the bundle does not change that answer. The entitlement
exists so WKWebView can reach its own networking process, which is what Probe 1 measured; it
grants a capability the app never exercises.

**The answer belongs in the bundle, not in a dialog.** Build 4 was uploaded without it and
landed as **Missing Compliance**, which blocks submission until someone clicks through a
question in App Store Connect. There is no API for it either: `buildBundles` returns
`403 FORBIDDEN` to this key, so build 4 had to be answered by hand.

`ITSAppUsesNonExemptEncryption` in `src-tauri/Info.plist` answers it at build time, and Tauri
merges that file into the bundle on macOS. Measured on a real bundle rather than assumed:

```
LSUIElement                      true
ITSAppUsesNonExemptEncryption    false
```

So build 4 needed the dialog and build 5 onwards will not.

## The five app-level fields that block Add for Review

"Add for Review" refuses until all of these are set, and it lists them all at once rather
than one at a time. None of them is part of a build, so none of them needs an upload. They
live in four different places, which is the only reason this section exists.

| Field | Where | Answer |
|---|---|---|
| Contact Information | version page, App Review Information | name, phone with country code, email |
| Sign-in required | version page, App Review Information | No: the app has no accounts |
| Content Rights | App Information | Yes, third-party content, rights confirmed |
| App Privacy | App Privacy | Data Not Collected, then **Publish** |
| Price | Pricing and Availability | Free, all countries |
| Age Rating | App Information | every question at its lowest value: 4+ |

**App Privacy has its own Publish button.** Answering the questionnaire saves a draft, and a
draft does not satisfy submission: the blocker still reads "an Admin must provide information
about the app's privacy practices". The answers were entered on 25 August 2026 and the
submission was still blocked on 26 August, which is what published means here.

**The sign-in error does not name the checkbox that causes it.** Leaving Sign-in required on
produces "User name - This field is required" and "Password - This field is required" with no
indication of which section they belong to. They are the demo account fields, they only exist
while that checkbox is ticked, and there is no account to supply: untick it and they go.

**Content Rights is Yes because the room art is licensed, not owned.** The rooms and
characters are the LimeZu Modern Interiors pack. Most games that bundle an asset pack answer
No, on the reading that licensed art is part of the app rather than third-party content the
app shows, and that reading is defensible. Yes is chosen anyway: it is true, it costs one
checkbox and no documentation, and it agrees with the constraint that already shapes the
release scripts, which is that the pack cannot be redistributed and only its composed output
ships.

**Unrestricted web access is No, and that is measured rather than assumed.** The only URL the
app can open is the constant `PRIVACY_POLICY_URL` in `src-tauri/src/commands.rs`, opened in the
default browser through `NSWorkspace`. There is deliberately no `open_url(url)` command, so the
webview cannot open an arbitrary address, and `src-tauri/capabilities/default.json` grants no
shell and no opener plugin. An in-app browser is what that question is about, and there is not
one.

## Attaching the build: the picker only shows a processed build

The Build section stays empty, with no error and no progress indicator, until Apple finishes
processing the upload. That looks identical to a failed upload from the browser. The way to tell
the difference without guessing is the API:

```sh
set -a; . tools/.release-env; set +a
JWT=$(xcrun altool --generate-jwt --apiKey "$ASC_API_KEY_ID" --apiIssuer "$ASC_API_ISSUER_ID" \
  2>&1 | grep -o 'eyJ[A-Za-z0-9._-]*' | head -1)
curl -s -H "Authorization: Bearer $JWT" \
  'https://api.appstoreconnect.apple.com/v1/apps/6804925509/builds?limit=10'
```

`processingState` is the answer: `PROCESSING` means wait, `VALID` means the picker will show it
after a page refresh, `INVALID` or `FAILED` means the upload needs redoing. Build 4 read `VALID`
with `minOsVersion 10.15` about twenty minutes after the upload.

Two traps in that snippet, both hit while writing it. `--generate-jwt` prints the token to
**stderr**, so `2>/dev/null` silently discards the thing being captured and leaves an empty
variable. And the token is a live credential for twenty minutes, so it belongs in a variable and
a header, never in a command line or an echo.

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

2560x1600. Six through 0.3.2, seven from 0.4.0, in this order:

1. The popover in **comeback**, which is the moment the whole product exists for.
2. The **builder** open, mid-choice. New in 0.4.0.
3. The popover open with the room in the **awake** state, showing three tracked projects.
4. The popover in **dozing**.
5. The popover in **asleep**.
6. The pet on a desktop, over a real wallpaper.
7. The share card at full size.

**The builder takes slot 2, and the argument is the thumbnail strip rather than the feature.**
It is the reason for 0.4.0, so the pull is towards slot 1, and slot 1 is not available: comeback
is the moment the product exists for and it was moved there deliberately in 0.3.2. What settles
slot 2 over slot 3 is that shots 1, 3, 4 and 5 are the same panel in front of the same wallpaper,
four rooms apart. A stranger scrolling the gallery sees the first two or three at thumbnail size,
and putting the builder second is the only placement where the second thing they see is not a
near-copy of the first.

**The existing files were renumbered when the builder shot landed.** `2-awake` through `6-card`
each moved up one. The names are a note to the person uploading and nothing else, per the
paragraph below, but a set where two files claim slot 2 is exactly how the 0.3.1 order went
wrong.

**The listing went live with this order wrong, and the file names did not prevent it.** The
shots are numbered `1-comeback` through `6-card` in `docs/store-shots/`, which is the order
argued for below, and the pet shot still ended up in slot 1 on the store. App Store Connect's
media panel orders by the position a file is dropped into, not by its name, and six files
dropped together do not land in name order. So the numbering is a note to the person uploading
and nothing more: after uploading, read the slots back off the panel. Resized to the 640px the
gallery actually shows, shot 5 is a photograph of water with a speck in one corner, which is the
whole argument below arriving as a consequence.

**Apple's limit is ten and its minimum is one.** The App Store Connect panel reads "Drag up to
3 app previews and 10 screenshots here", so ten is a ceiling and not a target. Six was the number
that earned its place through 0.3.2, and the builder is the first addition since that pays for
the dilution: it is a screen the other six cannot imply, rather than a seventh angle on them.

**The pet used to be first, and shooting it is what changed the order.** The old note called it
"the product's face". The pet is 64x64, so 128 physical pixels inside a 2560x1600 frame, which
is five per cent of the width: in a gallery thumbnail it is not visible at all, and the first
shot read as a photograph of the wallpaper it happened to be standing on. The face of this
product is the room. The pet shot still earns its place, because "it lives on your desktop" is
a claim a popover cannot make, but it is a supporting shot and not the opener.

**The wallpaper has to be mid-tone, and both extremes fail for different reasons.** Measured on
27 August by replacing the wallpaper colour behind a real capture and comparing at the 640px the
gallery shows. The sprite is outlined in near-black, so on a black desktop the outline stops
working as an edge: the figure loses its outer boundary and its dark trousers and boots, and what
still reads is the cap, the visor and a patch of shirt, floating. A busy mid-tone wallpaper fails
the other way, which is what the original water shot was. A flat mid-tone, `srgb(77,83,208)` in
the shot that replaced it, keeps the whole figure as one shape with a defined edge. There is a
second reason not to go dark that has nothing to do with the sprite: the App Store product page is
itself dark, so a near-black screenshot loses its own frame against the page.

Put the pet in a corner where the wallpaper is calm. On the wallpaper the first version was shot
against, the bottom-right corner was dense foliage at exactly the pet's scale and the sprite did
not separate from it at all; the bottom-left corner was flat water and it read immediately at full
size, which turned out to be the wrong test.

**Reshot on 27 August, and the fix was the frame rather than the pet.** Flat mid-tone wallpaper
solved the sprite and created a new problem: at gallery size a 2560x1600 frame of nothing but flat
colour reads as a blue rectangle, not as a desktop. So the pet goes in the **top-left** corner and
the crop is `tl`, which puts the **menu bar** in frame. That one dark band is what makes the image
read as somebody's Mac instead of a swatch, and it gives the pet a scale to be small against,
which is the claim the shot exists to make. It costs nothing in privacy on this display: `tl`
takes x=0..2560 of a 6720-wide capture, so the clock, the date, the status icons and the desktop
files all sit outside the frame, and the only menu titles inside it are Finder's.

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
the tray icon still close the popover, so there is always a way out.

That was not enough on its own. The celebration is a thirty minute one-shot and **any** close
ends it, so two attempts at shot 4 were lost to a stray click: the popover reopened showing
`awake`, which is correct behaviour and a useless photograph. `KEEPGOING_HOLD_COMEBACK`
suspends the cap and the resolution, so the room can be opened and closed as often as it takes.
It deliberately does not suspend the third exit: a held comeback still ends the moment the
resting state stops being awake, because a celebration over a project that has gone quiet again
would be a different bug wearing this one's clothes. Asserted in
`momentum::tests::a_held_comeback_outlives_both_the_cap_and_the_close`.

Both are debug builds only, for the same reason as the clock and the state path, and measured
rather than asserted. In the release binary:

```
KEEPGOING_HOLD_COMEBACK  0        PROBE              0
KEEPGOING_PIN_POPOVER    0        drawsBackground    0
KEEPGOING_CLOCK_SCALE    0        fullScreenEnabled  0
KEEPGOING_MASCOT_STATE   0
```

### The commands

**Use `popover`, not `tr`, for the popover shots.** The popover is anchored under the tray
icon, which on a 6718px capture leaves its left edge 161px outside a top-right window, so `tr`
slices the thing being photographed. `popover` measures instead: it masks the panel's own colour
from `src/style.css` and takes the connected component whose width matches the window's known
704 physical pixels, then centres the crop on it. Both halves of that were arrived at by
watching the naive version fail on a photographic wallpaper, and both are worth keeping. A
luminance threshold matched the sea, and a bounding box, even keyed on the right colour, is the
union of every match, so dark foliage on the far side of the screen dragged it across most of
the display.

Screenshots land wherever `defaults read com.apple.screencapture location` says, which is not
always the Desktop.

Shoot against a plain desktop. A full-screen browser or editor behind the popover makes a busy
screenshot and puts someone else's interface in the listing.

```sh
tools/hold-state.sh comeback                           # then open it FROM THE MENU BAR ICON
tools/store-shots.sh clip 1 comeback popover

tools/hold-state.sh awake                              # click +, then the HAIR tab
tools/store-shots.sh clip 2 builder popover

tools/hold-state.sh awake
tools/store-shots.sh clip 3 awake popover

tools/hold-state.sh dozing
tools/store-shots.sh clip 4 dozing popover

tools/hold-state.sh asleep
tools/store-shots.sh clip 5 asleep popover

tools/hold-state.sh awake                              # popover closed, pet in a calm corner
tools/store-shots.sh clip 6 pet bl

# Click Share Status, which puts the card on the clipboard, then:
tools/store-shots.sh card --clip 7 card

tools/store-shots.sh check                             # every file exactly 2560x1600
```

`grab` needs Screen & System Audio Recording permission for the terminal. Without it
`screencapture` fails with "could not create image from display", which reads like a bug and
is a permission. The way round it needs no permission change: take the shot with shift-cmd-5,
choosing the whole display rather than a region, and hand the file over.

```sh
tools/store-shots.sh crop ~/Downloads/'Screenshot ....png' 3 dozing tr
tools/store-shots.sh clip 3 dozing tr                  # ctrl-shift-cmd-3, whole screen
```

**Do not paste a capture into a chat window or a document to move it around.** That path
downscales: a 4002x2768 capture arrived as 2000x1383 that way, which is both too small for the
listing and resampled, the one thing this whole section exists to avoid. The clipboard itself
holds the capture at native resolution, which is why `clip` exists.

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

### The builder shot, for 0.4.0

Stage it with `hold-state.sh awake`, which is what sets `KEEPGOING_PIN_POPOVER`: the builder
lives inside the popover, so it is unphotographable for exactly the reason the four room shots
were. Open the popover **from the menu bar icon**, so the panel lands where shots 1 and 3 to 5
have it, then click the `+` slot to open the builder.

**Open the popover from the menu bar icon.** Not optional here, and not only for consistency
with the other four: `find_popover` solves for X and hardcodes `+0` for Y
(`tools/store-shots.sh:106`), because the panel is assumed to hang under the tray icon. A
popover opened from the mascot lands in the bottom-right corner, and `popover` then frames the
wallpaper it is standing on rather than the app. Doing it by hand does not rescue it: that panel
ends near y=2650 on a 2836-tall capture, so no 1600-tall window holds both it and the menu bar.

**Take it on the WEAR tab, mid-choice rather than empty.** This was HAIR until a real capture
settled it. WEAR draws thirteen whole figures in different outfits, which reads as "choose what
they wear" at the 640px the gallery shows; HAIR draws fourteen near-identical heads separated by
a few pixels of fringe, and EYES is one row. Both tabs draw the same two-row shape, so the grid
is the only thing that distinguishes them and the grid favours WEAR. Choose an outfit before
shooting so a swatch carries the selected outline; an untouched builder photographs as a grid
with nothing happening in it.

**The top of the panel is the preview, and it is not the room.** `builder.js:209` hides the room
and swaps in the untinted `awake-back` plate with the character centred, so there is no blue wash
and nothing in front of them. It fills the room's own box (`popover.css:297`, `inset: 0`), and the
quote, the project rows, the buttons and the character strip all hide with it. So the panel
geometry is identical to shots 1 and 3 to 5 and only what is inside it differs.

**Build something with contrast in it.** The preview is the half of the frame a thumbnail can
read: at 640px the swatch grid is texture and the character is the only thing with a shape. A
strong hair colour and glasses read at gallery size; a subtle build does not.

**There is no zoomed version of this shot, or of any of them.** 2560x1600 is exactly 1280x800 of
logical space on a 2x display, and the popover is 352x540 logical, so the panel is always
704x1080 physical: 27.5% of the frame's width and 67.5% of its height. The only thing a crop
chooses is where the window sits, never how large the panel is inside it. Getting closer would
mean scaling, and nothing in `store-shots.sh` scales a screenshot.

**A `popover` crop now checks itself**, and it was this shot that made it necessary. Two
attempts came back as 2560x1600 files with no app in them, and nothing in the pipeline noticed:
`check` verifies the size, which a crop that missed the subject entirely also has. `crop_to`
asserts the panel is present, whole and centred, prints `panel 698x1068+931+77, centred` when it
is, and deletes the file when it is not. The two historical failures were replayed against it
and both are caught, one as a missing panel and one as a clipped one.

The check runs for `popover` only, because it is the only corner that implies a popover is the
subject. If you fall back to `tools/store-shots.sh clip 2 builder tr` and a manual `+X+Y`,
nothing verifies the result and it has to be opened by eye.

### Taken

All seven, and `store-shots.sh check` reports every one at exactly 2560x1600. The set was
renumbered when the builder shot landed, so file names and slots agree again.

| Slot | File | What is in it |
|---|---|---|
| 1 | `1-comeback.png` | popover, comeback, "I KNEW IT.", rows at 3 minutes / 6 days / 14 days |
| 2 | `2-builder.png` | the builder, WEAR tab, an outfit selected, the mascot in the preview |
| 3 | `3-awake.png` | popover, awake, "Something moved today. That counts." |
| 4 | `4-dozing.png` | popover, dozing, "Taking five. Same here." |
| 5 | `5-asleep.png` | popover, asleep, "Sleeping, not gone. Wake me whenever." |
| 6 | `6-pet.png` | the pet alone, top-left corner, flat mid-tone wallpaper |
| 7 | `7-card.png` | the comeback share card, BACK!!!, at 2x on its mat |

**The panel lands in the same place in all five popover shots, and that is measured rather than
eyeballed.** Masking `#14141c` and taking the connected component 698px wide puts it at `+931+69`
in shot 3 and `+931+77` in shot 2: identical horizontally, four logical pixels apart vertically.
Two earlier attempts at shot 2 failed this and both were opened from the mascot.

**Three cosmetic departures in shot 2, all accepted.** A menu bar glyph is sliced by the right
frame edge, where the older shots happen to end on a whole icon; the menu bar's mean value is 101
against 91 for the other four, because more status items are lit; and the wallpaper is a
different frame of a rotating set. None of the three survives the downscale to the 640px the
gallery shows, and shot 5 already departs from shots 1, 3 and 4 on wallpaper.

**`5-pet.png` was reshot on 27 August** onto a flat mid-tone wallpaper in the top-left corner,
cropped `tl`; the "over water" row above is the original and the reasoning is in the wallpaper
paragraph. The five popover shots are the 25 and 26 August set and do not need retaking for
0.4.0: nothing in this version changes the room, the quote line or the project rows.

The menu bar icon and not the mascot: the popover hangs off whichever of the two opened it, so
opening it from the mascot puts the panel in the bottom-right corner and the four shots no
longer share a frame.

Shots 1 to 4 share a frame deliberately: same wallpaper, same popover position, same menu bar in
view, so the four states read as four states of one thing rather than four screenshots. The
first attempt at shot 1 was taken without the menu bar and had to be redone for that reason
alone.

**`asleep` was missing from the first set, and that was a copy problem rather than a taste
one.** The description says "they doze, then sleep", the site has a section headed "Four moods",
and the screenshots showed three. Promising four and showing three is the kind of gap a reader
notices without being able to name it.

**Operating mode has no screenshot, deliberately.** The description gives it its own bullet, so
it was considered and then measured against `popover.css`: the entire visual difference is one
glyph, `○` becoming `●`, plus the row's timestamp turning the accent colour. There is no label.
Nobody browsing the App Store could infer "this project runs without commits and the mascot
ignores it" from a recoloured dot, so the claim stays in the description where words can carry
it.

### The pet shot is thin, and that is a judgement call rather than a defect

Shot 5 is 95 per cent flat water with a 128 pixel sprite in the corner. Two readings of it are
both true: the emptiness is the product's actual pitch, since the pet sits in a corner and does
not ask for anything, and a screenshot that is mostly wallpaper is a weak screenshot. It is
fifth rather than first for that reason.

The stronger version, if this is ever reshot: the pet over a fullscreen window. The description
claims "visible over fullscreen apps" and no other shot supports that claim. It needs a window
with nothing private in it, which is the only reason it was not done this time.

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

## Verifying what the store actually delivered

`tools/verify-store-copy.sh` runs every part of the spec section 9 list that a script can answer,
against an installed App Store copy, and prints the rest of the list so the two halves do not
drift apart.

**The store copy is the artifact that closes the gap this project could not close before.** The
pkg `release-mas.sh` uploads is signed `3rd Party Mac Developer Installer` and carries no
provisioning profile, so it cannot be installed and launched here; the Developer ID build from
`install-local.sh` runs but is not sandboxed, so it cannot answer a sandbox question. Once a
version is live, one artifact is both the shipped bits and runnable, and it is the one users get.

**It installs under a different name if a local build is in the way.** The store copy landed as
`/Applications/Momentum Mascot 2.app`, because `install-local.sh` already owned
`/Applications/Momentum Mascot.app`. Two copies of the same bundle id then run side by side, and
they do not fight over state: the sandboxed one reads the container, the unsandboxed one reads
`~/.keepgoing/mascot/state.json`. Delete the local build and reinstall from the store to get the
name back, which was done for 0.3.2 and works.

**The collision then happened in the other direction, and the local installers had to give
way.** With 0.3.2 installed from the store, `install-local.sh` failed: it ended in `rm -rf
/Applications/Momentum Mascot.app`, the store copy there is owned by `root`, and the removal was
denied on every file inside it. `sudo` is the wrong answer. That copy is the only artifact that
is both sandboxed and the exact shipped bits, `verify-store-copy.sh` defaults to reading it from
that path, and putting it back is a redownload and a fresh set of bookmarks. So
`tools/replace-app.sh` now owns the install step for both `install-local.sh` and
`install-sandboxed.sh`: it looks for `Contents/_MASReceipt`, and when the store copy holds
`/Applications` it installs to `~/Applications` instead and says so. `APP_DIR` forces a folder,
and pointing it at a store copy is refused rather than escalated. Side by side is the same
arrangement that already worked above, only with the names the other way round.

**Deleting the app does not delete the container.** Measured on 1 September 2026 while doing
exactly that: the local build was removed, 0.3.2 installed fresh from the store, and the script
found three tracked projects with three security-scoped bookmarks already there. macOS leaves
`~/Library/Containers/<id>` behind when an app is dragged to the Trash, so a reinstall inherits
the state rather than starting empty. Good for a user and worth knowing before treating a
delete-and-reinstall as a clean-slate test: it is not one. `Data/.keepgoing` inside the container
is the thing to remove for that, per correction 3 in the plan's execution log.

Run against 0.3.1 build 4, on 27 August 2026, the day it went live:

```
provenance   _MASReceipt present, Authority=Apple Mac OS Application Signing, signature verifies
sandbox      app-sandbox, files.user-selected.read-only, files.bookmarks.app-scope all present
             state file inside the container, 3 projects, 3 with a security-scoped bookmark
private API  drawsBackground / fullScreenEnabled: 0
             KEEPGOING_CLOCK_SCALE, MASCOT_STATE, PIN_POPOVER, HOLD_COMEBACK all absent
bundle       version 0.3.1, build 4, x86_64 arm64, LSUIElement, developer-tools
```

**Two of those lines are new information rather than a repeat of the upload gate.** The private
API check has only ever been run on a binary built here; this is the first time it has been run
on the bits Apple delivered. And `x86_64 arm64` says the store did not thin the binary on the way
through, which nothing in this repository had established either way.

Run again against 0.3.2 build 5, on 1 September 2026, the day after it went live:

```
all mechanical checks passed.
provenance   _MASReceipt present, Authority=Apple Mac OS Application Signing, signature verifies
sandbox      all three entitlements present, state file in the container (4151 bytes)
             3 tracked projects, 3 with a security-scoped bookmark
private API  drawsBackground / fullScreenEnabled: 0, all four debug overrides absent
bundle       version 0.3.2, build 5, x86_64 arm64, LSUIElement, developer-tools
```

**`build 5` rather than `0.3.2` in that last line is the check working.** `release-mas.sh:239`
stamps `CFBundleVersion` with its own counter, so a copy reading `build 0.3.2` is a local build
wearing the store's name. That is what was installed before this run, and reading the two fields
apart is the cheapest way to tell the artifacts apart before trusting anything else the script
says.

**One thing on the by-eye list cannot be photographed, and it is not a gap in the tooling.** The
popover shows the tracked project list, so a capture of it on this machine carries real project
names. The version line added in 0.3.2 therefore has to be read off the screen by the person
sitting at it: `strings` cannot confirm it either, because Tauri compresses the embedded frontend
assets into the binary and none of `src/index.html` survives as a searchable string. Measured on
the store copy: `id="version"`, `limezu.itch.io` and `Add Project` all return 0, which is the
expected result rather than a failure. `getVersion` returns 2, and that is a trap worth naming:
those are the Rust side's `core:app` command name, so they prove the command is compiled in and
say nothing about whether the page calls it.

## The 0.4.0 queue

Written before the build, so the panel session is a list to work through rather than a
discovery exercise. 0.3.1 took six attempts to press Add for Review and none of them were about
the binary; the fix for that is this list.

Everything above is already updated for 0.4.0 and is the source to copy from.

**Before either script runs**

- [ ] Take the builder screenshot and renumber the set. `### The builder shot, for 0.4.0`.
- [ ] `tools/store-shots.sh check` reports seven files at exactly 2560x1600.

**The two scripts, in this order**

- [ ] `tools/release.sh minor`, which is what makes 0.4.0 exist: it bumps
      `tauri.conf.json`, `Cargo.toml` and `Cargo.lock`, dates the `[Unreleased]` bullets into a
      `## 0.4.0` section, rewrites both version strings on `site/index.html`, commits, tags,
      pushes, builds and notarizes the disk image, and publishes the GitHub release.
- [ ] `tools/release-mas.sh --upload`, which burns build 6. One run, not two: `--validate-app`
      runs before `--upload-package` under `set -eu`, so a validation failure stops it before
      anything is sent.

**In App Store Connect**

- [ ] Read the live 0.3.2 copy back off the API before editing anything. The command is under
      Description. The panel does not report what a new version dropped.
- [ ] Create the 0.4.0 version and attach build 6, once it has finished processing.
- [ ] **Promotional Text**: re-enter all 162 characters. It is not copied forward and an empty
      one is a legal listing that warns about nothing.
- [ ] **What's New**: the 0.4.0 block, 1433 characters.
- [ ] **Description**: the build-your-own clause is the only change. Paste the whole block
      rather than editing in place, so the line breaks are the ones in this file.
- [ ] **Screenshots**: drop the seven one at a time in slot order, then read the slots back.
      Drop order sets the slots, not file names, and a released version's order cannot be
      changed afterwards.
- [ ] **App Review Information > Notes**: the review notes plus the rewritten addendum, 3886
      characters. Point 7 changed and the old wording is now false.
- [ ] Export compliance, answered by hand: no encryption.
- [ ] Add for Review, then Submit.

**After**

- [ ] Merge PR #3, which is what publishes the builder section on keepgoing.dev. Held until the
      release ships by decision, because the site must describe only what people can download.
- [ ] Add the rows to the submission log below.
- [ ] `tools/verify-store-copy.sh` against `/Applications` the day it goes live, and not before:
      a Developer ID or locally signed copy fails the provenance and sandbox checks by
      definition.

## What rode on 0.3.2, and what it cost

**Everything on this list shipped on 31 August 2026.** It is kept rather than deleted because the
reason each item was waiting is the part worth reading, and because the shape of it recurs: a
listing defect that cannot be fixed on a live version becomes a queue against the next one.

**Screenshots on a released version cannot be reordered.** Tried on 27 August 2026, the day
0.3.1 went live: the media panel does not accept a change to a version that is already
distributing. So the fix for shot 1 is not a fix to the listing, it is the next version, and
that has a price worth stating before anyone reaches for it.

**A store-only version does not exist in this repository.** Version bumping lives in
`tools/release.sh` and deliberately nowhere else, so that the two channels cannot disagree about
what a version is (`tools/release-mas.sh:11`). `release-mas.sh` reads the version out of
`tauri.conf.json` and only owns the build number, in `tools/.mas-build`. So 0.3.2 means a tagged
release, a notarized disk image, and a GitHub release, and then a store upload. **Do not spend a
version on a screenshot order.** Let it ride with the next real change, and take the whole list
below with it.

- ~~**Reorder the screenshots**: `5-pet` out of slot 1, comeback into it.~~ **Done on 31 August**,
  and read back off the API rather than the panel: comeback in slot 1, `5-pet` in slot 5, all six
  `COMPLETE` at 2560x1600. Read the slots back after uploading either way, because the file
  numbering does not survive the drop.
- ~~**Re-shoot `5-pet`**~~: **shot on 27 August, shipped on 31 August**, from
  `docs/store-shots/5-pet.png`. Flat mid-tone wallpaper, pet in the top-left corner, cropped
  `tl` so the menu bar is in frame. The reasoning is above; the short version is that a zoom was
  never available, because the pet is 64x64 logical and the frame must be exactly 2560x1600, so
  making it bigger means scaling and every shot here is a crop on purpose.
- ~~**The App Preview**, which is not the review video.~~ **Recorded, encoded and uploaded on
  30 and 31 August**, one `DESKTOP` set. The two encoding requirements it turned up are in the
  App Previews section above, along with the SSIM check that the re-encode did not resample the
  pixel art. That one was 43 seconds with Terminal and
  Finder in it, which is what App Review asked for and the opposite of what a preview may
  contain. A preview is 15 to 30 seconds, up to three, public, in the gallery ahead of the
  screenshots, and only footage of the app itself. The content was decided above: asleep to
  comeback, because the measured constraint on `drive-states.sh` rules out a full arc at any clock
  scale. `tools/preview-take.sh` performs that take hands off while you record.
- ~~**Task 18 step 2**, the manual test list from spec section 9.~~ **Run and closed on 28 August**,
  against a locally signed sandboxed copy, because that is the only kind of build that can answer a
  sandbox question and still launch. Eight items pass, one is not runnable on this hardware (both
  displays are 2x), and the three defects it found are fixed. **Still open against build 5**: it
  has to be run on the build that is actually uploaded, which is the point of the step, and the
  store copy that can answer it only became installable when 0.3.2 went live.
- ~~**The two `watcher.rs` defects the test list turned up on 28 August.**~~ **Fixed the same
  day**, and a third with them: a file inside an ignored directory was counting as work, because
  the matcher only ever looked at the path it was handed. Written up in the plan under that step.
  Not a listing matter, but the reason 0.3.2 had something in it worth spending a version on.
- ~~**The two defects the by-eye half of that list turned up.**~~ **Fixed the same day.** The
  popover was invisible over a fullscreen app while the pet was visible over it, and a tracked
  worktree whose git folder sits outside the picker's grant read as an ordinary healthy project
  and never recorded a commit. Both are in the plan under Task 18 step 2. The first of them is a
  listing matter after all: the App Preview is app footage only, and "click the pet, the panel
  opens" is the obvious opening beat.

## Submission log

A row for every outcome, including rejections and what they cited. This log is the actual
deliverable of the project: the point was to learn the process end to end, and a rejection
reason is worth more than a clean pass.

| Date | Version | Build | Result |
|---|---|---|---|
| 2026-08-25 | 0.3.1 | 3 | validated only, not uploaded: `VERIFY SUCCEEDED with no errors, 1 warning` (90889, TestFlight profile) |
| 2026-08-26 | 0.3.1 | 4 | `UPLOAD SUCCEEDED with no errors, 1 warning` (90889 again). Delivery UUID `8297c738-5a78-419c-99e7-d4f63e2fd308`, 7197291 bytes. Attached to 0.3.1 and **submitted for review** the same day, after answering export compliance by hand. |
| 2026-08-27 | 0.3.1 | 4 | **Rejected, guideline 2.1 Information Needed.** Apple's standard new-app questionnaire: a screen recording plus seven answers, no finding about the binary. Reply and recording plan in `docs/app-store-review-notes.md`. No new build required. |
| 2026-08-27 | 0.3.1 | 4 | Answered the same day: two replies in Resolution Center (the field holds 4000 characters and the answers measure 5302), a 43 second 1920x1080 screen recording attached, and the Notes field updated with the addendum. Then resubmitted, because the metadata edit had moved the version to Ready for Review. Same build, no upload. |
| 2026-08-27 | 0.3.1 | 4 | **Approved.** "Review of your submission has been completed. It is now eligible for distribution." Submitted 04:21 PDT, accepted the same day. Submission ID `c0dfeea1-544a-4452-8120-447c4732a7d4`, App Store URL `https://apps.apple.com/app/momentum-mascot/id6804925509`. |
| 2026-08-31 | 0.3.2 | 5 | `UPLOAD SUCCEEDED with no errors, 1 warning` (90889 a third time). Delivery UUID `ce4b6ca0-e770-439d-b5e5-f24ac837811f`, 7207763 bytes. Reached `VALID`, minimum macOS 10.15. |
| 2026-08-31 | 0.3.2 | 5 | **Submitted for review** at 08:03 +07, Submission ID `67103224-ab09-4462-9f99-b8db71caecf1`. No rejection and no questionnaire: the 2.1 answers were in the Notes field from the first submission of this version, which is what 0.3.1 taught. |
| 2026-08-31 | 0.3.2 | 5 | **Approved and released**, `2026-08-31T17:03:34Z`. Sixteen hours from Submit, unattended. Verified without credentials from `https://itunes.apple.com/lookup?id=6804925509&entity=macSoftware`: version 0.3.2, six screenshots, release notes present, 4942018 bytes delivered against 7207763 uploaded. |

**Submitting took six tries, none of them about the build.** After the build was attached,
"Add for Review" refused five times over listing fields, all recorded above: contact information,
content rights, an unpublished privacy questionnaire, no price, and an unanswered age rating,
then a sixth time over the demo account fields that Sign-in required had turned on. The build
itself was never the obstacle. Worth knowing for the next version: allow a session for the
listing that has nothing to do with compiling anything.

**Approved on the same day it was answered, on the same build that was rejected.** The 2.1
questionnaire cost one round trip of a few hours, and nothing in it required a new upload: build 4
was accepted exactly as it was first delivered on 26 August. Worth carrying into the next version,
because it inverts the instinct a rejection produces. The reflex is to go and change the app. The
correct first question is whether App Review asked about the app at all.

**Editing metadata after a rejection takes the version out of the queue.** Updating the Notes
field and attaching the recording moved 0.3.1 from Rejected to **Ready for Review**, which is not
a queue position: it is the state before Submit to App Review is pressed. Replying in Resolution
Center does not put the version back either. Both are needed, in that order, and the reply is
what the reviewer reads while the resubmission is what gets read at all.

**The rejection was not about the app, and it was not 4.2.** Guideline 2.1 Information Needed
arrived the day after submission and asked for a screen recording and seven pieces of
information about what the app is, what it was tested on, and what third-party material it
contains. None of it is a finding: it is the questionnaire Apple sends new apps, and the same
build stays in review while it is answered. The reply is in `docs/app-store-review-notes.md`,
which also holds the shot list for the recording. Two things learned worth carrying forward.
The review notes already on the listing did not prevent this, so budget for the questionnaire
rather than hoping good notes make it unnecessary; and the seven answers belong in the Notes
field from the first submission of every version, which is what Apple's closing line asks for.

**0.3.2 passed on the first try, and the reason is a field rather than the app.** 0.3.1 took six
attempts to submit and one rejection to approve; 0.3.2 took one of each. Nothing about the build
was better. What changed is that the seven 2.1 answers were sitting in the Notes field before
Submit was pressed, which is exactly what Apple's closing line on the 0.3.1 rejection asked for
and what the paragraph below it predicted. Sixteen hours, no Resolution Center round trip.

**The 90889 email arrives again on every version, and it still means nothing.** Apple mails the
missing-provisioning-profile warning after a successful delivery, subject line about "one or more
issues with a recent delivery". Build 4 got it and was approved; build 5 got it and was approved.
`docs/app-store.md` section 5 has the TN3125 reading; the short version is that it is a TestFlight
eligibility statement and this app uses no restricted entitlements.

**Three listing defects were found by reading the API rather than the panel**, on the morning of
submission, and all three would have shipped otherwise. `promotionalText` is not copied to a new
version while `description`, `keywords`, `supportUrl` and `marketingUrl` all are, so the 0.3.2
draft held zero characters where the live 0.3.1 held 162, and an empty promotional text is a legal
listing that warns about nothing at submission. The live description carried 26 real line breaks
from this document's own 90-column source wrapping. And this document had drifted from the live
text by three commas typed straight into the panel. The lesson is at the top of the file: read the
shipping copy back out of the API before every submission, because the panel will not tell you
what it lost.

**What was not done before submitting.** Task 18 step 2 asks for the full manual test list from
spec section 9 against the exact build being submitted. It was not re-run against build 4. The
private API gate was verified on build 4 itself, and the comeback path was exercised while
staging the screenshots, but the rest of the list was last run against an earlier build. That is
a real gap rather than an oversight worth hiding: if review rejects on function, this is the
first thing to rule out, and the list runs before the next upload either way.

**It did not run before build 5 either, and the reason is structural.** The only artifact that is
both the shipped bits and launchable is the store copy, and the store copy does not exist until
the version is approved, which is after the moment the check is supposed to protect. So this step
is always retrospective by construction. What it can still do is catch a defect before anyone
installs the update, which is why `tools/verify-store-copy.sh` runs the mechanical half against
`/Applications` the day a version goes live and prints the by-eye half underneath it. Do not run
it before installing from the store: a Developer ID or locally signed copy fails the provenance
and sandbox checks by definition, and the script says so rather than pretending otherwise.

**The upload is one run, not two.** Task 18 said to rehearse with `tools/release-mas.sh` and then
re-run with `--upload`, which burns two build numbers, because the counter increments on every
run rather than only on an upload. It does not need to: the script is `set -eu` and
`--validate-app` runs before `--upload-package`, so a validation failure stops it before
anything is sent. One run with `--upload` did the whole pipeline, and build 3 stayed the last
rehearsal rather than becoming a wasted number.

Every gate reported clean on the build that was actually uploaded, which is the only build where
that claim means anything:

```
architectures: x86_64 arm64
private API check: clean
CFBundleShortVersionString: 0.3.1
CFBundleVersion:            4
```

`private API check: clean` is the one that mattered most. It is the whole reason the native pet
rewrite happened, and Phase 6 could not complete until `drawsBackground` and `fullScreenEnabled`
were out of the binary. This is the first time that gate has been reported on a build that left
the machine.
