#!/usr/bin/env bash
#
# End-to-end check on src/baker.js: runs the app's bake probe, then reassembles the same build
# with ImageMagick and compares the pixels.
#
# tools/verify-baker.sh checks the arithmetic without a browser. This checks the drawing, which
# needs one, so it launches the debug app and reads what it wrote.
#
# The tolerance mirrors tools/verify-plates.sh: AE 0, or every differing pixel at a max
# per-channel delta of 1, which is -colorize rounding to integer quantum and not visible.

set -uo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
A="$ROOT/src/assets"
BUILD='{"skin":"Body_03","eyes":"Eyes_02","outfit":"Outfit_11_04","hair":"Hairstyle_11_03","accessory":"Accessory_15_Glasses_05"}'
SKIN=Body_03 EYES=Eyes_02 OUTFIT=Outfit_11_04 HAIR=Hairstyle_11_03 ACC=Accessory_15_Glasses_05

STATE_DIR=$(mktemp -d -t verify-bake)
WORK=$(mktemp -d -t verify-bake-work)
trap 'rm -rf "$STATE_DIR" "$WORK"; pkill -f "[m]omentum-mascot" 2>/dev/null' EXIT

BIN="$ROOT/src-tauri/target/debug/momentum-mascot"
[ -x "$BIN" ] || { echo "no debug binary; run cargo build --manifest-path src-tauri/Cargo.toml" >&2; exit 1; }

echo "running the bake probe"
KEEPGOING_MASCOT_STATE="$STATE_DIR/state.json" MASCOT_BAKE_PROBE="$BUILD" \
  "$BIN" >"$WORK/app.log" 2>&1 &
app=$!

art="$STATE_DIR/custom"
for _ in $(seq 1 60); do
  [ -f "$art/pet/run.png" ] && break
  sleep 1
done
kill "$app" 2>/dev/null

[ -f "$art/pet/run.png" ] || { echo "FAIL the probe wrote nothing"; tail -5 "$WORK/app.log"; exit 1; }

FRAMES=$(jq -r '.frames' "$A/character-layout.json")
fail=0

check() {  # check <surface> <state>
  local surface=$1 state=$2
  local want strip="" i=0
  want="$art/$( [ "$surface" = room ] && echo rooms || echo pet )/$state.png"
  [ -f "$want" ] || { echo "FAIL  $surface/$state  not written"; fail=1; return; }
  while [ "$i" -lt "$FRAMES" ]; do
    "$ROOT/tools/assemble-built.sh" "$surface" "$state" "$i" "$WORK/$surface-$state-$i.png" \
      "$SKIN" "$EYES" "$OUTFIT" "$HAIR" "$ACC" || { echo "FAIL  $surface/$state oracle"; fail=1; return; }
    strip="$strip $WORK/$surface-$state-$i.png"
    i=$((i + 1))
  done
  magick $strip +append PNG32:"$WORK/$surface-$state-want.png"
  local raw ae delta
  raw=$(magick compare -metric AE "$WORK/$surface-$state-want.png" "$want" null: 2>&1)
  ae=$(printf '%s' "$raw" | sed -n 's/.*(\(.*\)).*/\1/p'); [ -n "$ae" ] || ae=$raw
  delta=$(magick "$WORK/$surface-$state-want.png" "$want" -compose difference -composite \
    -colorspace Gray -format '%[fx:maxima*255]' info:)
  if [ "$ae" = "0" ]; then
    echo "ok    $surface/$state"
  elif [ "${delta%%.*}" -le 1 ]; then
    echo "ok    $surface/$state  ($ae px at delta $delta, rounding)"
  else
    echo "FAIL  $surface/$state  AE=$ae maxdelta=$delta"; fail=1
  fi
}

for state in $(jq -r '.states | keys[]' "$A/character-layout.json"); do
  [ "$(jq -r ".states.\"$state\".room // empty" "$A/character-layout.json")" = "" ] || check room "$state"
  [ "$(jq -r ".states.\"$state\".pet // empty" "$A/character-layout.json")" = "" ] || check pet "$state"
done

exit $fail
