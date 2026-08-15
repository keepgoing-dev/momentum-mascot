#!/usr/bin/env bash
#
# Composes the four room states from a local licensed copy of Modern Interiors.
#
# This script contains coordinates, not art. It is safe to commit. The pack it
# reads is not: see spec-v2.md section 4.2. Output lands in docs/mockups/, which
# is gitignored for the same reason.
#
# Every state is a 12-frame loop. The still PNG is frame 0.
#
# Usage:  tools/compose-rooms.sh [state ...]        default: all four
# Env:    MASCOT_PACK   root of moderninteriors-win   (default below)
#         MASCOT_OUT    output directory              (default docs/mockups)
#         MASCOT_CHAR   premade character number      (default 07)
#         MASCOT_ZOOM   preview scale factor          (default 4)
#
set -euo pipefail

PACK="${MASCOT_PACK:-$HOME/Workspace/OneQode/projects/repos/oneqode-pixel-assets/moderninteriors-win}"
OUT="${MASCOT_OUT:-docs/mockups}"
CHAR="${MASCOT_CHAR:-07}"
ZOOM="${MASCOT_ZOOM:-4}"

# When set, the app's own asset tree is written too: one room strip and one pet strip per
# state, under the character's own folder. Set by tools/build-app-assets.sh, which is the
# only caller that wants it. Review artifacts and shipped assets are kept apart on purpose,
# because the GIFs are for looking at and are never shipped.
APP_OUT="${MASCOT_APP_OUT:-}"

[ -d "$PACK" ] || { echo "asset pack not found: $PACK" >&2; exit 1; }
mkdir -p "$OUT"
WORK=$(mktemp -d -t mascot-compose)
trap 'rm -rf "$WORK"' EXIT

# ---------------------------------------------------------------- source roots

I16="$PACK/1_Interiors/16x16"
SIN="$I16/Theme_Sorter_Singles"
RB="$I16/Room_Builder_subfiles"
CHARS="$PACK/2_Characters/Character_Generator/0_Premade_Characters/16x16"
UI="$PACK/4_User_Interface_Elements"
ANIM="$PACK/3_Animated_objects/16x16/spritesheets"

CHAR_SHEET="$CHARS/Premade_Character_$CHAR.png"
EMOTES="$UI/UI_thinking_emotes_animation_16x16.png"

# ---------------------------------------------------------------- room geometry
# 10 by 7 tiles on the 16px native grid. Wall band is the top 34px, which is
# exactly one wall strip from the Room Builder sheet: 2px outline, 5px top cap,
# 20px face, 4px baseboard, 2px outline. Do not round this to 32.

W=160
H=112
WALL_H=34

# ---------------------------------------------------------------- surface picks
# Both are deliberate choices from the Room Builder sheets, replacing the
# placeholders that were sampled out of Generic_Home_1.
#   floor  0,384  light warm plank, quiet enough to sit under furniture
#   wall   0,224  warm beige with a faint weave, plus its own baseboard

FLOOR_TILE="16x16+0+384"
WALL_STRIP="16x34+16+222"

# ---------------------------------------------------------------- prop placement
# Everything is "<sprite> at +x+y" where x,y is the sprite's top-left in room
# pixels. Bottom edges matter more than top edges: a prop reads as standing on
# the floor when its base lands on a tile boundary.

# The bookshelf is TWO sprites. Singles_60 is 32x48 and is not a whole bookcase:
# it is the left and middle of a 48-wide four-bay one, cut off mid-shelf with no
# right-hand frame and no right leg. Singles_61 (16x48) is the missing cap. Used
# alone, 60 loses a corner, which is exactly what it looks like. Nothing in the
# filenames says this; the pack numbers wide props as a body plus a cap, and the
# only way to tell is to butt them together and look.
BOOKSHELF_BODY="$SIN/5_Classroom_and_Library_Singles/Classroom_and_Library_Singles_60.png"   # 32x48
BOOKSHELF_CAP="$SIN/5_Classroom_and_Library_Singles/Classroom_and_Library_Singles_61.png"    # 16x48
BOOKSHELF_AT="+106+6"
BOOKSHELF_CAP_AT="+138+6"

# A SINGLE bed, 16 wide, not the 32-wide double the prototype used. The sleep
# animation is a 16px head laid on a pillow; on a double bed that head reads as
# a doll dropped on the covers, because the bed is twice the width of the person.
BED="$SIN/4_Bedroom_Singles/Bedroom_Singles_149.png"                                    # 16x48
BED_AT="+10+34"

