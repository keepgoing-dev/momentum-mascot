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

## Install

Pre-built releases are on the [GitHub Releases](https://github.com/keepgoing-dev/momentum-mascot/releases)
page. Download `Momentum Mascot.dmg`, open it, and drag the app to `Applications`.

That is the whole process. The build is signed with a Developer ID certificate and notarized by
Apple, so it opens with no Gatekeeper warning and nothing to run in Terminal.

The app does not start on login yet, so open it yourself after a restart.

## Privacy

There is no network layer. The app makes zero network requests, has no accounts, no telemetry
and no hosted anything, and all of that is permanent rather than "not yet".

Everything it knows lives in one local file, `~/.keepgoing/mascot/state.json`, which you can
read. The share card cannot contain a project name, a path, a commit message, a hash or a
timestamp, and it is built so that there is no way to express one rather than a rule about
remembering not to.

## Building it

```sh
tools/build-app-assets.sh          # composites the rooms, the pet, the icon and the fonts
cargo run --manifest-path src-tauri/Cargo.toml
```

The first step needs a local licensed copy of **LimeZu's Modern Interiors** (the full pack, not
the free tiles), and reads it from `$MASCOT_PACK`. The composed art is deliberately **not** in
version control: the licence permits shipping it compiled into an application and forbids
redistributing it as assets, so this repository holds the coordinates that produce the art and
you bring your own copy of the pack. `docs/asset-picks.md` is the manifest, and
`tools/compose-rooms.sh` is the compositor.

`tools/drive-states.sh` runs the whole four-state arc past you in about two minutes on an
accelerated clock, instead of the three days the real 24 and 72 hour thresholds would take. It
uses a throwaway repository and a throwaway state file, and the clock it needs is read in debug
builds only.

## Packaging it

### Manual build

```sh
cargo install tauri-cli --version "^2.0" --locked
cargo tauri build --target universal-apple-darwin
```

That produces `Momentum Mascot.app` and a `.dmg` under
`src-tauri/target/universal-apple-darwin/release/bundle/`, universal so it runs natively on both
Apple Silicon and Intel.

### Automated release

The preferred release path builds locally on the machine that already has the licensed asset
pack, then publishes the `.dmg` and tag automatically:

```sh
tools/release.sh patch     # 0.1.0 -> 0.1.1
tools/release.sh minor     # 0.1.0 -> 0.2.0
tools/release.sh 0.1.1     # explicit version
```

This script:

1. Checks the signing certificate and notarization credentials before anything is pushed
2. Bumps the version in `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, and `Cargo.lock`
3. Dates the matching section in `CHANGELOG.md`
4. Commits and tags the release
5. Builds the universal `.dmg`, signed with a Developer ID certificate
6. Notarizes the `.dmg` with Apple and staples the ticket to it
7. Creates the GitHub Release and uploads the `.dmg`

Credentials live in `tools/.release-env`, which is gitignored. Copy
`tools/.release-env.example` and fill it in once; the one-time Apple setup is in
[`docs/notarization.md`](docs/notarization.md).

Set `MASCOT_PACK` first if you want to recomposite the art; otherwise it uses the assets already
on disk.

```sh
MASCOT_PACK=/path/to/moderninteriors-win tools/release.sh patch
```

### CI release (optional)

If you ever want to build entirely in GitHub Actions, `.github/workflows/release.yml` is also
available. It needs a `MASCOT_PACK_URL` repository secret pointing to a privately hosted archive
of the licensed pack, because the pack cannot be committed to git.

**Check which `cargo` is on your `PATH` first.** If an older standalone Rust install sits in
`/usr/local/bin`, it shadows rustup's and you will silently get an x86_64-only build on an Apple
Silicon machine, which runs under Rosetta and reports no problem at all. `rustc -vV | grep host`
is the check, and `PATH="$HOME/.cargo/bin:$PATH"` in front of the build command is the fix.

**Releases are signed and notarized**, which needs an Apple Developer Program membership, a
Developer ID Application certificate, and an app-specific password. The whole setup is in
[`docs/notarization.md`](docs/notarization.md), and `tools/release.sh` refuses to start without
it rather than discovering the problem after it has already pushed a tag.

`MASCOT_SKIP_NOTARIZE=1 tools/release.sh patch` builds ad-hoc signed instead. That build only
opens on the machine that produced it, so it is for testing the release plumbing, never for
publishing: on macOS Sequoia and later, Control-click then **Open** no longer bypasses
Gatekeeper, and the user has to dig through System Settings to Privacy & Security to allow it.

macOS is the current target. Windows is a build target rather than a rewrite, and Linux is
honestly uncertain: Wayland does not let applications position their own windows, so the pet is
essentially not implementable there, and tray support varies by desktop environment.

Nothing can float over a game that has taken an exclusive display through `CGDisplayCapture`.
That is an OS-level guarantee rather than a bug here.

## Credits

Built by **Hoa Trinh**. [hoatrinh.dev](https://hoatrinh.dev) ·
[github.com/mrth2](https://github.com/mrth2) ·
[linkedin.com/in/hoa-trinh-dev](https://www.linkedin.com/in/hoa-trinh-dev)

**Art: [limezu.itch.io](https://limezu.itch.io)**, LimeZu's Modern Interiors. Required by the
licence, and the reason this thing looks like anything at all.

**Type: Departure Mono** by Helena Zhang, under the SIL Open Font License 1.1, vendored in
`assets/fonts/departure-mono/` with its licence text.

## Licence

The code is **MIT**, in `LICENSE`. That covers everything in this repository, which is the whole
point of the arrangement described above: the pack art is not here, so the permissive licence on
the code cannot accidentally speak for art that is not the author's to give away.

What MIT does **not** cover, and what forking this does not grant you:

- **LimeZu's Modern Interiors.** Everything under `src/assets/` and `docs/mockups/`, plus the
  application icon, is composed from it at build time and is ignored by git. The licence permits
  shipping it compiled into an application and forbids redistributing it as assets. You need your
  own copy of the pack, and the credit to `limezu.itch.io` is required in anything you ship.
- **Departure Mono**, which is OFL 1.1 rather than MIT, with its terms in
  `assets/fonts/departure-mono/LICENSE`.

## The design

`docs/spec-v2.md` is the specification, and it argues for its decisions rather than just listing
them. `docs/initial-spec.md` is the first draft, kept unchanged so the two can be read side by
side.
