#!/usr/bin/env bash
#
# Act out asleep -> comeback, hands off, while you record the screen.
#
# `hold-state.sh` holds one state still, which is what a photographer wants and the exact
# opposite of what a video wants: the App Preview is about the TRANSITION, so the thing being
# recorded has to happen on camera. `hold-state.sh comeback` stages a comeback that has already
# fired and is pinned open; this stages a real asleep and then fires the comeback mid-take, from
# the same throwaway repositories, by committing into one of them.
#
# The full arc cannot be filmed. `drive-states.sh` puts its comeback commit at AT_ASLEEP + 30
# with those 30 seconds fixed in wall-clock, so at every clock scale the commit lands past 32
# seconds and an App Preview is 30. Asleep -> comeback is the payload anyway.
#
# Why the app is not asked to fake anything: the only debug overrides in play are the state file
# and the popover pin. The commit is a real commit, the watcher is the real watcher (250 ms
# debounce, `watcher.rs`), and `src/popover.js:211` redraws an OPEN popover on the `mood` event,
# so the room changes under the viewer's eyes without anyone touching the machine.
#
# You record; this performs. macOS puts an orange recording dot in the menu bar that no
# application can suppress, so crop the menu bar off afterwards - the opening beat is a click on
# the pet, not on the tray icon, so nothing is lost.
#
# Usage:  tools/preview-take.sh [character] [--keep-terminal]
#
# Needs Accessibility permission for whatever runs it (synthetic pointer and Escape), checked up
# front so it fails before it stages anything rather than halfway through a take.
#
# Nothing here touches ~/.keepgoing/mascot/state.json or any repository of yours.

set -euo pipefail

CHARACTER=07
HIDE=1
for arg in "$@"; do
  case "$arg" in
    --keep-terminal) HIDE= ;;
    -h|--help) sed -n '2,29p' "$0" | sed 's/^#\{1,2\} \{0,1\}//'; exit 0 ;;
    *) CHARACTER="$arg" ;;
  esac
done

BIN="src-tauri/target/debug/momentum-mascot"
[ -d src-tauri ] || { echo "run this from the repository root" >&2; exit 1; }

# The beats, in seconds. The sum is kept under 30 so a take fits an App Preview without a second
# edit, and LEAD is on top of that because you trim it off.
LEAD=6            # the terminal hides and the screen settles
HOLD_ASLEEP=4     # the pet asleep in its corner, before anything moves
GLIDE=1.6         # the pointer travelling to the pet
READ_NIGHT=5      # the night room, long enough to read
READ_COMEBACK=7   # "YOU CAME BACK.", the shot the listing is for
AFTER_CLOSE=4     # the popover gone, the pet awake

# Same ages as `hold-state.sh asleep`: 5 d, 12 d, 20 d. The first decides the mood, because the
# mood comes from the newest commit across every project, and the other two are there so the
# list reads like a real one.
AGES=(7200 17280 28800)
NAMES=(pixel-diary tiny-synth dotfiles)

WORK=$(mktemp -d -t mascot-take)
restore_terminal() {
  [ -n "$HIDE" ] && [ -n "${FRONT:-}" ] && \
    osascript -e "tell application \"$FRONT\" to activate" >/dev/null 2>&1
  return 0
}
trap 'kill ${APP_PID:-} 2>/dev/null || true; restore_terminal; rm -rf "$WORK"' EXIT

cat > "$WORK/stage.swift" <<'SWIFT'
import AppKit
import ApplicationServices

// `pet.rs`: SIZE and MARGIN, both logical points. Duplicated rather than derived, because there
// is no way to ask a running app where it put its window. The rehearsal below is what catches a
// divergence, since a wrong corner here glides the pointer onto empty desktop.
let PET: CGFloat = 64
let MARGIN: CGFloat = 20

