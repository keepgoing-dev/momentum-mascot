# Built-mascot slot ("+") Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a fourth character-picker slot, marked `+`, that opens a builder inside the popover and produces a mascot composited from LimeZu Character Generator layers, indistinguishable downstream from a shipped premade.

**Architecture:** `tools/compose-rooms.sh` gains character-less back and front plates plus a machine-readable manifest of every placement it knows. A new `tools/compose-layers.sh` cuts a curated palette into pre-tinted 432x32 layer strips. At runtime the builder previews by stacking DOM layers over the untinted back plate, and on Done a canvas baker assembles nine PNGs into the state directory. Everything downstream reads those PNGs exactly as it reads bundled ones.

**Tech Stack:** bash + ImageMagick 7 (`magick`), Rust / Tauri 2 (no new crates), vanilla ES modules (no npm, no bundler, no JS test runner).

**Spec:** `docs/superpowers/specs/2026-09-03-mascot-builder-design.md`

## Global Constraints

Copied verbatim from the spec. Every task's requirements implicitly include these.

- **No new Cargo dependencies.** The bake happens in canvas precisely so no image crate enters an App Review binary (spec 5.1).
- **`tauri.conf.json`'s `csp` and `assetProtocol` must end byte-identical to their current values.** The design goes through `blob:`, which the existing CSP already permits. If either changed, the design was abandoned somewhere (spec 10.6).
- **`store::CHARACTERS` stays `["07", "12", "20"]`** and does not gain `"custom"`. It means "the shipped premades" everywhere it is used (spec 7.1).
- **Every pixel asset is authored on the 16x16 native grid and scaled only at integer factors** (`src/style.css`). No fractional scaling anywhere.
- **`strings -a <binary> | grep -cE 'drawsBackground|fullScreenEnabled'` stays 0** (spec 10.7).
- **No em dashes** in any file this plan creates or edits.
- **Comments:** no comment may exceed two lines, and the default number of comments in a diff is zero. Match the surrounding file's density. `compose-rooms.sh` is heavily commented and new blocks there may match it; new Rust and JS should not invent commentary.
- The licensed pack is required for every asset task and is read from `$MASCOT_PACK`. Tasks 6 through 12 need no pack.

## Manifest shape

Tasks 1 through 5 and 10 all touch `src/assets/character-layout.json`. This is its final shape, built up across tasks 1, 2, 3 and 4. Later tasks may assume it whole.

```json
{
  "frames": 12,
  "room": { "w": 160, "h": 112 },
  "pet":  { "w": 32,  "h": 32 },
  "layerStrip": {
    "frame": { "w": 16, "h": 32 },
    "ranges": {
      "idle":         [0, 1],
      "run":          [1, 7],
      "seated":       [7, 13],
      "sleep":        [13, 19],
      "idleDozing":   [19, 20],
      "sleepAsleep":  [20, 26],
      "idleComeback": [26, 27]
    }
  },
  "states": {
    "awake": {
      "room": {
        "char": { "x": 111, "y": 41, "hop": [0,0,0,0,0,0,0,0,0,0,0,0], "range": "seated" },
        "overlays": []
      },
      "pet": {
        "char": { "x": 0, "y": -2, "hop": [0,0,0,0,0,0,-1,-1,-1,-1,-1,-1],
                  "range": "seated", "frame": 0 },
        "overlays": []
      }
    }
  }
}
```

`range` names a frame range in `layerStrip.ranges`. A `char` with a `frame` key repeats that one
frame for all twelve; without it the range advances one frame per output frame, wrapping.
An overlay is `{ "sprite": "coffee", "dx": 12, "dy": -4, "frames": 6 }`, with `dx`/`dy` measured
from the character's **hopped** top-left, matching `shift_pos` in the compositor.

---

### Task 1: Awake plates and the manifest

Awake first because it is the only state with no hop and no tint, and the only one whose front
plate is non-empty. It proves the whole plate mechanism on the simplest case.

**Files:**
- Modify: `tools/compose-rooms.sh` (add plate emission and manifest emission)
- Create: `tools/verify-plates.sh`

**Interfaces:**
- Produces: `src/assets/plates/awake-back.png` and `src/assets/plates/awake-front.png`, both 1920x112 PNG32; `src/assets/character-layout.json` containing `frames`, `room`, `pet`, `layerStrip` and `states.awake`.
- Produces: `tools/verify-plates.sh [state...]`, exit 0 when every requested state reassembles to `magick compare -metric AE` of 0 against the shipped room strip, for all three premades. With no argument it checks every state present in the manifest.

- [ ] **Step 1: Write the failing verification script**

Create `tools/verify-plates.sh`:

```bash
#!/usr/bin/env bash
#
# Reassembles each shipped room strip from its plates and asserts pixel identity.
# This is what stops the JS baker and the shell compositor drifting apart.

set -uo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
A="$ROOT/src/assets"
MANIFEST="$A/character-layout.json"
[ -f "$MANIFEST" ] || { echo "no manifest: $MANIFEST" >&2; exit 1; }

jq_get() { jq -r "$1" "$MANIFEST"; }

STATES=${*:-$(jq_get '.states | keys[]')}
FRAMES=$(jq_get '.frames')
CW=$(jq_get '.room.w'); CH=$(jq_get '.room.h')
WORK=$(mktemp -d -t verify-plates); trap 'rm -rf "$WORK"' EXIT

fail=0
for char in 07 12 20; do
  for state in $STATES; do
    want="$A/rooms/$char/$state.png"
    [ -f "$want" ] || { echo "MISSING $want" >&2; fail=1; continue; }
    strip=""
    for i in $(seq 0 $((FRAMES - 1))); do
      "$ROOT/tools/assemble-frame.sh" "$char" "$state" "$i" "$WORK/f$i.png" || { fail=1; break; }
      strip="$strip $WORK/f$i.png"
    done
    magick $strip +append PNG32:"$WORK/got.png"
    ae=$(magick compare -metric AE "$want" "$WORK/got.png" null: 2>&1 || true)
    if [ "$ae" = "0" ]; then
      echo "ok    $char/$state"
    else
      echo "FAIL  $char/$state  AE=$ae"
      fail=1
    fi
  done
done
exit $fail
```

- [ ] **Step 2: Run it to verify it fails**

Run: `tools/verify-plates.sh awake`
Expected: FAIL with `no manifest: .../character-layout.json`

- [ ] **Step 3: Emit the awake plates from the compositor**

In `tools/compose-rooms.sh`, beside `frame_awake`, add:

