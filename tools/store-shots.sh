#!/usr/bin/env bash
#
# The five App Store screenshots, at exactly 2560x1600, with nothing resampled.
#
# The listing sells pixel art, so a screenshot that has been through a resize is not a smaller
# problem than a wrong one: soft edges are the single most visible way for this listing to look
# amateur. The whole design of this script is one idea. A 2x display's native capture is
# already 2x the logical size, 2560x1600 is exactly 1280x800 of logical space, and a CROP is
# not a resize. So every shot is a full-display capture cropped to an integer offset, and no
# path in here ever scales a screenshot.
#
# The one thing that is scaled is the share card, and it is scaled by exactly 2 with a point
# filter, which replicates pixels rather than blending them.
#
# Usage:
#   tools/store-shots.sh grab  <n> <name> [corner] [display]   capture and crop
#   tools/store-shots.sh crop  <file> <n> <name> [corner]      crop a capture you took
#   tools/store-shots.sh clip  <n> <name> [corner]             crop the capture on the clipboard
#   tools/store-shots.sh card  <file>|--clip <n> <name>        frame a 1200x630 share card
#   tools/store-shots.sh check                                 verify every shot's size
#
#   corner:  tr (default) | br | tl | bl | c
#
# `grab` needs Screen & System Audio Recording permission for the terminal, in System Settings
# > Privacy & Security. Without it `screencapture` fails with "could not create image from
# display", which reads like a bug and is a permission. `crop` and `clip` are the ways round it,
# and neither needs a permission change: take the shot with shift-cmd-5, whole display rather
# than a region, and either hand over the saved file or hold ctrl so it lands on the clipboard.
# Prefer those two to pasting a capture into a chat or a document, which usually downscales it.
#
# Output goes to docs/store-shots/, which is gitignored for the same reason docs/mockups is:
# these are derived LimeZu art. Uploading them to the listing is presentation of the app and
# not redistribution of the pack. See docs/app-store-licence-check.md.

set -euo pipefail

W=2560
H=1600
OUT="${MASCOT_SHOTS:-docs/store-shots}"

[ -f Cargo.toml ] || [ -d src-tauri ] || { echo "run this from the repository root" >&2; exit 1; }
command -v magick >/dev/null || { echo "ImageMagick is not installed" >&2; exit 1; }

size_of() {  # size_of <file> -> "WxH"
  magick identify -format '%wx%h' "$1"
}

# Where in a native capture the 2560x1600 window sits. Integer offsets only, which is the
# entire correctness argument: an odd offset would still be a crop, but a half-logical-pixel
# one, and the pet is drawn on the logical grid.
offset_for() {  # offset_for <corner> <srcW> <srcH> -> "+X+Y"
  local corner=$1 sw=$2 sh=$3
  if [ "$sw" -lt "$W" ] || [ "$sh" -lt "$H" ]; then
    echo "source is ${sw}x${sh}, smaller than ${W}x${H}: capture the whole display, not a region" >&2
    return 1
  fi
  local right=$((sw - W)) bottom=$((sh - H))
  case "$corner" in
    tr) echo "+$right+0" ;;
    br) echo "+$right+$bottom" ;;
    tl) echo "+0+0" ;;
    bl) echo "+0+$bottom" ;;
    c)  echo "+$((right / 2))+$((bottom / 2))" ;;
    *)  echo "unknown corner: $corner (tr, br, tl, bl, c)" >&2; return 1 ;;
  esac
}

# The clipboard, because ctrl-shift-cmd-3 and the "Copy to Clipboard" option in the shift-cmd-5
# panel are how a capture gets taken when the terminal has no Screen Recording permission, and
# they need none. The pasteboard holds the capture at NATIVE resolution, which is the part that
# matters: pasting the same capture into a chat window or a document usually does not.
#
# `«class PNGf»` is the pasteboard's PNG flavour, and asking for it fails rather than converting
# if the clipboard holds something else.
from_clipboard() {  # from_clipboard -> path of a temp png
  local out
  out=$(mktemp -t mascot-clip).png
  osascript >/dev/null 2>&1 <<AS
set f to (open for access (POSIX file "$out") with write permission)
write (the clipboard as «class PNGf») to f
close access f
AS
  [ -s "$out" ] || { echo "no PNG on the clipboard" >&2; return 1; }
  echo "$out"
}