MAP="$SIN/5_Classroom_and_Library_Singles/Classroom_and_Library_Singles_31.png"         # 32x32
MAP_AT="+64+2"

LAMP="$SIN/2_Living_Room_Singles/Living_Room_Singles_87.png"                            # 16x48
LAMP_AT="+30+36"

PLANT="$SIN/2_Living_Room_Singles/Living_Room_Singles_17.png"                           # 16x48
PLANT_AT="+2+64"

# Kept red after comparing against the blue and the two small rugs. Red is the
# worst of the four under the 34% asleep tint, where it goes maroon, and the
# best in comeback, where the saturation boost makes it the thing your eye lands
# on. Comeback is the state the product is for. docs/mockups/rug-variants.png
# is the comparison; swapping is one crop.
#
# The crop is the rug's exact bounds, x 146..205 and y 68..107, found by dumping
# the region as text and reading where the outline's alpha goes from 0 to 255.
# An earlier 62x40+144+64 was four rows short at the bottom, so the rug stood in
# the room with its bottom outline and the last of the red band sliced off.
#
# That is easy to get wrong and worth knowing about: the pack pads sprites with
# THE OUTLINE COLOUR AT ALPHA ZERO (#3A3A5000 sitting next to #3A3A50FF), so on a
# dark background the padding and the real edge look identical, and a crop that
# looks right is not. Trimming does not help either, because the sheet is dense
# enough that a hand-drawn box catches the neighbouring sprite.
RUG_SRC="$I16/Theme_Sorter/1_Generic_16x16.png"
RUG_CROP="60x40+146+68"
RUG_AT="+48+68"

# The workstation, and the one piece of composition this room actually turns on.
#
# The pack has no computer desk and no seated back view, so a front-facing sprite
# and a top-down desk cannot both be right about which way a screen points. The
# pack solves this itself in animated_receptionist: a grey tower seen from
# BEHIND, sitting on a counter, with the user's head above it. Nothing has to
# face the wrong way. The computer here is cropped out of that animation, which
# is the only place in 48,000 files it exists as art.
#
# Draw order is load-bearing: character, then desk, then computer. The desk
# covers the character's lower body, which is what makes them read as sitting
# at it rather than standing behind it.

DESK="$SIN/5_Classroom_and_Library_Singles/Classroom_and_Library_Singles_25.png"        # 32x32
DESK_AT="+108+58"

COMPUTER_SRC="$ANIM/animated_receptionist.png"
COMPUTER_CROP_H="16x11"
COMPUTER_CROP_Y=12
# The receptionist's 7 frames only ever put the tower in one of three states:
# 0/2/4 are identical, 1/3 are identical, 5/6 are identical. Three frames is the
# whole animation, and three divides 12.
COMPUTER_XS="0 16 80"
COMPUTER_AT="+111+65"

COFFEE_SRC="$ANIM/animated_coffee.png"
# Rotated so frame 0 has the plume. Frame 0 is the still that goes into the
# share image, and the cup without steam is a grey blob at this size.
COFFEE_XS="32 48 64 80 0 16"

# The cat. 576x16 is 12 frames of 48x16, not 36 of 16x16, and the cat is drawn
# in the middle of each cell with the tail trailing left. Cropping every frame
# at the same sub-offset is what stops it twitching sideways: the union of all
# 12 bounding boxes is x 7..34, so 28x16+7 holds every frame with no jitter.
# The body never moves. Only the tail does, which is why the cat can be in all
# four states without ever making the room feel busy.
CAT_SRC="$ANIM/animated_cat.png"
CAT_CELL=48
CAT_CROP_W=28
CAT_CROP_X=7
CAT_FRAME_COUNT=12
CAT_AT="+76+84"

# ---------------------------------------------------------------- character
# All four states use frames from one premade sheet, so switching characters
# costs one PNG. Sprites are 16x32 and are placed by their top-left.
#
# Row 0 is NOT an idle animation. It is one pose per facing, in the order
# left, up, right, down, so x=0 is a left-facing side view: a hat with a sliver
# of cheek under it, which is what the standing character used to be. The
# front-facing pose is x=48. Proof rather than guesswork: flopping x=0
# reproduces x=32 to within 4 pixels, so those two are the side pair.
IDLE_X=48
IDLE_Y=0

# The run is the sheet's side-view walk row (y=32), six frames per facing in the same
# left/up/right/down order as row 0. The right-facing six (x=192..272) are the run: a real
# stride with a 1px vertical bob baked in, unlike the front-facing walk which is mostly
# static. The frontend flips the strip with scaleX(-1) for leftward travel, so one strip
# serves both directions and only the right-facing half is ever cropped.
RUN_XS="192 208 224 240 256 272"
RUN_Y=32