```bash
plate_awake_back() {  # plate_awake_back <i> <out>
  local i=$1 out=$2
  magick "$WORK/base.png" \
    "$(nth "$i" $CAT_FRAMES)" -geometry "$CAT_AT" -composite \
    PNG32:"$out"
}

plate_awake_front() {
  local i=$1 out=$2
  magick -size ${W}x${H} xc:none \
    "$DESK"                        -geometry "$DESK_AT"     -composite \
    "$(nth "$i" $COMPUTER_FRAMES)" -geometry "$COMPUTER_AT" -composite \
    PNG32:"$out"
}
```

- [ ] **Step 4: Write the plates into the app tree**

Where the script already writes `$APP_OUT/rooms/$CHAR/$s.png`, add a plate pass that runs once
per invocation rather than once per character, since plates carry no character:

```bash
if [ -n "${MASCOT_APP_OUT:-}" ]; then
  mkdir -p "$APP_OUT/plates"
  for half in back front; do
    strip=""
    for i in $(seq 0 $((FRAMES - 1))); do
      "plate_awake_$half" "$i" "$WORK/plate-awake-$half-$i.png"
      strip="$strip $WORK/plate-awake-$half-$i.png"
    done
    magick $strip +append PNG32:"$APP_OUT/plates/awake-$half.png"
  done
fi
```

- [ ] **Step 5: Emit the manifest**

Add a `write_manifest` function that prints the JSON from the script's own variables, so no value
is typed twice:

```bash
write_manifest() {  # write_manifest <out>
  cat > "$1" <<JSON
{
  "frames": $FRAMES,
  "room": { "w": $W, "h": $H },
  "pet":  { "w": $PET_W, "h": $PET_H },
  "layerStrip": {
    "frame": { "w": 16, "h": 32 },
    "ranges": {
      "idle": [0, 1], "run": [1, 7], "seated": [7, 13], "sleep": [13, 19],
      "idleDozing": [19, 20], "sleepAsleep": [20, 26], "idleComeback": [26, 27]
    }
  },
  "states": {
    "awake": {
      "room": {
        "char": { "x": ${CHAR_AWAKE_AT#+}, "hop": [$(echo 0 0 0 0 0 0 0 0 0 0 0 0 | tr ' ' ',')],
                  "range": "seated" },
        "overlays": []
      },
      "pet": {
        "char": { "x": $PET_CHAR_X, "y": $PET_CHAR_Y,
                  "hop": [$(echo $PET_BREATH | tr ' ' ',')], "range": "seated", "frame": 0 },
        "overlays": []
      }
    }
  }
}
JSON
}
```

Note `CHAR_AWAKE_AT` is the string `+111+41`. Split it into `x` and `y` with the same parsing
`shift_pos` uses rather than hard-coding 111 and 41:

```bash
at_x() { local a=${1%+*}; printf '%d' "${a#+}"; }
at_y() { printf '%d' "${1##*+}"; }
```

Call `write_manifest "$APP_OUT/character-layout.json"` inside the same `MASCOT_APP_OUT` guard.

- [ ] **Step 6: Write the frame assembler the verifier calls**

Create `tools/assemble-frame.sh`, the shell oracle. Tasks 2, 3, 4 and 10 all reuse it:

```bash
#!/usr/bin/env bash
#
# The shell oracle: one room frame, assembled from plates the way the JS baker assembles it.

set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
A="$ROOT/src/assets"
M="$A/character-layout.json"
char=$1 state=$2 i=$3 out=$4

q() { jq -r "$1" "$M"; }
cx=$(q ".states.\"$state\".room.char.x")
cy=$(q ".states.\"$state\".room.char.y")
hop=$(q ".states.\"$state\".room.char.hop[$i]")
range=$(q ".states.\"$state\".room.char.range")
lo=$(q ".layerStrip.ranges.$range[0]")
hi=$(q ".layerStrip.ranges.$range[1]")
single=$(q ".states.\"$state\".room.char.frame // \"\"")

n=$((hi - lo))
if [ -n "$single" ]; then k=$((lo + single)); else k=$((lo + i % n)); fi

magick "$A/plates/$state-back.png" -crop "160x112+$((i * 160))+0" +repage PNG32:"/tmp/.af-back.png"
magick "$A/layers/premade/$char.png" -crop "16x32+$((k * 16))+0" +repage PNG32:"/tmp/.af-char.png"

args=("/tmp/.af-back.png" "/tmp/.af-char.png" -geometry "+$cx+$((cy + hop))" -composite)
if [ -f "$A/plates/$state-front.png" ]; then
  magick "$A/plates/$state-front.png" -crop "160x112+$((i * 160))+0" +repage PNG32:"/tmp/.af-front.png"
  args+=("/tmp/.af-front.png" -geometry "+0+0" -composite)
fi
magick "${args[@]}" PNG32:"$out"
```

- [ ] **Step 7: Emit premade layer strips so the oracle has character art**

The oracle needs each premade cut into the same 432x32 strip layout as a built mascot, otherwise
it is testing a different thing from what ships. In `compose-rooms.sh`, under the same
`MASCOT_APP_OUT` guard:

```bash
mkdir -p "$APP_OUT/layers/premade"
magick \
  "$(nth 0 $IDLE_FRAMES)" $RUN_FRAMES $SEATED_FRAMES $SLEEP_FRAMES \
  \( "$(nth 0 $IDLE_FRAMES)" -fill "$TINT_COLOUR" -colorize "$TINT_DOZING" \) \
  \( $SLEEP_FRAMES -fill "$TINT_COLOUR" -colorize "$TINT_ASLEEP" \) \
  \( "$(nth 0 $IDLE_FRAMES)" -modulate "$COMEBACK_MODULATE" \) \
  +append PNG32:"$APP_OUT/layers/premade/$CHAR.png"
```

- [ ] **Step 8: Rebuild assets and run the verifier**

Run: `tools/build-app-assets.sh && tools/verify-plates.sh awake`
Expected: three `ok` lines, `07/awake`, `12/awake`, `20/awake`, exit 0.

If AE is non-zero, the likely cause is the front plate being composited onto an opaque canvas
rather than `xc:none`, which would paint the desk's transparent margin over the character.

- [ ] **Step 9: Commit**

```bash
git add tools/compose-rooms.sh tools/verify-plates.sh tools/assemble-frame.sh
git commit -m "Split the awake room into plates the character sits between"
```

---

### Task 2: Dozing and comeback, the hop and the tracking overlays

**Files:**
- Modify: `tools/compose-rooms.sh`
- Modify: `tools/assemble-frame.sh`

