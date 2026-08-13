# Asset picks - Modern Interiors

Exact source files for the room. All paths are relative to:

```
~/Workspace/OneQode/projects/repos/oneqode-pixel-assets/moderninteriors-win/
```

Everything below comes from the **full paid pack** only. Nothing from `Modern tiles_Free/`
(non-commercial licence). Credit `limezu.itch.io` is required in the shipped app.

**This file is the manifest; `tools/compose-rooms.sh` is the executable version of it.** Run that
script to regenerate all four states. Nothing here needs to be placed by hand, and the coordinates
below are kept in sync with the script rather than duplicated from memory.

```sh
tools/compose-rooms.sh              # all four states into docs/mockups/
tools/compose-rooms.sh awake        # just one
MASCOT_CHAR=12 tools/compose-rooms.sh    # a different premade character
```

Per state it writes the still (`state-<s>-160x112.png`), the 12-frame strip the app consumes
(`state-<s>-strip-12f.png`, 1920x112), and a GIF at that state's own rate for review
(`state-<s>.gif`). Plus `states-four.png` and `states-four.gif` for looking at all four together.

`tools/build-app-assets.sh` is what the application consumes. It runs the room compositor once per
shipped character (07, 12, 20) with `MASCOT_APP_OUT` set, so each run also writes a room strip and
a **pet strip** into `src/assets/`, then draws the icons and copies the font. That tree is build
output and stays out of git for the same reason `docs/mockups/` does.

`tools/compose-share.sh` then builds the 1200x630 share card for each state from those stills,
writing `share-<s>-1200x630.png` and a `share-four.png` contact sheet. It consumes frame 0 only,
so run the room compositor first. Review both in `docs/mockups/preview.html` (the rooms) and
`docs/mockups/share-preview.html` (the cards, at 1:1 and down a thumbnail ladder, on light and
dark timelines).

## Room geometry

- Composed on the **16x16 native grid**.
- Room is **160x112** source pixels (10 by 7 tiles).
- Wall band is the top **34px**, floor is y 34-111.

34 is not a rounding error. One wall strip from the Room Builder sheet is exactly 34px: 2px
outline, 5px top cap, 20px face, 4px baseboard, 2px outline. Rounding it to 32 cuts the baseboard.

## Surfaces

Both are deliberate picks from the Room Builder sheets, replacing the placeholders that were
sampled out of an assembled home in the first pass.

| Piece | File | Crop | Notes |
|---|---|---|---|
| Floor | `1_Interiors/16x16/Room_Builder_subfiles/Room_Builder_Floors_16x16.png` | `16x16+0+384` | light warm plank |
| Wall | `..._subfiles/Room_Builder_Walls_16x16.png` | `16x34+16+222` | warm beige, includes its own baseboard |

**Sheet layout, for picking a different one.** Floors: each design is a 48x32 block at
`x ∈ {0,64,128,192}`, `y = 32 + 32k`; any 16x16 inside a block is a tile. Walls: three
style-columns at `x ∈ {0,176,352}`, one style per 32px row, and the full 34px strip starts 2px
above the row boundary.

## Props

| Piece | File | Size | Placed at |
|---|---|---|---|
| Bed (single, vertical) | `1_Interiors/16x16/Theme_Sorter_Singles/4_Bedroom_Singles/Bedroom_Singles_149.png` | 16x48 | +10+34 |
| Floor lamp | `.../2_Living_Room_Singles/Living_Room_Singles_87.png` | 16x48 | +30+36 |
| World map | `.../5_Classroom_and_Library_Singles/Classroom_and_Library_Singles_31.png` | 32x32 | +64+2 |
| Bookshelf body | `.../5_Classroom_and_Library_Singles/Classroom_and_Library_Singles_60.png` | 32x48 | +106+6 |
| Bookshelf cap | `.../5_Classroom_and_Library_Singles/Classroom_and_Library_Singles_61.png` | 16x48 | +138+6 |
| Potted plant | `.../2_Living_Room_Singles/Living_Room_Singles_17.png` | 16x48 | +2+64 |
| Desk | `.../5_Classroom_and_Library_Singles/Classroom_and_Library_Singles_25.png` | 32x32 | +108+58 |
| Computer | `3_Animated_objects/16x16/spritesheets/animated_receptionist.png` crop `16x11+0+12` | 16x11 | +111+65 |
| Coffee | `.../spritesheets/animated_coffee.png` crop `16x32+32+0` | 16x32 | beside the character |
| Rug | `1_Interiors/16x16/Theme_Sorter/1_Generic_16x16.png` crop `60x40+146+68` | 60x40 | +48+68 |
| Cat | `3_Animated_objects/16x16/spritesheets/animated_cat.png` crop `28x16+7` per 48px cell | 28x16 | +76+84 |

