# Build your own mascot: the "+" slot

**Date:** 2026-09-03
**Status:** design approved in brainstorming. Ready to plan.

**Extends** `docs/spec-v2.md` section 6.3, which shipped three premade characters and said
"adding a fourth later is an asset drop rather than a spec". That remains true of a fourth
*premade*. It is not true of a *built* character, because a built character cannot be baked at
build time, and this document is the difference.

**Corrects** two lines of section 6.3 that the shipped app already overtook. It says character
selection is "the entire selection mechanism: no picker UI and no settings screen" - a visible
picker shipped anyway (`src/index.html:23`, `src/popover.js:27`), and the guardrail it was
protecting turned out to be *no recurring configuration*, not *no configuration*. A builder is a
one-time act. That distinction is the whole justification for this feature and is argued in
section 1.

**Revision note (2026-09-03, during planning).** Section 4 as first approved was wrong about two
things, found by reading `compose-rooms.sh:400-460` line by line while drawing up tasks. The
corrections are kept visible rather than tidied away, because both are traps a reader would
otherwise re-enter:

- The character's placement is **per frame, not per state** - dozing and comeback hop, and the
  coffee and emotes hop with it (section 4.4). The original front-plate table assumed a fixed
  placement and was therefore unbuildable for two of the four states.
- Every state but awake applies **a colour operation over the finished frame**, character
  included (section 4.5). The original design would have had to reimplement ImageMagick's
  `-colorize` and `-modulate` in canvas. Measurement showed pre-tinting at build time is
  equivalent, which removes colour maths from the baker entirely.

**Preserves** the invariant the rest of the app is built on: **a character is a set of PNGs.**
The popover sets `backgroundImage` from a room strip, `share.js:187` loads the same strip, and
`sprite.rs:87` resolves a pet strip under the bundle's resource directory. Nothing downstream of
the bake step learns that a character was built rather than shipped.

---

## 1. Goal and non-goals

The goal is one extra slot in the character picker, marked `+`, which opens a builder inside the
existing popover and produces a mascot composited from LimeZu's Character Generator layers. The
mascot then behaves exactly like a premade in every surface: room, desktop pet, share card.

The premise this serves is the one the app already runs on. The mascot works because the person
in the room stands in for you - waiting, dozing, leaping out of bed. Three premades is a
compromise on that; a built mascot is the thing itself.

Non-goals, explicitly:

- **More premade characters.** 3-to-6 was considered and rejected *because* of this feature:
  premades bake into rooms, the builder un-bakes them, and doing both means curating twice and
  discarding the first pass.
- **Multiple custom slots.** One custom mascot. See section 7.1.
- **Exposing the pack's full generator.** 432 layer variants and 139 million combinations. See
  section 3.
- **A settings screen.** The builder is reached from the picker and returns to the panel. It is
  the only mode the popover has, and it has no preferences in it.
- **Retiring click-to-cycle.** `popover.js:197` keeps working. See section 7.3.

---

## 2. What the pack provides, measured

Everything in this section was measured against the licensed pack, not inferred from filenames.
`docs/asset-picks.md` records why that distinction is load-bearing here.

### 2.1 Layer sheets share the premade sheets' geometry exactly

| Category | 16x16 variants | Sheet size |
|---|---|---|
| Bodies | 9 | 927x656 |
| Eyes | 7 | 896x656 |
| Hairstyles | 200 | 896x656 |
| Outfits | 132 | 896x656 |
| Accessories | 84 (19 families) | 896x656 |

`Premade_Character_NN.png` is also 896x656. **Every crop in `tools/compose-rooms.sh` therefore
applies to a layer sheet unchanged** - the idle pose at `16x32+48+0`, the run row at `y=32`
`x=192..272`, the seated loop at `y=192` `x=48..128`, and the sleep loop at `y=96` `x=0..80`.
No new coordinate archaeology is required, which is the single biggest reason this feature is
tractable at all.

Bodies being 927 wide rather than 896 is padding on the right-hand edge. Every crop this project
takes is at `x <= 272`, so it is not reachable and needs no handling.

### 2.2 The stacking order is the pack's, and the sleep row needs no special case

`CHARACTER_GENERATOR.txt` documents the order: **body, eyes, outfit, hairstyle, accessory.**

The row that looked most likely to break it is the sleep row, because `asset-picks.md` records
that the sleep loop "needs reading, not indexing" and uses "this character's own hat". Mean alpha
over the sleep crop (`96x32+0+96`):

