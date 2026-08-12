# Asset picks - Modern Interiors

Exact source files for the room mockup. All paths are relative to:

```
~/Workspace/OneQode/projects/repos/oneqode-pixel-assets/moderninteriors-win/
```

Everything below comes from the **full paid pack** only. Nothing from `Modern tiles_Free/`
(non-commercial licence). Credit `limezu.itch.io` is required in the shipped app.

## Room geometry

- Composed on the **16x16 native grid**.
- Room is **160x112** source pixels (10 by 7 tiles).
- Wall band occupies y 0-33, floor y 34-111.

Note: this is larger than the 144x96 in earlier drafts. At 2x that is 320x224, which no longer
fits a 320px popover with padding. Resolved in favour of the larger room: `spec-v2.md` now
specifies 160x112 with the popover widened from 320px to 352px.

## Props

| Piece | File | Size | Placed at |
|---|---|---|---|
| Bookshelf | `1_Interiors/16x16/Theme_Sorter_Singles/5_Classroom_and_Library_Singles/Classroom_and_Library_Singles_60.png` | 32x48 | +0+6 |
| Bed (vertical) | `1_Interiors/16x16/Theme_Sorter_Singles/4_Bedroom_Singles/Bedroom_Singles_229.png` | 32x48 | +34+18 |
| World map | `1_Interiors/16x16/Theme_Sorter_Singles/5_Classroom_and_Library_Singles/Classroom_and_Library_Singles_31.png` | 32x32 | +82+2 |
| Floor lamp | `1_Interiors/16x16/Theme_Sorter_Singles/2_Living_Room_Singles/Living_Room_Singles_81.png` | 16x48 | +68+24 |
| Desk | `1_Interiors/16x16/Theme_Sorter_Singles/5_Classroom_and_Library_Singles/Classroom_and_Library_Singles_25.png` | 32x32 | +114+24 |
| Monitor | `1_Interiors/16x16/Theme_Sorter_Singles/2_Living_Room_Singles/Living_Room_Singles_8.png` | 16x16 | +116+28 |
| Potted plant | `1_Interiors/16x16/Theme_Sorter_Singles/2_Living_Room_Singles/Living_Room_Singles_17.png` | 16x48 | +2+64 |
| Rug | `1_Interiors/16x16/Theme_Sorter/1_Generic_16x16.png` crop `54x34+149+70` | 54x34 | +46+74 |
| Floor tile | `6_Home_Designs/Generic_Home_Designs/16x16/Generic_Home_1_preview_16x16.png` crop `16x16+96+56` | 16x16 | tiled |
| Wall column | same file, crop `1x34+45+0` | 1x34 | tiled |

Floor and wall are sampled from an assembled home because they are guaranteed to be a valid
combination. Both should be replaced with a deliberate choice from
`1_Interiors/16x16/Room_Builder_subfiles/Room_Builder_Floors_16x16.png` and
`Room_Builder_Walls_16x16.png` (about 20 wallpaper styles) once the palette is decided.

## Characters

Shortlisted: **07**, **12**, **20**.

```
2_Characters/Character_Generator/0_Premade_Characters/16x16/Premade_Character_07.png
                                                          /Premade_Character_12.png
                                                          /Premade_Character_20.png
```

Each sheet is 896x656, a 56 by 41 grid of 16px cells, and **every premade character carries the
identical animation set**. That is why supporting three characters costs three PNGs rather than
three sets of rooms.

Frame used for the idle pose: `-crop 16x32+0+0`.

## The sleep animation is layered

Found on each character sheet around `+100+84`, labelled `sleep` in the sheet itself. It is not a
single sprite. It composes as:

1. a **vertical (top-down) bed** base with pillow
2. the character's **bare head** sprite
3. a **blanket** overlay drawn on top

This is why the room needs a vertical bed such as `Bedroom_Singles_229`. A side-view bed has no
sleeping pose to go with it, and the side-view beds (`Bedroom_Singles_1` through `~96`) are for
placing against a side wall only.

Consequence for the architecture: the character must be a **separate layer composited over the
room**, not baked into the room image. That also makes character selection nearly free.

## The four states

Rendered in `states-four.png`. Same room every time. Only three things change: **where the
character is, how the room is lit, and which emote is showing.**

| State | Trigger | Character | Lighting | Emote |
|---|---|---|---|---|
| Awake | latest commit < 24h | seated behind the desk, monitor on | normal | none |
| Dozing | 24h to 72h | standing away from the desk | 10% blue tint | `Z` |
| Asleep | >= 72h | in bed under the blanket | 34% blue tint | `Z` |
| Comeback | asleep -> awake | out of bed, on the rug | +13% brightness, +20% saturation | `!` plus two sparkles |

Character frames, all from the same premade sheet:

| Frame | Crop from `Premade_Character_NN.png` |
|---|---|
| Idle (standing, facing down) | `16x32+0+0` |
| Seated | `16x32+0+192` |
| Sleeping (head plus blanket overlay) | `16x32+191+96` |

The sleeping frame is an **overlay**, drawn on top of the bed at `+8+4` relative to the bed's
top-left. It carries the character's head and a blanket, so it composites onto any bed.

Emotes, all from `4_User_Interface_Elements/UI_thinking_emotes_animation_16x16.png`:

| Emote | Crop |
|---|---|
| `Z` (sleep) | `16x16+96+80` |
| `!` (comeback) | `16x16+0+80` |
| sparkle | `16x16+64+96` |

## What actually animates

The room is a static background. Only two layers move, which keeps the whole thing within the
spec's "2 to 4 frames at 2 to 6 fps" budget:

1. **The character**, a few frames from its row on the sheet.
2. **The emote**, which is already a 2-frame loop. The pack's own note on the emote sheet reads
   "sample animation, just swap the last 2", so the bubble pops in and the icon alternates.

Lighting is a flat colour multiply over the composed frame, not per-pixel art, so it costs nothing
and can be tweaked without redrawing.

## Object catalogues

Rendered contact sheets with index numbers, for picking replacements without hunting through the
48,000 files:

- `objects-conference.png` - 68 objects (office chairs 37-39, posters 54-55, counters 25-32)
- `objects-classroom.png` - 75 objects (desks 25-26 and 49-52, bookshelves 55-75, map 31, globe 34-35)
- `objects-livingroom.png` - 122 objects (screens 3-8, plants 13-18, lamps 71-88, cabinets 37-44)
- `bed-options-vertical.png` - vertical beds 217-241, bunk beds 131-139

To regenerate one for another theme, montage the folder under
`1_Interiors/16x16/Theme_Sorter_Singles/<theme>/` with each file labelled by its trailing number.

## Open

- Wallpaper and floor: currently sampled placeholders, not chosen.
- Rug: the red/gold one is loud. Other rugs sit in `1_Generic_16x16.png` around y 64-104.
- The desk is a school desk with a book on it. There is no dedicated computer desk anywhere in the
  pack, so the workstation is desk + separate monitor sprite.