Upper-left wall (x 0-63) and the area above the bed are deliberately bare, because the share
image overlays type there (spec section 5.2).

### Crop bounds are read off the alpha, never eyeballed

The pack pads its sprites with **the outline colour at alpha zero**. Around the rug, `#3a3a5000`
sits directly against `#3a3a50ff`. On any dark background the padding and the real edge are the
same colour, so a crop can look correct and still be wrong: the rug shipped for a while as
`62x40+144+64`, which starts four rows above the sprite and therefore ends four rows short, and it
stood in the room with its bottom outline and the last of its red band sliced off.

Trimming does not rescue this. The theme sheets are dense enough that any hand-drawn box around one
sprite catches its neighbours, so `-trim` returns the box you gave it.

What works is dumping the region as text and finding where alpha changes:

```sh
magick "$SHEET" -crop 90x60+135+58 +repage txt:- | less
```

Each line is `x,y: (r,g,b,a) #RRGGBBAA`. The real edge is the first row or column whose outline
pixels are `ff`, not `00`. The rug is `x 146..205, y 68..107`, so `60x40+146+68`.

### A wide prop is a body plus a cap

`Classroom_and_Library_Singles_60` is 32x48 and looks like a whole bookcase in a file listing. It
is not. It is the left two-thirds of a 48-wide four-bay one, cut off mid-shelf with no right-hand
frame and no right leg, and `Singles_61` (16x48) is the missing cap. Butt them together and the
shelf is whole; place the body alone and it stands in the room with a corner sliced off.

Nothing in the filenames says this. The theme folders number wide props as consecutive files, so
the test is mechanical: **before placing any multi-tile prop, composite it against the next number
and look.** The same shape shows up all through the classroom sheet - 57/58 are complete 32-wide
two-bay shelves, 59 is a complete 16-wide one, and 60/61, 67/68 and 74/75 are body-and-cap pairs.

### The computer is the one cropped sprite

The pack has no computer desk and no standalone monitor. It does have `animated_receptionist`,
which is a character behind a counter with a grey tower **seen from behind**, and that is the only
place in 48,000 files this art exists. Cropping it out is explicitly permitted by the licence
("edit and use the asset in any commercial or non commercial project").

Taking it also solves the perspective problem, rather than working around it: a top-down desk and
a front-facing sprite cannot both be right about which way a screen points, unless the screen
points away from the viewer. The pack's own answer is the right one.

### Draw order is load-bearing

```
floor → wall → rug → map → bookshelf → bed → lamp → plant     (the static base)
  → cat → CHARACTER → desk → computer → emote                 (per frame, per state)
```

The character goes down **before** the desk so the desk covers their lower body. That occlusion is
the entire difference between reading as *sitting at* the desk and *standing behind* it.

Everything up to the plant is baked once into a base image. Everything after it is composited per
frame, which is why the desk is not in the base: it has to land on top of the character.

The cat goes down before the character in every state, so if the two ever overlap the person wins.
As placed they do not overlap, and the character positions on the rug were nudged left to keep it
that way.

## Characters

Shortlisted: **07**, **12**, **20**.

```
2_Characters/Character_Generator/0_Premade_Characters/16x16/Premade_Character_07.png
                                                          /Premade_Character_12.png
                                                          /Premade_Character_20.png
```

Each sheet is 896x656 and **every premade character carries the identical animation set**, so
supporting three characters costs three PNGs rather than three sets of rooms. `MASCOT_CHAR=12`
regenerates every state against a different one.

