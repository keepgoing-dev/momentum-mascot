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
# Needs Accessibility permission for whatever runs it, because it drives the real pointer, and
# checks for it before staging anything rather than halfway through a take.
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
# The log outlives the run on purpose. The take hides the terminal and deletes its own
# workspace, so without this a take that goes wrong leaves nothing at all to look at afterwards.
LOG="${TMPDIR:-/tmp}/momentum-take.log"
trap 'kill ${APP_PID:-} 2>/dev/null || true; restore_terminal;
      cp "$WORK/app.log" "$LOG" 2>/dev/null || true; rm -rf "$WORK"' EXIT

cat > "$WORK/stage.swift" <<'SWIFT'
import AppKit
import ApplicationServices

// `pet.rs`: SIZE and MARGIN, both logical points. Duplicated rather than derived, because there
// is no way to ask a running app where it put its window. The rehearsal below is what catches a
// divergence, since a wrong corner here glides the pointer onto empty desktop.
let PET: CGFloat = 64
let MARGIN: CGFloat = 20

/// `pet.rs` `anchors()[3]`, the bottom-right corner of the PRIMARY display, by the same
/// arithmetic: AppKit measures from the bottom-left of that display upwards, Tauri from the
/// top-left downwards.
///
/// The primary display, and not `NSScreen.main`, which is a different thing on a second
/// reading: main is where the active window is, so the take would stage itself on whichever
/// display the terminal happened to be on. The primary is the one macOS calls the main display
/// in System Settings, which is where the menu bar is, which is the one anybody records.
///
/// The conversion back to CGEvent's coordinates divides by one screen's scale, which is only
/// right while every display shares it, so a mismatch is reported rather than assumed away.
func geometry() {
    guard let primary = NSScreen.screens.first else { exit(1) }
    let scale = primary.backingScaleFactor
    if NSScreen.screens.contains(where: { $0.backingScaleFactor != scale }) {
        FileHandle.standardError.write(
            "  displays disagree on scale factor; check the rehearsal carefully\n"
                .data(using: .utf8)!)
    }
    let visible = primary.visibleFrame
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
    // The take checks every window it opens against these, because a window on the wrong
    // display is a ruined recording that looks fine from the terminal.
    print("SCREEN_W=\(Int(primary.frame.size.width))")
    print("SCREEN_H=\(Int(primary.frame.size.height))")
    print("SCREEN_NAME='\(primary.localizedName)'")
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

/// Every on-screen window this app owns, as WIDTH HEIGHT X Y in logical points, so the take can
/// tell an open popover from a closed one, and one on the recording display from one that is
/// not, instead of assuming its own clicks all landed.
func windows(pid: Int) {
    let list = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements],
                                          kCGNullWindowID) as? [[String: Any]] ?? []
    for w in list where (w[kCGWindowOwnerPID as String] as? Int) == pid {
        let b = w[kCGWindowBounds as String] as? [String: CGFloat] ?? [:]
        print("\(Int(b["Width"] ?? 0)) \(Int(b["Height"] ?? 0)) "
            + "\(Int(b["X"] ?? 0)) \(Int(b["Y"] ?? 0))")
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
case "windows" where args.count == 3:
    windows(pid: Int(args[2]) ?? -1)
case "park" where args.count == 4:
    CGWarpMouseCursorPosition(point(args))
case "move" where args.count == 5:
    glide(to: point(args), seconds: Double(args[4]) ?? 1)
case "click" where args.count == 5:
    let target = point(args)
    glide(to: target, seconds: Double(args[4]) ?? 1)
    click(at: target)
default:
    FileHandle.standardError.write(
        "usage: stage trusted|geometry|windows PID|park X Y|move X Y SECONDS|click X Y SECONDS\n"
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

  This needs Accessibility permission to move and click the real pointer, and the terminal
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

# Pinned: the popover closes on focus loss, and hiding the terminal is a focus change. A click
# on the pet still closes it, which is the beat that needs it. Debug builds only.
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

popover_open() {
  "$WORK/stage" windows "$APP_PID" | awk '$1 > 200 { open = 1 } END { exit !open }'
}

# Where the popover actually is, as X Y, or nothing when it is closed.
popover_at() {
  "$WORK/stage" windows "$APP_PID" | awk '$1 > 200 { print $3, $4; exit }'
}

# Click the pet until the popover is actually in the state asked for, rather than assuming the
# click landed. A dropped click is not hypothetical - the app ignores a reopen within 250 ms of a
# close - and because every step below is a toggle, ONE missed click inverts every open and close
# after it. That turns the take's own click into a close, and the recording is of nothing at all.
toggle_to() {  # toggle_to open|closed [glide]
  local want=$1 glide=${2:-0.1} n
  for n in 1 2 3; do
    if [ "$want" = open ] && popover_open; then return 0; fi
    if [ "$want" = closed ] && ! popover_open; then return 0; fi
    "$WORK/stage" click "$CLICK_X" "$CLICK_Y" "$glide"
    sleep 0.8
  done
  return 1
}

# `copy.rs` rotates the four comeback lines rather than picking one at random, deliberately, so
# that a recording is reproducible - and `show_popover` advances the turn on every open. The
# take opens the popover exactly once, which lands on the second line, so the popover is opened
# and closed three times here to make the take's own open the fourth. It is the difference
# between a stranger reading "YOU CAME BACK." and reading "Woke up for this. Worth it."
printf '  winding the quote to the line the listing wants...\n'
for _ in 1 2 3; do
  toggle_to open   || { echo "  the popover will not open; ctrl-c and say so" >&2; exit 1; }
  # On the recording display, checked once, here, where there is still a terminal to say so in.
  # This is not defensive coding: the popover used to hang off the tray icon, macOS moves the
  # menu bar's status items to whichever display is active, and a take recorded on the laptop
  # opened its popover on the other monitor - correct-looking from here and empty on camera.
  read -r px py <<<"$(popover_at)"
  if [ "${px:-0}" -lt 0 ] || [ "${py:-0}" -lt 0 ] ||
     [ "${px:-0}" -ge "$SCREEN_W" ] || [ "${py:-0}" -ge "$SCREEN_H" ]; then
    printf '  the popover opened at %s,%s, which is off %s.\n' "$px" "$py" "$SCREEN_NAME" >&2
    printf '  ctrl-c: the take would record an empty screen.\n' >&2
    exit 1
  fi
  toggle_to closed || { echo "  the popover will not close; ctrl-c and say so" >&2; exit 1; }
done
"$WORK/stage" park "$PARK_X" "$PARK_Y"

TOTAL=$(LC_ALL=C awk "BEGIN{print $HOLD_ASLEEP+$GLIDE+$READ_NIGHT+$READ_COMEBACK+$AFTER_CLOSE}")
cat <<PLAN

  character $CHARACTER, asleep on the real clock, pet in the bottom-right corner of
  $SCREEN_NAME. That is your main display, and it is the one to record; if you meant to
  record the other one, make it the main display in System Settings > Displays first.

  the take, once it starts:

    ${HOLD_ASLEEP}s   the pet asleep in the corner
    ${GLIDE}s   the pointer travels to it and clicks
    ${READ_NIGHT}s   the popover, on the night room
    -    a commit lands and the room becomes the comeback
    ${READ_COMEBACK}s   the comeback room, on "YOU CAME BACK."
    -    a second click closes the popover
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

# The travelling click, then a check rather than a hope. A second click here is visible on
# camera and still better than a take with no popover in it.
"$WORK/stage" click "$CLICK_X" "$CLICK_Y" "$GLIDE"
sleep 0.8
toggle_to open 0.25 || true
sleep "$READ_NIGHT"

# The comeback, fired for real. `--allow-empty` so the work tree is never written: the only thing
# that changes is `.git/logs/HEAD`, which is the reflog the app actually reads.
GIT_COMMITTER_DATE="@$(date +%s) +0000" git -C "$WORK/${NAMES[0]}" \
  -c user.email=take@local -c user.name=take \
  commit -q --allow-empty -m "wake up"
sleep "$READ_COMEBACK"

# A second click on the pet, not Escape. A synthetic Escape does not reach this popover: the
# pet and the popover are both non-activating panels, so the app never becomes frontmost, and
# measured from both ways in - opened by the pet and opened by the tray icon - the panel was
# still open a second after the keystroke, while a click closed it every time. It is also the
# better beat: click to open, click to close is a complete sentence about how the thing works.
"$WORK/stage" click "$CLICK_X" "$CLICK_Y" 0.35
sleep 0.8
toggle_to closed 0.25 || true
sleep "$AFTER_CLOSE"

restore_terminal
printf '\n  stop your recording.\n\n'
if grep -q comeback "$WORK/app.log"; then
  printf '  the app reached the comeback: %s\n' \
    "$(grep comeback "$WORK/app.log" | tail -1 | tr -s ' ' | sed 's/^ //')"
else
  printf '  WARNING: the app never reached the comeback, so the take is not usable.\n'
  printf '  the log below is what to send back.\n'
fi
cat <<DONE

  trim the lead-in and everything after the last beat, then crop the menu bar off to lose the
  orange recording dot. The app is still running so the pet does not vanish out of the last
  frame - ctrl-c here when the recording is saved.

  this run's app log is kept at $LOG

DONE
wait "$APP_PID"