**Interfaces:**
- Consumes: the manifest and `assemble-frame.sh` from Task 1.
- Produces: `plates/dozing-back.png`, `plates/comeback-back.png` (no front plates for either), `src/assets/shared/{coffee,dots,bang,spark}.png` as `+append` strips, and `states.dozing` / `states.comeback` in the manifest carrying real hop arrays and overlay entries.

- [ ] **Step 1: Extend the verifier's reach**

No code change. Run: `tools/verify-plates.sh dozing comeback`
Expected: FAIL, `jq` returns `null` for `.states."dozing".room.char.x`.

- [ ] **Step 2: Emit the back plates**

These two states put desk and computer *behind* the character, so both live in the back plate:

```bash
plate_hopper_back() {  # plate_hopper_back <i> <out> <colorize|modulate spec>
  local i=$1 out=$2
  magick "$WORK/base.png" \
    "$(nth "$i" $CAT_FRAMES)"      -geometry "$CAT_AT"      -composite \
    "$DESK"                        -geometry "$DESK_AT"     -composite \
    "$(nth "$i" $COMPUTER_FRAMES)" -geometry "$COMPUTER_AT" -composite \
    PNG32:"$out"
}
```

Then tint per state after assembly, matching `frame_dozing` and `frame_comeback`:

```bash
plate_dozing_back() {
  plate_hopper_back "$1" "$2"
  magick "$2" -fill "$TINT_COLOUR" -colorize "$TINT_DOZING" PNG32:"$2"
}

plate_comeback_back() {
  plate_hopper_back "$1" "$2"
  magick "$2" -modulate "$COMEBACK_MODULATE" PNG32:"$2"
}
```

- [ ] **Step 3: Emit the tracking overlays as shared strips**

Coffee and the emotes are character-independent art placed at character-relative offsets, so they
ship once rather than per layer. Under the `MASCOT_APP_OUT` guard:

```bash
mkdir -p "$APP_OUT/shared"
magick $COFFEE_FRAMES +append PNG32:"$APP_OUT/shared/coffee.png"
magick $DOTS_FRAMES   +append PNG32:"$APP_OUT/shared/dots.png"
magick $BANG_FRAMES   +append PNG32:"$APP_OUT/shared/bang.png"
magick $SPARK_FRAMES  +append PNG32:"$APP_OUT/shared/spark.png"
magick $Z_FRAMES      +append PNG32:"$APP_OUT/shared/z.png"
magick "$BLANKET"     +append PNG32:"$APP_OUT/shared/blanket.png"
```

These are emitted untinted. The baker tints nothing, so each state's overlay art must be
pre-tinted per state exactly as the character strip is. Emit a tinted copy per consuming state:

```bash
magick "$APP_OUT/shared/coffee.png" -fill "$TINT_COLOUR" -colorize "$TINT_DOZING" \
  PNG32:"$APP_OUT/shared/coffee-dozing.png"
magick "$APP_OUT/shared/dots.png"   -fill "$TINT_COLOUR" -colorize "$TINT_DOZING" \
  PNG32:"$APP_OUT/shared/dots-dozing.png"
magick "$APP_OUT/shared/bang.png"   -modulate "$COMEBACK_MODULATE" \
  PNG32:"$APP_OUT/shared/bang-comeback.png"
magick "$APP_OUT/shared/spark.png"  -modulate "$COMEBACK_MODULATE" \
  PNG32:"$APP_OUT/shared/spark-comeback.png"
magick "$APP_OUT/shared/z.png"      -fill "$TINT_COLOUR" -colorize "$TINT_ASLEEP" \
  PNG32:"$APP_OUT/shared/z-asleep.png"
```

- [ ] **Step 4: Add both states to the manifest**

Inside `write_manifest`, add. Note `dx`/`dy` are `EMOTE_DX`/`EMOTE_DY` for the emotes and the
literal 12 and -4 that `frame_dozing` uses for the coffee, read from the script's variables:

```
"dozing": {
  "room": {
    "char": { "x": $(at_x "$CHAR_DOZING_AT"), "y": $(at_y "$CHAR_DOZING_AT"),
              "hop": [$(echo $HOP_DOZING | tr ' ' ',')],
              "range": "idleDozing", "frame": 0 },
    "overlays": [
      { "sprite": "coffee-dozing", "dx": 12, "dy": -4, "frames": 6 },
      { "sprite": "dots-dozing", "dx": $EMOTE_DX, "dy": $EMOTE_DY, "frames": 2 }
    ]
  },
  ...
},
"comeback": {
  "room": {
    "char": { "x": $(at_x "$CHAR_COMEBACK_AT"), "y": $(at_y "$CHAR_COMEBACK_AT"),
              "hop": [$(echo $HOP_COMEBACK | tr ' ' ',')],
              "range": "idleComeback", "frame": 0 },
    "overlays": [
      { "sprite": "bang-comeback",  "dx": $EMOTE_DX, "dy": $EMOTE_DY, "frames": 2 },
      { "sprite": "spark-comeback", "dx": -9, "dy": -4, "frames": 2 },
      { "sprite": "spark-comeback", "dx": 21, "dy": -4, "frames": 2 }
    ]
  },
  ...
}
```

- [ ] **Step 5: Teach the oracle to place overlays**

In `tools/assemble-frame.sh`, after the character composite and before the front plate:

```bash
count=$(q ".states.\"$state\".room.overlays | length")
for o in $(seq 0 $((count - 1))); do
  sp=$(q ".states.\"$state\".room.overlays[$o].sprite")
  dx=$(q ".states.\"$state\".room.overlays[$o].dx")
  dy=$(q ".states.\"$state\".room.overlays[$o].dy")
  nf=$(q ".states.\"$state\".room.overlays[$o].frames")
  magick "$A/shared/$sp.png" -crop "16x16+$(( (i % nf) * 16 ))+0" +repage PNG32:"/tmp/.af-o$o.png"
  args+=("/tmp/.af-o$o.png" -geometry "+$((cx + dx))+$((cy + hop + dy))" -composite)
done
```

The overlay offset is measured from the **hopped** position, which is why `hop` is inside the
`dy` expression and not outside it. Getting this wrong produces a character that bobs while its
coffee stays still.

- [ ] **Step 6: Rebuild and verify**

Run: `tools/build-app-assets.sh && tools/verify-plates.sh awake dozing comeback`
Expected: nine `ok` lines, exit 0.

- [ ] **Step 7: Commit**

```bash
git add tools/compose-rooms.sh tools/assemble-frame.sh
git commit -m "Plate the two states that hop, with their overlays tracking the character"
```

---

### Task 3: Asleep, and the full four-state verification

**Files:**
- Modify: `tools/compose-rooms.sh`