| Layer | sleep row | seated row | standing |
|---|---|---|---|
| Body_05 | 0.340 | 0.526 | 0.527 |
| Hairstyle_27_06 | 0.286 | 0.286 | 0.286 |
| Accessory_04_Snapback_05 | 0.277 | - | - |
| Outfit_25_01 | **0** | 0.135 | 0.162 |
| Eyes_05 | **0** | - | - |

Outfits and eyes are **empty in the sleep row**, and this was confirmed across four sampled
outfits rather than one. The character is under a duvet there, so the layers that would be
covered draw nothing, and the layers that stay visible - hair, hat - draw normally. A naive
composite in the documented order produces a correct sleeping character with no branch.

This is the finding that turns the sleep row from the feature's biggest risk into a non-event.
It must be re-checked for every outfit and accessory admitted to the palette (section 11).

### 2.3 The sprite sits at an eight-row offset

Alpha by row band on `Body_05` at `16x32+48+0`: rows 0-7 are empty, rows 8-11 are 0.28, rows
12-15 are 0.81. Content is 16x24 at an 8-row offset, which is what `spec-v2.md:321` already
records for the emote arithmetic.

**This is a trap for the swatch crops.** A head swatch taken as `16x16+48+0` is eight rows of
nothing over the top half of a skull, which is how the first pass of the mockups rendered nine
distinct skin tones as nine identical red blobs. The head band is `16x16+48+6`; for skin
swatches specifically, which must show the face rather than the hair, `16x16+48+7` with no
hairstyle layer.

---

## 3. The shipped palette

**Curated, not exposed.** Nine skin tones and seven eyes are shipped whole because they are
identity and there are only sixteen of them. The three large categories are cut down:

| Category | Shipped | Of |
|---|---|---|
| Skin | 9 | 9 |
| Eyes | 7 | 7 |
| Hair | ~20 | 200 |
| Outfit | ~16 | 132 |
| Accessory | ~10 including "none" | 84 |

That is roughly **62 layer strips and about 200,000 combinations**.

Two reasons for curating rather than shipping all 432:

1. **Tone.** The accessory list runs from `Glasses`, `Beanie` and `Beard` to `Zombie_Brain`,
   `Party_Cone` and `Bataclava`. This app's voice is quiet and sincere - "the mascot never dies,
   it waits". A party cone is a different product. The palette keeps the everyday half and drops
   the novelty half. `Glasses` is the single most identifying item available to a developer
   audience and is not optional.
2. **Licence.** `README.md` states the licence "permits shipping it compiled into an application
   and forbids redistributing it as assets", and `docs/app-store-licence-check.md` is the
   existing record on this. Shipping a curated derived subset is the same act as shipping the
   composited rooms, which is already settled. Shipping the generator's whole library is
   materially closer to redistributing it, and this design declines to test that line for a
   feature that does not need it.

**One accessory slot, not two.** The generator documents a single accessory. Glasses and a beard
occupy disjoint pixels and would stack, but taking two would be a deliberate deviation from the
pack's own order and would add a second row to a control budget that has ~280px. Deferred, not
refused; revisit if it is the most-asked-for thing.

The exact per-category picks are a curation task for the implementation plan, not a design
decision. The plan must record the chosen file list in `docs/asset-picks.md`, which is the
manifest for exactly this kind of thing.

---

## 4. The room becomes two plates

### 4.1 The z-order differs by state, and this is the constraint everything else follows from

From `tools/compose-rooms.sh`:

| State | Composite order | Lines |
|---|---|---|
| awake | base, cat, **character**, desk, computer | 415-419 |
| dozing | base, cat, desk, computer, **character**, coffee, dots | 426-432 |
| asleep | base, cat, desk, computer, **character**, z | 438-443 |
| comeback | base, cat, desk, computer, **character**, bang, sparks | 450-457 |

**In `awake` the desk lands on top of the character.** `popover.js` already names why: the
occlusion "is the entire difference between reading as sitting at the desk and standing behind
it." A runtime character drawn over a finished room would stand in front of its own desk.

### 4.2 Back plate and front plate

Each state gains two character-less strips, both 1920x112, twelve frames:

| State | Back plate | Front plate | Character-tracking overlays |
|---|---|---|---|
| awake | base + cat | desk + computer | - |
| dozing | base + cat + desk + computer | *empty* | coffee, dots |
| asleep | base + cat + desk + computer | z | - |
| comeback | base + cat + desk + computer | *empty* | bang, spark ×2 |

