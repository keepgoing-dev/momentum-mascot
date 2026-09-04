#!/usr/bin/env bash
#
# Cuts the curated Character Generator palette into the strips the builder composites from.
#
# One strip per layer variant, 448x32, 28 frames, in exactly the layout
# tools/compose-rooms.sh writes for the premades, so the baker treats built and shipped
# characters identically. Frames 19, 20-25 and 26 are pre-tinted because the baker only ever
# does source-over (spec section 4.5).
#
# The palette itself is docs/asset-picks.md's "Generator palette" section; the lists below are
# the executable half of it.
#
# Usage:  MASCOT_PACK=... tools/compose-layers.sh <out-dir>

set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
. "$ROOT/tools/lib/tints.sh"

PACK="${MASCOT_PACK:-$HOME/Workspace/OneQode/projects/repos/oneqode-pixel-assets/moderninteriors-win}"
G="$PACK/2_Characters/Character_Generator"
OUT="${1:-$ROOT/src/assets}"
[ -d "$G" ] || { echo "asset pack not found: $G" >&2; exit 1; }

WORK=$(mktemp -d -t compose-layers)
trap 'rm -rf "$WORK"' EXIT

# ---------------------------------------------------------------- the palette
# Skin and eyes ship whole: sixteen files, and both are identity.
# seq -w pads to the width of the largest value, which is one digit here, so -f is required.
SKIN=$(seq -f 'Body_%02g' 1 9)
EYES=$(seq -f 'Eyes_%02g' 1 7)

# Hair and outfit are style x colour. Styles are curated for distinctness at 16px; colours
# ship whole, because hair colour is identity in the same way skin tone is. Styles 27-29 are
# excluded: they render stylised cyan and do not respond to the colour axis at all.
HAIR_STYLES="01 02 03 04 05 08 09 10 11 13 15 18 22 26"
HAIR_COLOURS="01 02 03 04 05 06 07"
OUTFIT_STYLES="01 02 03 04 07 08 11 12 13 16 17 19 21"
OUTFIT_COLOURS="01 02 03 04"

# One everyday family per entry, colours whole. The novelty half of the pack is deliberately
# absent: no zombie brain, party cone, dino snapback, balaclava, police hat, ladybug or bee.
ACCESSORY_FAMILIES="03_Backpack 04_Snapback 11_Beanie 12_Mustache
13_Beard 14_Gloves 15_Glasses 16_Monocle"

# Eight is what one row of the colour picker holds. Backpack ships ten, and a 4px strap cannot
# tell the second green from the first, nor the second blue.
ACCESSORY_SKIP="Accessory_03_Backpack_04 Accessory_03_Backpack_10"

# A 16px swatch spans y6-21 or y12-27, and no single crop holds both a beanie at y8 and a pair
# of gloves at y26.
ACCESSORY_HATS="04_Snapback 11_Beanie"

# ---------------------------------------------------------------- crops
IDLE="16x32+48+0"
RUN_XS="192 208 224 240 256 272"
SEATED_XS="48 64 80 96 112 128"
SLEEP_XS="0 16 32 48 64 80"
BLANKET="16x16+192+112"

cut_at() {  # cut_at <sheet> <geom> <out>
  magick "$1" -crop "$2" +repage PNG32:"$3"
}

cut_row() {  # cut_row <sheet> <y> <name> <x>...
  local sheet=$1 y=$2 name=$3; shift 3
  local n=0
  for x in "$@"; do
    cut_at "$sheet" "16x32+$x+$y" "$WORK/$name-$n.png"
    n=$((n + 1))
  done
}

