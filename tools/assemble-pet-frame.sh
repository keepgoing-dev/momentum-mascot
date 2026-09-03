#!/usr/bin/env bash
#
# The shell oracle for one pet frame. Differs from the room's in three ways: the canvas is
# empty rather than a plate, offsets may be negative, and asleep lays a blanket over the
# character. The pet takes no tint in any state (section 6.1 of docs/spec-v2.md).
#
# Usage:  tools/assemble-pet-frame.sh <char> <state> <frame> <out.png>

set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
A="$ROOT/src/assets"
M="$A/character-layout.json"
char=$1 state=$2 i=$3 out=$4

W=$(mktemp -d -t assemble-pet-frame)
trap 'rm -rf "$W"' EXIT

q() { jq -r "$1" "$M"; }

pw=$(q '.pet.w'); ph=$(q '.pet.h')
cx=$(q ".states.\"$state\".pet.char.x")
cy=$(q ".states.\"$state\".pet.char.y")
hop=$(q ".states.\"$state\".pet.char.hop[$i]")
range=$(q ".states.\"$state\".pet.char.range")
lo=$(q ".layerStrip.ranges.$range[0]")
hi=$(q ".layerStrip.ranges.$range[1]")
single=$(q ".states.\"$state\".pet.char.frame // empty")

n=$((hi - lo))
if [ -n "$single" ]; then k=$((lo + single)); else k=$((lo + i % n)); fi
y=$((cy + hop))

magick "$A/layers/premade/$char.png" -crop "16x32+$((k * 16))+0" +repage PNG32:"$W/char.png"
args=(-size ${pw}x${ph} xc:none "$W/char.png" -geometry "$(printf '%+d%+d' "$cx" "$y")" -composite)

# The blanket is NOT generic art: it differs on every character sheet and every body, so it
# rides in the layer strip with the character rather than in shared/.
blanket_dy=$(q ".states.\"$state\".pet.blanketDy // empty")
if [ -n "$blanket_dy" ]; then
  bl=$(q '.layerStrip.ranges.blanket[0]')
  magick "$A/layers/premade/$char.png" -crop "16x32+$((bl * 16))+0" +repage PNG32:"$W/blanket.png"
  args+=("$W/blanket.png" -geometry "$(printf '%+d%+d' "$cx" "$((y + blanket_dy))")" -composite)
fi

count=$(q ".states.\"$state\".pet.overlays | length")
o=0
while [ "$o" -lt "$count" ]; do
  sp=$(q ".states.\"$state\".pet.overlays[$o].sprite")
  dx=$(q ".states.\"$state\".pet.overlays[$o].dx")
  dy=$(q ".states.\"$state\".pet.overlays[$o].dy")
  nf=$(q ".states.\"$state\".pet.overlays[$o].frames")
  osz=$(magick identify -format '%w %h' "$A/shared/$sp.png")
  ow=$(( ${osz% *} / nf )); oh=${osz#* }
  magick "$A/shared/$sp.png" -crop "${ow}x${oh}+$(( (i % nf) * ow ))+0" +repage PNG32:"$W/o$o.png"
  args+=("$W/o$o.png" -geometry "$(printf '%+d%+d' "$((cx + dx))" "$((y + dy))")" -composite)
  o=$((o + 1))
done

magick "${args[@]}" PNG32:"$out"
