# Changelog

## 0.3.2 (2026-08-30)

- On the Mac App Store from 31 August 2026: https://apps.apple.com/app/momentum-mascot/id6804925509. Approved on the first submission, sixteen hours after it was sent.
- Added: the popover shows the app version on the credit line. With no Dock icon, no menu bar of its own and no About window, there was nowhere in the app to read it, which matters the moment you want to report something.
- Fixed: a `.DS_Store` no longer counts as working on a project. Opening a dormant project's folder in Finder was enough to wake the mascot, which could spend the comeback celebration.
- Fixed: creating or removing a folder no longer counts as working on a project.
- Fixed: files inside a directory your `.gitignore` excludes no longer count as working on a project. Build output and dependency folders were being read as activity.
- Fixed: the popover now appears over fullscreen apps. The mascot was already visible there, so clicking it opened a panel you could not see.
- Fixed: a tracked `git worktree` whose git folder is outside the folder you picked now says so, instead of looking healthy and never recording a commit.
- Fixed: a project whose folder path goes through a symbolic link is now watched properly. It was tracked and read correctly on startup, and then never noticed another commit for as long as the app stayed open, while looking perfectly healthy the whole time.
- Fixed: the popover now opens next to whatever you clicked - above the mascot, or under the menu bar icon - rather than in the middle of the screen. Two separate faults produced that: the app only ever learned where the menu bar icon was from a click on the icon itself, so opening from the mascot first positioned nothing at all; and on two displays the popover could then open on the screen you were not looking at, because macOS moves menu bar icons to whichever display is active while the mascot stays where you left it.

## [Unreleased]

- Added: build your own mascot. The character picker has a fourth slot, marked `+`, which opens a builder inside the popover: a skin tone, eyes, a hairstyle and its colour, an outfit and its colour, and one accessory. That is 9 skin tones, 7 pairs of eyes, 14 hairstyles in 7 colours, 13 outfits in 4 colours and 42 accessories, with a Shuffle button for when nothing in particular comes to mind. The room itself is the preview, so you are looking at the mascot in the place they are going to live rather than at a form.
- Added: a mascot you built is a character like any of the three premades. They appear in the room, as the desktop pet and on the share card, and clicking the character in the room cycles through them along with the rest. Choosing the slot again while it is already selected reopens the builder, which is the only way back to it: there is still no settings screen.
- Added: the menu bar icon's right-click menu has a Support item, which opens the contact details on keepgoing.dev. Nothing inside the app said where to report a bug or ask for help; the App Store product page was the only place that did.
- Fixed: the mascot no longer disappears when you drag it off the edge of a screen. A drag kept moving it after the pointer had stopped against the edge, and once it was clear of every display it had nowhere to snap back to, so it stayed there invisibly. Reopening the app was the only way to get it back. It now runs home to the nearest corner of the display it left. One screen is enough to have hit this; a second display was not needed.
- Fixed: on more than one display, the mascot's corners are measured from the display it is actually on. They were read from whichever screen held the window you were working in, so a mascot on a second screen could run to a corner of the first one.


## 0.3.1 (2026-08-20)

- Release 0.3.1.
- On the Mac App Store from 27 August 2026: https://apps.apple.com/app/momentum-mascot/id6804925509. Same version, built and signed separately by `tools/release-mas.sh`, sandboxed where the disk image is not.

## 0.3.0 (2026-08-18)

- Release 0.3.0.

## 0.2.0 (2026-08-18)

- Release 0.2.0.
- Added a visible character picker under the room so users can switch mascots directly. Clicking the mascot still cycles through the three characters.

## 0.1.2 (2026-08-17)

- Release 0.1.2.

## 0.1.1 (2026-08-17)

- Release 0.1.1.

Versions follow [semantic versioning](https://semver.org). The state file carries its own
`version` field, currently `2.0`, and it moves independently of this one: the app version says
what changed for a person, the schema version says whether an older build can still read the
file.

## 0.1.0

The first build that runs. It is complete against `docs/spec-v2.md` as an application and
incomplete as a release: see the limits at the bottom, which are the reason this is 0.1.0 rather
than 1.0.0.

### The mascot

- Four moods read from the reflog of the repositories you point it at: **awake** under 24 hours,
  **dozing** from 24 to 72, **asleep** past 72, and the **comeback**, which fires when a real
  commit lands after a sleep. Checking out a branch or pulling cannot trigger it.
- A **64x64 desktop pet** in the bottom right corner, floating above other windows, on every
  space, and clickable to open the popover. It has no other affordance on purpose: no drag, no
  hover state, no right-click menu.
- An **animated room** per mood in a popover from the menu bar, with the tracked project list,
  a folder picker to add one, untracking, and a click on the character to cycle through the
  three characters.
- A **1200x630 share card** copied to the clipboard, carrying the room and the mood and nothing
  that could identify a project: no name, no path, no message, no hash, no timestamp. It is
  built so there is no way to express one, rather than a rule about remembering not to.

### How it behaves

- No network layer at all, no accounts, no telemetry. Zero requests, permanently.
- Everything it knows lives in `~/.keepgoing/mascot/state.json`, created on first run. Writes
  are atomic, and a corrupt or hand-edited file starts the app rather than stopping it.
- Reads exactly one thing per repository: when you last actually committed.
- No git hooks are installed in your repositories. Untracking is a line removed from JSON, and
  uninstalling leaves nothing behind but that one file.
- No Dock icon, no app switcher entry, no notifications, and nothing anywhere that tells you how
  long it has been since you last committed.

### Packaging

- Universal macOS build, native on both Apple Silicon and Intel.
- Art from **LimeZu's Modern Interiors**, composed at build time from a local licensed copy of
  the pack rather than committed. Type is **Departure Mono** under the OFL 1.1.

### Known limits

- **Ad-hoc signed, not notarized.** Gatekeeper refuses to open it on any machine other than the
  one that built it, until somebody right-clicks and chooses Open. Fixing that needs a paid
  Apple Developer ID.
- **It does not start when you log in.** After a restart you open it yourself.
- **macOS only.** Windows is a build target rather than a rewrite. Linux is uncertain: Wayland
  does not let an application position its own window, so the pet is close to unimplementable
  there.
- Nothing can float over a game that has taken an exclusive display through `CGDisplayCapture`.
  That is an OS guarantee rather than a defect here.