# 28 frames: idle, run x6, seated x6, sleep x6, then the three tinted room frames and the
# blanket. Categories other than skin contribute nothing to the blanket frame, which is
# correct: the body is what supplies it, exactly as the premade sheets do.
strip_for() {  # strip_for <sheet> <out> <with-blanket:0|1>
  local sheet=$1 out=$2 with_blanket=$3

  cut_at "$sheet" "$IDLE" "$WORK/idle.png"
  cut_row "$sheet" 32 run $RUN_XS
  cut_row "$sheet" 192 seated $SEATED_XS
  cut_row "$sheet" 96 sleep $SLEEP_XS

  local run=() seated=() slp=()
  for n in 0 1 2 3 4 5; do
    run+=("$WORK/run-$n.png"); seated+=("$WORK/seated-$n.png"); slp+=("$WORK/sleep-$n.png")
  done

  if [ "$with_blanket" = 1 ]; then
    magick "$sheet" -crop "$BLANKET" +repage \
      -background none -gravity NorthWest -extent 16x32 PNG32:"$WORK/blanket.png"
  else
    magick -size 16x32 xc:none PNG32:"$WORK/blanket.png"
  fi

  magick \
    "$WORK/idle.png" "${run[@]}" "${seated[@]}" "${slp[@]}" \
    \( "$WORK/idle.png" -fill "$TINT_COLOUR" -colorize "$TINT_DOZING" \) \
    \( "${slp[@]}" -fill "$TINT_COLOUR" -colorize "$TINT_ASLEEP" \) \
    \( "$WORK/idle.png" -modulate "$COMEBACK_MODULATE" \) \
    "$WORK/blanket.png" \
    +append PNG32:"$out"
}

# The head band, per spec section 2.3. Skin shows the face and so carries no hair.
swatch() {  # swatch <out> <y> <sheet>...
  local out=$1 y=$2; shift 2
  local args=(-size 16x16 xc:none)
  for s in "$@"; do args+=(\( "$s" -crop "16x16+48+$y" +repage \) -composite); done
  magick "${args[@]}" PNG32:"$out"
}

REF_BODY="$G/Bodies/16x16/Body_04.png"
REF_EYES="$G/Eyes/16x16/Eyes_01.png"
REF_HAIR_STYLE=03
REF_HAIR="$G/Hairstyles/16x16/Hairstyle_${REF_HAIR_STYLE}_03.png"
REF_OUTFIT="$G/Outfits/16x16/Outfit_01_01.png"

# An eye is two pixels, a brow over an iris, and only the iris colour changes between the
# seven. Cropped at 16px they are the same picture, so the swatch draws that colour big.
swatch_eyes() {  # swatch_eyes <out> <sheet>
  local ground brow iris
  ground=$(magick "$REF_BODY" -format '%[pixel:p{53,16}]' info:)
  brow=$(magick "$2" -format '%[pixel:p{53,20}]' info:)
  iris=$(magick "$2" -format '%[pixel:p{53,21}]' info:)
  magick -size 16x16 "xc:$ground" \
    -fill "$brow" -draw "rectangle 2,5 6,6 rectangle 9,5 13,6" \
    -fill "$iris" -draw "rectangle 2,7 6,10 rectangle 9,7 13,10" \
    PNG32:"$1"
}

hair_band() {  # hair_band <colour>: the reference style's head band, one line per colour in it
  magick "$G/Hairstyles/16x16/Hairstyle_${REF_HAIR_STYLE}_$1.png" -crop "16x16+48+6" +repage \
    -unique-colors txt:- | tail -n +2
}

# Whatever the head band holds in all seven colours is the outline and the drop shadow, so what
# is left is the hair itself.
HAIR_SHARED=$(for c in $HAIR_COLOURS; do hair_band "$c" | awk '{print $3}'; done | sort | uniq -c |
  awk -v n="$(echo "$HAIR_COLOURS" | wc -w)" '$1 == n {print $2}' | tr '\n' ' ')

hair_shades() {  # hair_shades <colour>: its hair shades, lightest first
  hair_band "$1" | awk -v skip="$HAIR_SHARED" '
    { if (index(skip, $3)) next
      split($2, p, /[(),]/)
      printf "%.1f %s\n", 0.299 * p[2] + 0.587 * p[3] + 0.114 * p[4], $3 }' |
    sort -rn | awk '{print $2}'
}

chip() {  # chip <out> <shade>...: a flat swatch, one band per shade in the order given
  local out=$1; shift
  local n=$# i=0 lo hi s
  local args=(-size 16x16 xc:none)
  for s in "$@"; do
    lo=$((i * 16 / n)); hi=$((((i + 1) * 16 / n) - 1))
    args+=(-fill "$s" -draw "rectangle 0,$lo 15,$hi")
    i=$((i + 1))
  done
  magick "${args[@]}" PNG32:"$out"
}