| Frame | Crop | Placed at |
|---|---|---|
| Standing, facing the viewer | `16x32+48+0` | dozing +54+56, comeback +54+64 |
| Seated loop, 6 frames | `16x32+{48,64,80,96,112,128}+192` | awake +111+41 |
| Sleeping loop, 6 frames, this character's own hat | `16x32+{0,16,32,48,64,80}+96` | +10+35 |

### Row 0 is one pose per facing, not an idle animation

Row 0 has four sprites and they are **left, up, right, down**. So `x=0`, the obvious crop, is a
left-facing side view: at 16px that is a hat with a sliver of cheek under it and no readable
person. The standing character was that sprite for the whole first pass, in both dozing and
comeback, and it read as a blob rather than as a mistake, which is why nobody caught it.

The front-facing pose is **`x=48`**. This is checkable rather than a matter of taste: flopping
`x=0` reproduces `x=32` to within four pixels, which proves those two are the side pair and
therefore that the remaining two are front and back. The walk row at `y=32` groups the same way,
six frames per facing in the same order, so its front-facing walk starts at `x=288`.

### The sleep row needs reading, not indexing

Row `y=96` on every character sheet is labelled `sleep` and is a recipe, not a sprite:

- `128,96` a 16-wide bed with a pillow
- `160,96` a **bald** head, eyes closed
- `192,96` that same bald head with a tan blanket, a generic overlay for any bed
- `0..80,96` the same head **wearing this character's own hat**, six frames, no blanket

Take `0,96` and let the bed supply the blanket. The hat is the whole reason for picking a
character, and a bald head throws it away. (An earlier draft of this file said `191,96`; the
sprite actually starts at 192, and it is the wrong sprite anyway.)

**This is why the bed must be a single.** The sleeping head is 16px wide. On a 32-wide double
bed it reads as a doll dropped on the covers. Vertical singles are `Bedroom_Singles_140` through
`189`, all 16x48; doubles are `217` upward. Side-view beds have no sleeping pose at all.

## Emotes

All from `4_User_Interface_Elements/UI_thinking_emotes_animation_16x16.png`.

| Emote | Crop | Offset from the character's top-left |
|---|---|---|
| `Z` (dozing) | `16x16+96+80` | +8, -13 |
| `Z` (asleep) | same | +13, -9 |
| `!` (comeback) | `16x16+0+80` | +8, -13 |
| sparkle | `16x16+64+96` | -9,-4 and +21,-4 |

The emote is anchored to the **character**, never to the room. In the first prototype the `Z`
floated over the desk, which read as the room being sleepy rather than the person. A sleeper gets
its own offset because they have no headroom: the `Z` drifts right instead of up, or it lands in
the wall.

## The four states

Same room every time. Only three things change: **where the character is, how the room is lit,
and which emote is showing.**

| State | Trigger | Character | Lighting | Emote |
|---|---|---|---|---|
| Awake | latest commit < 24h | at the desk, face above the monitor | normal | none |
| Dozing | 24h to 72h | away from the desk, coffee steaming | 10% `#3050a0` | `Z` |
| Asleep | >= 72h | in bed, hat on, under the blanket | 34% `#3050a0` | `Z` |
| Comeback | asleep -> awake | out of bed, on the rug | +13% brightness, +20% saturation | `!` plus two sparkles |

Lighting is a flat colour over the composed frame, not per-pixel art, so retuning a mood is one
number.

## What actually animates

Every state is a **12-frame loop**. Twelve is arithmetic, not taste: it is divisible by every
layer's own frame count, so each layer is indexed `frame % count` and every cycle closes in the
same place with no seam. Change the loop length and the layer counts need rechecking.

| Layer | Frames | Source |
|---|---|---|
| Cat | 12 | `animated_cat.png`, tail only |
| Character, seated | 6 | row `y=192`, the sheet's own `4-9 loop` |
| Character, sleeping | 6 | row `y=96`, `x=0..80` |
| Coffee steam | 6 | `animated_coffee.png` |
| Computer | 3 | `animated_receptionist.png` |
| Emote | 2 | adjacent cells on the emote sheet |