**Interfaces:**
- Produces: `plates/asleep-back.png`, `plates/asleep-front.png`, `states.asleep` in the manifest.
- Produces: `tools/verify-plates.sh` passing with no arguments, all four states, all three premades.

- [ ] **Step 1: Run the verifier over everything**

Run: `tools/verify-plates.sh`
Expected: FAIL on the three `*/asleep` rows only.

- [ ] **Step 2: Emit the asleep plates**

Asleep has a static character, so the `z` emote can be pre-baked into a front plate:

```bash
plate_asleep_back() {
  local i=$1 out=$2
  plate_hopper_back "$i" "$out"
  magick "$out" -fill "$TINT_COLOUR" -colorize "$TINT_ASLEEP" PNG32:"$out"
}

plate_asleep_front() {
  local i=$1 out=$2
  magick -size ${W}x${H} xc:none \
    "$(nth "$i" $Z_FRAMES)" \
      -geometry "$(shift_pos "$SLEEP_OVERLAY_AT" "$SLEEP_EMOTE_DX" "$SLEEP_EMOTE_DY")" -composite \
    -fill "$TINT_COLOUR" -colorize "$TINT_ASLEEP" PNG32:"$out"
}
```

- [ ] **Step 3: Add asleep to the manifest**

```
"asleep": {
  "room": {
    "char": { "x": $(at_x "$SLEEP_OVERLAY_AT"), "y": $(at_y "$SLEEP_OVERLAY_AT"),
              "hop": [0,0,0,0,0,0,0,0,0,0,0,0], "range": "sleepAsleep" },
    "overlays": []
  },
  ...
}
```

- [ ] **Step 4: Rebuild and verify all four states**

Run: `tools/build-app-assets.sh && tools/verify-plates.sh`
Expected: twelve `ok` lines, exit 0.

This is spec test 2 passing in full. It proves the plate split, the hop arrays, the overlay
deltas and the pre-tinting are all faithful to the compositor in one assertion.

- [ ] **Step 5: Commit**

```bash
git add tools/compose-rooms.sh
git commit -m "Plate the asleep room and close the four-state reassembly"
```

---

### Task 4: The pet half of the manifest

The pet has its own geometry (32x32 cell, `PET_BREATH` instead of the room hop, a blanket) and
applies no tint at all. It needs verifying separately or the baker will get it wrong silently.

**Files:**
- Modify: `tools/compose-rooms.sh`
- Create: `tools/assemble-pet-frame.sh`
- Modify: `tools/verify-plates.sh`

**Interfaces:**
- Produces: `states.<state>.pet` for all four states plus `run`, and `tools/assemble-pet-frame.sh <char> <state> <i> <out>`.
- Produces: `verify-plates.sh` also comparing against `src/assets/pet/<char>/<state>.png`.

- [ ] **Step 1: Add the failing pet comparison to the verifier**

In `tools/verify-plates.sh`, after the room loop, add a pet loop calling
`tools/assemble-pet-frame.sh` and comparing against `$A/pet/$char/$state.png`, over
`$STATES` plus `run`.

Run: `tools/verify-plates.sh`
Expected: room rows `ok`, every pet row FAIL (script missing).

- [ ] **Step 2: Write the pet oracle**

Create `tools/assemble-pet-frame.sh`. It differs from the room oracle in three ways: the canvas is
`xc:none` at 32x32 rather than a plate, offsets may be negative so it uses `%+d%+d` formatting,
and asleep composites the blanket after the character.

```bash
#!/usr/bin/env bash
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
A="$ROOT/src/assets"; M="$A/character-layout.json"
char=$1 state=$2 i=$3 out=$4
q() { jq -r "$1" "$M"; }
pw=$(q '.pet.w'); ph=$(q '.pet.h')
cx=$(q ".states.\"$state\".pet.char.x"); cy=$(q ".states.\"$state\".pet.char.y")
hop=$(q ".states.\"$state\".pet.char.hop[$i]")
range=$(q ".states.\"$state\".pet.char.range")
lo=$(q ".layerStrip.ranges.$range[0]"); hi=$(q ".layerStrip.ranges.$range[1]")
single=$(q ".states.\"$state\".pet.char.frame // \"\"")
n=$((hi - lo))
if [ -n "$single" ]; then k=$((lo + single)); else k=$((lo + i % n)); fi
magick "$A/layers/premade/$char.png" -crop "16x32+$((k * 16))+0" +repage PNG32:"/tmp/.pf-char.png"
args=(-size ${pw}x${ph} xc:none "/tmp/.pf-char.png" -geometry "$(printf '%+d%+d' "$cx" "$((cy + hop))")" -composite)
blanket=$(q ".states.\"$state\".pet.blanket // \"\"")
if [ -n "$blanket" ]; then
  bdy=$(q ".states.\"$state\".pet.blanketDy")
  args+=("$A/shared/blanket.png" -geometry "$(printf '%+d%+d' "$cx" "$((cy + hop + bdy))")" -composite)
fi
count=$(q ".states.\"$state\".pet.overlays | length")
for o in $(seq 0 $((count - 1))); do
  sp=$(q ".states.\"$state\".pet.overlays[$o].sprite")
  dx=$(q ".states.\"$state\".pet.overlays[$o].dx"); dy=$(q ".states.\"$state\".pet.overlays[$o].dy")
  nf=$(q ".states.\"$state\".pet.overlays[$o].frames")
  magick "$A/shared/$sp.png" -crop "16x16+$(( (i % nf) * 16 ))+0" +repage PNG32:"/tmp/.pf-o$o.png"
  args+=("/tmp/.pf-o$o.png" -geometry "$(printf '%+d%+d' "$((cx + dx))" "$((cy + hop + dy))")" -composite)
done
magick "${args[@]}" PNG32:"$out"
```

- [ ] **Step 3: Fill in the pet manifest entries**

Read every value from the script's `PET_*` constants. The pet overlays use the **untinted**
sprite names, because `frame_pet_*` applies no colour operation:

```
"awake":    pet char x=$PET_CHAR_X y=$PET_CHAR_Y hop=$PET_BREATH range=seated frame=0, overlays []
"dozing":   pet char ... hop=$PET_BREATH  range=idle frame=0,
            overlays [{ dots, dx=$PET_EMOTE_DX, dy=$PET_EMOTE_DY, frames 2 }]
"asleep":   pet char x=$PET_CHAR_X y=$PET_SLEEP_Y hop=[0 x12] range=sleep,
            blanket=true, blanketDy=$PET_BLANKET_DY,
            overlays [{ z, dx=$PET_EMOTE_DX, dy=$PET_EMOTE_DY, frames 2 }]
"comeback": pet char ... hop=$HOP_COMEBACK range=idle frame=0,
            overlays [{ bang, dx=$PET_EMOTE_DX, dy=$PET_EMOTE_DY, frames 2 },
                      { spark, dx=$PET_SPARK_DX, dy=$PET_SPARK_DY, frames 2 }]
"run":      pet char x=$PET_RUN_X y=$PET_CHAR_Y hop=[0 x12] range=run, overlays []
```

