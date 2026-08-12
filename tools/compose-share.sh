#!/bin/sh
# compose-share.sh - build the 1200x630 share card for each of the four states.
#
# Phase 2 of docs/spec-v2.md section 12. The card is both the growth mechanism and the
# validation instrument (section 5), so it is designed as a static image before any code
# generates one. This script is that design, in the same form as tools/compose-rooms.sh:
# coordinates rather than art, and therefore safe to commit.
#
# Input:  docs/mockups/state-<s>-160x112.png, the frame-0 stills from compose-rooms.sh.
# Output: docs/mockups/share-<s>-1200x630.png, plus a 2x2 contact sheet.
#
# Env:  MASCOT_OUT   output directory (default docs/mockups)
#
# Run tools/compose-rooms.sh first; this consumes its stills.

set -eu

ROOT=$(cd "$(dirname "$0")/.." && pwd)
OUT="${MASCOT_OUT:-$ROOT/docs/mockups}"
FONT="$ROOT/assets/fonts/departure-mono/DepartureMono-Regular.otf"

[ -f "$FONT" ] || { echo "font not found: $FONT" >&2; exit 1; }
[ -f "$OUT/state-awake-160x112.png" ] || { echo "run tools/compose-rooms.sh first" >&2; exit 1; }

# ---------------------------------------------------------------------------
# Canvas
# ---------------------------------------------------------------------------
# 1200x630 is the standard social card. The room is 10:7 and the card is 1.91:1, so the
# room is integer-scaled and matted, never scaled fractionally to fill (section 5.2).

CW=1200 ; CH=630
SCALE=5                       # 160x112 -> 800x560
RW=$((160 * SCALE))
RH=$((112 * SCALE))
RX=$(( (CW - RW) / 2 ))       # 200, so 200px of mat either side
RY=12                         # 12px below the top edge, per section 5.2
FRAME=$SCALE                  # a one-room-pixel mount line around the picture
BAND=$((RY + RH + FRAME))     # 577: top of the footer band

# ---------------------------------------------------------------------------
# Type
# ---------------------------------------------------------------------------
# Departure Mono (SIL OFL 1.1) is drawn on an 11px cell with a 7px advance, and renders
# pixel-exact only at integer multiples of 11. Verified by upscaling an 11px render 5x
# with a point filter and diffing it against a direct 55px render: 0 pixels different.
# So every size here is 11 * n, and n is that text's pixel unit.
#
#   11px = 1x   7px/char        33px = 3x  21px/char
#   22px = 2x  14px/char        55px = 5x  35px/char, the room's own unit
#
# Chosen over Press Start 2P and Silkscreen. Press Start 2P is a fixed 8x8 cell, so the
# longest quote in section 4.6 runs to 1663px at 5x and does not fit an 800px room at any
# integer scale. Silkscreen renders lowercase as small caps, and an all-caps quote line
# contradicts the warm voice section 4.6 specifies. Departure Mono is the only one of the
# three that fits the popover's 320px room on one line: 42 chars at 11px is 296px.
FS_LABEL=55                   # shares the room's pixel unit, because it sits on the room
FS_QUOTE=22                   # sits on the mat, so it is free to be a caption
FS_URL=22                     # the one thing a viewer needs in order to find the tool
FS_META=11
ADV_URL=14                    # advances at those sizes, for right-aligning
ADV_META=7

# Sampled from the room itself so the mat is family with the art rather than a generic
# dark grey: #3a3a50 is the pack's outline colour, and the mat is that darkened.
MAT="#191924"
MOUNT="#3a3a50"
CREAM="#f0e6d2"
DIM_CREAM="#b6ab98"           # the URL: second in the hierarchy, behind the quote
SHADOW="#14141c"
MUTED="#7c7c96"

# The wordmark and the URL are the same string. Section 5.2 asked for both, and rendering
# "keepgoing.dev" delivers both in one, which buys the size the wordmark needs: at 11px it
# was invisible in a timeline thumbnail, and a growth mechanism nobody can read the name of
# is not one.
URL="keepgoing.dev"
CREDIT="art: limezu.itch.io"

# ---------------------------------------------------------------------------
# Per-state copy
# ---------------------------------------------------------------------------
# One representative quote per state, verbatim from section 4.6. The asleep card carries
# the longest line in the whole pool at 42 characters, so it is the layout's stress test
# as well as its hardest legibility case, being the dimmest room.
#
# Section 5.3: no project names, paths, commit messages, hashes, or timestamps. Ever.
# There is nothing in this file that could carry one, which is the point.

label_for() {
  case $1 in
    awake)    printf 'AWAKE' ;;
    dozing)   printf 'DOZING' ;;
    asleep)   printf 'DREAMING' ;;
    comeback) printf 'BACK!!!' ;;
  esac
}

quote_for() {
  case $1 in
    awake)    printf "I saw that commit. I'm telling everyone." ;;
    dozing)   printf "Still warm. I've got the seat." ;;
    asleep)   printf "Dreaming about that thing you're building." ;;
    comeback) printf 'YOU CAME BACK.' ;;
  esac
}

# The label carries the state's temperature: warm gold awake, cooling as it sleeps, hottest
# on the comeback. The first pass used the state's own tint at full strength and
# #8f9ec8-on-dimmed-grey was unreadable, so these are pushed to the light end of each hue
# and the outline does the contrast work rather than the fill.
accent_for() {
  case $1 in
    awake)    printf '#f5c65c' ;;
    dozing)   printf '#dce4f4' ;;
    asleep)   printf '#e4ecff' ;;
    comeback) printf '#ffd45e' ;;
  esac
}

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

