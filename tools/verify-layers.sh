#!/usr/bin/env bash
#
# Shape and content checks on the built-mascot layer strips.
#
# The sleep-row assertion is spec section 10 test 1: outfits and eyes draw nothing there
# because the character is under a duvet, and an item that does draw would clip through it.

set -uo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
A="${1:-$ROOT/src/assets}"
[ -d "$A/layers/skin" ] || { echo "no layers in $A/layers" >&2; exit 1; }

fail=0
n=0

for f in "$A"/layers/*/*.png; do
  n=$((n + 1))
  size=$(magick identify -format '%wx%h' "$f")
  [ "$size" = "448x32" ] || { echo "FAIL $f is $size not 448x32"; fail=1; }
done

# Frames 13-18 are the sleep loop, so the crop starts at 13*16 = 208.
for f in "$A"/layers/outfit/*.png "$A"/layers/eyes/*.png; do
  a=$(magick "$f" -crop 96x32+208+0 +repage -format '%[fx:mean.a]' info:)
  [ "$a" = "0" ] || { echo "FAIL $(basename "$f") draws in the sleep row (mean alpha $a)"; fail=1; }
done

# Only the body supplies the blanket, frame 27 at 27*16 = 432.
for f in "$A"/layers/skin/*.png; do
  a=$(magick "$f" -crop 16x32+432+0 +repage -format '%[fx:mean.a]' info:)
  [ "$a" = "0" ] && { echo "FAIL $(basename "$f") has no blanket"; fail=1; }
done
for cat in eyes hair outfit accessory; do
  for f in "$A"/layers/$cat/*.png; do
    a=$(magick "$f" -crop 16x32+432+0 +repage -format '%[fx:mean.a]' info:)
    [ "$a" = "0" ] || { echo "FAIL $cat/$(basename "$f") draws a blanket"; fail=1; }
  done
done

# The blanket differs per body. It was shipped once until pet/12 and pet/20 failed
# reassembly at delta 27, so this guards the fix rather than the original assumption.
seen=$(for f in "$A"/layers/skin/*.png; do
  magick "$f" -crop 16x32+432+0 +repage -format '%#\n' info:
done | sort -u | wc -l | tr -d ' ')
total=$(ls "$A"/layers/skin/*.png | wc -l | tr -d ' ')
[ "$seen" = "$total" ] || { echo "FAIL blankets are not per-body ($seen distinct of $total)"; fail=1; }

echo "checked $n strips"
exit $fail