| State | Rate | Loop length |
|---|---|---|
| Awake | 6 fps | 2.0s |
| Dozing | 3 fps | 4.0s |
| Asleep | 2 fps | 6.0s |
| Comeback | 8 fps | 1.5s |

### The pack annotates its own loops, in pixels

Two frame tables were read off the sheets rather than guessed, and both were right:

- Row `y=192` has **`4-9 loop`** written next to it in pixel letters. Those are 1-indexed, so the
  loop is `x=48` through `x=128`.
- The emote sheet says **"sample animation, just swap the last 2"**, so each emote is a pair of
  adjacent cells: `Z` at 96 and 112, `!` at 0 and 16, sparkle at 64 and 80.

Both the seated row and the sleep row are **palindromes**: four frames differing from rest by 15 to
36 pixels, two differing by ~165 to 200, then back. The small frames are breathing, the big pair is
the character leaning in. Diffing frames against frame 0 is the fastest way to find a loop's shape
in a row you do not recognise.

### The computer has three states, not seven

`animated_receptionist` has 7 frames, but inside the computer crop, frames 0/2/4 are identical,
1/3 are identical, and 5/6 are identical. Three frames is the entire animation, and three divides
12, so the table is `x = 0, 16, 80`.

### The cat is 12 frames of 48x16, not 36 of 16x16

`animated_cat.png` is 576x16, which looks like 36 tiles and is not. Each cell is **48x16**, with
the cat drawn in the middle and the tail trailing left into the empty space. The union of all 12
bounding boxes is `x 7..34`, so cropping every frame at the same `28x16+7` sub-offset holds all of
them and, crucially, stops the cat twitching sideways between frames.

The body never moves; only the tail does. That is exactly why one animal can be in all four states
without making the room feel busy.

### A standing pose has no breathing frames, so the offset supplies them

Row 0 gives one static front pose and nothing else, so dozing and comeback animate the character
by moving it vertically: `+1px` for half the loop is a breath, `-2px` twice per loop is a hop. At
16px both read, because the character is only 32px tall. It is composition, not new art.

### The coffee frame order is rotated

`COFFEE_XS` starts at `x=32`, not `x=0`. Frame 0 of the sheet is the cup at rest with no steam,
and at this size a cup with no steam is a grey blob. Frame 0 of the *loop* is the still that goes
into the share image, so the plume has to be there.

## Object catalogues

Rendered contact sheets with index numbers, for picking replacements without hunting through the
48,000 files:

- `objects-conference.png` - 68 objects (office chairs 37-39, posters 54-55, counters 25-32)
- `objects-classroom.png` - 75 objects (desks 25-26 and 49-52, bookshelves 55-75, map 31, globe 34-35)
- `objects-livingroom.png` - 122 objects (screens 3-8, plants 13-18, lamps 71-88, cabinets 37-44)
- `bed-options-vertical.png` - vertical doubles 217-241, bunks 131-139. Singles 140-189 are not
  on this sheet and are the ones the sleep animation needs.
- `rug-variants.png` - the four rugs compared in place, in both awake and dozing
- `shelves.png` - classroom shelves 53-75 labelled with their sizes, which is the sheet that shows
  the body-and-cap pattern at a glance

To regenerate one for another theme, montage the folder under
`1_Interiors/16x16/Theme_Sorter_Singles/<theme>/` with each file labelled by its trailing number.

## Settled, with the reasoning

- **Floor and wall** are chosen, not sampled. Warm light plank and warm beige.
- **Rug**: kept red after comparing blue, small green, and small red in place. Red is the worst of
  the four under the asleep tint, where it goes maroon, and the best in comeback, where the
  saturation boost makes it the thing your eye lands on. Comeback is the state the product is for.
  Its right edge lands at x107 and the desk starts at x108, so the two meet with no gap and no
  overlap. If a different rug is swapped in, read its bounds off the alpha first: the crop in
  `rug-variants.png` was measured the same wrong way this one was.
- **Lamp**: cream shade on a wooden base, replacing the red one, which pulled focus harder than
  the character did.