# One chip per colour, not per style: a hair colour is a ramp every style shares, and the
# builder's colour row is a colour picker rather than a second listing of heads.
swatch_hair_colours() {  # swatch_hair_colours <out-dir>
  local c
  mkdir -p "$1"
  for c in $HAIR_COLOURS; do chip "$1/$c.png" $(hair_shades "$c"); done
}

# The sprite outline, which every garment shares and so is never a colourway's own colour.
OUTLINE="#3A3A50FF #46465EFF"

outfit_band() {  # outfit_band <style> <colour>: opaque torso colours, count then rgba then hex
  magick "$G/Outfits/16x16/Outfit_$1_$2.png" -crop "16x16+48+16" +repage \
    -format %c histogram:info:- | tr -d ':' | awk '$3 ~ /FF$/ {print $1, $2, $3}'
}

outfit_colours() {  # outfit_colours <style>: the colourways that style actually ships
  local c
  for c in $OUTFIT_COLOURS; do
    if [ -f "$G/Outfits/16x16/Outfit_$1_$c.png" ]; then echo "$c"; fi
  done
}

shades() {  # shades <skip>: the three busiest colours of the band on stdin, lightest first
  awk -v skip="$1" '!index(skip, $3)' | sort -rn | head -3 |
    awk '{ split($2, p, /[(),]/)
           printf "%.1f %s\n", 0.299 * p[2] + 0.587 * p[3] + 0.114 * p[4], $3 }' |
    sort -rn | awk '{print $2}'
}

# Outfit colours are per-style palettes rather than one shared ramp, so a chip is derived from
# the garment: whatever all of a style's colourways hold is its shading, not its colour.
swatch_outfit_colours() {  # swatch_outfit_colours <out-dir>
  local out=$1 st c cs skip
  mkdir -p "$out"
  for st in $OUTFIT_STYLES; do
    cs=$(outfit_colours "$st")
    skip=$(for c in $cs; do outfit_band "$st" "$c" | awk '{print $3}'; done | sort | uniq -c |
      awk -v n="$(echo "$cs" | wc -w)" '$1 == n {print $2}' | tr '\n' ' ')
    for c in $cs; do
      chip "$out/Outfit_${st}_${c}.png" $(outfit_band "$st" "$c" | shades "$skip $OUTLINE")
    done
  done
}

accessory_band() {  # accessory_band <id>: its opaque colours, count then rgba then hex
  magick "$G/Accessories/16x16/$1.png" -crop "16x32+48+0" +repage \
    -format %c histogram:info:- | tr -d ':' | awk '$3 ~ /FF$/ {print $1, $2, $3}'
}

accessory_colours() {  # accessory_colours <family>: the colours that family offers
  local f id
  for f in "$G/Accessories/16x16/Accessory_$1_"*.png; do
    id=$(basename "$f" .png)
    case " $ACCESSORY_SKIP " in *" $id "*) continue ;; esac
    echo "${id##*_}"
  done
}

# Accessory colours are per-family palettes like outfits, not one shared ramp, so the chip is
# derived from the sprite: what all of a family's colours hold is its outline and shading.
swatch_accessory_colours() {  # swatch_accessory_colours <out-dir>
  local out=$1 fam c cs skip s
  mkdir -p "$out"
  for fam in $ACCESSORY_FAMILIES; do
    cs=$(accessory_colours "$fam")
    skip=$(for c in $cs; do accessory_band "Accessory_${fam}_$c" | awk '{print $3}'; done |
      sort | uniq -c | awk -v n="$(echo "$cs" | wc -w)" '$1 == n {print $2}' | tr '\n' ' ')
    for c in $cs; do
      s=$(accessory_band "Accessory_${fam}_$c" | shades "$skip $OUTLINE")
      # Glasses 06 is the untinted pair: nothing is its own, so it wears the family's shading.
      [ -n "$s" ] || s=$(accessory_band "Accessory_${fam}_$c" | shades "$OUTLINE")
      chip "$out/Accessory_${fam}_${c}.png" $s
    done
  done
}