The character composites between the plates. Membership differs per state, which is why the
plates are generated by `compose-rooms.sh` - the script that already owns these positions -
rather than described a second time anywhere else.

**A front plate can only hold what does not move with the character.** Awake and asleep have a
static character, so desk, computer and the `z` emote are pre-baked. Dozing and comeback hop
(section 4.4), and their coffee and emotes are positioned by `shift_pos`/`emote_pos` relative to
the hopped position, so those overlays cannot be baked into a plate. They ship as their own
strips and the baker places them per frame from the manifest.

**The three premade characters keep their existing baked rooms.** Regenerating them through the
plate path would be tidier and risks a visual regression in shipped art for no user-facing gain.
The duplication is deliberate and costs roughly 80KB.

### 4.3 The builder pose

While the builder is open the preview shows the character **front-facing, centred, at `+72+56`**,
with no coffee and no emote, regardless of actual mood.

`72` is `(160 - 16) / 2`. The dozing placement at `+54` was tried first and is wrong for this
job: it puts the character left of centre with the floor lamp crowding one shoulder and the feet
on the rug's border. Centred, the whole body is clear of the lamp and the desk.

Forcing the pose is not a nicety. Mood is derived from reflog activity, so a user whose projects
have gone quiet would otherwise be dressing a character they can only see asleep under a duvet.

At `+72+56` the character's feet land just above the cat at `+76+84`. The compositor's existing
rule covers it - the cat goes down before the character "so if the two ever overlap the person
wins" - and the builder preview keeps that order.

The preview takes no overlay and no tint: it is the untinted back plate, the character, nothing
else. Tinting the preview would mean judging a hairstyle through a 34% blue wash.

### 4.4 The character hops, so placement is per frame

`compose-rooms.sh:308-309`:

```
HOP_DOZING="0 0 0 0 0 0 1 1 1 1 1 1"
HOP_COMEBACK="0 -1 -2 -2 -1 0 0 -1 -2 -2 -1 0"
```

These are vertical offsets applied to the character's placement per frame, and the coffee
(`shift_pos "$at" 12 -4`), the emotes (`emote_pos "$at"`) and the comeback sparkles
(`shift_pos "$at" -9 -4` and `+21 -4`) are all positioned from the hopped value. Awake and asleep
have no hop.

The manifest (section 5.2) therefore carries a **twelve-entry offset array per state**, not a
single placement, plus the overlay deltas. A baker that reads one placement per state produces a
character standing still inside a room that is breathing, which reads as a bug in the animation
rather than as a bug in the maths.

Note also that `IDLE_FRAMES` is a **single** sprite reused across all twelve frames: dozing and
comeback animate entirely through the hop and the emote, not through character art.

### 4.5 The tint is applied at build time, not at bake time

Every state but awake finishes with a colour operation over the whole frame, character included
(`compose-rooms.sh:315-318`):

| State | Operation |
|---|---|
| awake | none |
| dozing | `-fill #3050a0 -colorize 10` |
| asleep | `-fill #3050a0 -colorize 34` |
| comeback | `-modulate 113,120` (brightness, saturation) |

The script's own comment calls this "a flat colour applied over the finished frame, not redrawn
art."

The naive reading is that the baker must apply the tint after compositing the character, and must
therefore reimplement both operators in canvas. **It does not.** Both plates and character layer
strips ship **pre-tinted per state**, and the baker does nothing but source-over.

This is exact rather than approximate, for two different reasons:

- `-colorize` is a per-pixel linear blend, `T(x) = (1-k)x + kc`. Source-over is also linear in
  its operands, so `T(A) over T(B) == T(A over B)` identically, at any alpha. Dozing and asleep
  are therefore exact for every possible build.
- `-modulate` is an HSL operation and does **not** commute in general. It commutes exactly where
  alpha is 0 or 255, because source-over there simply selects one operand. Measured partial-alpha
  pixel counts inside the used crops:

  | Sheet | idle | seated | sleep | run |
  |---|---|---|---|---|
  | Premade 07 / 12 / 20 | 0 | 0 | 0 | 0 |
  | Body / Eyes / Outfit / Glasses (sampled) | 0 | 0 | 0 | 0 |
  | Hairstyle_27_06 | 4 | 24 | 24 | 24 |
  | Hairstyle_11_03 | 8 | 48 | 48 | 24 |

  The three premades are pure binary alpha in every crop this project takes, which is why the
  shipped room strips are fully opaque and why the reassembly test in section 10 can demand
  pixel-identity. Hairstyles are the only category carrying antialiasing.