crop_to() {  # crop_to <src> <dest> <corner>
  local src=$1 dest=$2 corner=$3
  local dims sw sh at
  dims=$(size_of "$src"); sw=${dims%x*}; sh=${dims#*x}
  at=$(offset_for "$corner" "$sw" "$sh")
  mkdir -p "$(dirname "$dest")"
  magick "$src" -crop "${W}x${H}${at}" +repage "$dest"
  printf '  %s  %s  cropped %s from %s at %s\n' "$(size_of "$dest")" "$dest" "$corner" "$dims" "$at"
}

case "${1:-}" in

grab)
  n=${2:?slot number}; name=${3:?slot name}; corner=${4:-tr}; display=${5:-1}
  tmp=$(mktemp -t mascot-shot).png
  trap 'rm -f "$tmp"' EXIT
  # -x is no shutter sound, -D picks the display. Deliberately NOT -R: a rect capture would
  # hand screencapture the scaling decision, and this script exists to keep it.
  screencapture -x -D "$display" "$tmp"
  crop_to "$tmp" "$OUT/$n-$name.png" "$corner"
  ;;

crop)
  src=${2:?source file}; n=${3:?slot number}; name=${4:?slot name}; corner=${5:-tr}
  crop_to "$src" "$OUT/$n-$name.png" "$corner"
  ;;

clip)
  n=${2:?slot number}; name=${3:?slot name}; corner=${4:-tr}
  src=$(from_clipboard) || exit 1
  trap 'rm -f "$src"' EXIT
  crop_to "$src" "$OUT/$n-$name.png" "$corner"
  ;;

card)
  src=${2:?source file, or --clip}; n=${3:?slot number}; name=${4:?slot name}
  if [ "$src" = --clip ]; then
    # The only place the app puts the card: there is no "save as" and there should not be one.
    src=$(from_clipboard) || { echo "click Share Status first" >&2; exit 1; }
    trap 'rm -f "$src"' EXIT
  fi
  dims=$(size_of "$src")
  [ "$dims" = "1200x630" ] || { echo "expected a 1200x630 share card, got $dims" >&2; exit 1; }
  mkdir -p "$OUT"
  # 2400x1260 on a 2560x1600 mat. The card is scaled by exactly 2 with a point filter, so
  # every source pixel becomes four identical ones and no edge is softened. The mat is the
  # card's own background (share.js CARD.mat), so the frame reads as part of the card rather
  # than as a screenshot of a card on a page.
  magick "$src" -filter point -resize 200% \
    -background '#191924' -gravity center -extent "${W}x${H}" \
    "$OUT/$n-$name.png"
  printf '  %s  %s  card at 2x on its own mat\n' "$(size_of "$OUT/$n-$name.png")" "$OUT/$n-$name.png"
  ;;

check)
  [ -d "$OUT" ] || { echo "no shots in $OUT" >&2; exit 1; }
  bad=0 count=0
  for f in "$OUT"/*.png; do
    [ -e "$f" ] || continue
    count=$((count + 1))
    dims=$(size_of "$f")
    if [ "$dims" = "${W}x${H}" ]; then
      printf '  ok    %s  %s\n' "$dims" "$f"
    else
      printf '  WRONG %s  %s\n' "$dims" "$f"; bad=1
    fi
  done
  printf '\n  %d shots, %s\n' "$count" "$([ "$bad" = 0 ] && echo "all ${W}x${H}" || echo "SOME WRONG")"
  [ "$count" = 5 ] || printf '  the listing wants 5, in the order in docs/app-store-listing.md\n'
  exit "$bad"
  ;;

*)
  sed -n '2,30p' "$0" | sed 's/^#\{1,2\} \{0,1\}//'
  exit 1
  ;;
esac
