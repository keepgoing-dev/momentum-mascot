#!/usr/bin/env bash
#
# The awake room strips the site's builder section cycles through, one per example mascot.
# Composited output only: the pack's licence covers shipping that, not the layers themselves.
#
# Usage:  tools/site-built-strips.sh

set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
OUT="$ROOT/site/assets/built"
FRAMES=$(jq -r '.frames' "$ROOT/src/assets/character-layout.json")

# skin eyes outfit hair accessory
BUILDS=(
  "Body_02 Eyes_03 Outfit_04_02 Hairstyle_05_03 Accessory_15_Glasses_02"
  "Body_06 Eyes_01 Outfit_13_01 Hairstyle_22_06"
  "Body_04 Eyes_05 Outfit_08_03 Hairstyle_02_01 Accessory_11_Beanie_03"
  "Body_08 Eyes_02 Outfit_17_03 Hairstyle_26_02 Accessory_13_Beard_04"
)

mkdir -p "$OUT"
W=$(mktemp -d -t site-built-strips)
trap 'rm -rf "$W"' EXIT

n=0
for build in "${BUILDS[@]}"; do
  n=$((n + 1))
  name=$(printf '%02d' "$n")
  frames=()
  i=0
  while [ "$i" -lt "$FRAMES" ]; do
    "$ROOT/tools/assemble-built.sh" room awake "$i" "$W/$name-$i.png" $build
    frames+=("$W/$name-$i.png")
    i=$((i + 1))
  done
  magick "${frames[@]}" +append PNG32:"$OUT/$name.png"
  echo "$OUT/$name.png"
done