`run` has a `pet` key and no `room` key. Every consumer must treat a missing `room` as "this
state does not appear in the popover".

- [ ] **Step 4: Rebuild and verify**

Run: `tools/build-app-assets.sh && tools/verify-plates.sh`
Expected: twelve room `ok` rows and fifteen pet `ok` rows, exit 0.

- [ ] **Step 5: Commit**

```bash
git add tools/compose-rooms.sh tools/assemble-pet-frame.sh tools/verify-plates.sh
git commit -m "Cover the pet in the manifest and the reassembly check"
```

---

### Task 5: The curated palette and its layer strips

**Files:**
- Create: `tools/compose-layers.sh`
- Modify: `tools/build-app-assets.sh`
- Modify: `docs/asset-picks.md`
- Create: `tools/verify-layers.sh`

**Interfaces:**
- Produces: `src/assets/layers/<category>/<id>.png`, each 432x32 in the frame layout given in the plan header, for categories `skin`, `eyes`, `outfit`, `hair`, `accessory`.
- Produces: `src/assets/swatches/<category>/<id>.png`, each 16x16.
- Produces: `src/assets/layers/index.json`, a per-category ordered list of ids, which the builder UI reads so the grid is not hard-coded.

- [ ] **Step 1: Write the failing layer verification**

Create `tools/verify-layers.sh` asserting, for every strip in `src/assets/layers/`:

```bash
#!/usr/bin/env bash
set -uo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd); A="$ROOT/src/assets"
fail=0
for f in "$A"/layers/*/*.png; do
  size=$(magick identify -format '%wx%h' "$f")
  [ "$size" = "432x32" ] || { echo "FAIL $f is $size not 432x32"; fail=1; }
done
# Spec test 1: outfits and eyes draw nothing in the sleep row.
for f in "$A"/layers/outfit/*.png "$A"/layers/eyes/*.png; do
  a=$(magick "$f" -crop 96x32+208+0 +repage -format '%[fx:mean.a]' info:)
  [ "$a" = "0" ] || { echo "FAIL $f draws in the sleep row (mean alpha $a)"; fail=1; }
done
exit $fail
```

The sleep-row crop is `+208`, being frames 13 through 18 at 16px each.

Run: `tools/verify-layers.sh`
Expected: exit 0 vacuously, no files yet. Add `[ -d "$A/layers/skin" ] || { echo "no layers"; exit 1; }` as the first check so it fails honestly.

- [ ] **Step 2: Choose the palette and record it**

Add a `## Generator palette` section to `docs/asset-picks.md` listing the exact chosen filenames
per category. Follow the existing table style in that file.

This step needs eyes on the art and cannot be done from filenames. Generate a contact sheet of
head swatches per category first (`swatch` from step 4), look at it, and pick.

Acceptance, from spec section 3:

| Category | Count | Rule |
|---|---|---|
| skin | exactly 9 | every `Body_0*.png`, no choice to make |
| eyes | exactly 7 | every `Eyes_0*.png` |
| hair | 18 to 22 | distinguishable from each other at 16px, spread across lengths and colours |
| outfit | 14 to 18 | everyday clothing |
| accessory | 9 to 11 | includes a `none` entry; `Glasses` is mandatory |

Excluded by name, non-negotiable: `Zombie_Brain`, `Party_Cone`, `Dino_Snapback`, `Bataclava`,
`Policeman_Hat`, `Ladybug`, `Bee`. These are the novelty half that spec section 3 rejects on tone.

Two hairstyles that differ only in a shade nobody can tell apart at 16 pixels are one hairstyle
and a wasted grid cell. Prefer fewer, more distinct picks over hitting the top of the range.

- [ ] **Step 3: Write the layer compositor**

Create `tools/compose-layers.sh` reading the palette lists and emitting, per variant, the 27-frame
strip in the documented order. The tints come from the same `TINT_*` and `COMEBACK_MODULATE`
values `compose-rooms.sh` uses, so source them rather than copying:

```bash
strip_for() {  # strip_for <sheet> <out>
  local sheet=$1 out=$2
  local idle; idle=$(cut1 "$sheet" 16x32 48 0)
  local run;  run=$(cutn "$sheet" 16x32 32 192 208 224 240 256 272)
  local seat; seat=$(cutn "$sheet" 16x32 192 48 64 80 96 112 128)
  local slp;  slp=$(cutn "$sheet" 16x32 96 0 16 32 48 64 80)
  magick "$idle" $run $seat $slp \
    \( "$idle" -fill "$TINT_COLOUR" -colorize "$TINT_DOZING" \) \
    \( $slp    -fill "$TINT_COLOUR" -colorize "$TINT_ASLEEP" \) \
    \( "$idle" -modulate "$COMEBACK_MODULATE" \) \
    +append PNG32:"$out"
}
```

- [ ] **Step 4: Emit the swatches**

Head crops per spec section 2.3. Skin uses `+48+7` with body and eyes only; every other category
uses `+48+6` composited over a fixed reference body and eyes so the item is visible in context:

```bash
swatch() {  # swatch <out> <sheet>...
  local out=$1; shift
  local args=(-size 16x16 xc:none)
  for s in "$@"; do args+=(\( "$s" -crop 16x16+48+6 +repage \) -composite); done
  magick "${args[@]}" PNG32:"$out"
}
```

- [ ] **Step 5: Emit the blanket as frame 27, per body**

**Answered during Task 4: the blanket is not generic.** Its crop differs on all three premade
sheets and on every body sampled, which made `pet/12/asleep` and `pet/20/asleep` fail reassembly
at a max delta of 27 while `pet/07` passed. It is frame 27 of the layer strip, taken from the
body layer, padded to the 16x32 cell with `-background none -gravity NorthWest -extent 16x32`.

The check below is kept as a regression guard, inverted: it now asserts the blankets **differ**,
so that a future pack update quietly making them uniform does not go unnoticed.

```bash
ref=""; for b in $SKIN_SHEETS; do
  magick "$b" -crop 16x16+192+112 +repage PNG32:/tmp/.bl.png
  h=$(magick /tmp/.bl.png -format %# info:)
  [ -z "$ref" ] && ref=$h
  [ "$h" = "$ref" ] || { echo "FAIL blanket differs in $b"; exit 1; }
done
```