- **Desk**: still a school desk with a book and a mug, because the pack has no computer desk. The
  computer now sits where the book was, and the mug survives.
- **Bookshelf**: `60` plus its cap `61`, at 48 wide, with 6px of wall left to the right of it.
  Alone, `60` loses a corner. `58` is a complete 32-wide two-bay shelf and is the drop-in
  alternative if 48 ever feels too heavy on that wall.
- **Cat**: on the rug, in every state, and not a state variable. The room was built without it
  first on purpose, and then it earned its way in: the asleep room has to read as cosy rather than
  abandoned, and one live animal does more for that than any furniture. The characters on the rug
  moved 6 and 12 pixels left to give it room.

## The share card

Not pack art, but it is composed from it, so its coordinates belong in the same manifest.
`tools/compose-share.sh` is the executable version. Canvas 1200x630 on `#191924`, which is the
pack's outline colour `#3a3a50` darkened so the mat is family with the art.

| Element | Size | Position | Notes |
| --- | --- | --- | --- |
| Room | 800x560 | `+200+12` | 160x112 at exactly 5x, in a 5px `#3a3a50` mount |
| State label | 55px type | `+220`, baseline `120` | 5x the font cell, so the room's own pixel unit |
| Quote | 22px type | `+200`, baseline `609` | in the footer band, never on the room |
| Credit | 11px type | right edge `184`, baseline `609` | left mat |
| URL | 22px type | right edge `1184`, baseline `609` | right mat, carries the wordmark too |

### The label is outlined, not shadowed

The label sits on the wall, and the wall runs from full-brightness beige (`#cbb297`) when awake to
a 34%-blue-tinted grey when asleep. A single fill plus a one-direction drop shadow reads on one end
of that range and washes out on the other: the first pass used each state's own tint and
`#8f9ec8` on the dimmed wall was unreadable. A **one-unit outline on all four sides** makes the
label independent of what is behind it, so one colour rule covers all four states. Fills are
`#f5c65c` awake, `#dce4f4` dozing, `#e4ecff` asleep, `#ffd45e` comeback: light end of each hue,
with the outline doing the contrast work.

### The quote could not go where the spec put it

Spec section 5.2 originally put the quote "along the room's lower strip". There is no such strip.
The bottom twelve rows hold the plant, the rug's gold-and-navy edge, and the cat, and the upper
wall is already carrying the label. Two candidate layouts were built and rejected by looking:

- **Room left, type in a right-hand column.** Never touches the art and the label reads perfectly
  on the dark mat, but a 310px column leaves the middle of the card empty and the picture stops
  reading as a matted picture.
- **Label and quote both on the wall.** Fails outright. The wall block is 310x165 at 5x on paper,
  but its top rows are the wall's trim, so the label lands on the trim and the quote's third line
  lands on the bed and the `Z`.

What was actually short was the footer band: a 5x room leaves 70px of vertical slack and the quote
plus a meta row plus margins needs 78px. Fixed by running the band across the **full 1200px**
instead of only the room's 800px column, which moves the credit and URL into the side mats. Those
mats were dead space in every candidate.

### Type sizes are all multiples of 11

Departure Mono (SIL OFL 1.1, vendored at `assets/fonts/departure-mono/`) is drawn on an 11px cell
with a 7px advance and is pixel-exact only at integer multiples of 11. Verified, not assumed: an
11px render upscaled 5x with a point filter diffs to **0 pixels** against a direct 55px render.

| Size | Unit | Advance | Used for |
| --- | --- | --- | --- |
| 11px | 1x | 7px | credit; the popover quote, where 42 chars is 296px and fits the 320px room |
| 22px | 2x | 14px | card quote and URL |
| 55px | 5x | 35px | state label, matching the room |

Press Start 2P and Silkscreen were rejected on measurements. Press Start 2P is a fixed 8x8 cell, so
the longest quote runs 1663px at 5x and overflows even the popover at 1x. Silkscreen renders
lowercase as small caps, and an all-caps quote contradicts the voice in spec section 4.6.
`+antialias` is mandatory throughout: an antialiased pixel font is just a blurry font.

### The room's temperature is what survives downscaling

