#!/usr/bin/env bash
#
# Hold one state still, for as long as it takes to photograph it.
#
# `drive-states.sh` runs the whole arc past you on an accelerated clock, which is the right
# tool for watching the transitions and the wrong one for a screenshot: the state you want is
# on screen for seconds, and the clock keeps moving while you frame the shot. This does the
# opposite. It leaves the clock alone and backdates the commits instead, so the state it
# stages is the state the app reads for hours.
#
# The timestamp the app reads is the REFLOG ENTRY's, not the commit's author date, so
# `GIT_COMMITTER_DATE` is the knob and `--date` would do nothing (reflog.rs documents why the
# reflog wins: an amend rewrites committer time, the reflog records when the user did the
# thing).
#
# Nothing here touches ~/.keepgoing/mascot/state.json or any repository of yours: it builds
# throwaway repositories in a temporary directory and points the debug-only
# KEEPGOING_MASCOT_STATE override at a throwaway state file.
#
# Usage:  tools/hold-state.sh awake|dozing|asleep|comeback [character]
#
# Runs in the foreground and holds until ctrl-c. Take the shots with
# `tools/store-shots.sh` from another terminal.

set -euo pipefail

STATE_NAME="${1:-awake}"
CHARACTER="${2:-07}"
BIN="src-tauri/target/debug/momentum-mascot"

[ -f Cargo.toml ] || [ -d src-tauri ] || { echo "run this from the repository root" >&2; exit 1; }

# Minutes since the last qualifying commit, per project. The first number decides the state,
# because the mood is derived from the NEWEST commit across every project (section 4.4), and
# the rest are there so the project list looks like a real one rather than three copies of the
# same row.
#
# Every first entry is chosen with headroom, not on a boundary: 12 minutes has most of a day
# before it dozes, and 30 hours has 42 hours before it falls asleep. A shot taken at 23h59m
# would be a different shot by the time you looked at it.
case "$STATE_NAME" in
  awake)    AGES=(12 300 1800)           ;;  # 12 min, 5 h, 30 h
  dozing)   AGES=(1800 5760 12960)       ;;  # 30 h, 4 d, 9 d
  asleep)   AGES=(7200 17280 28800)      ;;  # 5 d, 12 d, 20 d
  comeback) AGES=(3 8640 20160)          ;;  # 3 min, 6 d, 14 d
  *) echo "unknown state: $STATE_NAME (awake, dozing, asleep, comeback)" >&2; exit 1 ;;
esac
NAMES=(pixel-diary tiny-synth dotfiles)

# The comeback is a TRANSITION, not a resting state, so it cannot be staged by a timestamp
# alone. What fires it is asleep -> awake, and the previous half of that pair lives in the
# state file as `last_displayed_state`. Seeding it is the same path as the restart case the
# app already supports: quit while asleep, commit, relaunch (spec section 4.5).
LAST_DISPLAYED=null
HOLD_COMEBACK=
if [ "$STATE_NAME" = comeback ]; then
  LAST_DISPLAYED='"asleep"'
  # Otherwise the celebration is a thirty minute one-shot that any close also ends, which is
  # correct for a user and useless for a photographer.
  HOLD_COMEBACK=1
fi

cargo build --manifest-path src-tauri/Cargo.toml

WORK=$(mktemp -d -t mascot-hold)
STATE="$WORK/state.json"
trap 'kill ${APP_PID:-} 2>/dev/null || true; rm -rf "$WORK"' EXIT

NOW=$(date +%s)
ROWS=""
for i in 0 1 2; do
  name="${NAMES[$i]}"
  repo="$WORK/$name"
  ts=$((NOW - AGES[i] * 60))
  git init -q "$repo"
  GIT_COMMITTER_DATE="@$ts +0000" git -C "$repo" \
    -c user.email=hold@local -c user.name=hold \
    commit -q --allow-empty -m "work"
  # `last_commit_at` is deliberately null: the app re-reads every reflog on startup, so the
  # value under test comes from the repository. `last_active_at` too, and that one matters
  # more: it is the max of the two that decides the mood, and a working-tree mtime from five
  # minutes ago would pin every state to awake. These repositories have no files at all,
  # which is why the commits are `--allow-empty`.
  ROWS="$ROWS${ROWS:+,}
    {
      \"id\": \"hold-$i\",
      \"path\": \"$repo\",
      \"name\": \"$name\",
      \"added_at\": $ts,
      \"last_commit_at\": null,
      \"last_active_at\": null
    }"
done

cat > "$STATE" <<JSON
{
  "version": "3.1",
  "last_displayed_state": $LAST_DISPLAYED,
  "character_id": "$CHARACTER",
  "pet_position": null,
  "tracked_projects": [$ROWS
  ]
}
JSON

printf '\n  holding %s, character %s, real clock.\n\n' "$STATE_NAME" "$CHARACTER"
# Raw ages, not the popover's own wording. copy.rs owns that wording and this script has no
# business having a second copy of it that can drift.
for i in 0 1 2; do
  printf '    %-14s %s\n' "${NAMES[$i]}" "$(LC_ALL=C awk "BEGIN{m=${AGES[$i]}; \
    if (m<60) printf \"%dm\", m; \
    else if (m<1440) printf \"%gh\", m/60; \
    else printf \"%gd\", m/1440}") old"
done
if [ "$STATE_NAME" = comeback ]; then
  cat <<'INFO'

  the comeback is HELD: neither the 30 minute cap nor closing the popover will end it, so the
  popover can be opened and closed as often as it takes. Debug builds only.
INFO
fi
printf '\n  the popover is PINNED: it will not close when the screenshot tool takes the focus.\n'
printf '  escape or the tray icon still close it. ctrl-c here when the shots are taken.\n\n'

# Pinned, because the popover closes on focus loss and every way of triggering a screen capture
# takes the focus first. Escape and the tray icon still close it. Debug builds only.
# Through `env`, not a bare assignment prefix. The shell decides what is an assignment BEFORE
# it expands anything, so `${HOLD_COMEBACK:+VAR=1}` in that position expands into a command name
# and exits 127. `env` takes its assignments after expansion.
env KEEPGOING_PIN_POPOVER=1 \
  ${HOLD_COMEBACK:+KEEPGOING_HOLD_COMEBACK=1} \
  KEEPGOING_MASCOT_STATE="$STATE" "$BIN" &
APP_PID=$!
wait "$APP_PID"