# text <file> <size> <fill> <x> <baseline-y> <string>
# +antialias is mandatory. An antialiased pixel font is just a blurry font.
text() {
  magick "$1" +antialias -font "$FONT" -pointsize "$2" -fill "$3" \
    -annotate "+$4+$5" "$6" PNG32:"$1"
}

# text_shadow <file> <size> <fill> <x> <y> <string> <unit>
# A one-unit hard offset, never a blur. Enough for type on the flat mat.
text_shadow() {
  text "$1" "$2" "$SHADOW" "$(($4 + $7))" "$(($5 + $7))" "$6"
  text "$1" "$2" "$3" "$4" "$5" "$6"
}

# text_outline <file> <size> <fill> <x> <y> <string> <unit>
# A full one-unit outline on all four sides, which is how the pack outlines its own art.
# A drop shadow alone was not enough for the label: it sits on the wall, and the wall runs
# from full-brightness beige when awake to dimmed grey-blue when asleep, so any single fill
# is low-contrast against one end of that range. An outline makes the label independent of
# what is behind it, which means one colour rule works for all four states instead of four.
text_outline() {
  _f=$1 _s=$2 _c=$3 _x=$4 _y=$5 _t=$6 _u=$7
  text "$_f" "$_s" "$SHADOW" "$((_x - _u))" "$_y"          "$_t"
  text "$_f" "$_s" "$SHADOW" "$((_x + _u))" "$_y"          "$_t"
  text "$_f" "$_s" "$SHADOW" "$_x"          "$((_y - _u))" "$_t"
  text "$_f" "$_s" "$SHADOW" "$_x"          "$((_y + _u))" "$_t"
  text "$_f" "$_s" "$_c"     "$_x"          "$_y"          "$_t"
}

# text_right <file> <size> <fill> <right-edge-x> <y> <string> <advance>
text_right() {
  _len=$(printf '%s' "$6" | wc -c | tr -d ' ')
  text "$1" "$2" "$3" "$(($4 - _len * $7))" "$5" "$6"
}

# ---------------------------------------------------------------------------
# The card
# ---------------------------------------------------------------------------
# Settled by rendering three candidates and looking at them. The two that lost:
#
#   Room left, type in a right-hand column. Never touches the art and the label reads
#   perfectly on the mat, but a 310px column leaves the middle of the card empty and the
#   picture stops being a matted picture.
#
#   Label and quote both on the wall. Fails outright. The wall block is 310x165 at 5x on
#   paper, but the top rows are the wall's trim, so the label lands on the trim and the
#   quote's third line lands on the bed and the Z emote.
#
# And the arithmetic that decided where the quote goes. Section 5.2 put it "overlaid along
# the room's lower strip", and there is no such strip: the plant, the rug's gold-and-navy
# edge, and the cat fill the bottom twelve rows. It cannot go on the wall either. So it
# goes on the mat, in the footer band, and the band is only just big enough. A 5x room
# leaves 70px of vertical slack; a 22px quote plus a meta row plus margins needs 78px.
# The fix is that the band spans the full 1200px rather than only the room's 800px column,
# so the wordmark and credit live in the side mats, which were dead space in every
# candidate, and the quote gets the band's whole height to itself.
card() {
  state=$1 out=$2

  magick -size "${CW}x${CH}" "xc:$MAT" PNG32:"$out"
  magick "$OUT/state-$state-160x112.png" -filter point -resize "$((SCALE * 100))%" \
    +repage -bordercolor "$MOUNT" -border "$FRAME" PNG32:"$WORK/room.png"
  magick "$out" "$WORK/room.png" -geometry "+$((RX - FRAME))+$((RY - FRAME))" \
    -composite PNG32:"$out"

  # State label, on the upper-left wall. Room-space x 0..62 by y 8..33 is plain wallpaper
  # (the trim ends at y=8, the map starts at x=64, the bed at y=34), which at 5x is a
  # clear 310x125 block. "DREAMING" is the longest label at 8 * 35 = 280px, so it fits
  # with 30px to spare, and the map is the thing it would hit if it grew.
  text_outline "$out" "$FS_LABEL" "$(accent_for "$state")" \
    $((RX + 20)) $((RY + 108)) "$(label_for "$state")" "$SCALE"

  # Footer band: one baseline across the full width, reading credit, quote, URL. The credit
  # is the smallest thing on the card and the only one with a legal obligation behind it, so
  # it goes in the left mat where it is present and legible without competing. The quote
  # keeps the room's column. The URL is right-aligned to the canvas rather than to the room,
  # which puts 214px between it and the longest quote instead of 30px, so the two never read
  # as one sentence.
  text_right "$out" "$FS_META" "$MUTED" $((RX - 16)) $((BAND + 32)) "$CREDIT" "$ADV_META"
  text_shadow "$out" "$FS_QUOTE" "$CREAM" $RX $((BAND + 32)) "$(quote_for "$state")" 2
  text_right "$out" "$FS_URL" "$DIM_CREAM" $((CW - 16)) $((BAND + 32)) "$URL" "$ADV_URL"
}

# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

mkdir -p "$OUT"
sheet=""
for state in awake dozing asleep comeback; do
  out="$OUT/share-$state-1200x630.png"
  card "$state" "$out"
  sheet="$sheet $out"
  echo "  $out"
done

magick montage $sheet -tile 2x2 -geometry +10+10 -background "#0c0c12" \
  PNG32:"$OUT/share-four.png"
echo "  $OUT/share-four.png"
