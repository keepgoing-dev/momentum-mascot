#!/usr/bin/env bash
#
# Reassembles each shipped strip from its plates and asserts pixel identity.
# This is what stops the JS baker and the shell compositor drifting apart.
#
# Usage:  tools/verify-plates.sh [state...]      default: every state in the manifest

set -uo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
A="$ROOT/src/assets"
MANIFEST="$A/character-layout.json"
[ -f "$MANIFEST" ] || { echo "no manifest: $MANIFEST" >&2; exit 1; }

q() { jq -r "$1" "$MANIFEST"; }

FRAMES=$(q '.frames')
STATES=${*:-$(q '.states | keys[]' | tr '\n' ' ')}
WORK=$(mktemp -d -t verify-plates)
trap 'rm -rf "$WORK"' EXIT

fail=0

compare_strip() {  # compare_strip <label> <want> <assembler> <char> <state>
  local label=$1 want=$2 asm=$3 char=$4 state=$5
  local slug=${label//\//-}
  if [ ! -x "$ROOT/tools/$asm" ]; then echo "skip  $label  no $asm yet"; return; fi
  [ -f "$want" ] || { echo "MISS  $label  no $want"; fail=1; return; }
  local strip="" i=0
  while [ "$i" -lt "$FRAMES" ]; do
    if ! "$ROOT/tools/$asm" "$char" "$state" "$i" "$WORK/$slug-$i.png" 2>"$WORK/err"; then
      echo "FAIL  $label  assembler: $(head -1 "$WORK/err")"; fail=1; return
    fi
    strip="$strip $WORK/$slug-$i.png"
    i=$((i + 1))
  done
  magick $strip +append PNG32:"$WORK/$slug-got.png"
  local ae
  # ImageMagick 7 Q16 prints AE as "<count*65535> (<count>)". The parenthesised value is
  # the pixel count; the bare one is quantum-scaled and reads as a wild number.
  local raw; raw=$(magick compare -metric AE "$want" "$WORK/$slug-got.png" null: 2>&1)
  ae=$(printf '%s' "$raw" | sed -n 's/.*(\(.*\)).*/\1/p'); [ -n "$ae" ] || ae=$raw
  local delta; delta=$(magick "$want" "$WORK/$slug-got.png" -compose difference -composite \
    -colorspace Gray -format '%[fx:maxima*255]' info:)
  # A tinted state cannot reach AE 0: -colorize rounds to integer quantum, so tinting the
  # sprites and tinting the finished frame disagree by one on antialiased pixels.
  if [ "$ae" = "0" ]; then
    echo "ok    $label"
  elif [ "${delta%%.*}" -le 1 ]; then
    echo "ok    $label  ($ae px at delta $delta, rounding)"
  else
    echo "FAIL  $label  AE=$ae maxdelta=$delta"; fail=1
  fi
}

# A state absent from the manifest is a failure, not a skip. A state present but with no
# room half (run) is a legitimate skip. Conflating the two makes an empty manifest pass.
for state in $STATES; do
  [ "$(q ".states.\"$state\" // empty")" = "" ] && { echo "FAIL  $state  not in the manifest"; fail=1; }
done

for char in 07 12 20; do
  for state in $STATES; do
    [ "$(q ".states.\"$state\" // empty")" = "" ] && continue
    [ "$(q ".states.\"$state\".room // empty")" = "" ] || \
      compare_strip "room/$char/$state" "$A/rooms/$char/$state.png" assemble-frame.sh "$char" "$state"
    [ "$(q ".states.\"$state\".pet // empty")" = "" ] || \
      compare_strip "pet/$char/$state" "$A/pet/$char/$state.png" assemble-pet-frame.sh "$char" "$state"
  done
done

exit $fail
