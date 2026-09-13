# keepgoing.dev becomes a two-product site in its own repository

**Date:** 2026-09-13
**Status:** design approved in brainstorming. Ready to plan.

**Blocks** `2026-09-13-momentum-mascot-launch-design.md`. Every link in that plan points at a
URL this document defines, and the backlinks a launch earns are permanent. The site has to be
finished and verified before the first post goes out, not during.

**Supersedes** the assumption in `docs/launch-copy.md` that "the site" and "the mascot page"
are the same page. After this, they are not.

## 1. The problem

`site/` is the Momentum Mascot's landing page, it lives inside the Momentum Mascot application
repository, and it is served at the root of a domain named after neither. That was correct while
there was one product. There are now two: Commitropolis is at v0.9.0 with a fixed ship date and
a launch spec of its own, and it has nowhere on the web that is not itch.io.

Three separate things are tangled together and each wants a different answer:

- **Brand.** keepgoing.dev is a domain for a body of work, and it currently renders as one
  product's marketing page, titled "Momentum Mascot" in the `<title>` tag.
- **Repository.** A typo on a landing page currently rides in the same history as the Rust, and
  `tools/release.sh` already does seven things without also owning web deploys.
- **URLs.** Commitropolis cannot have a page at all until one of the first two is solved.

## 2. The root page is load-bearing, and not in the usual way

A normal homepage can be rewritten freely. This one cannot, because three references to it are
already fixed in ways that cannot be recalled:

- **`src-tauri/src/tray.rs:22`** sets `SUPPORT_URL` to `https://keepgoing.dev/#support`. That is
  the Support item in the menu bar's right-click menu, shipped in 0.4.0 and in the Mac App Store
  build. It is compiled into binaries already on other people's machines. The anchor `#support`
  must keep resolving on the **root** page for as long as those builds run, which is forever.
- **`src/share.js:25`** prints the bare string `keepgoing.dev` onto the 1200x630 share card. The
  card is the growth loop. Whatever the root serves is where every person who saw a pixel room
  and typed the domain arrives, and they arrived wanting the mascot.
- **`/privacy`** is registered with App Store Connect as the Privacy Policy URL
  (`docs/app-store-listing.md:333`). It cannot move. `docs/app-store-listing.md:336` also records that
  the domain serves a page for unknown paths, so this has to be checked by content rather than
  by status code.

## 3. The decision, and what it costs

**The root becomes a hub. The mascot moves to `/momentum-mascot`.**

The case against was real and is recorded here because it will look obvious in hindsight if the
numbers disappoint: a hub converts worse than a product page, and it does so at the exact moment
traffic peaks. Launch visitors and share-card visitors both arrive wanting one specific thing,
and this design puts a click in front of it.

It wins anyway on permanence. The launch is the only time this project will earn a large number
of links at once, and those links are forever. If the mascot's page is going to be
`/momentum-mascot` eventually, then pointing the launch at the root means the best links this
project will ever get survive on a redirect and describe a page that changed meaning. Doing the
restructure afterwards costs strictly more than doing it now, and the exchange rate never
improves.

The conversion cost is mitigated by construction, not by hope: **the hub is a poster, not an
index.** One screen, the mascot card first and above the fold carrying `hero.gif`, the
Commitropolis card second. It is not a list of links with a paragraph of positioning above it.

## 4. URL map

| Path | Serves | Why it is pinned |
|---|---|---|
| `/` | Hub. Mascot card first with `hero.gif`, Commitropolis card second, `#support` section, footer | Shipped binaries open `/#support`. Share-card traffic lands here. |
| `/momentum-mascot` | Today's `site/index.html`, moved essentially verbatim, canonical updated | The launch target and the new App Store Marketing URL |
| `/momentum-mascot#support` | Same section as the hub's | Lets the App Store listing point somewhere product-specific |
| `/commitropolis` | Teaser: premise, screenshots, link to the itch.io page | No date and no promise, per section 6 |
| `/privacy` | Unchanged, at exactly this path | Registered with App Store Connect |
| `/install.sh`, `/add-ons` | 404 | Retired KeepGoing CLI. No special handling, by decision. |

