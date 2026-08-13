#!/bin/sh
#
# Draws the app's own two marks:
#
#   src-tauri/icons/tray.png   the menu bar template image (spec section 6.2)
#   src-tauri/icons/icon.png   the application icon, which Tauri requires exists
#
# This is the one piece of art in the project that is NOT from the pack, and it is committed
# rather than gitignored for that reason. Two consequences worth knowing: the app still
# compiles on a machine with no licensed copy of Modern Interiors, and nothing here is
# covered by the pack's redistribution restriction.
#
# Why it is drawn rather than cropped. The first attempt cropped the character's own head off
# the idle sprite and reduced it to a silhouette, on the reasonable theory that the thing in
# the menu bar should be the thing in the room. It rendered as a solid black blob: at 16px a
# filled silhouette has no internal structure left, and every internal edge in the sprite is a
# colour change rather than a hole. Extracting the outline colour instead gave structure but
# read as a burger. The lesson is that a menu bar template is a different register from pixel
# art, and it is drawn AT the size it is used rather than reduced to it.
#
# The mark is a capped head and shoulders with two eye holes. Holes rather than dots, because
# a template image is a shape plus an alpha channel: macOS supplies the colour, so the only
# contrast available is between ink and no ink.

set -eu

ROOT=$(cd "$(dirname "$0")/.." && pwd)
OUT="$ROOT/src-tauri/icons/tray.png"
APP_ICON="$ROOT/src-tauri/icons/icon.png"

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$(dirname "$OUT")"

# '#' is ink, '.' is transparent. Row 5 is one pixel wider on each side: that ledge is the
# cap's brim, and it is the only thing separating this from a bare head.
cat > "$WORK/glyph.txt" <<'GLYPH'
................
................
....########....
...##########...
...##########...
..############..
...##########...
...##.####.##...
...##########...
....########....
.....######.....
................
...##########...
.##############.
.##############.
................
GLYPH

{
  echo P1
  echo "16 16"
  sed 's/\./0 /g; s/#/1 /g' "$WORK/glyph.txt"
} > "$WORK/glyph.pbm"

# The glyph is the alpha channel of a fully black image. PBM writes ink as black, so it is
# negated to become opacity.
magick "$WORK/glyph.pbm" -alpha off -negate PNG32:"$WORK/mask.png"
magick -size 16x16 xc:black PNG32:"$WORK/ink.png"
magick "$WORK/ink.png" "$WORK/mask.png" -alpha off -compose CopyOpacity -composite PNG32:"$OUT"
echo "wrote $OUT"

# The application icon. A menu-bar-only app barely shows one, but Tauri requires it to exist,
# so it is the same mark in the product's own colours: the share card's mat behind the
# popover's cream, scaled by a whole number with a point filter like everything else here.
magick -size 16x16 xc:'#f0e6d2' PNG32:"$WORK/cream.png"
magick "$WORK/cream.png" "$WORK/mask.png" -alpha off -compose CopyOpacity -composite \
  -filter point -resize 1600% \
  -background '#191924' -gravity center -extent 512x512 \
  PNG32:"$APP_ICON"
echo "wrote $APP_ICON"