# Row 192 is seated, and the sheet has "4-9 loop" written next to it in pixels.
# Those are 1-indexed, so the loop is x=48..128. Its shape is palindromic: four
# frames that differ from rest by 15 to 36 pixels, two that differ by ~195, then
# back. The big pair is the character leaning in. That is the typing loop.
SEATED_XS="48 64 80 96 112 128"
SEATED_Y=192

# The sleep row is the one place the sheet needs reading rather than indexing.
# 192,96 is a BALD head with a tan blanket, a generic overlay for any bed. 0,96
# is the same head wearing this character's own hat, with no blanket. The hat is
# the whole point of picking a character, so take 0,96 and let the bed supply
# the blanket. (asset-picks.md said 191,96; the sprite actually starts at 192.)
# The six frames are the same palindrome as the seated row: 0/1/4/5 identical,
# 2/3 differing by 164px. That is breathing.
SLEEP_XS="0 16 32 48 64 80"
SLEEP_Y=96

# The blanket, which the pet needs and the room does not.
#
# 192,96 is not the clean overlay the note above implies: it is the sleeper already composited
# into a bed, so the cell carries headboard posts and side rails as well. The blanket itself is
# the bottom half, rows 16..32, and those sixteen rows are clean. Its outermost columns are the
# pack's own outline navy rather than bed rail, so the band needs no trimming sideways.
#
# It is one frame, not six. The room's bed supplies a static blanket too, so a still blanket
# with a breathing head inside it is the same arrangement, not a shortcut.
SLEEP_BLANKET_X=192
SLEEP_BLANKET_Y=112

CHAR_AWAKE_AT="+111+41"       # behind the desk, head clear of the computer
CHAR_DOZING_AT="+54+56"       # away from the desk, coffee in hand
CHAR_COMEBACK_AT="+54+64"     # out of bed, on the rug
SLEEP_OVERLAY_AT="+10+35"     # head centred on the bed's pillow, bed-local y 8..16

# ---------------------------------------------------------------- emotes
# The emote always sits above the character's own head. In the prototype the Z
# floated over the desk instead, which read as the room being sleepy rather
# than the person. Each emote is a 2-frame pair on adjacent cells: the sheet's
# own note reads "sample animation, just swap the last 2".

EMOTE_Z_XS="96 112"
EMOTE_Z_Y=80
EMOTE_BANG_XS="0 16"
EMOTE_BANG_Y=80
EMOTE_SPARK_XS="64 80"
EMOTE_SPARK_Y=96

# Dozing gets the three dots, and asleep keeps the Z. They used to share the Z, and sharing it
# was the whole reason the two states were indistinguishable on the pet: the emote is what
# carries state there, so two states with one emote are one state as far as anyone glancing at
# it is concerned. The dots are a two-frame pulse that reads as trailing off rather than as
# sleeping, which is what 24 hours away actually is.
EMOTE_DOTS_XS="32 48"
EMOTE_DOTS_Y=144
EMOTE_DX=8                    # offset from the character's top-left
EMOTE_DY=-13
SLEEP_EMOTE_DX=13             # a sleeper has no headroom, so the Z drifts right
SLEEP_EMOTE_DY=-9

# ---------------------------------------------------------------- the pet
# The desktop pet (spec section 6.1) is the character ONLY, never the room: the pet is the
# character, the popover is the scene. 64x64 on screen is a 32x32 cell at 2x.
#
# The cell is decided by measurement, not by taste. A character sprite is 16x32 whose CONTENT
# is 16x24 at +0+8, so there are exactly 8 transparent rows above the head. An emote is 16x16
# with 15 rows of content. In the room the emote sits 13px ABOVE the character and clears the
# head completely; in a 32px cell there is no such room, and 8 rows will not hold 15.
#
# So the emote moves from above the head to BESIDE it, which is the one arrangement where the
# two 16-wide sprites tile the 32-wide cell with no overlap at all. Character in the left
# half, emote in the right, aligned with the head rather than the body.
#
# The character sits 2px high of the cell floor so that both directions are available without
# clipping: the comeback hop goes 2px up, the asleep slump 2px down, and neither loses a foot.
PET_W=32
PET_H=32
PET_CHAR_X=0
PET_CHAR_Y=-2
PET_RUN_X=8                  # the run is side-view; centred so a scaleX(-1) flip stays put
PET_EMOTE_DX=16               # relative to the character, so the emote moves with them
PET_EMOTE_DY=4
PET_SPARK_DX=16
PET_SPARK_DY=18