So the only inexactness in the whole design is **comeback, on a built mascot, at up to 48
antialiased hair pixels per frame, all at alpha ≤ 105/255**. That is accepted (section 11) rather
than paid for with two hand-written colour operators whose failure mode is a mismatch across
every pixel instead of forty-eight.

---

## 5. Compositing: bake on save

### 5.1 Where it happens

**The webview's canvas, on Done.** Not Rust, and not at display time.

Rust was rejected because `Cargo.toml` has no image crate and adding one to a binary that goes
through App Review, to duplicate what canvas already does, buys nothing.

Display-time layering was rejected for the pet specifically. The pet is a native CALayer stepping
`contentsRect` across a 384x32 strip, and `pet.rs` records that this AppKit configuration is
history-dependent - "a configuration that measures differently depending on its past is not one
to shave". Five sublayers is exactly the kind of change that invalidates that hard-won recipe.

So: **layer live in the DOM for the builder's preview, bake once on Done.** The preview is three
stacked elements over the back plate - cheap, instant, and it never touches the pet. The baked
output is an ordinary set of strips.

### 5.2 The bake reads a manifest, so the placements have one source of truth

The bake step is the compositor performed again in a different language, and the failure mode is
drift: a placement changed in the shell script and not in the JS.

`tools/compose-rooms.sh` therefore emits `src/assets/character-layout.json` alongside the plates.
The JS baker consumes it and hard-codes none of it. The script stays the source of truth; the
manifest is its machine-readable half, the same relationship `asset-picks.md` has to it in prose.

Per state it must carry:

- the base character placement, and the **twelve-entry hop array** added to it (section 4.4)
- which frame range of the layer strip that state draws from, and whether the frames advance or
  a single sprite repeats
- each character-tracking overlay: its sprite, its per-frame delta from the hopped placement, and
  its own frame count (the emotes are two-frame pairs)
- the same for the pet: `PET_BREATH`, the pet-local emote deltas, the blanket

Anything the baker would otherwise have to know as a constant belongs here. The test for whether
a value is missing is mechanical: if changing it in `compose-rooms.sh` alone would make the built
mascot disagree with the premades, it is not in the manifest yet.

### 5.3 Output, and why no asset protocol is needed

On Done the baker writes, next to `state.json`:

```
<state dir>/custom/rooms/{awake,dozing,asleep,comeback}.png   1920x112
<state dir>/custom/pet/{awake,dozing,asleep,comeback,run}.png  384x32
<state dir>/custom/build.json                                  the five layer ids
```

`<state dir>` is the directory `store::default_path()` already resolves to, so this inherits the
sandbox behaviour worked out in `store.rs:80` for free: the App Store build lands in the
container, the DMG build in `~/.keepgoing/mascot/`, and no code branches on which.

**The popover reads those bytes back through a Tauri command and makes a blob URL.** The existing
CSP is `img-src 'self' data: blob:` (`tauri.conf.json:28`) - `blob:` is already permitted, so
this needs **no `assetProtocol` scope and no CSP change**, which is the cheapest possible answer
and keeps the App Store surface identical. `share.js:187` takes the same blob URL.

The pet needs none of this: it reads a file path natively. `sprite::resolve_path` gains one
branch - the custom id resolves under the state directory instead of `resource_dir()`.

### 5.4 Invalidation

`build.json` holds the five layer ids. The strips are regenerated whenever it changes, which is
only on Done.

A missing or unreadable strip falls back to `CHARACTERS[0]`. This is a **new check at render
time, not the existing branch** at `store.rs:140` - that one validates the id as the state file
loads and cannot see whether the art behind a valid id is actually on disk. Both exist for the
same reason and they are not the same code: a half-written cache must degrade to a premade rather
than to a room with no person in it.

The bake is nine PNGs of pixel art. It runs on Done, once, and does not need progress UI.

---

## 6. The builder UI

### 6.1 A takeover of the existing popover

No second window. The popover is 352x540, fixed, undecorated, `alwaysOnTop`, and the credit line
notes the app has "no about window and no settings screen". A second window would need the
`appkit::show_over_fullscreen` treatment and would be a new surface in an app whose whole pitch is
that it has one.

The builder replaces the panel's contents below the room. The room stays.

### 6.2 Layout

Top to bottom, inside the existing 320px content column:

1. **Preview.** The 320x224 room, back plate + live character + nothing in front, character in
   the builder pose (section 4.3).
