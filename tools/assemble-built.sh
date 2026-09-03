#!/usr/bin/env bash
#
# The shell oracle for a BUILT mascot: the same assembly as assemble-frame.sh, but stacking
# the five generator layers in the pack's order instead of cropping one premade sheet.
#
# Usage:  tools/assemble-built.sh <room|pet> <state> <frame> <out.png> <skin> <eyes> <outfit> <hair> [accessory]

set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
A="$ROOT/src/assets"
M="$A/character-layout.json"
surface=$1 state=$2 i=$3 out=$4 skin=$5 eyes=$6 outfit=$7 hair=$8 accessory=${9:-}

W=$(mktemp -d -t assemble-built)
trap 'rm -rf "$W"' EXIT
q() { jq -r "$1" "$M"; }

cx=$(q ".states.\"$state\".$surface.char.x")
cy=$(q ".states.\"$state\".$surface.char.y")
hop=$(q ".states.\"$state\".$surface.char.hop[$i]")
range=$(q ".states.\"$state\".$surface.char.range")
lo=$(q ".layerStrip.ranges.$range[0]"); hi=$(q ".layerStrip.ranges.$range[1]")
single=$(q ".states.\"$state\".$surface.char.frame // empty")
n=$((hi - lo))
if [ -n "$single" ]; then k=$((lo + single)); else k=$((lo + i % n)); fi
y=$((cy + hop))

# The five layers, in the pack's documented order, cropped to the same strip frame.
magick -size 16x32 xc:none PNG32:"$W/char.png"
for pair in "skin:$skin" "eyes:$eyes" "outfit:$outfit" "hair:$hair" "accessory:$accessory"; do
  cat=${pair%%:*}; id=${pair#*:}
  [ -n "$id" ] || continue
  magick "$A/layers/$cat/$id.png" -crop "16x32+$((k * 16))+0" +repage PNG32:"$W/l.png"
  magick "$W/char.png" "$W/l.png" -geometry +0+0 -composite PNG32:"$W/char.png"
done

if [ "$surface" = room ]; then
  rw=$(q '.room.w'); rh=$(q '.room.h')
  magick "$A/plates/$state-back.png" -crop "${rw}x${rh}+$((i * rw))+0" +repage PNG32:"$W/back.png"
  args=("$W/back.png" "$W/char.png" -geometry "+$cx+$y" -composite)
else
  pw=$(q '.pet.w'); ph=$(q '.pet.h')
  args=(-size ${pw}x${ph} xc:none "$W/char.png" -geometry "$(printf '%+d%+d' "$cx" "$y")" -composite)
  bdy=$(q ".states.\"$state\".pet.blanketDy // empty")
  if [ -n "$bdy" ]; then
    bl=$(q '.layerStrip.ranges.blanket[0]')
    magick "$A/layers/skin/$skin.png" -crop "16x32+$((bl * 16))+0" +repage PNG32:"$W/bl.png"
    args+=("$W/bl.png" -geometry "$(printf '%+d%+d' "$cx" "$((y + bdy))")" -composite)
  fi
fi

count=$(q ".states.\"$state\".$surface.overlays | length")
o=0
while [ "$o" -lt "$count" ]; do
  sp=$(q ".states.\"$state\".$surface.overlays[$o].sprite")
  dx=$(q ".states.\"$state\".$surface.overlays[$o].dx")
  dy=$(q ".states.\"$state\".$surface.overlays[$o].dy")
  nf=$(q ".states.\"$state\".$surface.overlays[$o].frames")
  osz=$(magick identify -format '%w %h' "$A/shared/$sp.png")
  ow=$(( ${osz% *} / nf )); oh=${osz#* }
  magick "$A/shared/$sp.png" -crop "${ow}x${oh}+$(( (i % nf) * ow ))+0" +repage PNG32:"$W/o$o.png"
  args+=("$W/o$o.png" -geometry "$(printf '%+d%+d' "$((cx + dx))" "$((y + dy))")" -composite)
  o=$((o + 1))
done

if [ "$surface" = room ] && [ "$(q ".states.\"$state\".room.front // empty")" = "true" ]; then
  rw=$(q '.room.w'); rh=$(q '.room.h')
  magick "$A/plates/$state-front.png" -crop "${rw}x${rh}+$((i * rw))+0" +repage PNG32:"$W/front.png"
  args+=("$W/front.png" -geometry "+0+0" -composite)
fi

magick "${args[@]}" PNG32:"$out"
