# Momentum Mascot

A retro pixel character who lives in a tiny room on your desktop and reflects how your side
projects are going. Commit something and they are at their desk. Take a few days off and they
doze, then sleep, still holding your place. Come back after a long silence and they leap out of
bed.

It is not a productivity tool. There are no streaks, no scores, no notifications, and nothing
in it will ever tell you how long it has been since you last committed.

**The mascot never dies. It waits.**

## What it does

- Watches the reflog of the git repositories you point it at, and reads exactly one thing from
  each: when you last actually committed. Not messages, not diffs, not branch names.
- Shows the mood as a **64x64 desktop pet** in the corner of your screen, and as a full
  **animated room** in a popover from the menu bar.
- Copies a 1200x630 **share card** to your clipboard, carrying the room and the mood and
  nothing identifying at all.

Four states: awake under 24 hours, dozing from 24 to 72, asleep past 72, and the comeback,
which fires when a real commit lands after a sleep. Checking out a branch or pulling does not
count as work and cannot trigger it.

## Privacy

There is no network layer. The app makes zero network requests, has no accounts, no telemetry
and no hosted anything, and all of that is permanent rather than "not yet".

Everything it knows lives in one local file, `~/.keepgoing/mascot/state.json`, which you can
read. The share card cannot contain a project name, a path, a commit message, a hash or a
timestamp, and it is built so that there is no way to express one rather than a rule about
remembering not to.

## Building it

```sh
tools/build-app-assets.sh          # composites the rooms, the pet and the fonts
cargo build --manifest-path src-tauri/Cargo.toml --release
```

The first step needs a local licensed copy of **LimeZu's Modern Interiors** (the full pack, not
the free tiles), and reads it from `$MASCOT_PACK`. The composed art is deliberately **not** in
version control: the licence permits shipping it compiled into an application and forbids
redistributing it as assets, so this repository holds the coordinates that produce the art and
you bring your own copy of the pack. `docs/asset-picks.md` is the manifest, and
`tools/compose-rooms.sh` is the compositor.

macOS is the current target. Windows is a build target rather than a rewrite, and Linux is
honestly uncertain: Wayland does not let applications position their own windows, so the pet is
essentially not implementable there, and tray support varies by desktop environment.

Nothing can float over a game that has taken an exclusive display through `CGDisplayCapture`.
That is an OS-level guarantee rather than a bug here.

## Credits

**Art: [limezu.itch.io](https://limezu.itch.io)**, LimeZu's Modern Interiors. Required by the
licence, and the reason this thing looks like anything at all.

**Type: Departure Mono** by Helena Zhang, under the SIL Open Font License 1.1, vendored in
`assets/fonts/departure-mono/` with its licence text.

## The design

`docs/spec-v2.md` is the specification, and it argues for its decisions rather than just listing
them. `docs/initial-spec.md` is the first draft, kept unchanged so the two can be read side by
side.
