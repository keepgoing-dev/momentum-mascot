#!/usr/bin/env bash
#
# Drive all four states past the pet in about two minutes instead of three days.
#
# The thresholds are 24 and 72 hours (spec section 8.2), so the only honest way to look at
# dozing and asleep is to run the clock fast. This is what `KEEPGOING_CLOCK_SCALE` exists for,
# and the scale is read in DEBUG BUILDS ONLY, so a release binary cannot be driven this way and
# there is nothing here a user could stumble into.
#
# It sets up a throwaway repository, points a throwaway state file at it, launches the app, and
# commits at the right moment so the comeback happens while you are watching. Nothing touches
# ~/.keepgoing/mascot/state.json or any repository of yours.
#
# Usage:  tools/drive-states.sh [scale]        default 3600, so one real second is one hour
#
# Watch two things at once: the transitions printed here, and the pet in the corner of the
# screen. The printed log is the state machine; the pet is the part no test can check.

set -euo pipefail

SCALE="${1:-3600}"
BIN="src-tauri/target/debug/momentum-mascot"

# Seconds of real time per simulated hour, and the two thresholds expressed in it.
HOUR=$(LC_ALL=C awk "BEGIN{printf \"%.3f\", 3600/$SCALE}")
AT_DOZING=$(LC_ALL=C awk "BEGIN{printf \"%.0f\", 24*3600/$SCALE}")
AT_ASLEEP=$(LC_ALL=C awk "BEGIN{printf \"%.0f\", 72*3600/$SCALE}")
# The comeback commit lands a fixed THIRTY REAL SECONDS after asleep rather than a fixed number
# of simulated hours, because the number being chosen here is how long a person gets to look at
# the blanket, and that is wall-clock time no matter what the clock is doing. This is the same
# split the comeback cap itself makes (section 8.1).
AT_COMMIT=$(LC_ALL=C awk "BEGIN{printf \"%.0f\", (72*3600/$SCALE)+30}")

[ -f Cargo.toml ] || [ -d src-tauri ] || { echo "run this from the repository root" >&2; exit 1; }
cargo build --manifest-path src-tauri/Cargo.toml

WORK=$(mktemp -d -t mascot-drive)
REPO="$WORK/repo"
STATE="$WORK/state.json"
trap 'kill ${APP_PID:-} ${COMMIT_PID:-} 2>/dev/null || true; rm -rf "$WORK"' EXIT

# A repository with NO commits yet, which the app accepts (section 9.1): a fresh repository is
# a legitimate thing to track and reads as awake because there is nothing to be late about.
# The first commit is made further down, immediately before launch.
git init -q "$REPO"

commit_now() {  # commit_now <message>
  git -C "$REPO" -c user.email=drive@local -c user.name=drive \
    commit -q --allow-empty -m "$1"
}

# Seeding the state file is what avoids the folder picker, which cannot be scripted. The app
# re-reads the reflog on startup regardless, so `last_commit_at` is deliberately null here:
# the value under test comes from the repository, not from this file.
cat > "$STATE" <<JSON
{
  "version": "2.0",
  "last_displayed_state": null,
  "character_id": "07",
  "pet_position": null,
  "tracked_projects": [
    {
      "id": "drive",
      "path": "$REPO",
      "name": "drive-repo",
      "added_at": $(date +%s),
      "last_commit_at": null
    }
  ]
}
JSON

printf '\n'
printf 'clock at %sx, so one simulated hour takes %ss of real time.\n\n' "$SCALE" "$HOUR"
printf '  expected, from the moment the app starts:\n'
printf '    %-8s %-9s %s\n' "t+0s"            "awake"    "standing, no emote"
printf '    %-8s %-9s %s\n' "t+${AT_DOZING}s" "dozing"   'seated, "..." emote'
printf '    %-8s %-9s %s\n' "t+${AT_ASLEEP}s" "asleep"   'tucked under a blanket, "Z" emote'
printf '    %-8s %-9s %s\n' "t+${AT_COMMIT}s" "comeback" "a commit lands here, made for you"
cat <<INFO

  after the comeback the state returns to DOZING rather than to awake, and that is correct: the
  comeback holds for as long as the project is awake, and 24 simulated hours pass ${AT_DOZING}s later. At
  1x that same rule reads as a day of being awake, which is the behaviour being modelled.

  the left column below is hours since the last commit, on the simulated clock. ctrl-c to stop.

INFO

# The commit that triggers the comeback, made from a subshell so the app keeps the terminal.
# A plain `--allow-empty` commit qualifies (section 9.2): what does NOT qualify is a checkout
# or a pull, and swapping this line for `git -C "$REPO" checkout -b x` is the check for that.
(
  sleep "$AT_COMMIT"
  echo ">>> committing now"
  commit_now "back at it"
) &
COMMIT_PID=$!

# The anchor commit goes here rather than at the top, and the position is the point. Every
# timestamp entering the app is mapped onto the accelerated timeline (section 8.1), so a commit
# made N real seconds before launch is N*SCALE seconds old on that timeline. At 3600x a commit
# made during setup would be hours old before anything started, and at a high enough scale the
# run would open in `dozing` and awake would never be seen at all.
commit_now "first"

KEEPGOING_CLOCK_SCALE="$SCALE" KEEPGOING_MASCOT_STATE="$STATE" "$BIN" &
APP_PID=$!
wait "$APP_PID"