2. **Category tabs.** Five: skin, hair, eyes, wear, extra.
3. **Swatch grid.** 32px swatches, 8px gaps, eight per row. 16 visible without scrolling, which
   covers every category except hair at a scroll of one row.
4. **Actions.** Cancel, Shuffle, Done.

Budget: 540 less 224 of room, less 32 of padding, leaves ~284px. Tabs 28, three swatch rows 120,
actions 34, gaps ~30 - about 212. It fits, with the margin going to hair.

Swatches are head crops at `16x16+48+6` (section 2.3), except skin, which uses `+48+7` with no
hair so the face is visible.

### 6.3 Shuffle

Kept. It is the only control that does not correspond to a decision the user is making, which is
the case against it, but it is also the fastest way to learn what the palette can do on first
open, and an empty builder with five untouched tabs is a worse first frame than a random mascot.
It randomises all five categories at once.

### 6.4 Entering and leaving

The picker's fourth button is a dashed `+` while no custom mascot exists, and the mascot's head
once one does. Clicking it when empty opens the builder; clicking it when filled selects the
custom mascot, and clicking it *again while selected* re-opens the builder to edit. Four buttons
at 32px with 12px gaps is 152px in a 320px column; the ceiling before the row wraps is seven.

Cancel restores the previously selected character and discards the in-progress build. Done bakes,
selects the custom mascot, and returns to the panel.

---

## 7. State and schema

### 7.1 The record

`SCHEMA_VERSION` goes `3.1` to `3.2`. One new optional key beside `character_id`:

```json
"custom_character": {
  "body": "Body_03", "eyes": "Eyes_02", "outfit": "Outfit_11_04",
  "hair": "Hairstyle_11_03", "accessory": "Accessory_15_Glasses_05"
}
```

`accessory` is nullable; the other four are required when the object is present. `character_id`
becomes `"custom"` when the built mascot is selected, so the selection stays a single string and
every existing consumer of `character_id` keeps working.

`store::CHARACTERS` stays `["07", "12", "20"]` and does **not** gain `"custom"`. It means "the
shipped premades" in every place it is used, and widening it would silently change the fallback
at `store.rs:141` and the cycle at `momentum.rs:308`. Validity of `"custom"` is a separate
predicate: the id is valid when `custom_character` is present.

### 7.2 Downgrade

`store.rs:140` already filters an unrecognised `character_id` and falls back to `CHARACTERS[0]`,
and its comment says why: "A future release that ships more characters must not break the state
file of a user who downgrades." That branch was written for this and needs no change - an older
build reading a `3.2` file sees `"custom"`, does not recognise it, and shows character 07. The
`custom_character` object is preserved on rewrite by the same tolerance the loader already
applies to unknown keys, which the plan must verify rather than assume.

### 7.3 The two selection paths

`set_character` (`commands.rs:118`) gains `"custom"` as valid when `custom_character` is present.

`cycle_character` (`momentum.rs:303`) currently indexes `CHARACTERS` directly. Since
`CHARACTERS` does not gain `"custom"`, the cycle must build its own sequence: `CHARACTERS`, then
`"custom"` appended when `custom_character` is present. Four long with a built mascot, three
without, ending on custom before returning to 07.

The tests at `momentum.rs:712-714` assert the three-step cycle. They must gain a four-step case
rather than be edited to fit, because the three-step behaviour is still correct for a user who
has never opened the builder and is the more common path.

---

## 8. The share card

`share.js:187` loads `assets/rooms/<id>/<mood>.png` and draws frame 0. With the custom id it
takes the blob URL instead. Nothing else changes.

**The privacy properties are unchanged and this is worth stating explicitly**, because the share
card is the one artifact in this app with a hard privacy contract. A built mascot is pixel art
composited from a licensed pack. It carries no project name, path, commit message, hash or
timestamp, and it cannot be made to. `README.md`'s claim that the card "is built so that there is
no way to express one rather than a rule about remembering not to" survives this feature intact.

---

## 9. Build pipeline

`tools/compose-rooms.sh`:
- emits back and front plates per state, character-less
- emits `character-layout.json` (section 5.2)