/// `pet.rs` `anchors()[3]`, the bottom-right corner, by the same arithmetic: AppKit measures
/// from the bottom-left of the PRIMARY screen upwards, Tauri from the top-left downwards.
///
/// The conversion back to CGEvent's coordinates divides by one screen's scale, which is only
/// right while every display shares it, so a mismatch is reported rather than assumed away.
func geometry() {
    guard let main = NSScreen.main, let primary = NSScreen.screens.first else { exit(1) }
    let scale = main.backingScaleFactor
    if NSScreen.screens.contains(where: { $0.backingScaleFactor != scale }) {
        FileHandle.standardError.write(
            "  displays disagree on scale factor; check the rehearsal carefully\n"
                .data(using: .utf8)!)
    }
    let visible = main.visibleFrame
    let right = (visible.origin.x + visible.size.width) * scale
    let bottom = (primary.frame.size.height - visible.origin.y) * scale
    let extent = PET * scale
    let x = right - extent - MARGIN * scale
    let y = bottom - extent - MARGIN * scale
    print("PET_X=\(Int(x))")
    print("PET_Y=\(Int(y))")
    // CGEvent works in logical points from the top-left of the primary display, so the click
    // target is the pet's centre divided back out of Tauri's physical pixels.
    print("CLICK_X=\(Int((x + extent / 2) / scale))")
    print("CLICK_Y=\(Int((y + extent / 2) / scale))")
    // Where the pointer starts every take, so the travel is the same length each time instead
    // of however far the mouse happened to be left from the corner.
    print("PARK_X=\(Int(visible.midX))")
    print("PARK_Y=\(Int(primary.frame.size.height - visible.midY))")
}

/// Move the real pointer, in steps, easing out. A warp would teleport the cursor, and a
/// teleporting cursor is the one thing in the take that would read as a machine.
func glide(to target: CGPoint, seconds: Double) {
    let start = CGEvent(source: nil)?.location ?? target
    let steps = max(1, Int(seconds * 60))
    let frame = useconds_t(seconds * 1_000_000 / Double(steps))
    for i in 1...steps {
        let t = Double(i) / Double(steps)
        let eased = 1 - pow(1 - t, 3)
        let p = CGPoint(x: start.x + (target.x - start.x) * eased,
                        y: start.y + (target.y - start.y) * eased)
        CGEvent(mouseEventSource: nil, mouseType: .mouseMoved,
                mouseCursorPosition: p, mouseButton: .left)?.post(tap: .cghidEventTap)
        usleep(frame)
    }
}

func click(at p: CGPoint) {
    for type in [CGEventType.leftMouseDown, .leftMouseUp] {
        CGEvent(mouseEventSource: nil, mouseType: type,
                mouseCursorPosition: p, mouseButton: .left)?.post(tap: .cghidEventTap)
        usleep(90_000)
    }
}

func key(_ code: CGKeyCode) {
    let source = CGEventSource(stateID: .hidSystemState)
    for down in [true, false] {
        CGEvent(keyboardEventSource: source, virtualKey: code, keyDown: down)?
            .post(tap: .cghidEventTap)
        usleep(40_000)
    }
}

func point(_ args: [String]) -> CGPoint {
    guard let x = Double(args[2]), let y = Double(args[3]) else { exit(2) }
    return CGPoint(x: x, y: y)
}

let args = CommandLine.arguments
switch args.count > 1 ? args[1] : "" {
case "trusted":
    exit(AXIsProcessTrusted() ? 0 : 1)
case "geometry":
    geometry()
case "park" where args.count == 4:
    CGWarpMouseCursorPosition(point(args))
case "move" where args.count == 5:
    glide(to: point(args), seconds: Double(args[4]) ?? 1)
case "click" where args.count == 5:
    let target = point(args)
    glide(to: target, seconds: Double(args[4]) ?? 1)
    click(at: target)
case "escape":
    key(53)
default:
    FileHandle.standardError.write(
        "usage: stage trusted|geometry|park X Y|move X Y SECONDS|click X Y SECONDS|escape\n"
            .data(using: .utf8)!)
    exit(2)
}
SWIFT

printf '\n  building...\n'
swiftc -O -o "$WORK/stage" "$WORK/stage.swift"
cargo build --manifest-path src-tauri/Cargo.toml

# Before anything is staged. A take that dies at the click has already wasted the recording.
if ! "$WORK/stage" trusted; then
  cat >&2 <<'ERR'

  This needs Accessibility permission to move the pointer and press Escape, and the terminal
  running it does not have it. System Settings > Privacy & Security > Accessibility, add the
  terminal, then run this again. The permission belongs to the terminal, not to this script.

ERR
  exit 1
fi

eval "$("$WORK/stage" geometry)"

NOW=$(date +%s)
ROWS=""
for i in 0 1 2; do
  repo="$WORK/${NAMES[$i]}"
  ts=$((NOW - AGES[i] * 60))
  git init -q "$repo"
  GIT_COMMITTER_DATE="@$ts +0000" git -C "$repo" \
    -c user.email=take@local -c user.name=take \
    commit -q --allow-empty -m "work"
  # Null timestamps for the same reason as `hold-state.sh`: the app re-reads every reflog on
  # startup, and a working-tree mtime from a minute ago would pin the mood to awake.
  ROWS="$ROWS${ROWS:+,}
    {
      \"id\": \"take-$i\",
      \"path\": \"$repo\",
      \"name\": \"${NAMES[$i]}\",
      \"added_at\": $ts,
      \"last_commit_at\": null,
      \"last_active_at\": null
    }"
