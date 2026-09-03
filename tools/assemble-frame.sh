#!/usr/bin/env bash
#
# The shell oracle: one room frame, assembled from plates the way the JS baker assembles it.
# Every placement comes from the manifest, so this and src/baker.js cannot drift apart
# without tools/verify-plates.sh going red.
#
# Usage:  tools/assemble-frame.sh <char> <state> <frame> <out.png>

set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
A="$ROOT/src/assets"
M="$A/character-layout.json"
char=$1 state=$2 i=$3 out=$4

W=$(mktemp -d -t assemble-frame)
trap 'rm -rf "$W"' EXIT

q() { jq -r "$1" "$M"; }

rw=$(q '.room.w'); rh=$(q '.room.h')
cx=$(q ".states.\"$state\".room.char.x")
cy=$(q ".states.\"$state\".room.char.y")
hop=$(q ".states.\"$state\".room.char.hop[$i]")
range=$(q ".states.\"$state\".room.char.range")
lo=$(q ".layerStrip.ranges.$range[0]")
hi=$(q ".layerStrip.ranges.$range[1]")
single=$(q ".states.\"$state\".room.char.frame // empty")

n=$((hi - lo))
if [ -n "$single" ]; then k=$((lo + single)); else k=$((lo + i % n)); fi

y=$((cy + hop))

magick "$A/plates/$state-back.png" -crop "${rw}x${rh}+$((i * rw))+0" +repage PNG32:"$W/back.png"
magick "$A/layers/premade/$char.png" -crop "16x32+$((k * 16))+0" +repage PNG32:"$W/char.png"

args=("$W/back.png" "$W/char.png" -geometry "+$cx+$y" -composite)

count=$(q ".states.\"$state\".room.overlays | length")
o=0
while [ "$o" -lt "$count" ]; do
  sp=$(q ".states.\"$state\".room.overlays[$o].sprite")
  dx=$(q ".states.\"$state\".room.overlays[$o].dx")
  dy=$(q ".states.\"$state\".room.overlays[$o].dy")
  nf=$(q ".states.\"$state\".room.overlays[$o].frames")
  # Overlays are not all square: the coffee is 16x32, the emotes 16x16.
  osz=$(magick identify -format '%w %h' "$A/shared/$sp.png")
  ow=$(( ${osz% *} / nf )); oh=${osz#* }
  magick "$A/shared/$sp.png" -crop "${ow}x${oh}+$(( (i % nf) * ow ))+0" +repage PNG32:"$W/o$o.png"
  args+=("$W/o$o.png" -geometry "+$((cx + dx))+$((y + dy))" -composite)
  o=$((o + 1))
done

if [ -f "$A/plates/$state-front.png" ]; then
  magick "$A/plates/$state-front.png" -crop "${rw}x${rh}+$((i * rw))+0" +repage PNG32:"$W/front.png"
  args+=("$W/front.png" -geometry "+0+0" -composite)
fi

magick "${args[@]}" PNG32:"$out"