New `tools/compose-layers.sh`:
- crops the curated palette to **432x32 strips, 27 frames**, laid out as:

  | Frames | Contents | Tint |
  |---|---|---|
  | 0 | idle | none |
  | 1-6 | run | none |
  | 7-12 | seated | none |
  | 13-18 | sleep | none |
  | 19 | idle | `-colorize 10` (dozing room) |
  | 20-25 | sleep | `-colorize 34` (asleep room) |
  | 26 | idle | `-modulate 113,120` (comeback room) |

  Frames 0-18 are untinted because **every pet frame is untinted** - `frame_pet_*` applies no
  colour operation at all - and because awake's room is untinted and the builder preview is
  untinted. Only the three tinted room states need extra art, and dozing and comeback need one
  frame each rather than six, because `IDLE_FRAMES` is a single repeated sprite (section 4.4).

  The frame indices live in the manifest, not in the baker.

- output `src/assets/layers/<category>/<id>.png`
- emits the swatch crops the builder's grid uses (section 6.2)
- emits the shared sprites the baker places but that are not per-layer: coffee, the four emote
  pairs, and the sleep blanket. The blanket is generic art rather than per-character
  (`asset-picks.md` calls `192,96` "a generic overlay for any bed"), so it ships once; the plan
  verifies it is byte-identical across the sheets in the palette rather than assuming it.

`tools/build-app-assets.sh` calls both. `CHARACTERS="07 12 20"` at line 31 is untouched.

`tauri.conf.json` `resources` currently maps `"../src/assets/pet": "pet"`. The pet strips for
premades stay there; layer strips are read by the webview and live under `src/assets/`, which is
`frontendDist` and needs no resource mapping.

---

## 10. Acceptance tests

1. **Sleep row.** For every outfit and accessory admitted to the palette, the mean alpha of
   `96x32+0+96` matches the expectation in section 2.2 - zero for outfits, non-zero for
   accessories that are headwear. A novelty accessory that draws a body item in the sleep row
   would clip through the duvet, and this is the check that catches it.
2. **Plate reassembly.** For each state and each of the twelve frames, back plate + premade
   character at the manifest's hopped placement + character-tracking overlays + front plate is
   **pixel-identical** to the currently shipped `rooms/<id>/<mood>.png`, for all three premades
   and all four states. `magick compare -metric AE` must report 0.

   This is the most valuable test in the plan and it is achievable exactly, because the premades
   are binary-alpha (section 4.5). It proves in one assertion that the plate split, the hop
   arrays, the overlay deltas and the pre-tinting are all faithful to the compositor. If it
   passes for twelve frames across four states and three characters, the manifest is complete.

   Run it against the pet strips too: the same reassembly against `pet/<id>/<mood>.png`.

2b. **Comeback drift on a built mascot.** Assemble one comeback frame for a build using a
   hairstyle with partial alpha, both ways - pre-tinted layers composited, versus composited then
   modulated - and record `magick compare -metric AE` and the maximum per-channel delta. Section
   4.5 predicts a small non-zero count confined to antialiased hair pixels. **The number goes in
   the plan's notes.** If it exceeds a max per-channel delta of 8, or touches pixels at full
   alpha, the analysis in 4.5 is wrong and the accepted trade in section 11 must be revisited
   rather than shipped.
3. **Bake output.** Room strips are 1920x112, pet strips 384x32.
4. **Downgrade.** A `3.2` state file with `character_id: "custom"` loaded by the `3.1` loader
   yields `"07"`, and a `3.2` loader round-trips `custom_character` unchanged.
5. **Cycle.** `cycle_character` is three long with no custom mascot and four with one.
6. **CSP.** `tauri.conf.json`'s `csp` and `assetProtocol` are byte-identical to before. If either
   changed, the blob-URL design was abandoned somewhere and the App Store surface moved with it.
7. **Private API.** `strings -a <binary> | grep -cE 'drawsBackground|fullScreenEnabled'` is still
   0, per the existing App Store spec's section 2.1.

---

## 11. Deliberately accepted

- **Premade rooms stay baked**, duplicating ~80KB against the plates. Buys zero regression risk
  in shipped art (section 4.2).
- **One accessory.** Glasses or a beard, not both (section 3).
- **No progress UI on Done.** Nine small PNGs.
- **The bake is JS, the compositor is shell.** Mitigated by the manifest (section 5.2), not
  eliminated. If they drift, test 2 is what catches it.
- **Comeback is approximate on a built mascot**, at up to 48 antialiased hairstyle pixels per
  frame, all at alpha ≤ 105/255 (section 4.5). Exact for all three premades, and exact for every
  build in the other three states. Bought in exchange for zero hand-written colour operators in
  the baker, whose failure mode would be a mismatch across every pixel rather than forty-eight.
  Test 2b is what holds this claim to a number.
