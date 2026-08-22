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