# Motion is RESERVED on the pet, and this is a rule rather than a preference (section 6.1).
# Every room state animates, but the pet is the one surface sitting in peripheral vision all
# day, and a sprite that moves constantly AND largely in the corner of someone's eye is the
# thing people quit. The distinction is amplitude, not presence: a 1px breath is fine, a tail
# sweep or a hop is not, which is why the cat and the typing loop stay in the room.
PET_BREATH="0 0 0 0 0 0 -1 -1 -1 -1 -1 -1"

# Where the blanket's top edge sits relative to the sleeping head. 16 puts it a row under the
# chin, so the head is tucked in rather than resting on top of the covers.
PET_BLANKET_DY=16

# The sleeping pose reserves its clearance at the BOTTOM of the cell, not the top, which is the
# opposite of every other state and the reason it does not use PET_CHAR_Y.
#
# The others sit 2px high so a 2px hop has somewhere to go. A sleeper never hops, and the sheets
# put the head's topmost row at +2 for two of the three characters, so borrowing those 2px above
# would leave the cap touching the cell edge with nothing to spare. Spending them below instead
# lets the blanket run off the bottom of the frame, which is how bedding behaves.
PET_SLEEP_Y=0

# The pet's own rates. Slower than the room everywhere except the comeback, which is the one
# moment the pet is allowed to be loud.
PET_FPS_AWAKE=3
PET_FPS_DOZING=2
PET_FPS_ASLEEP=2
PET_FPS_COMEBACK=8
PET_FPS_RUN=16

# ---------------------------------------------------------------- animation
# One loop length for every state, so each layer is indexed frame % count and
# every layer's cycle closes at the same place. 12 is divisible by every layer
# count in use: 12 (cat), 6 (character, coffee), 3 (computer), 2 (emote).
#
# Amplitude is what separates the states, not whether they move at all. The
# room is awake at 6fps and asleep at 2fps with the same 12 frames.
FRAMES=12
FPS_AWAKE=6
FPS_DOZING=3
FPS_ASLEEP=2
FPS_COMEBACK=8
FPS_SHEET=4                   # the side-by-side contact sheet, in lockstep

# A standing pose has no breathing frames on the sheet, so the vertical offset
# supplies them. One pixel is a breath; two is a hop. Both read at 16px because
# the character is only 32 tall.
HOP_DOZING="0 0 0 0 0 0 1 1 1 1 1 1"
HOP_COMEBACK="0 -1 -2 -2 -1 0 0 -1 -2 -2 -1 0"

# ---------------------------------------------------------------- lighting
# A flat colour applied over the finished frame, not redrawn art. Retuning a
# mood is one number here.

TINT_COLOUR="#3050a0"
TINT_DOZING=10
TINT_ASLEEP=34
COMEBACK_MODULATE="113,120"   # brightness,saturation

# ================================================================== helpers

