# LimeZu Modern Interiors: App Store distribution check

**Date:** 2026-08-22
**Verdict:** clear. A free Mac App Store listing is permitted.

Checked against the licence text shipped with the full pack, at
`$MASCOT_PACK/LICENSE.txt`. Quoted in full because it is four lines and the whole
question turns on them:

> MODERN INTERIORS FULL VERSION LICENSE
> -
> YOU CAN:
> -Edit and use the asset in any commercial  or non commercial project
> -Use the asset in any commercial  or non commercial project
> -
> YOU CAN'T:
> - Resell or distribute the asset to others
> - Edit and resell the asset to others
> -
> - Credits required (limezu.itch.io)

Why this permits the store build:

- Shipping the art composited into an application is **use in a project**, which the
  licence permits with no channel restriction. It does not distinguish App Store from
  direct download, and it does not distinguish free from paid.
- The store build **redistributes no assets**. `src/assets/` is composited at build time
  from a local licensed copy and is gitignored. Nothing standalone leaves the machine.
- The credit is present in the app (`src/index.html:42`, the credit line under the buttons)
  and in the bundle's `copyright` field (`src-tauri/tauri.conf.json:52`), which carries
  into the listing.

The one thing to keep true: if a future release ever exposes the raw sprite sheets as
files a user can extract or export, that becomes distribution and this verdict no longer
covers it.

---

# Addendum for 0.4.0: the builder ships layers, not just rooms

**Date:** 2026-09-04
**Verdict:** still clear, and closer to the line than 0.3.2 was. Read this before adding
anything to the builder.

The check above was written when every LimeZu pixel in the bundle was a finished scene. The
mascot builder changes what is in the bundle, and the change is the exact one the closing
sentence above was written to catch, so it gets answered rather than assumed.

## What is new in the bundle

`src/assets/` now also carries:

- `layers/`, about 844K: the curated Character Generator palette, cut into 16x32 frame strips
  and pre-tinted. 9 skin tones, 7 eyes, 98 hair (14 styles x 7 colours), 47 outfits (13 styles
  x 4 colours) and 42 accessories, listed in `layers/index.json`.
- `swatches/`, about 1.2M: single-frame crops of those, which is what the picker draws.
- `plates/`: the rooms split into a back and a front half so a built character can be composited
  between them at runtime.

`frontendDist` is `../src`, so all of it lands in the app's Resources and can be copied out of
the bundle by anyone who opens it in Finder.

## Why this is still use rather than distribution

- **The permission has no channel or form attached.** "Use the asset in any commercial or non
  commercial project" is what the pack grants, and a sprite layer inside an application's
  resources is the ordinary shape of that permission. Every game that ships a spritesheet is
  doing this. Reading the prohibition as covering it would mean the licence permits nothing.
- **What is prohibited is redistributing the pack as a pack.** "Resell or distribute the asset
  to others" and "Edit and resell the asset to others" both describe handing someone the
  material to use themselves. Nothing here is offered as material: it is a resource an
  application reads.
- **This is a derived, curated subset, not the library.** 42 accessories of the pack's 432, cut
  to this app's frame geometry and pre-tinted for its four moods. Someone who extracted it would
  have this app's art, not the Character Generator.
- **The app offers no export.** There is no way to get a layer, a swatch or a built strip out of
  the app: the builder writes composited PNGs into the container and nothing reads them back out
  to disk. The share card is a composited 1200x630 image with no layer in it.
- **The credit is unchanged and still present**, in the popover's credit line and in the
  bundle's `copyright` field.

## Where the reasoning is weakest, stated rather than buried

A composited room is a scene: extracting it gets you a picture of this app. A layer strip is a
part, and a part is reusable in a way a scene is not. So the honest position is that 0.3.2 was
comfortably inside the permission and 0.4.0 is inside it on the strength of the four points
above rather than self-evidently. That is a reason to hold the line where it is, not a reason
the line has moved.

## What would cross it

- **An export or "save my mascot as a PNG" feature.** This is the one to watch, because it is a
  reasonable-sounding request. A composited mascot is arguable; anything that writes a layer or
  a swatch to a user-chosen path is not.
- **Shipping the generator's full library** instead of the curated palette. The builder design
  declined this for tone reasons as well and the licence is the second reason
  (`docs/superpowers/specs/2026-09-03-mascot-builder-design.md`, section 3).
- **Putting layers or swatches on a public URL.** `site/` may carry composited rooms and cards
  only, which is why `tools/site-built-strips.sh` generates finished room strips rather than
  publishing the swatch art the builder draws. A file on keepgoing.dev is a download, and a
  download is distribution on anyone's reading.