done

# `last_displayed_state` stays null on purpose. The comeback is asleep -> awake, and here the app
# watches itself make that transition rather than being handed the first half of it: it starts
# genuinely asleep, and the commit below is what wakes it. Seeding the field is `hold-state.sh`'s
# trick for staging a comeback that has already happened, which is the wrong thing to film.
#
# `pet_position` is pinned to the corner the app would have chosen anyway, so the pointer knows
# where to go. `place()` keeps a saved position that is still on screen and otherwise falls back
# to this same corner, so both paths land in the same place.
cat > "$WORK/state.json" <<JSON
{
  "version": "3.1",
  "last_displayed_state": null,
  "character_id": "$CHARACTER",
  "pet_position": { "x": $PET_X, "y": $PET_Y },
  "tracked_projects": [$ROWS
  ]
}
JSON

# Pinned: the popover closes on focus loss, and hiding the terminal is a focus change. Escape
# still closes it, which is the beat that needs it. Debug builds only.
env KEEPGOING_PIN_POPOVER=1 KEEPGOING_MASCOT_STATE="$WORK/state.json" "$BIN" \
  >"$WORK/app.log" 2>&1 &
APP_PID=$!
sleep 2

# The rehearsal. Two displays, or a `pet.rs` constant that moved, and the corner computed above
# is not the corner the pet is in - which is invisible until the take is already recorded and
# the pointer glides to bare desktop. So it glides there now, while there is still a terminal to
# read, and parks in the middle afterwards so every take travels the same distance.
"$WORK/stage" move "$CLICK_X" "$CLICK_Y" 0.7
printf '\n  the pointer is on the pet now. If it is not, ctrl-c: everything below assumes it.\n'
sleep 1
"$WORK/stage" park "$PARK_X" "$PARK_Y"

TOTAL=$(LC_ALL=C awk "BEGIN{print $HOLD_ASLEEP+$GLIDE+$READ_NIGHT+$READ_COMEBACK+$AFTER_CLOSE}")
cat <<PLAN

  character $CHARACTER, asleep on the real clock, pet in the bottom-right corner.

  the take, once it starts:

    ${HOLD_ASLEEP}s   the pet asleep in the corner
    ${GLIDE}s   the pointer travels to it and clicks
    ${READ_NIGHT}s   the popover, on the night room
    -    a commit lands and the room becomes the comeback
    ${READ_COMEBACK}s   "YOU CAME BACK."
    -    escape closes the popover
    ${AFTER_CLOSE}s   the pet awake, alone

  ${TOTAL}s on camera, after a ${LEAD}s lead-in you trim off. Do not touch the mouse or the
  keyboard from the moment you press Enter until it says stop.

PLAN

if [ -n "$HIDE" ]; then
  FRONT=$(osascript -e 'tell application "System Events" to name of first process whose frontmost is true' 2>/dev/null || true)
  printf '  %s hides itself when you press Enter, and comes back at the end.\n' "${FRONT:-the terminal}"
  printf '  pass --keep-terminal to switch Spaces by hand instead.\n\n'
fi

printf '  start your screen recording now, then press Enter.'
read -r _
printf '\n'

if [ -n "$HIDE" ] && [ -n "${FRONT:-}" ]; then
  osascript -e "tell application \"System Events\" to set visible of process \"$FRONT\" to false" \
    >/dev/null 2>&1 || true
fi

sleep "$LEAD"
sleep "$HOLD_ASLEEP"

"$WORK/stage" click "$CLICK_X" "$CLICK_Y" "$GLIDE"
sleep "$READ_NIGHT"

# The comeback, fired for real. `--allow-empty` so the work tree is never written: the only thing
# that changes is `.git/logs/HEAD`, which is the reflog the app actually reads.
GIT_COMMITTER_DATE="@$(date +%s) +0000" git -C "$WORK/${NAMES[0]}" \
  -c user.email=take@local -c user.name=take \
  commit -q --allow-empty -m "wake up"
sleep "$READ_COMEBACK"

"$WORK/stage" escape
sleep "$AFTER_CLOSE"

restore_terminal
cat <<'DONE'

  stop your recording.

  trim the lead-in and everything after the last beat, then crop the menu bar off to lose the
  orange recording dot. The app is still running so the pet does not vanish out of the last
  frame - ctrl-c here when the recording is saved.

DONE
wait "$APP_PID"