If it fails, the blanket becomes per-skin art and the manifest gains a per-layer blanket
reference. Record which happened.

- [ ] **Step 6: Wire into the asset build and verify**

Add `"$ROOT/tools/compose-layers.sh"` to `tools/build-app-assets.sh` after the room loop.

Run: `tools/build-app-assets.sh && tools/verify-layers.sh`
Expected: exit 0, no FAIL lines.

- [ ] **Step 7: Commit**

```bash
git add tools/compose-layers.sh tools/verify-layers.sh tools/build-app-assets.sh docs/asset-picks.md
git commit -m "Cut the curated generator palette into pre-tinted layer strips"
```

---

### Task 6: The state schema

First Rust task. Nothing here depends on the asset tasks.

**Files:**
- Modify: `src-tauri/src/store.rs`

**Interfaces:**
- Produces: `pub struct CustomCharacter { body, eyes, outfit, hair, accessory }` where the first four are `String` and `accessory` is `Option<String>`.
- Produces: `StateFile.custom_character: Option<CustomCharacter>`, `SCHEMA_VERSION = "3.2"`.

- [ ] **Step 1: Write the failing tests**

In `src-tauri/src/store.rs`'s test module:

```rust
#[test]
fn custom_character_round_trips() {
    let dir = tempdir();
    let path = dir.join("state.json");
    let mut state = StateFile::empty();
    state.custom_character = Some(CustomCharacter {
        body: "Body_03".into(),
        eyes: "Eyes_02".into(),
        outfit: "Outfit_11_04".into(),
        hair: "Hairstyle_11_03".into(),
        accessory: Some("Accessory_15_Glasses_05".into()),
    });
    save(&path, &state).unwrap();
    assert_eq!(load(&path).custom_character, state.custom_character);
}

#[test]
fn custom_character_accessory_is_optional() {
    let dir = tempdir();
    let path = dir.join("state.json");
    let mut state = StateFile::empty();
    state.custom_character = Some(CustomCharacter {
        body: "Body_01".into(), eyes: "Eyes_01".into(), outfit: "Outfit_01_01".into(),
        hair: "Hairstyle_05_02".into(), accessory: None,
    });
    save(&path, &state).unwrap();
    assert_eq!(load(&path).custom_character.unwrap().accessory, None);
}

#[test]
fn a_partial_custom_character_is_dropped_not_defaulted() {
    let path = write_json(r#"{"version":"3.2","custom_character":{"body":"Body_01"}}"#);
    assert_eq!(load(&path).custom_character, None);
}
```

The third test is the one that matters. A half-written build must yield `None` so the picker shows
`+` again, not a mascot missing its face.

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml custom_character`
Expected: FAIL, `cannot find struct CustomCharacter`.

- [ ] **Step 3: Implement**

Add the struct with `#[derive(Debug, Clone, PartialEq, Eq)]`, parse it in `load` beside the
existing `character_id` branch, and write it in `save`. Parse defensively: all four required
fields present or the whole object is `None`.

Bump `SCHEMA_VERSION` to `"3.2"`.

- [ ] **Step 4: Run to verify they pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: all tests pass, count up from 98.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/store.rs
git commit -m "Carry a built mascot's five layer ids in the state file"
```

---

### Task 7: Downgrade and selection

**Files:**
- Modify: `src-tauri/src/store.rs`
- Modify: `src-tauri/src/momentum.rs`
- Modify: `src-tauri/src/commands.rs`

**Interfaces:**
- Consumes: `CustomCharacter` from Task 6.
- Produces: `pub const CUSTOM_ID: &str = "custom";` in `store`.
- Produces: `Momentum::cycle_character` returning a four-long cycle when `custom_character` is present.
- Produces: `commands::set_character` accepting `"custom"` when `custom_character` is present.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn an_unknown_character_id_still_falls_back() {
    let path = write_json(r#"{"version":"3.2","character_id":"custom"}"#);
    assert_eq!(load(&path).character_id, CHARACTERS[0]);
}

#[test]
fn custom_is_kept_when_the_build_is_present() {
    let path = write_json(
        r#"{"version":"3.2","character_id":"custom","custom_character":
            {"body":"Body_01","eyes":"Eyes_01","outfit":"Outfit_01_01","hair":"Hairstyle_05_02"}}"#,
    );
    assert_eq!(load(&path).character_id, "custom");
}

#[test]
fn the_cycle_is_four_long_with_a_built_mascot() {
    let mut m = momentum_with_custom();
    assert_eq!(m.cycle_character(), "12");
    assert_eq!(m.cycle_character(), "20");
    assert_eq!(m.cycle_character(), "custom");
    assert_eq!(m.cycle_character(), "07");
}
```

The existing three-step assertions at `momentum.rs:712-714` stay exactly as they are. They are
still correct for a user who has never opened the builder, which is the more common path.

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml character`
Expected: FAIL on `custom_is_kept_when_the_build_is_present` and the four-long cycle.

- [ ] **Step 3: Implement**

In `store::load`, the `character_id` filter becomes: accept an id in `CHARACTERS`, or `CUSTOM_ID`
when `custom_character` parsed to `Some`. Note the ordering constraint, since `custom_character`
must be parsed before `character_id` is validated.

In `momentum::cycle_character`, build the sequence rather than indexing `CHARACTERS`:

```rust
let mut order: Vec<&str> = store::CHARACTERS.to_vec();
if self.state.custom_character.is_some() {
    order.push(store::CUSTOM_ID);
}
```

In `commands::set_character`, widen the guard the same way.

- [ ] **Step 4: Run to verify they pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/store.rs src-tauri/src/momentum.rs src-tauri/src/commands.rs
git commit -m "Make custom a selectable id without widening CHARACTERS"
```

---

### Task 8: Where the custom art lives

**Files:**
- Modify: `src-tauri/src/sprite.rs`
- Create: `src-tauri/src/custom.rs`
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Produces: `custom::dir(app) -> PathBuf`, the `custom/` folder beside `state.json`.
- Produces: `custom::relative_art_path(name: &str) -> Option<PathBuf>`, `None` for any name outside the allowlist. Tasks 9 and 12 call this exact name.
- Produces: `custom::has_art(dir: &Path) -> bool`, true only when all nine strips exist.
- Produces: `sprite::resolve_path` returning the custom pet strip for `CUSTOM_ID`.

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn art_names_are_allowlisted_not_sanitised() {
    for bad in ["../state.json", "rooms/../../x", "rooms/awake/../y", "", "rooms/AWAKE"] {
        assert_eq!(relative_art_path(bad), None, "{bad} should be rejected");
    }
    for good in ["rooms/awake", "rooms/comeback", "pet/run", "pet/asleep"] {
        assert!(relative_art_path(good).is_some(), "{good} should be accepted");
    }
}

