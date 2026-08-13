# Preview harnesses

Two pages for looking at the frontend without running the app, which matters on a machine
where the app itself cannot be screenshotted.

They live here rather than in `src/` for one reason: `frontendDist` is `../src`, so anything
in that folder is compiled into the shipped binary. A `<base href="/src/">` tag is what lets
them sit outside it and still resolve every relative path the way the real pages do.

```sh
python3 -m http.server 8731        # from the repository root, not from src/
open http://localhost:8731/tools/preview/card.html
open http://localhost:8731/tools/preview/popover.html?state=asleep
```

**`card.html`** composes the share card exactly as Share Status does, then diffs it against
`docs/mockups/share-comeback-1200x630.png`, the card `tools/compose-share.sh` produces. It
reports the number of differing pixels, split into the room, the label, and everything else.

The number to expect is **zero**, and it is worth knowing why that is achievable rather than
merely nice. The canvas draws the room at exactly 5x with smoothing off, and it renders type
by drawing at the font's native 11px cell, thresholding the alpha to one bit, and scaling by a
whole number. That last step is the canvas equivalent of ImageMagick's `+antialias`, which is
not optional: an antialiased pixel font is just a blurry font. When the two agree to the
pixel, the app is provably shipping the composition that was designed and looked at in Phase 2
rather than an approximation of it.

It also caught a real bug, which is the argument for keeping it. The card composed silently in
a fallback serif when called from a page that had not linked the stylesheet holding the
`@font-face` rule: hard-edged, correctly laid out, entirely the wrong typeface, and no error
anywhere. `share.js` now registers the font itself and refuses to compose if the advance is
not 7px on an 11px cell.

**`popover.html?state=<mood>`** renders the real popover markup, stylesheet and script against
a stubbed backend. One state per load, deliberately: `popover.js` binds to element ids, so four
panels in one document would all be driven by whichever `getElementById` returned first, and
the screenshot would quietly show one state four times. That was found by looking at exactly
that mistake.

The stub implements only what `popover.js` reaches for. If that list grows, the popover has
grown a dependency on the backend that it probably should not have.