`sitemap.xml` gains `/momentum-mascot` and `/commitropolis`. `robots.txt` is unchanged.

The root keeps returning 200 with different content rather than redirecting anywhere. There is
nothing to redirect *to*: the old root and the new root are both real pages, they are just
different ones.

## 5. The repository

`keepgoing-dev/keepgoing.dev`, public, static, no build step, as today.

Seed it with `git subtree split --prefix=site` rather than copying files in. The page has real
history worth keeping, including the reasons behind several of its claims, and the split is one
command. If the split proves awkward for any reason, a plain copy is an acceptable fallback and
not worth a fight.

Two rules have to travel with the files, because both currently live in the repository being
left behind and would otherwise be orphaned:

- **Composited art only.** `docs/app-store-licence-check.md:98` records that `site/` may carry
  composited rooms and cards and never layers or swatches, which is why
  `tools/site-built-strips.sh` emits finished strips. That constraint is the reason a public
  site repository is compatible with LimeZu's licence at all, and it must be written into the
  new repository's README rather than assumed.
- **The site cannot rebuild its own images.** Every GIF and PNG it serves is generated by tools
  in the mascot repository that need a licensed `$MASCOT_PACK`. The new repository holds the
  output as committed files and documents where it came from.

## 6. `/commitropolis` at mascot-launch time

A teaser: the premise, screenshots, and a link to the itch.io page. The audience capture is
**itch's own Follow button**, which notifies followers on release and costs nothing to operate.

No email signup. A list would need to be run and would need a privacy line, on a domain whose
entire pitch is that nothing phones home. The optics are worse than the list is worth.

No date, and no language that implies shipping is settled.
`commitropolis/docs/superpowers/specs/2026-09-10-launch-and-distribution-design.md` gates the
ship on three strangers returning the next day and states plainly that not launching is a real
outcome. The page must be truthful about that, and a teaser that promises delivery would make
this document the reason the project could not later walk away cheaply.

This assumes an itch.io page exists or can be put up as a draft. If it cannot, the card falls
back to the minimal placeholder: premise, one screenshot, link to the GitHub repository.

## 7. Cutover

1. New Cloudflare Pages project built from the new repository.
2. Verify everything on its `*.pages.dev` preview URL, including the checks in section 8.
3. Move the custom domain off the existing `keepgoing` Pages project
   (`.wrangler/cache/pages.json` records the account and project name).
4. Re-run the checks against `keepgoing.dev` itself.
5. Remove `site/` from the mascot repository and leave a pointer in its README.
6. Update App Store Connect: Marketing URL to `/momentum-mascot`, Support URL stays at the root.
   Both are editable without submitting a binary. The tray URL is not editable at all, which is
   why the root keeps `#support`.

This happens in Phase 0 of the launch, while nothing is pointing at the domain. Doing it with a
live thread is the one sequencing mistake in this plan that cannot be undone.

## 8. Verification

Three checks, all cheap, all of which have failed silently for someone before:

- `curl -sS https://keepgoing.dev/privacy | grep -q "<title>Privacy Policy"`. By content, not by
  status, for the reason in `docs/app-store-listing.md:336`.
- The root page contains an element with `id="support"`, and so does `/momentum-mascot`. This is
  the check that protects every shipped binary's Support menu item, and it is the regression
  most likely to go unnoticed, because nothing in the app reports a broken link.
- No file under the new repository matches the layer or swatch outputs described in
  `docs/app-store-licence-check.md:98`.

## 9. Open questions

- Whether an itch.io page for Commitropolis exists yet, which decides between the teaser in
  section 6 and its fallback.
- Whether `git subtree split` produces a clean history given `site/` was added and moved during
  the repository's life. Fallback is a plain copy.