#[test]
fn custom_pet_sprites_resolve_outside_the_bundle() {
    assert_eq!(relative_path("custom", "run"), PathBuf::from("custom/pet/run.png"));
}

#[test]
fn a_half_written_cache_is_not_usable_art() {
    let dir = tempdir();
    std::fs::create_dir_all(dir.join("custom/rooms")).unwrap();
    std::fs::write(dir.join("custom/rooms/awake.png"), b"x").unwrap();
    assert!(!has_art(&dir));
}
```

Allowlisting rather than sanitising is the point: the set of legal names is nine strings, so the
check is membership, not escaping.

`has_art` is spec section 5.4's render-time check. It is deliberately not the load-time filter in
`store.rs`, which validates the id and cannot see whether the art behind a valid id is on disk.

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test --manifest-path src-tauri/Cargo.toml art_names`
Expected: FAIL, `cannot find function relative_art_path`.

- [ ] **Step 3: Implement**

```rust
pub const ROOM_MOODS: [&str; 4] = ["awake", "dozing", "asleep", "comeback"];
pub const PET_MOODS: [&str; 5] = ["awake", "dozing", "asleep", "comeback", "run"];

pub fn relative_art_path(name: &str) -> Option<PathBuf> {
    let (kind, mood) = name.split_once('/')?;
    let ok = match kind {
        "rooms" => ROOM_MOODS.contains(&mood),
        "pet" => PET_MOODS.contains(&mood),
        _ => false,
    };
    ok.then(|| PathBuf::from("custom").join(kind).join(format!("{mood}.png")))
}
```

Add the `CUSTOM_ID` branch to `sprite::resolve_path` so it joins the state directory rather than
`resource_dir()`.

Add `has_art`, checking all four room strips and all five pet strips exist and are non-empty.
`app::publish` reports it to the frontend as `custom_art_ready`, so `render` in Task 12 can fall
back to `CHARACTERS[0]` rather than painting a room with nobody in it.

- [ ] **Step 4: Run to verify they pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/custom.rs src-tauri/src/sprite.rs src-tauri/src/main.rs
git commit -m "Resolve a built mascot's art beside the state file"
```

---

### Task 9: The bytes bridge

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Produces: `write_custom_art(app, name: String, png: Vec<u8>) -> Result<(), String>`
- Produces: `read_custom_art(app, name: String) -> Result<Vec<u8>, String>`
- Produces: `save_custom_character(app, build: CustomCharacter) -> Result<(), String>`, which persists the build, selects `"custom"`, and republishes.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn writing_art_under_a_rejected_name_is_an_error_not_a_write() {
    let dir = tempdir();
    assert!(write_art_to(&dir, "../escape", &[1, 2, 3]).is_err());
    assert!(!dir.join("escape").exists());
    assert!(!dir.parent().unwrap().join("escape").exists());
}
```

Test the path-resolving half rather than the `#[tauri::command]` wrapper, which needs an
`AppHandle`. The command body should be two lines over a testable free function.

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test --manifest-path src-tauri/Cargo.toml writing_art`
Expected: FAIL, function not found.

- [ ] **Step 3: Implement**

`write_art_to(dir, name, bytes)` resolves through `custom::relative_art_path`, creates parent
directories, and writes. The commands are thin wrappers resolving the state directory and
delegating. Register all three in `main.rs`'s `invoke_handler`.

`save_custom_character` sets `character_id` to `CUSTOM_ID` and calls `app::publish`, matching how
`set_character` already ends.

- [ ] **Step 4: Run to verify it passes**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "Move built-mascot art across the bridge as bytes"
```

---

### Task 10: The baker

**Files:**
- Create: `src/baker.js`
- Modify: `src/popover.js`
- Create: `tools/verify-bake.sh`

**Interfaces:**
- Consumes: the manifest, the layer strips, `write_custom_art`.
- Produces: `bake(manifest, build, assets) -> Map<string, Blob>`, keyed by the nine art names.
- Produces: `MASCOT_BAKE_PROBE` debug env, matching the `MASCOT_PROBE_FRAMES` precedent in `sprite.rs`: in a debug build only, the popover bakes one fixed build on load and writes it, so a shell script can compare it against the oracle.

- [ ] **Step 1: Write the failing comparison script**

Create `tools/verify-bake.sh`. It bakes a fixed build through the app, assembles the same build
through `assemble-frame.sh`, and asserts AE of 0 across all nine outputs. The fixed build is
declared in the script so both sides read the same one.

Run: `tools/verify-bake.sh`
Expected: FAIL, no probe output.

- [ ] **Step 2: Write the placement half of the baker**

```javascript
export function framePlacement(manifest, state, surface, i) {
  const s = manifest.states[state][surface];
  const hop = s.char.hop[i % manifest.frames];
  return { x: s.char.x, y: s.char.y + hop };
}

export function frameIndex(manifest, state, surface, i) {
  const c = manifest.states[state][surface].char;
  const [lo, hi] = manifest.layerStrip.ranges[c.range];
  return c.frame === undefined ? lo + (i % (hi - lo)) : lo + c.frame;
}
```

These two mirror `assemble-frame.sh` exactly. Every other placement in the baker derives from
`framePlacement`, so an overlay is `x + dx`, `y + dy` with `y` already hopped.

- [ ] **Step 3: Write the compositing half**

`bake` draws, per state and per frame, onto a `<canvas>` at native resolution: back plate frame,
character layers in the pack's order (body, eyes, outfit, hair, accessory, skipping a null
accessory), overlays, then the front plate if one exists. It uses `drawImage` only, with no
filter, no globalAlpha and no globalCompositeOperation other than the default `source-over`.

Anything that tints is a bug: the strips are pre-tinted (spec 4.5).

Export via `canvas.convertToBlob({ type: "image/png" })` on an `OffscreenCanvas`.

- [ ] **Step 4: Wire the probe**

In `popover.js`, behind a debug-only check mirroring how `clock_scale` is already gated, bake the
fixed build on load and send each blob through `write_custom_art`.

- [ ] **Step 5: Run the comparison**

Run: `tools/build-app-assets.sh && tools/verify-bake.sh`
Expected: nine `ok` rows, exit 0.

- [ ] **Step 6: Measure the comeback drift (spec test 2b)**