shift_pos() {  # shift_pos +x+y dx dy  ->  +x+y
  local at=$1 x y
  x=${at%+*}; x=${x#+}
  y=${at##*+}
  printf '+%d+%d' "$((x + $2))" "$((y + $3))"
}

emote_pos() {  # emote_pos +x+y  ->  +x+y shifted to sit above a character
  shift_pos "$1" "$EMOTE_DX" "$EMOTE_DY"
}

# shift_pos cannot express a negative offset, because it splits the string on '+'. The pet
# needs them, so the pet works in plain integers and formats at the point of use.
geom() {  # geom <x> <y>  ->  +x+y, with either sign
  printf '%+d%+d' "$1" "$2"
}

nth() {  # nth <i> <word> ...   0-indexed, wraps. No arrays: zsh indexes from 1.
  local i=$1; shift
  i=$(( i % $# ))
  shift "$i"
  printf '%s' "$1"
}

cut_row() {  # cut_row <name> <sheet> <WxH> <y> <x> ...   -> $WORK/<name>-N.png
  local name=$1 sheet=$2 size=$3 y=$4; shift 4
  local n=0 list=""
  for x in "$@"; do
    magick "$sheet" -crop "$size+$x+$y" +repage PNG32:"$WORK/$name-$n.png"
    list="$list $WORK/$name-$n.png"
    n=$((n + 1))
  done
  printf '%s' "${list# }"
}

# ================================================================== composition

build_base() {
  # Floor first, wall painted over the top band so props can overhang it.
  # PNG32 is not decoration: a canvas made with xc:none is written as
  # greyscale-plus-alpha, and every colour composited onto it afterwards is
  # silently converted to grey.
  magick -size ${W}x${H} xc:none PNG32:"$WORK/base.png"

  magick "$RB/Room_Builder_Floors_16x16.png" -crop "$FLOOR_TILE" +repage "$WORK/floor.png"
  magick -size ${W}x$((H - WALL_H)) tile:"$WORK/floor.png" "$WORK/floorband.png"
  magick "$WORK/base.png" "$WORK/floorband.png" -geometry "+0+$WALL_H" -composite PNG32:"$WORK/base.png"

  magick "$RB/Room_Builder_Walls_16x16.png" -crop "$WALL_STRIP" +repage "$WORK/wall.png"
  magick -size ${W}x${WALL_H} tile:"$WORK/wall.png" "$WORK/wallband.png"
  magick "$WORK/base.png" "$WORK/wallband.png" -geometry "+0+0" -composite PNG32:"$WORK/base.png"

  # Static props, back to front. The desk is deliberately absent: it is
  # composited per state, after the character, so that it occludes them.
  magick "$RUG_SRC" -crop "$RUG_CROP" +repage "$WORK/rug.png"
  magick "$WORK/base.png" "$WORK/rug.png"     -geometry "$RUG_AT"           -composite \
                          "$MAP"              -geometry "$MAP_AT"           -composite \
                          "$BOOKSHELF_BODY"   -geometry "$BOOKSHELF_AT"     -composite \
                          "$BOOKSHELF_CAP"    -geometry "$BOOKSHELF_CAP_AT" -composite \
                          "$BED"              -geometry "$BED_AT"           -composite \
                          "$LAMP"             -geometry "$LAMP_AT"          -composite \
                          "$PLANT"            -geometry "$PLANT_AT"         -composite \
                          PNG32:"$WORK/base.png"
}

build_layers() {
  COMPUTER_FRAMES=$(cut_row computer "$COMPUTER_SRC" "$COMPUTER_CROP_H" "$COMPUTER_CROP_Y" $COMPUTER_XS)
  COFFEE_FRAMES=$(cut_row coffee "$COFFEE_SRC" 16x32 0 $COFFEE_XS)

  local n=0 xs=""
  while [ $n -lt $CAT_FRAME_COUNT ]; do
    xs="$xs $((n * CAT_CELL + CAT_CROP_X))"
    n=$((n + 1))
  done
  CAT_FRAMES=$(cut_row cat "$CAT_SRC" "${CAT_CROP_W}x16" 0 $xs)

  IDLE_FRAMES=$(cut_row idle   "$CHAR_SHEET" 16x32 "$IDLE_Y"   $IDLE_X)
  SEATED_FRAMES=$(cut_row seated "$CHAR_SHEET" 16x32 "$SEATED_Y" $SEATED_XS)
  SLEEP_FRAMES=$(cut_row sleep  "$CHAR_SHEET" 16x32 "$SLEEP_Y"  $SLEEP_XS)
  RUN_FRAMES=$(cut_row run      "$CHAR_SHEET" 16x32 "$RUN_Y"    $RUN_XS)

  BLANKET=$(cut_row blanket "$CHAR_SHEET" 16x16 "$SLEEP_BLANKET_Y" $SLEEP_BLANKET_X)

  Z_FRAMES=$(cut_row z         "$EMOTES" 16x16 "$EMOTE_Z_Y"     $EMOTE_Z_XS)
  DOTS_FRAMES=$(cut_row dots   "$EMOTES" 16x16 "$EMOTE_DOTS_Y"  $EMOTE_DOTS_XS)
  BANG_FRAMES=$(cut_row bang   "$EMOTES" 16x16 "$EMOTE_BANG_Y"  $EMOTE_BANG_XS)
  SPARK_FRAMES=$(cut_row spark "$EMOTES" 16x16 "$EMOTE_SPARK_Y" $EMOTE_SPARK_XS)
}

# The cat goes down before the character in every state: it is a background
# animal, and if the two ever overlap the person wins.
frame_awake() {  # frame_awake <i> <out>
  local i=$1 out=$2
  magick "$WORK/base.png" \
    "$(nth "$i" $CAT_FRAMES)"      -geometry "$CAT_AT"         -composite \
    "$(nth "$i" $SEATED_FRAMES)"   -geometry "$CHAR_AWAKE_AT"  -composite \
    "$DESK"                        -geometry "$DESK_AT"        -composite \
    "$(nth "$i" $COMPUTER_FRAMES)" -geometry "$COMPUTER_AT"    -composite \
    PNG32:"$out"
}

frame_dozing() {  # away from the desk with a coffee, not standing next to it
  local i=$1 out=$2
  local at; at=$(shift_pos "$CHAR_DOZING_AT" 0 "$(nth "$i" $HOP_DOZING)")
  magick "$WORK/base.png" \
    "$(nth "$i" $CAT_FRAMES)"      -geometry "$CAT_AT"      -composite \
    "$DESK"                        -geometry "$DESK_AT"     -composite \
    "$(nth "$i" $COMPUTER_FRAMES)" -geometry "$COMPUTER_AT" -composite \
    "$(nth "$i" $IDLE_FRAMES)"     -geometry "$at"          -composite \
    "$(nth "$i" $COFFEE_FRAMES)"   -geometry "$(shift_pos "$at" 12 -4)" -composite \
    "$(nth "$i" $DOTS_FRAMES)"     -geometry "$(emote_pos "$at")"       -composite \
    -fill "$TINT_COLOUR" -colorize "$TINT_DOZING" PNG32:"$out"
}

frame_asleep() {
  local i=$1 out=$2
  magick "$WORK/base.png" \
    "$(nth "$i" $CAT_FRAMES)"      -geometry "$CAT_AT"           -composite \
    "$DESK"                        -geometry "$DESK_AT"          -composite \
    "$(nth "$i" $COMPUTER_FRAMES)" -geometry "$COMPUTER_AT"      -composite \
    "$(nth "$i" $SLEEP_FRAMES)"    -geometry "$SLEEP_OVERLAY_AT" -composite \
    "$(nth "$i" $Z_FRAMES)" -geometry "$(shift_pos "$SLEEP_OVERLAY_AT" "$SLEEP_EMOTE_DX" "$SLEEP_EMOTE_DY")" -composite \
    -fill "$TINT_COLOUR" -colorize "$TINT_ASLEEP" PNG32:"$out"
}

frame_comeback() {
  local i=$1 out=$2
  local at; at=$(shift_pos "$CHAR_COMEBACK_AT" 0 "$(nth "$i" $HOP_COMEBACK)")
  magick "$WORK/base.png" \
    "$(nth "$i" $CAT_FRAMES)"      -geometry "$CAT_AT"      -composite \
    "$DESK"                        -geometry "$DESK_AT"     -composite \
    "$(nth "$i" $COMPUTER_FRAMES)" -geometry "$COMPUTER_AT" -composite \
    "$(nth "$i" $IDLE_FRAMES)"     -geometry "$at"          -composite \
    "$(nth "$i" $BANG_FRAMES)"     -geometry "$(emote_pos "$at")"        -composite \
    "$(nth "$i" $SPARK_FRAMES)"    -geometry "$(shift_pos "$at" -9 -4)"  -composite \
    "$(nth "$i" $SPARK_FRAMES)"    -geometry "$(shift_pos "$at" 21 -4)"  -composite \
    -modulate "$COMEBACK_MODULATE" PNG32:"$out"
}

# ---------------------------------------------------------------- the pet
# The art gap that was claimed here turned out not to exist, and the claim did real damage while
# it stood. It read: the pack has no sleeping character without a bed, so dozing and asleep must
# share the seated pose and be separated by a 2px slump. That was accepted rather than tested,
# and the result was a pet with three visible states instead of four, which is what the author
# said the moment he saw the two frames next to each other.
#
# The sleep row does need a bed to be read directly, but the blanket band is separable from the
# furniture around it, and a capped head under a blanket needs no bed to say asleep. What made
# this look impossible was reasoning about the sheet instead of cropping it.
#
# Awake is the seated pose, the same one the room shows at the desk. It is the rest frame of
# the seated row, never the typing loop: a pet leaning in to type in the corner of the eye all
# day is exactly the thing the amplitude rule bans. Dozing then takes the standing idle pose,
# which matches the room's dozing (standing with coffee) and undoes the old inversion where the
# pet's working state stood around while its dozing state sat down.

frame_pet_awake() {
  local i=$1 out=$2
  local dy=$((PET_CHAR_Y + $(nth "$i" $PET_BREATH)))
  magick -size ${PET_W}x${PET_H} xc:none \
    "$(nth 0 $SEATED_FRAMES)" -geometry "$(geom $PET_CHAR_X $dy)" -composite \
    PNG32:"$out"
}

frame_pet_dozing() {
  local i=$1 out=$2
  local dy=$((PET_CHAR_Y + $(nth "$i" $PET_BREATH)))
  magick -size ${PET_W}x${PET_H} xc:none \
    "$(nth "$i" $IDLE_FRAMES)" -geometry "$(geom $PET_CHAR_X $dy)" -composite \
    "$(nth "$i" $DOTS_FRAMES)" \
      -geometry "$(geom $((PET_CHAR_X + PET_EMOTE_DX)) $((dy + PET_EMOTE_DY)))" -composite \
    PNG32:"$out"
}

# Asleep is the sleeping pose under a blanket, and it is the only pet state that is not the
# standing or seated character.
#
# The first version had this sharing the seated pose with dozing and separated only by a 2px
# slump, on the reasoning that the pack has no sleeping character without a bed. That reasoning
# was wrong in a way only a person looking at the result catches: the two frames differed by two
# pixels and one emote they had in common, so the pet had three states rather than four. The
# blanket band exists on the character sheet independently of the furniture, which is what makes
# this possible without putting a bed on the pet.
#
# The head breathes on its own six frames and the blanket does not move, so what animates is a
# sleeper shifting under bedding rather than the bedding shifting with them.
frame_pet_asleep() {
  local i=$1 out=$2
  magick -size ${PET_W}x${PET_H} xc:none \
    "$(nth "$i" $SLEEP_FRAMES)" -geometry "$(geom $PET_CHAR_X $PET_SLEEP_Y)" -composite \
    "$BLANKET" -geometry "$(geom $PET_CHAR_X $((PET_SLEEP_Y + PET_BLANKET_DY)))" -composite \
    "$(nth "$i" $Z_FRAMES)" \
      -geometry "$(geom $((PET_CHAR_X + PET_EMOTE_DX)) $((PET_SLEEP_Y + PET_EMOTE_DY)))" \
      -composite \
    PNG32:"$out"
}

# The one state the pet is allowed to be loud in, and the reason the pet exists at all: the
# comeback plays out in peripheral vision the moment the commit lands, with no banner and no
# click. Only one sparkle rather than the room's two, because the cell has one 16x16 slot
# left once the character and the emote have tiled it.
frame_pet_comeback() {
  local i=$1 out=$2
  local dy=$((PET_CHAR_Y + $(nth "$i" $HOP_COMEBACK)))
  magick -size ${PET_W}x${PET_H} xc:none \
    "$(nth "$i" $IDLE_FRAMES)" -geometry "$(geom $PET_CHAR_X $dy)" -composite \
    "$(nth "$i" $BANG_FRAMES)" \
      -geometry "$(geom $((PET_CHAR_X + PET_EMOTE_DX)) $((dy + PET_EMOTE_DY)))" -composite \
    "$(nth "$i" $SPARK_FRAMES)" \
      -geometry "$(geom $((PET_CHAR_X + PET_SPARK_DX)) $((dy + PET_SPARK_DY)))" -composite \
    PNG32:"$out"
}

# The run is the only pet state that is not the front-facing or seated character: it is the
# sheet's side-view walk, so the mascot reads as running when the frontend plays it during a
# drag or a glide. It is the pet-only state, composed here rather than in the room loop, and
# the frontend flips it with scaleX(-1) to face whichever way it is travelling. No emote and
# no breath offset: the stride carries the motion and a hop on top of it would fight it.
frame_pet_run() {
  local i=$1 out=$2
  magick -size ${PET_W}x${PET_H} xc:none \
    "$(nth "$i" $RUN_FRAMES)" -geometry "$(geom $PET_RUN_X $PET_CHAR_Y)" -composite \
    PNG32:"$out"
}

pet_fps_for() {
  case $1 in
    awake)    printf '%s' "$PET_FPS_AWAKE" ;;
    dozing)   printf '%s' "$PET_FPS_DOZING" ;;
    asleep)   printf '%s' "$PET_FPS_ASLEEP" ;;
    comeback) printf '%s' "$PET_FPS_COMEBACK" ;;
    run)      printf '%s' "$PET_FPS_RUN" ;;
  esac
}

