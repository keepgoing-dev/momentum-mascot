#!/usr/bin/env bash
#
# Builds the application icon: the one people see in the Dock, in Finder, and on the disk image.
#
# This is NOT the tray icon, and the two are different jobs for the same reasons a menu bar
# template is a different register from pixel art (make-icons.sh). The tray mark is ink and no
# ink at 16px, drawn by hand, committed, and never from the pack. The app icon is 1024px of full
# colour, and at that size the honest answer to "what is this app" is the character themselves.
#
# So this one IS derived from the pack, which puts it under section 4.2: permitted to ship
# compiled into a distributed binary, forbidden to redistribute as an asset. It therefore lands
# in a gitignored directory alongside the rooms and the pet strips, and `tauri build` needs a
# licensed copy of the pack the same way the rest of the app does.
#
# `src-tauri/icons/icon.png` stays committed and stays the drawn mark. It is what
# `generate_context!` requires at compile time, so keeping it is what preserves the property
# that the app still builds on a machine with no pack at all.
#
# Env:  MASCOT_PACK   root of moderninteriors-win  (default matches compose-rooms.sh)
#       MASCOT_CHAR   premade character number     (default 07)
#
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
PACK="${MASCOT_PACK:-$HOME/Workspace/OneQode/projects/repos/oneqode-pixel-assets/moderninteriors-win}"
CHAR="${MASCOT_CHAR:-07}"
OUT="$ROOT/src-tauri/icons/bundle"

SHEET="$PACK/2_Characters/Character_Generator/0_Premade_Characters/16x16/Premade_Character_$CHAR.png"
[ -f "$SHEET" ] || { echo "character sheet not found: $SHEET" >&2; exit 1; }

# The front-facing idle pose, which is the same frame the awake pet uses. Row 0 is one pose per
# facing rather than an animation, and x=48 is the front (compose-rooms.sh explains how that
# was established). Cropped to its content so the 8 transparent rows above the head do not
# silently become part of the layout here.
IDLE_X=48
CANVAS=1024

# The macOS icon grid: the artwork sits inside a rounded square inset from the canvas rather
# than bleeding to the edge, and the system draws no mask of its own. 824 of 1024 with a corner
# radius of 185 is the Big Sur proportion, so the icon sits correctly next to system ones
# instead of looking slightly too large, which is the usual tell of a hand-rolled icon.
PLATE=824
RADIUS=185

# The app's own palette, from src/style.css: the popover's mat, and the mount line that frames
# the room in both the popover and the share card. Reusing them is what makes the icon read as
# this application rather than as a sprite on a square.
MAT="#191924"
MOUNT="#3a3a50"
EDGE=12

# Whole-number scaling only, here as everywhere. 26x on a 16x24 sprite is 416x624, which leaves
# an even margin inside the plate at the sides and a little more below than above: the character
# reads as standing on something rather than floating in the middle.
SCALE=26
DROP=24

WORK=$(mktemp -d -t mascot-icon)
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$OUT"

magick "$SHEET" -crop 16x32+$IDLE_X+0 +repage -trim +repage "$WORK/char.png"
magick "$WORK/char.png" -filter point -resize "$((SCALE * 100))%" "$WORK/big.png"

INSET=$(( (CANVAS - PLATE) / 2 ))
FAR=$(( INSET + PLATE ))
HALF_EDGE=$(( EDGE / 2 ))

magick -size ${CANVAS}x${CANVAS} xc:none \
  -fill "$MAT" -stroke "$MOUNT" -strokewidth $EDGE \
  -draw "roundrectangle $((INSET + HALF_EDGE)),$((INSET + HALF_EDGE)) $((FAR - HALF_EDGE)),$((FAR - HALF_EDGE)) $RADIUS,$RADIUS" \
  "$WORK/big.png" -gravity center -geometry "+0+$DROP" -composite \
  PNG32:"$WORK/icon-1024.png"

# The iconset macOS actually reads. The small sizes are resampled SMOOTHLY on purpose: nearest
# neighbour from 1024 to 16 does not preserve pixel art, it destroys it, and a 16px Finder row
# is not a place pixel art can survive intact anyway.
SET="$WORK/icon.iconset"
mkdir -p "$SET"
for spec in "16 icon_16x16" "32 icon_16x16@2x" "32 icon_32x32" "64 icon_32x32@2x" \
            "128 icon_128x128" "256 icon_128x128@2x" "256 icon_256x256" \
            "512 icon_256x256@2x" "512 icon_512x512" "1024 icon_512x512@2x"; do
  set -- $spec
  magick "$WORK/icon-1024.png" -filter Lanczos -resize "$1x$1" "PNG32:$SET/$2.png"
done

iconutil -c icns "$SET" -o "$OUT/icon.icns"
cp "$WORK/icon-1024.png" "$OUT/icon.png"
magick "$WORK/icon-1024.png" -filter Lanczos -resize 128x128 "PNG32:$OUT/128x128.png"
magick "$WORK/icon-1024.png" -filter Lanczos -resize 256x256 "PNG32:$OUT/128x128@2x.png"
magick "$WORK/icon-1024.png" -filter Lanczos -resize 32x32 "PNG32:$OUT/32x32.png"

echo "  $OUT/icon.icns"
echo "  $OUT/icon.png (1024)"