Assemble one comeback frame both ways for a build using a partial-alpha hairstyle: pre-tinted
layers composited, versus composited then modulated. Record `magick compare -metric AE` and the
maximum per-channel delta in a comment at the top of `tools/verify-bake.sh`.

Spec 4.5 predicts a small non-zero count confined to antialiased hair pixels. **If it exceeds a
max per-channel delta of 8, or touches any pixel at full alpha, stop.** The analysis in 4.5 is
wrong and the accepted trade in section 11 has to be revisited rather than shipped.

- [ ] **Step 7: Commit**

```bash
git add src/baker.js src/popover.js tools/verify-bake.sh
git commit -m "Bake a built mascot into the nine strips the app already reads"
```

---

### Task 11: The builder view

**Files:**
- Modify: `src/index.html`
- Modify: `src/popover.css`
- Create: `src/builder.js`

**Interfaces:**
- Consumes: `src/assets/layers/index.json`, the swatches, the plates, `bake` from Task 10.
- Produces: `openBuilder(current)` and `closeBuilder()`, and a `builder:done` event carrying the chosen `CustomCharacter`.

- [ ] **Step 1: Add the builder markup, hidden**

In `src/index.html`, after the picker, a `<section id="builder" hidden>` containing the tab row,
the swatch grid, and the three action buttons. Toggle it with the `hidden` property, matching how
`empty` and `error` are already handled in `popover.js`.

- [ ] **Step 2: Style it to the existing tokens**

In `src/popover.css`, follow the existing `.char-btn` rules: 32px cells, 8px gaps, `--edge`
outline, `--accent` when selected, `image-rendering: pixelated`. Add nothing to `style.css`, which
is shared with the pet window.

Budget from spec 6.2: tabs 28px, three swatch rows 120px, actions 34px, gaps about 30px, inside
the roughly 284px left under the room.

- [ ] **Step 3: Build the live preview**

Three stacked absolutely-positioned elements inside `.room-wrap`: the untinted awake back plate,
the character layers, nothing in front. The character sits at the builder pose from spec 4.3,
`+72+56` in room pixels, doubled to `+144+112` at the popover's 2x.

The preview is untinted and takes no overlay. Reuse the room's existing `steps()` animation so
the preview breathes rather than freezing.

- [ ] **Step 4: Wire the controls**

Tabs switch the grid's category. A swatch click updates the build and re-renders the preview only.
Shuffle randomises all five categories at once. Cancel restores the previously selected character
and closes. Done bakes, calls `write_custom_art` nine times, then `save_custom_character`.

- [ ] **Step 5: Verify by hand**

Run: `tools/drive-states.sh`

Check, and record the result in the commit message: the builder opens from `+`, the preview
updates on every swatch, Shuffle changes all five, Cancel leaves the previous character selected,
Done returns to the panel with the fourth slot filled and the room showing the built mascot.

- [ ] **Step 6: Commit**

```bash
git add src/index.html src/popover.css src/builder.js
git commit -m "Add the builder view to the popover"
```

---

### Task 12: The fourth slot, the room, and the share card

**Files:**
- Modify: `src/popover.js`
- Modify: `src/share.js`

**Interfaces:**
- Consumes: everything above.
- Produces: a four-button picker, custom room art via blob URL, and a share card that carries the built mascot.

- [ ] **Step 1: Add the fourth button**

In `buildCharacters`, append a slot after the three premades: a dashed `+` when
`payload.custom_character` is absent, the built mascot's pet frame when present. Clicking it when
absent opens the builder; when present but unselected it selects `"custom"`; when present and
already selected it re-opens the builder.

`CHARACTERS` at `popover.js:24` stays the three premades, matching the Rust constant.

- [ ] **Step 2: Resolve custom room art through a blob URL**

In `render`, when `payload.character_id === "custom"` and `payload.custom_art_ready` is true,
fetch the room strip via `read_custom_art` and set `backgroundImage` to
`URL.createObjectURL(blob)`. Revoke the previous URL on each render or the popover leaks one
object URL per mood change.

When `custom_art_ready` is false, render `CHARACTERS[0]` instead. This is spec section 5.4: a
half-written cache degrades to a premade rather than to a room with no person in it. Verify it by
deleting one file from `custom/rooms/` while the app runs and reopening the popover.

Confirm `tauri.conf.json` is untouched. The existing CSP already allows `blob:`.

- [ ] **Step 3: Point the share card at the same bytes**

`share.js:187` currently builds a path. Give it the same blob URL when the id is `"custom"`.

- [ ] **Step 4: Verify the whole arc**

Run: `tools/drive-states.sh`

Watch a built mascot through all four states in both the room and the pet, then copy a share card
and paste it somewhere. Record in the commit message that the card carries the built mascot and no
project name, path, message, hash or timestamp (spec section 8).

- [ ] **Step 5: Run every check**

```bash
cargo test --manifest-path src-tauri/Cargo.toml
tools/verify-plates.sh
tools/verify-layers.sh
tools/verify-bake.sh
git diff --stat HEAD~12 -- src-tauri/tauri.conf.json
cargo tauri build --target universal-apple-darwin
strings -a "src-tauri/target/universal-apple-darwin/release/bundle/macos/Momentum Mascot.app/Contents/MacOS/momentum-mascot" \
  | grep -cE 'drawsBackground|fullScreenEnabled'
```

Expected: all tests pass, all three verifiers exit 0, the `tauri.conf.json` diff is **empty**, and
the `strings` count is **0**.

A non-empty config diff means the CSP or `assetProtocol` moved and the App Store surface changed.
A non-zero `strings` count means a private API came back, which is spec section 10.7 and the
existing App Store spec's section 2.1. Neither is a warning; either one blocks the merge.

- [ ] **Step 6: Commit**

```bash
git add src/popover.js src/share.js
git commit -m "Give the built mascot its picker slot, its room, and its share card"
```

---

## Notes for the executor

- **Tasks 1 through 5 need the licensed pack** at `$MASCOT_PACK`. Tasks 6 through 9 need nothing but Rust and can be done in any order relative to the asset work.
- **Task 1's step 8 is the highest-value gate in the plan.** If reassembly is not pixel-exact for awake, do not proceed to Task 2. The failure is in the plates or the manifest, and every later task inherits it.
- `jq` is used by the verification scripts. It is not currently a project dependency. If it is unavailable, the alternative is `python3 -c` for the JSON reads, which is present on every macOS this app supports.
- The reassembly scripts write to `/tmp/.af-*.png` and `/tmp/.pf-*.png`. They are not concurrency-safe and are not meant to be run in parallel.