fps_for() {  # fps_for <state> -> frames per second
  case $1 in
    awake)    printf '%s' "$FPS_AWAKE" ;;
    dozing)   printf '%s' "$FPS_DOZING" ;;
    asleep)   printf '%s' "$FPS_ASLEEP" ;;
    comeback) printf '%s' "$FPS_COMEBACK" ;;
  esac
}

# ================================================================== run

STATES=("$@")
[ ${#STATES[@]} -eq 0 ] && STATES=(awake dozing asleep comeback)

build_base
build_layers

for s in "${STATES[@]}"; do
  strip=""
  i=0
  while [ $i -lt $FRAMES ]; do
    "frame_$s" "$i" "$WORK/$s-f$i.png"
    strip="$strip $WORK/$s-f$i.png"
    i=$((i + 1))
  done

  # The still is frame 0, which is the rest pose in every table.
  cp "$WORK/$s-f0.png" "$OUT/state-$s-${W}x${H}.png"

  # One horizontal strip per state is what the app consumes: a single sprite
  # sheet stepped with a CSS steps() animation, no runtime canvas loop.
  magick $strip +append PNG32:"$OUT/state-$s-strip-${FRAMES}f.png"

  # GIF is for looking at, not for shipping. Each state runs at its own rate.
  fps=$(fps_for "$s")
  magick -delay $((100 / fps)) -loop 0 $strip \
    -filter point -resize "$((ZOOM * 100))%" -layers optimize "$OUT/state-$s.gif"

  # The app's assets. The pet gets NO lighting tint, unlike the room: a blue multiply over a
  # character standing on someone's desktop wallpaper reads as a recoloured sprite rather
  # than as dim light, because there is no room around them for the light to be in. That is
  # why section 6.1 gives the emote the job the room's lighting does.
  if [ -n "$APP_OUT" ]; then
    petstrip=""
    i=0
    while [ $i -lt $FRAMES ]; do
      "frame_pet_$s" "$i" "$WORK/pet-$s-f$i.png"
      petstrip="$petstrip $WORK/pet-$s-f$i.png"
      i=$((i + 1))
    done
    mkdir -p "$APP_OUT/rooms/$CHAR" "$APP_OUT/pet/$CHAR"
    magick $petstrip +append PNG32:"$APP_OUT/pet/$CHAR/$s.png"
    cp "$OUT/state-$s-strip-${FRAMES}f.png" "$APP_OUT/rooms/$CHAR/$s.png"
    magick -delay $((100 / $(pet_fps_for "$s"))) -loop 0 $petstrip \
      -filter point -resize "$((ZOOM * 100))%" -layers optimize "$OUT/pet-$s.gif"
  fi

  echo "  $s: ${FRAMES}f at ${fps}fps"
done

# ---------------------------------------------------------------- the run strip
# Pet-only, so it is built outside the room loop above. The run is the side-view walk row
# stepped twice over the same 12 frames the other pet states use, so the frontend's steps(12)
# machinery plays it without a second case. No room state uses it and no contact-sheet tile is
# drawn for it: it is not one of the four rooms.
if [ -n "$APP_OUT" ]; then
  runstrip=""
  i=0
  while [ $i -lt $FRAMES ]; do
    frame_pet_run "$i" "$WORK/pet-run-f$i.png"
    runstrip="$runstrip $WORK/pet-run-f$i.png"
    i=$((i + 1))
  done
  magick $runstrip +append PNG32:"$APP_OUT/pet/$CHAR/run.png"
  magick -delay $((100 / $(pet_fps_for run))) -loop 0 $runstrip \
    -filter point -resize "$((ZOOM * 100))%" -layers optimize "$OUT/pet-run.gif"
fi

# ---------------------------------------------------------------- contact sheets
# All four side by side, in lockstep at one rate. Per-state rates live in the
# individual GIFs; this one exists to compare composition, not timing.
i=0
sheet_frames=""
while [ $i -lt $FRAMES ]; do
  tiles=""
  for s in awake dozing asleep comeback; do
    [ -f "$WORK/$s-f$i.png" ] || "frame_$s" "$i" "$WORK/$s-f$i.png"
    magick "$WORK/$s-f$i.png" -filter point -resize "$((ZOOM * 100))%" \
      -bordercolor '#101014' -border 4 \
      -background '#101014' -gravity South -splice 0x26 \
      -fill '#e8e8ee' -pointsize 18 -annotate +0+4 "$(echo "$s" | tr '[:lower:]' '[:upper:]')" \
      "$WORK/tile-$s-$i.png"
    tiles="$tiles $WORK/tile-$s-$i.png"
  done
  magick montage $tiles -tile 2x2 -geometry +6+6 -background '#101014' "$WORK/sheet-$i.png"
  sheet_frames="$sheet_frames $WORK/sheet-$i.png"
  i=$((i + 1))
done

cp "$WORK/sheet-0.png" "$OUT/states-four.png"
magick -delay $((100 / FPS_SHEET)) -loop 0 $sheet_frames -layers optimize "$OUT/states-four.gif"

echo "wrote $OUT/states-four.png, states-four.gif, four state-*.gif and four strips"