Tested with a Lanczos resample, which is what platforms actually re-encode with, not
nearest-neighbour. At 504px everything reads. At 280px the label still reads and the quote is at
its limit. At 180px all the type is gone and the state is **still** unmistakable, because the warm
room and the cool room do not look alike. The cheapest variable in the whole design, a flat colour
multiply, is the one that survives the most compression.

## The pet cell

The desktop pet is the character only, never the room, at 64x64 on screen: a **32x32 cell at 2x**.
The composition is decided by measurement rather than taste, and the measurement is the whole
finding.

| | value |
| --- | --- |
| cell | 32x32, 12 frames, so a 384x32 strip |
| character | 16x32 sprite at `+0-2` |
| emote | 16x16 at character `+16+4` |
| sparkle (comeback only) | 16x16 at character `+16+18` |
| breath | `0 0 0 0 0 0 -1 -1 -1 -1 -1 -1` |
| asleep slump | 2px down from dozing, same pose |
| rates | awake 3, dozing 2, asleep 2, comeback 8 |

### The emote had to move beside the head

In the room the emote sits 13px **above** the character and clears the head completely. That does
not fit here, and the numbers say so before any rendering does: a character sprite is 16x32 whose
content is `16x24+0+8`, so there are exactly **eight** transparent rows above the head, and an
emote is 16x16 with **fifteen** rows of content.

Beside the head is the one arrangement where two 16-wide sprites tile a 32-wide cell with no
overlap at all. Character in the left half, emote in the right, aligned with the head rather than
the body. The comeback gets one sparkle rather than the room's two, because one 16x16 slot is what
is left once the character and the emote have tiled the cell.

The character sits 2px clear of the cell floor so both directions are available: the comeback hop
goes 2px up, the asleep slump 2px down, and neither loses a foot.

### The pet is never tinted

The room's lighting is a flat colour multiply, and the pet does not get one. A blue multiply over a
character standing on somebody's desktop wallpaper reads as a **recoloured sprite**, not as dim
light, because there is no room around them for the light to be in. That is why the emote carries
the state on the pet and the lighting carries it in the room.

## The tray icon is drawn, not cropped

`tools/make-icons.sh`, and it is the only art here that is not from the pack, so it is committed.

The obvious approach was to crop the character's head off the idle sprite at the measured content
offset and reduce it to a silhouette. It renders as a **solid black blob**: at 16px a filled
silhouette has no internal structure, because every internal edge in a sprite is a colour change
rather than a hole, and a macOS template image has only ink and no ink to work with. Extracting the
pack's outline colour (`#3a3a50`, fuzz 18%) instead gave structure and read as a burger.

What works is drawing the mark **at** the size it is used rather than reducing to it: a capped head
and shoulders with two eye holes, on a 16x16 grid, checked at 16, 18, 22 and 32px against a light
and a dark bar. The eye holes survive all four. **A menu bar template is a different register from
pixel art.**

## Open

- Nothing blocking. The remaining Phase 1 step is the one-week live-with test (spec section 12),
  which tests whether the *states* feel right, not whether the pixels are well drawn.
- **The cat's coat is cool blue-grey** (`#8b8bab` and neighbours) in a room of warm oak and beige,
  so it is the coldest thing in the frame and takes slightly more attention than a cat on a rug
  should. A 10 to 15% warm colorize on that one layer would fix it and is one line in the
  compositor. Left alone deliberately: recolouring pack art leads to hand-tuned assets, and the
  week will say whether it actually bothers anyone.
- **The card has not been posted anywhere real.** It has been checked at 1:1, in simulated light and
  dark timelines, and down a thumbnail ladder to 180px, but a local HTML page is not a timeline.
  That is the remaining Phase 2 step and it is the author's, not the compositor's.
- **Dozing and asleep are nearly identical on the pet.** Same seated pose, 2px apart, same rate.
  It is the thinnest distinction in the app and it is spec section 17 question 1. The week will
  say whether a slump is enough.
- **The card is a still of a room that moves.** Every state is a 12-frame loop and the card takes
  frame 0. Whether the comeback in particular should travel as an animation is spec section 17
  question 4, deliberately not answered in Phase 2.
