# What is left to do, and what needs you

The site is built and verified locally. `tools/verify-site.sh` reports one failure,
`/commitropolis`, which cannot be retired without an itch.io account.

Nothing here has been published. The repository has no git remote, and
`keepgoing-dev/keepgoing.dev` does not exist on GitHub.

## 1. Publish the repository

```sh
cd ~/Workspace/KeepGoing.dev/keepgoing.dev
gh repo create keepgoing-dev/keepgoing.dev --public --source=. --remote=origin --push
```

Withheld deliberately: creating a public repository is a publish, and that is yours to decide.

**Do step 2 before you push, or accept that the hub is publicly reachable with a broken card.**
Creating the Cloudflare Pages project puts the site on a shareable `*.pages.dev` URL as soon as
the repository exists, and until `/commitropolis` is built the hub's second card is four dead
links over a broken image.

## 2. Task 6: the itch.io page and `/commitropolis`

Full steps are in `docs/superpowers/plans/2026-09-13-keepgoing-dev-site-split.md`, Task 6, in
the `momentum-mascot` repository. In short:

1. Run `~/Workspace/KeepGoing.dev/commitropolis/build/Commitropolis.app`, play until the city
   has buildings and residents, and capture three screenshots. Save them as
   `assets/commitropolis/hub.png` (wide, around 1200x630), `shot-01.png`, `shot-02.png`.
2. Create the itch.io account. The **tax interview is not needed** - it blocks selling, not
   following, and a public Follow button is the whole point of publishing early.
3. Publish a public project page using those screenshots. Restricted pages have no Follow
   button for strangers.
4. Create `commitropolis.html` from the plan's template, substituting the itch URL.

**`commitropolis.html`, not `commitropolis/index.html`.** Cloudflare Pages strips the trailing
slash for `.html` files and adds one for directory indexes, so the directory form would serve at
`/commitropolis/` while the page's own canonical and the sitemap declare `/commitropolis`.
`tools/verify-site.sh` already expects the `.html` form.

## 3. Task 7: the Cloudflare cutover

New Pages project from this repository, build command none, **build output directory `/`**.
Verify on the `*.pages.dev` preview with `tools/verify-site.sh https://<preview-host>` before
touching the domain. Then move `keepgoing.dev` off the old `keepgoing` project and re-verify.

Then open Support from an installed copy of Momentum Mascot. That is the only end-to-end test of
the constraint that matters most and no script can perform it.

## 4. Task 8: retire `site/` and update App Store Connect

Only after step 3 verifies. Marketing URL becomes `https://keepgoing.dev/momentum-mascot`.
Support URL stays at the root, because shipped binaries open `keepgoing.dev/#support`.

## Three things left undone on purpose

- `404.html` prose still says "Nobody is home in this room" and "the mascot is still waiting" on
  a page that now serves the whole domain. A copy decision, not a defect.
- `privacy.html` has no header, nav or footer, and no canonical or Open Graph tags, despite being
  the URL registered with App Store Connect. Pre-existing.
- No `:focus-visible` styles anywhere in `style.css`, and no `prefers-reduced-motion` block while
  the hub puts an animated GIF above the fold. Pre-existing, more visible now.