emit() {  # emit <category> <id> <sheet> <with-blanket> <swatch-mode>
  local cat=$1 id=$2 sheet=$3 blanket=$4 mode=$5
  mkdir -p "$OUT/layers/$cat" "$OUT/swatches/$cat"
  strip_for "$sheet" "$OUT/layers/$cat/$id.png" "$blanket"
  case "$mode" in
    face)  swatch "$OUT/swatches/$cat/$id.png" 7 "$sheet" "$REF_EYES" ;;
    eyes)  swatch_eyes "$OUT/swatches/$cat/$id.png" "$sheet" ;;
    head)  swatch "$OUT/swatches/$cat/$id.png" 6 "$REF_BODY" "$REF_EYES" "$sheet" ;;
    torso) swatch "$OUT/swatches/$cat/$id.png" 16 "$REF_BODY" "$sheet" ;;
    hatted) swatch "$OUT/swatches/$cat/$id.png" 6 "$REF_BODY" "$REF_EYES" "$REF_HAIR" \
              "$REF_OUTFIT" "$sheet" ;;
    worn)  swatch "$OUT/swatches/$cat/$id.png" 12 "$REF_BODY" "$REF_EYES" "$REF_HAIR" \
              "$REF_OUTFIT" "$sheet" ;;
  esac
}

json_list() {  # json_list <name> <item>...
  local name=$1; shift
  printf '  "%s": [' "$name"
  local first=1
  for i in "$@"; do
    [ $first = 1 ] || printf ', '
    printf '"%s"' "$i"
    first=0
  done
  printf ']'
}

echo "composing layers into $OUT/layers"

skin_ids=(); for b in $SKIN; do emit skin "$b" "$G/Bodies/16x16/$b.png" 1 face; skin_ids+=("$b"); done
eyes_ids=(); for e in $EYES; do emit eyes "$e" "$G/Eyes/16x16/$e.png" 0 eyes; eyes_ids+=("$e"); done

hair_ids=()
for st in $HAIR_STYLES; do for c in $HAIR_COLOURS; do
  id="Hairstyle_${st}_${c}"; f="$G/Hairstyles/16x16/$id.png"
  [ -f "$f" ] || { echo "  missing $id" >&2; continue; }
  emit hair "$id" "$f" 0 head; hair_ids+=("$id")
done; done
swatch_hair_colours "$OUT/swatches/hair-colour"

outfit_ids=()
for st in $OUTFIT_STYLES; do for c in $OUTFIT_COLOURS; do
  id="Outfit_${st}_${c}"; f="$G/Outfits/16x16/$id.png"
  [ -f "$f" ] || { echo "  missing $id" >&2; continue; }
  emit outfit "$id" "$f" 0 torso; outfit_ids+=("$id")
done; done
swatch_outfit_colours "$OUT/swatches/outfit-colour"

acc_ids=()
for fam in $ACCESSORY_FAMILIES; do
  case " $ACCESSORY_HATS " in *" $fam "*) mode=hatted ;; *) mode=worn ;; esac
  for c in $(accessory_colours "$fam"); do
    id="Accessory_${fam}_${c}"; f="$G/Accessories/16x16/$id.png"
    [ -f "$f" ] || { echo "  missing $id" >&2; continue; }
    emit accessory "$id" "$f" 0 "$mode"; acc_ids+=("$id")
  done
done
swatch_accessory_colours "$OUT/swatches/accessory-colour"

{
  printf '{\n'
  json_list skin "${skin_ids[@]}"; printf ',\n'
  json_list eyes "${eyes_ids[@]}"; printf ',\n'
  json_list hair "${hair_ids[@]}"; printf ',\n'
  json_list hairStyles $HAIR_STYLES; printf ',\n'
  json_list hairColours $HAIR_COLOURS; printf ',\n'
  json_list outfit "${outfit_ids[@]}"; printf ',\n'
  json_list outfitStyles $OUTFIT_STYLES; printf ',\n'
  json_list outfitColours $OUTFIT_COLOURS; printf ',\n'
  json_list accessory "${acc_ids[@]}"; printf '\n}\n'
} > "$OUT/layers/index.json"

echo "  skin ${#skin_ids[@]}, eyes ${#eyes_ids[@]}, hair ${#hair_ids[@]}, outfit ${#outfit_ids[@]}, accessory ${#acc_ids[@]}"
