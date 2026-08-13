# Changelog

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
