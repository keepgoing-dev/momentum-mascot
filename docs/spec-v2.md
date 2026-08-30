# Design Specification: KeepGoing Momentum Mascot

**Status:** approved design, pre-implementation
**Supersedes:** `docs/initial-spec.md` (kept unchanged for side-by-side comparison)
**Date:** 2026-08-12

---

## 1. Purpose and Success Criteria

KeepGoing Momentum Mascot is a small retro pixel character who lives in a tiny room in your system tray and reacts to whether your side projects are moving. It reads commit and working-tree activity from a handful of local git repositories you explicitly point it at, turns that into a mood, and shows you the character in that mood.

That is the whole product.

It is **not a productivity tool**. It will not help anyone write code faster, ship sooner, or plan better. It does not measure output, it does not score you, and it does not try to change your behaviour. It is a companion that sits beside a side project and is visibly happy when the project is alive.

**Target user:** the author, and developers like him. People who love side projects, have demanding day jobs, and go through long stretches where nothing gets committed because life is happening. The product must feel good to that person specifically, including during the stretches where they do nothing.

**Success:** users who love it, keep it running, and tell someone else unprompted. Screenshots in the wild. The author still running it six months after v1. **Not success:** revenue, install counts, or daily active use. A user who opens the popover once a week and smiles is a complete success.

If a community forms around this, that community may one day become something worth supporting financially. That is the only revenue path ever contemplated, and it is a consequence of the thing being loved, never a design input. The tool itself is not for sale, now or later.

**Why this is a reset:** the previous KeepGoing ecosystem (preserved at `../keepgoing-deprecated/`) grew to 12 applications, 4 shared packages, and 4 languages, carrying a licensing system, paid add-ons, and two internal marketing tools. It chased users and revenue simultaneously and drifted off-mission doing it. Anti-scope-creep is therefore a first-class design requirement here, on the same footing as the art.

---

## 2. Non-Goals

Permanent product positions, not v1 deferrals:

- **No monetization.** No pricing, licensing, tiers, add-ons, or upsell surface.
- **No accounts.** No sign-in, no identity, no user record anywhere.
- **No telemetry.** No analytics, crash reporting, usage pings, or "anonymous" counters.
- **No network calls.** Zero outbound requests at runtime. This is verifiable and should stay verifiable.
- **No behavioural nudging.** No streak pressure, no reminders to commit, no "you haven't committed in N days".
- **No productivity claims** anywhere in the copy, README, or landing surface.
- **No hosted component.** No sync, no server, no shared state between machines.

---

## 3. What v1 Is

> **Scope guardrail: if it needs a second process, a second language, or a settings screen, it is not in v1.**

v1 is one Tauri application with a tray icon and one popover. It ships exactly this:

1. A pixel character in a tiny room, in four states, driven by project activity.
2. A **desktop pet**: a small always-on-top character in the corner of the screen, showing the same state. This is the primary ambient surface and the primary way into the app.
3. A choice of three characters, cycled by clicking the character.
4. A tray icon that opens the popover and holds Quit.
5. A popover containing the room, a quote line, the list of tracked projects with relative times, and two buttons.
6. **Add Project**, a native folder picker that validates and tracks a git repository.
7. **Share Status**, which renders the room to an image and copies it to the clipboard.
8. A single local JSON state file.

Everything else is out. Explicitly:

| Not in v1 | Why |
| --- | --- |
| Settings screen | Every setting is a scope multiplier and a support surface. v1 ships opinionated defaults. |
| A second window **for UI chrome** (about, onboarding, stats, settings) | Chrome earns nothing. Note that the desktop pet **is** a second window, and it is in scope: it is the product's primary ambient surface, not chrome. This ban is narrow on purpose. |
| Stats, charts, history, streak counters | This is scoring, and scoring is the failure mode being avoided. |
| Accounts, sync, cloud anything | See non-goals. |
| Notifications (OS banners, sounds) | The tray icon is the only ambient signal. A banner that fires when you have not committed is guilt-ware. |
| CLI binary, IPC, custom URL scheme | Deleted from the design. See section 10.2. |
| Git hooks in user repositories | Rejected. See section 9.3. |
| Auto-update | Deferred until there is a second release worth shipping. |
| Light theme, skins, per-character rooms or copy | Each implies a setting or a second art pipeline. |
| A character picker UI, or more than three characters | Cycling on click needs no UI. More characters are assets, addable later without a spec. |

Anything on that table that becomes genuinely necessary gets its own spec and its own argument. Nothing arrives by accident during implementation.

---

## 4. The Character and the Room

**The character is the product.** The git tracking exists for one reason: to give the character a reason to feel something. Roughly 90% of the value here is the art, the room, and the personality. The remaining 10% is a file watcher and a timestamp comparison. This ordering is deliberate and inverts the previous spec, which opened with an architecture diagram and treated the mascot as a rendering detail. If the character is not lovable, a perfect state machine is worthless.

### 4.1 Art direction

The art is built from **LimeZu's Modern Interiors**, the full paid pack, for which the author holds a commercial license (section 4.2). This direction has been built and tested against the real pack rather than assumed. **The exact source file for every sprite, with crop coordinates and placements, lives in `docs/asset-picks.md`.** That manifest is the reference for pixel coordinates; this section carries only the decisions and constraints, and the two must not be duplicated into each other.

**The character lives in a tiny room.** Modern Interiors is an interiors pack, so the furniture is its strength, and the room is the concept rather than a backdrop. A scene carries story that a lone sprite cannot: someone asleep in bed across the room from a desk that still has their work on it says something a curled-up character on a blank background cannot. It also makes the share image substantially more compelling, which matters directly, because the share image is the growth mechanism (section 5).

**No original character art is required.** The pack ships every animation the four states need, including a `sleep` row, which is why Phase 1 is composition rather than drawing.

**Grid and geometry.** The pack ships everything pre-scaled at 16x16 (native), 32x32, and 48x48, and its own guidance is to pick one size and stay on it. **Compose at the 16x16 native grid and do all scaling in CSS at integer factors.**

- The room is **10 by 7 tiles, so 160x112 source pixels**. This grew from an earlier 9 by 6 because at that size the room read as empty once real furniture was placed. The extra row and column is what fixed it, which is a tested finding rather than a preference.
- **Popover: 2x, so 320x224.** This is why the popover is 352px wide rather than 320px (section 6.2).
- **Share image: 5x, so 800x560**, letterboxed inside the 1200x630 canvas per section 5.2.

Because the share image overlays text on the room, each room must be composed leaving the **upper-left wall area** and a **strip along the bottom** quiet enough to carry type.

**Seven constraints discovered by building it.** These are findings from the real pack, not preferences, and they bind the implementation:

1. **The bed must be a vertical (top-down) single.** The `sleep` animation is not a single sprite. It is a layered recipe: a vertical bed, plus the character's head, plus the bed's own blanket. Side-view beds have no sleeping pose at all, so choosing one would force drawing the single thing this pack was chosen to avoid. **Single, not double**, because the sleeping head is 16px wide: on a 32-wide double bed it reads as a doll dropped on the covers. Vertical singles are `Bedroom_Singles_140` through `189`.
2. **The character is a separate composited layer**, never baked into the room image. This follows directly from the layered sleep animation, and it is what makes character selection cheap (section 6.3) and the desktop pet possible at all (section 6.1).
3. **The pack contains no computer desk, but it does contain a computer.** There are school desks, reception counters, and dining tables, and no monitor sprite in any theme sheet. The one piece of computer art in the pack is inside `animated_receptionist`: a grey tower seen **from behind**, on a counter, with the user's head above it. Cropping it out is the workstation, and it is what resolves the perspective problem rather than working around it. A top-down desk and a front-facing sprite cannot both be right about which way a screen points unless the screen points away from the viewer.
4. **Draw order is part of the art, not an implementation detail.** The character is composited *before* the desk, so the desk covers their lower body. That single occlusion is the whole difference between reading as sitting at the desk and standing behind it.
5. **A wide prop ships as a body plus a separate cap, and nothing in the filename says so.** `Classroom_and_Library_Singles_60` is 32x48 and looks like a whole bookcase in a file listing. It is not: it is the left two-thirds of a 48-wide four-bay one, cut off mid-shelf with no right-hand frame and no right leg. `Singles_61` is the missing 16x48 cap. Used alone, the shelf loses a corner, which is exactly what it looks like in the room. The only way to tell a body from a whole prop is to butt the next number against it and look, so **every multi-tile prop gets checked for a cap before it is placed.**
6. **Row 0 of a character sheet is one pose per facing, not an idle animation.** The order is left, up, right, down, so `x=0` is a **left-facing side view**: at 16px that is a hat with a sliver of cheek under it and no readable person. The standing character in dozing and comeback was that sprite for the whole first pass. The front-facing pose is `x=48`. This is checkable rather than a matter of taste: flopping `x=0` reproduces `x=32` to within four pixels, which is what proves those two are the side pair.
7. **A crop cannot be eyeballed, because the pack pads sprites with the outline colour at alpha zero.** `#3a3a5000` sits directly against `#3a3a50ff` around the rug, so against any dark background the padding and the real edge are the same colour and a crop that looks correct can still be four rows short. Trimming does not rescue it either, since the theme sheets are dense enough that any hand-drawn box catches the neighbouring sprite. **Crop bounds are read off the pixels**, by dumping the region as text and finding where alpha changes. This is the constraint that applies to every crop in the manifest rather than to one sprite, and it is why constraints 5 and 6 were both possible in the first place.

**Animation. Every state moves, and amplitude is what separates them.** An earlier draft had the background static with only the character and the emote animating, on the grounds that motion is what makes an ambient thing intolerable. That reasoning is right about **amplitude** and wrong about **presence**. A room in which nothing at all moves reads as a screenshot, and a screenshot is not a companion; the thing has to look like it is still running even when the person is not.

So every state is a **12-frame loop**, and the state is carried by rate and amplitude rather than by whether anything is alive:

| State | Rate | What moves |
| --- | --- | --- |
| **Awake** | 6 fps | the seated typing loop, the computer, the cat's tail |
| **Dozing** | 3 fps | the coffee steam, the `Z`, a 1px breath, the cat's tail |
| **Asleep** | 2 fps | breathing under the blanket, the `Z`, the cat's tail |
| **Comeback** | 8 fps | a 2px hop, the `!`, two sparkles, the cat's tail |

Four things about this are load-bearing:

- **Twelve is chosen arithmetically, not aesthetically.** It is divisible by every layer's own frame count: 12 for the cat, 6 for the character and the coffee, 3 for the computer, 2 for the emotes. Each layer is indexed `frame % count`, so every cycle closes in the same place and the loop has no seam. Any other loop length needs the layer counts rechecked.
- **The pack's own annotations are the frame tables.** The seated row has `4-9 loop` written next to it in pixels, and the emote sheet reads "sample animation, just swap the last 2". Both are correct and both were found by looking at the sheet rather than guessing. The sleep row and the seated row are each palindromes: four frames differing from rest by a handful of pixels, two differing by a couple of hundred, then back. That shape is breathing and leaning in respectively.
- **A standing pose has no breathing frames on the sheet, so the vertical offset supplies them.** One pixel is a breath, two is a hop. Both read at this scale because the character is only 32px tall. This is composition, not new art.
- **It ships as one horizontal strip per state**, twelve frames wide, stepped with a CSS `steps(12)` animation. No animation library and no runtime canvas loop, which is the constraint section 10 already imposes.

The peripheral-vision concern that produced the original rule is real and it has not been dropped. It applies to the **pet**, which sits in the corner of the eye all day, rather than to the room, which is looked at deliberately in the popover and in the share image. Section 6.1 carries the pet's version of the rule.

### 4.2 Asset licensing

The license is settled, not an open question. `moderninteriors-win/LICENSE.txt` in the full pack states verbatim that you **can** "edit and use the asset in any commercial or non commercial project", **cannot** "resell or distribute the asset to others" or "edit and resell the asset to others", and that **credits are required (limezu.itch.io)**.

Three consequences, all binding:

1. **Use is unambiguously permitted**, commercial or not. Nothing about this product's use of the pack needs further clearance.
2. **Attribution is mandatory and is therefore a functional requirement**, not a courtesy. With no about window and no settings screen, it goes in three places: a small credit line at the bottom of the popover (`art: limezu.itch.io`), a line in the share image (section 5.2), and a credit in the README. The share image carries it because that is the artifact that travels.
3. **Only the full pack may be used.** The same asset repository also contains `Modern tiles_Free/`, whose license is **non-commercial only** and explicitly forbids commercial use even of edited sprites. Assets must come from `moderninteriors-win/` exclusively. Mixing the two would silently breach the license, so this is named here as a trap to avoid rather than left to be noticed later.

**Redistribution policy.** Raw pack assets are never committed to a public repository. If `momentum-mascot` is ever published, composed room scenes and sprite sheets stay out of version control and are composited in at build time from a local licensed copy. Shipping them compiled into a distributed binary is ordinary permitted use and needs no special handling. This mirrors the policy already applied in the assets repository, where the packs are gitignored and its README notes that extracted output should also be ignored if that repo ever goes public.

This policy is already in force here: **`docs/mockups/` is gitignored**, because the mockups are derived LimeZu art and are therefore covered by the same restriction. The manifest is deliberately **not** in that folder: `docs/asset-picks.md` names source files and coordinates rather than shipping any art, so it is committed, and Phase 1 is unreproducible without it. Anyone picking this project up gets the manifest but needs a licensed copy of the pack to regenerate the mockups, which is the correct outcome.

### 4.3 The mascot never dies. It waits.

The previous spec had a `Dead` state at 72 hours: "mascot is dead/ghost/crying". Rejected outright, for two reasons.

1. It punishes the exact user this targets for the exact thing that defines them. A developer with a demanding job will hit 72 hours constantly. Telling that person their companion died is telling them they failed at a hobby.
2. Guilt-ware gets uninstalled. The emotional low point of a tool is the moment a user decides whether to keep it. If the low point is a corpse, they quit. If the low point is someone asleep in a warm room still holding your place, they keep it, and they come back.

Every piece of copy passes one test: **would this line make a tired person feel worse about themselves?** If yes, it gets rewritten.

### 4.4 States

State derives from a single number: the most recent real activity across all non-operating tracked projects (sections 7 and 9). Not per project, not averaged, not weighted. Any real work anywhere counts. Operating projects are excluded; if none are left to evaluate, the mascot is awake by default.

**The room never changes.** It is one static background, and exactly three variables move on top of it:

1. **Character position**, moving desk to standing to bed to rug. This single axis carries the story, because position in a room reads as intent with no explanation needed.
2. **Lighting**, a flat colour multiply over the finished frame rather than redrawn art. Retuning a mood is one number.
3. **Emote**, one of `...`, `Z` or `!`. One per state, never shared between two of them: the emote is the state's name on the pet (section 6.1), so reusing one merges the states it is attached to.

| State | Trigger | Character | Lighting | Emote |
| --- | --- | --- | --- | --- |
| **Awake / hyped** | latest real activity < 24h | seated behind the desk, monitor on | normal | none |
| **Dozing** | 24h to 72h | standing, away from the desk | 10% blue tint | `Z` |
| **Asleep / dreaming** | >= 72h | in bed under the blanket | 34% blue tint | `Z` |
| **Comeback** | asleep to awake | out of bed, on the rug | +13% brightness, +20% saturation | `!` plus two sparkles |

The payoff is worth stating plainly: **four states cost four character frames plus two emotes, not four illustrated scenes.** This is the whole reason the room concept is affordable for one person working evenings.

Keeping the room identical across states does real work here: the asleep frame is the same intact room with the same desk and the same work still on it, only dimmer and with someone sleeping in it. That says *the project still exists and is still waiting*, which is the opposite of "you killed it". The asleep room must read as cosy, never abandoned. No dust, no cobwebs, no wilting plants.

**There is a cat, and it is not a fourth variable.** `animated_cat` lies on the rug with its tail flicking, identical in all four states, taking only the state's lighting. It was deliberately left out of the first pass on the argument that the room should earn its warmth without it. That was the right order to work in and the wrong conclusion to keep. The asleep room has to read as cosy rather than abandoned, and one living animal in it does more for that than any amount of furniture: a room with a sleeping person and a cat is a home, and a room with a sleeping person alone is closer to a hospital. It is also the cheapest thing in the room, being one sprite with no state behaviour attached to it at all.

There is no state beyond 72 hours. A project untouched for a year shows the same peacefully sleeping room as one untouched for four days. Escalating past asleep would reintroduce guilt through the back door.

### 4.5 The comeback state

This is the emotional payload of the entire product. Everything else is setup. The character dozing off over three days is the loaded spring; the moment a commit lands after a long silence and they leap out of bed is the release. It is the moment most likely to be screenshotted and most likely to make someone feel something about a piece of software. **Design it first and design it hardest.**

**Trigger:** derived state transitions from `asleep` to `awake`. A transition from `dozing` to `awake` does not trigger it. The user has to have been gone long enough for the return to mean something. Because only real commits and working-tree edits move the state (sections 9.1 and 9.4), the celebration cannot fire for someone who merely checked out a branch after three weeks away.

**The desktop pet solves the hardest problem in this design.** Earlier drafts had to accept that a popover-only product cannot guarantee the celebration is ever seen, and settled for "a celebration nobody attended is not a debt". That compromise is no longer necessary. The pet (section 6.1) is already on screen, so the comeback plays out in the user's peripheral vision at the moment it happens, with no notification banner and no click required. The one thing this design previously could not deliver, it now delivers by default, and the pet is the reason.

**Duration and resolution:**

- The pet celebrates **the moment the commit lands**. This is the real delivery of the moment, and it needs nothing from the user.
- The state persists **until the user opens the popover**, capped at **30 minutes**, whichever comes first. Opening the popover is the resolution: the user sees the full-room celebration, and on close it settles into `awake`.
- If the cap expires without the popover being opened, it settles into `awake` silently. No badge, no "you missed it", no queued notification. The pet has already done the emotional work, so nothing is owed.

**Restart safety:** the last displayed state is persisted, so a comeback still fires if the app was quit while asleep and relaunched after a commit landed. This matters because a plausible real sequence is: open laptop, app launches, commit, and the app must not miss the transition by being freshly started.

The popover remains the screenshottable version, because the room is the composed scene and the pet is only the character. The pet delivers the moment; the popover is where the user goes to look at it properly.

### 4.6 Tone and example copy

**Voice:** warm, encouraging, a little snarky, never guilt-inducing. The character is on your side without exception, is allowed to be funny about themselves, and is never funny at the user's expense.

Each state carries a short quote line in the popover, drawn from a small hardcoded pool and rotated so repeat views are not identical.

**Awake / hyped**
- "Look at you go."
- "Something moved today. That counts."
- "I saw that commit. I'm telling everyone."
- "Certified in motion."

**Dozing**
- "Still warm. I've got the seat."
- "Taking five. Same here."
- "Day off? Good. Rest is part of it."
- "No rush. It'll keep."

**Asleep / dreaming**
- "Dreaming about that thing you're building."
- "Sleeping, not gone. Wake me whenever."
- "I'll hold your place. However long it takes."
- "Zzz. The project's still there. So am I."

**Comeback**
- "YOU CAME BACK."
- "I KNEW IT."
- "Woke up for this. Worth it."
- "Best day. Objectively."

**Banned patterns:** elapsed-time shaming ("12 days since your last commit"), comparative framing ("you used to commit more"), pleading, and any second-person accusation. Relative timestamps in the project list are factual and permitted; the character never comments on them.

---

## 5. The Share Artifact

**There are two artifacts, and an earlier draft of this section collapsed them into one.** That was the mistake, and it survived until the card was actually built and looked at in a simulated timeline.

| | **The demo** (section 5.4) | **The card** (sections 5.1 to 5.3) |
| --- | --- | --- |
| Made by | the author, once | every user, repeatedly |
| Job | **explain the product** | **carry the mood, and the URL** |
| Shows | the transition, over time | one state, at one instant |
| Requires | the working app | the room art |
| Signal it gives | none, beyond its own reception | share volume, which is the honest one |

The critique that forced the split is simple and correct: **a single card explains nothing.** Post `Still warm. I've got the seat.` over a picture of a pixel room to someone who has never heard of this tool and they have no way to work out what it is. A still frame cannot show a transition, and this product *is* a transition. Section 4.5 already says the comeback is the emotional payload of the entire product and everything else is setup, so an artifact that can only ever show one frame was never going to be the thing that explains it.

- **Discovery is the demo's job.** No marketing budget, no ads, no network calls, so the only way anyone hears about this is someone posting it. What they need to see first is three days of silence compressed into a few seconds and then the character leaping out of bed. That is not a picture. It is a screen recording, and it needs the app to exist.
- **Growth after discovery is the card's job.** Once someone knows what KeepGoing is, a card of a sleeping room reads instantly, because **the context comes from the product being known, not from the image carrying it.** The card is a mood post between people who are already in on it, and it puts `keepgoing.dev` in front of the ones who are not.
- **Validation is still the card's job alone.** If users post cards, the art landed. A video the author posts once is a claim; a thousand cards posted by users is evidence. That is why the card is still worth having designed, and why share volume remains the only honest signal available without telemetry.

The consequence for the plan is real and is recorded in section 12: the card is finished as a **feature**, but it cannot be **validated** until the demo has created the context it depends on, and the demo cannot be made until the app runs. The dependency runs app, then demo, then context, then card. Designing the card before the app was still right, because the app has to render it either way, and every finding in section 5.2 would otherwise have surfaced as rework mid-implementation.

### 5.1 Behaviour

One press renders the current room to an image and copies it to the system clipboard. No dialog, no file save, no preview window; a brief inline "Copied." appears for about two seconds. It is generated in the webview by drawing to an offscreen `<canvas>` with image smoothing disabled, then `toBlob()` into the clipboard. Nothing is written to disk and nothing leaves the machine.

### 5.2 The artifact

There is a real tension here, worth stating rather than fudging. A room composed on a tile grid is roughly 10:7, while the social preview crop is roughly 1.91:1. **The resolution is to letterbox the integer-scaled room inside the wider canvas on the panel background, never to scale the room fractionally to fill it.** Integer scaling is non-negotiable; a half-pixel room is worse than a mat.

- **Canvas: 1200x630**, the standard social card size, filled with the dark panel background. `#191924`, which is the pack's own outline colour `#3a3a50` darkened, so the mat is family with the art rather than a generic dark grey.
- **Room: 800x560**, the 160x112 room at 5x integer scale, centred horizontally and sitting 12px below the top edge, inside a 5px mount line in `#3a3a50`. This leaves 200px bars either side and a 58px band along the bottom. The bars read as a mat around a framed picture, which suits pixel art. Only the room needs integer scaling; the surrounding background is flat colour, so the canvas is free to be any size.
- **State label:** a pixel-font word on the room's upper-left wall, at 55px, which is 5x the font's 11px cell and therefore **the same pixel unit as the art it sits on**: `AWAKE`, `DOZING`, `DREAMING`, or `BACK!!!`. It is **outlined on all four sides, not drop-shadowed.** The wall behind it runs from full-brightness beige when awake to dimmed grey-blue when asleep, and no single fill colour survives both ends of that range; an outline makes the label independent of what is behind it, so one colour rule covers all four states instead of four. The fill carries the state's temperature, warm gold awake through to near-white asleep and hottest on the comeback.
- **Quote line:** the same line currently shown in the popover, at 22px, **in the footer band and not on the room.** This corrects an earlier instruction to overlay it "along the room's lower strip", which the built room cannot accommodate: the plant, the rug's gold-and-navy edge, and the cat fill the bottom twelve rows, and the upper wall is already carrying the label. See the findings below.
- **Footer band:** one baseline across the full 1200px, reading `art: limezu.itch.io` at 11px in the left mat, the quote at 22px in the room's column, and `keepgoing.dev` at 22px right-aligned to the canvas edge. **The wordmark and the URL are the same string**, which is what buys the wordmark the size it needs: at 11px it was invisible in a timeline thumbnail, and a growth mechanism whose name nobody can read is not one. The credit is a license requirement (section 4.2) and is the smallest thing on the card, which is the correct weight for it: present and legible at full size, never competing.

**Three findings from building it**, in the same spirit as section 4.1.

1. **The band is the binding constraint, and it is 8px short.** A 5x room in a 630px canvas leaves 70px of vertical slack. A 22px quote plus a meta row plus workable margins needs 78px. The resolution is that the band spans the **full canvas width** rather than only the room's 800px column, so the credit and URL move out into the side mats, which were dead space in every candidate, and the quote gets the band's whole height to itself. The alternative was shrinking the room to 4x, and the room is the entire point of the card.
2. **The type cannot all share the room's pixel unit, and should not.** The label can, because it sits on the art, and it does. The quote cannot: at 5x the longest line in the pool is 1472px wide, and even wrapped it would cover a third of the room. So the quote is at 2x. This is not a compromise once the quote is on the mat rather than on the picture, because a caption is allowed to be finer than the thing it captions. The rule that comes out of it: **anything drawn on the room matches the room's unit; anything drawn on the mat is free.**
3. **The room's colour temperature, not the label, is what identifies the state at thumbnail size.** Tested by downscaling with a smooth resample, the way platforms actually re-encode. At 504px everything reads. At 280px the label still reads and the quote is at its limit. At 180px all the type is gone and the state is *still* unmistakable, because the warm room and the cool room do not look alike. That is a direct validation of section 4.4's decision to carry lighting as a flat colour multiply: the cheapest variable in the design turns out to be the one that survives the most compression.

### 5.3 Privacy

**The v1 share image never contains project names, repository paths, commit messages, hashes, or timestamps.** Not behind a toggle, not opt-in, not by default off.

A user sharing a picture of a happy pixel room should not have to audit it for their employer's repo name or their home directory path before posting. The failure mode is silent and irreversible, and the image loses nothing by omitting it. It communicates a *mood*, not a *report*.

**Section 5.4's demo is held to the same rule and it is harder there**, because a screen recording captures whatever is on screen. The demo is recorded against a throwaway repository with a deliberately boring name, on a clean desktop, and the recording is watched back before posting specifically to look for a leaked path, a browser tab title, or a notification banner. A privacy rule that only covers the artifact the code generates is not a privacy rule.

### 5.4 The demo

**The artifact that explains the product, made once, by the author.** A short screen recording, posted at launch and reusable as the top of the README and the landing page. It is not a feature, it takes no code beyond the debug clock below, and it is the thing most likely to decide whether anyone ever installs this.

**What it has to show, in order:** a commit landing and the pet waking; time passing and the pet dozing; more time passing and the room going to sleep; then a commit landing after the silence, and the comeback. That is the entire product, and it is legible without a single word of voiceover or caption. Roughly 20 to 30 seconds. The comeback is the payoff shot and gets the most screen time.

**It has to be a timelapse, because the real durations are 24 and 72 hours.** This is where the demo stops being a marketing task and becomes a spec item: it needs the clock to run fast.

**The debug clock is nearly free, and it is needed twice.** Section 8.1 already specifies state derivation as a pure function of the tracked timestamps and the current time. Feeding that function a scaled clock is exactly what a pure function is for, so time acceleration is a parameter rather than a feature: one injectable clock behind a debug-only environment variable, off and inert in a release build, never a setting and never in the UI.

The second reason matters more than the demo does. Section 18 requires that the awake to dozing to asleep transitions be verified **from time passing alone, without restarting the app**. Waiting three days per test run is not verification, it is hoping. Without an accelerated clock that requirement is untestable in practice, and an untestable line on a definition of done quietly becomes an unchecked one. So the demo and the test suite converge on the same small piece of plumbing, which is the cheapest kind of feature there is.

**`tools/drive-states.sh` is that check made repeatable**, and it exists because the alternative is remembering a sequence. It builds a throwaway repository, seeds a throwaway state file at it so the folder picker is not in the way, prints the schedule it expects, launches the app, and commits at the moment that puts the comeback in front of you. The whole arc takes about two minutes at the default 3600x, and it prints its transitions next to a pet you can watch while they happen. Two details in it are load-bearing rather than incidental:

- **The anchor commit is made immediately before launch, not during setup.** Every timestamp entering the app is mapped onto the accelerated timeline, so a commit is aged by the scale factor as much as anything else: at 3600x, three seconds spent writing files during setup makes the commit three hours old before the app starts, and at a high enough scale the run opens in `dozing` and awake is never seen at all.
- **The dwell before the comeback commit is thirty real seconds, not a number of simulated hours.** What that number controls is how long a person gets to look at the asleep frame, and that is wall-clock time whatever the clock is doing. It is the same split the comeback's own cap makes (section 8.1), reappearing in the test harness for exactly the same reason.

**Recorded with the real state machine, not a mock.** The clock is scaled and nothing else is faked: real commits into a real throwaway repository, the real watcher, the real derivation. A demo of a mocked path is a lie that also fails to test anything, and the point of recording it against the real thing is that making the demo *is* the integration test.

---

## 6. UI Surfaces

There are **four surfaces**, and each has one job. Naming them up front matters, because the earlier design put nearly all of the art behind a click, and that is a lot of craft for a popover opened twice a day.

| Surface | Seen | Job |
| --- | --- | --- |
| **Desktop pet** | always | ambient state, in peripheral vision |
| **Tray icon** | always | opening the popover, holding Quit |
| **Popover room** | on demand | the reward, the thing you look at |
| **Share image** | by other people | the audience and the growth mechanism |

The pet is the surface that makes the art worth making, because it is the only one that is always visible. The popover is where the craft is fully on display. The tray icon is now plumbing. The share image is section 5.

### 6.1 Desktop pet

The primary ambient surface, and the primary way into the app.

- **A 64x64 always-on-top window**, being the 32x32 character rendered at 2x. **The character only, never the room**, which is the same register split as the tray icon: the pet is the character, the popover is the scene.
- **Bottom-right by default, draggable between the four corners.** A drag is the only repositioning there is: the pet is moved by dragging it, and it snaps to the nearest of the four screen corners on release, never left where it was dropped. Free placement is deliberately not offered — the pet earns its keep by being ambient and *predictable*, and a pet dropped over content is a nuisance that gets quit. The corner choice is persisted in `state.json` (section 13), so it survives a restart with no schema change.
- **The bottom-right inset must clear the Dock explicitly.** `Monitor::work_area()` is not enough: on the author's display it returned a rect that reserved the menu bar and **not** the Dock band, so a pet placed relative to it sits underneath the Dock, which draws at window level 20. Placement uses the work area **plus** a Dock-aware inset. This is a tested finding (`spikes/always-on-top/RESULTS.md`), and it cost real time to diagnose because every AppKit property reported the window healthy while it was hidden.
- **A macOS-only `NSPanel` conversion is required** for the pet to be visible over fullscreen applications. Section 11, wall 1 has the recipe; section 10.3 accounts for it as the design's one platform-specific exception.
- **The pet's webview disables text selection, the context menu, and dragging.** Use `-webkit-user-select: none`, because plain `user-select` is not honoured in the WKWebView the pet renders in. Found by clicking the spike: the character is a picture, and picture-like things that highlight blue on click read as broken.
- **Clickable, not click-through.** Clicking it opens the popover. Click-through would make the pet purely decorative, and at 64x64 in a screen corner an accidental click is rare enough that the trade is easy. This makes the pet the primary entry point, with the tray icon secondary.
- **Motion is reserved on the pet specifically, and this is a rule rather than a preference.** Every room state animates (section 4.1), but the pet is the one surface sitting in peripheral vision all day, so it runs at the bottom of the range: dozing and asleep at 2 fps with the smallest amplitude in the loop, awake a slow idle, and only comeback loud. **The distinction is amplitude, not presence.** A sprite that moves constantly *and largely* in the corner of someone's eye is the thing people quit, and the pet's whole value depends on being tolerable to leave running. A 1px breath every half second is not that; a tail sweep or a hop is, which is why the cat and the hop stay in the room and out of the pet.
- **Emotes carry the state.** The pet shows the same emotes as the room (section 4.4), because it has no lighting cues to work with. The room dims; the pet cannot, so the emote does that job. The reason the pet cannot dim is worth stating rather than assuming: a blue multiply over a character standing on somebody's desktop wallpaper reads as a recoloured sprite, not as dim light, because there is no room around them for the light to be in.
- **Each state gets its own emote, and dozing's is `...` rather than `Z`.** This follows from the line above and was not honoured at first: dozing and asleep both carried the `Z`, which is a contradiction, because if the emote is what carries state then two states with one emote are one state to anyone glancing at it. `...` also says the truer thing. A day away from a project is trailing off, not sleeping, and the `Z` belongs to the state that has actually gone to bed. The change applies to the room as well as the pet, so a state reads the same way in both registers.

- **The emote sits beside the head, not above it, and that was decided by measurement.** A character sprite is 16x32 whose content is 16x24 at an 8 row offset, so there are exactly eight transparent rows above the head; an emote is 16x16 with fifteen rows of content. In the room the emote clears the head completely by sitting thirteen pixels above it, and in a 32px cell there is no such room. Moving it to the side is the one arrangement where two 16-wide sprites tile a 32-wide cell with **no overlap at all**: character in the left half, emote in the right, aligned with the head rather than the body. The character sits two pixels clear of the cell floor so the comeback hop has somewhere to go. The comeback carries one sparkle rather than the room's two, because that is how many 16x16 slots are left once the character and the emote have tiled the cell.

- **Asleep spends those two pixels at the bottom instead, and it is the one state that does.** A sleeper never hops, and the sheets put the sleeping head's topmost row at a two pixel offset for two of the three characters, so keeping the clearance above would leave the cap touching the cell edge with nothing in reserve. Spending it below lets the blanket run off the bottom of the frame, which is how bedding behaves.

**The art gap this section used to claim does not exist, and claiming it cost the pet a state.** The claim was that the pack has no sleeping character without a bed, since the sleep animation is a three-layer composite of vertical bed, bare head and blanket (section 4.1), so the pet could not show asleep the way the room does. What followed from it was a **seated pose with a `Z` for both dozing and asleep**, separated by a two pixel slump, and that shipped.

It survived exactly as long as it took the author to see the two frames side by side: they are the same drawing, and the pet had three visible states rather than four. **Asleep is now the sleeping pose under a blanket**, which the sheet supports perfectly well, because the blanket band is separable from the bed drawn around it and a capped head under a blanket needs no furniture to say asleep. **The bed itself does stay exclusive to the room** (the pet is the character, the room is the scene), and a blanket is not furniture: it comes off the character sheet, not the interiors sheet, which is the pack's own answer to the question.

The general lesson is the one that mattered here: **that gap came from reasoning about the sheet instead of cropping it.** The recorded conclusion was plausible, it was written down in a spec that argues for its decisions, and it was wrong, and no amount of re-reading the argument would have found that. Fifteen seconds of `magick -crop` did.

### 6.2 Tray icon

The tray icon is plumbing. Its only jobs are opening the popover and holding Quit.

- **A monochrome template image** on macOS, at 16x16, so it adapts to light and dark menu bars automatically. Template means macOS owns the colour: it renders the alpha channel in whatever the menu bar needs, including while the menu is highlighted.

- **It is drawn rather than cropped, and it is the one piece of art here that is not from the pack.** The obvious move was to crop the character's own head off the idle sprite, on the reasonable theory that the thing in the menu bar should be the thing in the room. It rendered as a solid black blob: at 16px a filled silhouette has no internal structure left, because every internal edge in a sprite is a colour change rather than a hole, and a template image has only ink and no ink to work with. Extracting the outline colour instead gave structure and read as a burger. What works is a mark drawn **at** the size it is used rather than reduced to it: a capped head and shoulders with two eye holes, checked at 16, 18, 22 and 32px against both a light and a dark bar. **A menu bar template is a different register from pixel art**, which is the general lesson, and it has two useful consequences: nothing about the icon is covered by the pack's redistribution restriction, so it is committed, and the app therefore compiles on a machine with no licensed copy of the pack.
- It **may** hint at state, but nothing depends on it doing so. This reverses an earlier decision to ship full-colour non-template icons, which existed only because four states could not be told apart as one-bit silhouettes at 16x16. With the pet carrying state ambiently, the icon no longer has to encode state at all, so the simpler and better-behaved option wins.
- Primary click opens the popover. Right click opens a native context menu with exactly two items, **Open** and **Quit**. That menu is the only place Quit lives, since there is no menu bar and no settings screen.

### 6.3 Popover

Fixed width of 352px, height sized to content, anchored to whichever surface opened it, closing on click-outside and `Esc`. Top to bottom:

> **Anchored to the opener, not always to the tray icon**, which is a correction to this line rather than to its intent: it was written when the tray icon was the only way in, and section 6.1 has since made the pet the primary one. macOS moves the menu bar's status items to whichever display is active, so on a two-display desktop the tray icon is frequently not on the same screen as the pet, and a popover anchored to the tray after a click on the pet opened on the *other monitor*. The panel now hangs off the pet when the pet was clicked and off the icon when the icon was clicked, opening downwards from an anchor in the top half of its display and upwards from one in the bottom half, clamped into that display's work area.

1. **Room panel.** 320x224, being the 160x112 room at 2x, which fits the 352px popover with 16px padding each side. The popover widened from 320px to 352px purely to accommodate the larger room (section 4.1).
2. **Character.** Clicking the character cycles to the next of the three shipped characters. This is the entire selection mechanism: no picker UI and no settings screen, so the guardrail holds. The choice persists in `state.json` as `character_id` (section 13) and applies to the pet as well as the room.
3. **Quote line.** One or two lines of pixel-font copy for the current state.
4. **Project list.** One row per tracked project: name on the left, relative time since its last activity on the right ("2 hours ago", "yesterday", "3 days ago", "a while back" past 30 days), or the word `operating` if the project is marked operating. No per-project moods, sorting, or counts. Two interactive elements: a small toggle that marks a project as operating, and a small `x` on hover that untracks it. Both earn their place because the alternative is hand-editing JSON. When nothing is tracked, a single line invites the user to add one.
5. **Buttons.** **Add Project** (section 7) and **Share Status** (section 5), side by side.
6. **Credit line.** A single small `art: limezu.itch.io` at the bottom. This is a license requirement (section 4.2), and with no about window or settings screen the popover is the only place it can live.

**Three characters, not one.** v1 ships premades 07, 12, and 20. The earlier position was one character on the grounds that choice implies a settings screen, and that reasoning was sound but is overtaken by a finding: because the sleep animation is layered, the character must be a separate composited layer over the room anyway (section 4.1). Once that is true, and since every premade sheet carries an identical animation set, three characters cost three PNGs rather than three sets of rooms. This is a **swap, not a skin system**: no per-character rooms, copy, or behaviour, and adding a fourth later is an asset drop rather than a spec.

### 6.4 Aesthetics

- **Dark only** in v1. A light palette implies a setting.
- **Typography: Departure Mono**, by Helena Zhang, under the SIL Open Font License 1.1, embedded and vendored at `assets/fonts/departure-mono/` with its license text. It carries the state label and the quote line; the project list and buttons use the system monospace stack, where legibility at small sizes beats character.

  It is drawn on an **11px cell with a 7px advance and renders pixel-exact only at integer multiples of 11**, so every size in the design is `11 * n` and `n` is that text's pixel unit: 11px in the popover, 22px for a share-card caption, 55px for the state label, which is 5x and therefore the room's own unit. Verified rather than assumed, by upscaling an 11px render 5x with a point filter and diffing it against a direct 55px render: zero pixels different.

  **Two candidates were rejected on measurements, not taste.** *Press Start 2P* is a fixed 8x8 cell, so its advance is a full em: the longest quote in section 4.6 runs to 1663px at 5x, fits no room at any integer scale, and overflows even the 320px popover room at 1x. *Silkscreen* renders lowercase as small caps, and an all-caps quote line contradicts the warm voice section 4.6 specifies. Departure Mono is the only one of the three that fits the popover's 320px room on one line, at 296px for 42 characters, and it is the only one with real lowercase and real punctuation at these sizes.

  **`+antialias` is not optional.** An antialiased pixel font is just a blurry font, and it is the one setting that makes the difference between type that belongs to the art and type that sits on top of it.
- All pixel assets are authored at the pack's **16x16 native grid** and scaled only in CSS, with `image-rendering: pixelated` at integer factors. No fractional scaling anywhere. No animation library and no runtime canvas loop in the popover.

---

## 7. Tracking Projects

**Add flow:** click **Add Project**, the native folder picker opens, the path is validated, and on success the project is appended to state, its last commit time is read, both its reflog and its working tree are watched, and the room re-evaluates. On failure a single short line appears inline in the popover. No modal, no alert dialog.

**Validation.** A folder is accepted only if it exists, contains a `.git` entry (a directory, or a file with a `gitdir:` pointer for linked worktrees and submodules, both resolved and both accepted), the resolved git directory has a readable `HEAD`, and the path is not already tracked. Re-adding an existing project is a friendly no-op, not an error. A repository with zero commits is accepted; its `last_commit_at` is null and it contributes nothing until it has a commit or a file change.

**Operating mode.** A tracked project can be marked as **operating**. This is a display-only tag: the project stays in the list, but it is excluded from the mascot's mood evaluation. It exists for projects whose current work is not commit-shaped — marketing, content, planning — so the mascot does not fall asleep while the user is busy elsewhere. If every tracked project is marked operating, the mascot has nothing to evaluate and is awake by default, the same as having no projects at all.

**What gets stored.** Per project: a generated id, the absolute path, a display name (the directory's base name, not user-editable in v1), when it was added, the last known commit timestamp, the last known working-tree activity timestamp, and whether it is operating. **Nothing else is read from the repository.** No commit messages, author identities, diffs, branch names, or file contents beyond whether a file changed and whether `.gitignore` says to ignore it.

There is no cap on tracked projects, but the design assumes a handful. The list scrolls beyond roughly 12 rows and is not virtualised.

---

## 8. State Model

### 8.1 Derivation

```
latest = max(last_commit_at, last_active_at) over all tracked projects
         that are not marked operating, ignoring nulls

if latest is null      -> awake      (nothing tracked/evaluated yet)
if now - latest < 24h  -> awake
if now - latest < 72h  -> dozing
otherwise              -> asleep

if previous_state == asleep and new_state == awake -> comeback
```

Thresholds are wall-clock durations, not calendar-day boundaries. A commit at 11pm does not become "yesterday's" at midnight.

The empty case resolves to `awake`, not `asleep`. A user who has just installed the app should meet a cheerful room.

State derivation is a **pure function of the tracked timestamps and the current time**, with no side effects. It is the most testable piece of the system and should be written that way, and kept independently replaceable.

**The current time is injected, never read from the system clock inside the function.** This costs one parameter and buys two things that are otherwise expensive: the boundary tests in section 15 become table-driven with no clock mocking, and the accelerated clock that section 5.4's demo and section 18's transition checks both need becomes a scale factor on the injected value rather than a code path of its own. The scale factor lives behind a debug-only environment variable, defaults to 1, and is compiled out or inert in a release build. It is not a setting, it does not appear in the UI, and it never touches `state.json`.

**Two corrections came out of building it, and neither is a detail.** Both were found by running the real state machine at 3600x and watching what it printed, which is exactly the check that clock exists to make possible. The paragraph above was right that the clock is cheap and wrong about what it is.

1. **Scaling `now` alone is broken, and it fails silently.** Git writes wall-clock timestamps while the app has moved on to a simulated one, so a commit made eighty real seconds after startup is born seventy-two simulated hours old and reads as ancient history. The first run of the timelapse crossed awake, dozing and asleep exactly on their thresholds and then sat there while a real `git commit` did nothing at all. **The scale defines a timeline anchored at startup, not a faster read of the current moment**, so every timestamp entering the app is mapped onto it and a commit made now is now at any scale. At scale 1 the mapping is the identity, so a release build is untouched by any of it.
2. **Not everything measured in time belongs on that timeline.** The comeback's 30 minute cap (section 4.5) is a dwell time on a piece of UI, and scaling it meant the celebration lasted half a real second: the one moment the product exists for was over before it could be seen, in the very recording made to show it. So there are **two clocks**, and the split is principled rather than a patch. Anything derived from git is on the app's timeline; anything about a person watching a screen is on the wall clock. The animation rates were never going to be scaled either, and this is the same kind of quantity.

### 8.2 Re-evaluation triggers

Three sources, and all must be handled:

1. **A commit lands**, event-driven via the watcher (section 9).
2. **A working-tree file changes**, event-driven via the watcher (section 9.4).
3. **Time passes.** The awake to dozing to asleep transitions happen with no event at all. This is easy to forget and is the more common transition in practice.

A tick re-evaluates state every 60 seconds. It is cheap because it compares in-memory timestamps and touches no disk.

### 8.3 Persistence

- A single JSON file: `~/.keepgoing/mascot/state.json` on macOS and Linux, `%APPDATA%\KeepGoing\Mascot\state.json` on Windows. **The extra folder is a correction, not drift, and the collision it avoids is not hypothetical.** This spec originally put `state.json` directly in `~/.keepgoing/`, and that directory already exists on the author's machine holding other KeepGoing tooling: databases, a socket, logs, `current-tasks.json`. It also already holds **`~/.keepgoing/state.json`**, written by the earlier KeepGoing CLI and carrying `lastSessionId` and `lastActivityAt`, with the type declared as `ProjectState` in that project's `packages/shared/src/types.ts`. The mascot writes atomically, by replacing the file rather than merging into it, so the original path would have destroyed that file on first launch, in silence, with no error on either side and nothing to indicate which tool had done it. Sharing the family directory is the part worth keeping; sharing the namespace is not.
- Writes are atomic: temporary file in the same directory, then rename. A crash mid-write must never leave a truncated state file.
- Reads are **resilient by contract**. A missing file, empty file, empty array, missing optional fields, unknown extra fields, and invalid JSON all resolve to sane defaults rather than an error or a crash. Losing the tracked list is a mild annoyance; refusing to start is a dead product.
- The stored state name is a cache for restart continuity and comeback detection, never trusted as truth. Current state is always recomputed from timestamps at load.

---

## 9. Activity Detection

### 9.1 Only real work counts

The mascot must respond to **work on the project**, not to a developer taking a look and leaving. This is a correctness requirement, not a nicety: `git checkout` and `git pull` move `HEAD` without the user writing anything, so treating all `HEAD` movement as momentum would let a **comeback celebration fire because someone checked out a branch after three weeks away**, hollowing out the single most important moment in the product.

Two signals count as real work: a qualifying commit in the reflog, and a non-ignored file change in the working tree.

Reflog lines have the form `<old-sha> <new-sha> <name> <email> <unix-ts> <tz>\t<message>`. The filter is on that trailing message.

**Counted** (message begins with `commit`): `commit:`, `commit (initial):`, `commit (amend):`, `commit (merge):`.

**Ignored:** `checkout:`, `pull:`, `merge <branch>:` (fast-forward, no new work), `reset:`, `clone:`, `rebase (pick):` and `rebase (finish):` (replaying existing commits, not new work).

### 9.2 Read algorithm

For each tracked project, the Rust backend watches `.git/logs/HEAD` with the `notify` crate. On an event, debounced by about 250ms to collapse the burst of writes a single git operation produces:

1. **Scan backwards from the end of the reflog until the first qualifying `commit` entry**, bounded to roughly the last 200 lines. This replaces a naive "read the last line", which would return the wrong answer any time the most recent operation was a checkout or a pull.
2. Take that entry's **reflog timestamp**, not the commit's committer time. For an amend or a rebase the committer time may be rewritten, while the reflog timestamp records when the user actually acted. "When did I last do work here" is a question about the user, not about the commit object.
3. If the scan passes the bound with no match, or the reflog is missing, empty, or unparseable, **fall back** to reading the committer timestamp from the commit `HEAD` points at. **This one path shells out to `git`, and it is the only place in the app that does.** The argument below against spawning a process is about the per-event path, which runs on every commit; this runs only when the cheap read has already failed, which in practice means a repository written by a GUI client with non-standard reflog messages. Reaching the commit object without `git` means inflating loose objects *and* parsing packfiles, which is a large amount of code for a rare fallback, and the degradation is graceful in both directions: with no `git` on `PATH` the worst case is a slightly stale timestamp, never a false comeback.
4. **Monotonicity rule: `last_commit_at` never decreases for a given project.** A new reading that is older than the stored value is discarded. Without this, checking out an older branch would drag the timestamp backwards and put the character to sleep despite recent work. "When did I last do work here" does not move backwards. An implementer would not infer this rule, so it is stated explicitly.
5. Update, persist, re-derive state, update the tray and popover.

Reflog parsing is preferred over shelling out per event because it is a single small read with no process spawn and no dependency on a `git` binary on `PATH`. Startup performs a one-time read of every tracked project, so commits made while the app was not running are picked up.

### 9.3 Why not git hooks

The previous spec installed a `post-commit` hook into each tracked repository. Rejected because it **collides** (husky, lefthook, pre-commit, and hand-rolled hooks all own `post-commit`, and merging into one safely is a real engineering problem for a mascot), because it **creates an uninstall problem** (deleting the app would leave shell fragments across the user's repositories pointing at a missing binary, breaking their commits, and a toy that can break `git commit` after being deleted is not a toy), and because it **requires a second executable** for the hook to call, which is exactly the dependency this design removes.

Watching the reflog gets the same near-instant update with zero footprint inside user repositories, through one code path on all three platforms. Untracking is a line removed from JSON, and uninstalling leaves nothing behind.

### 9.4 Working-tree activity

Not all project work is commit-shaped. Editing files before a commit, writing drafts, updating assets, or running local builds are all real activity, and the mascot should see them.

For each tracked project the app also watches the project's root directory with `notify`, recursively. On an event:

1. Ignore paths inside `.git` (those are handled by the reflog watcher).
2. Ignore paths matched by the project's own `.gitignore`. This is the most tech-stack-agnostic filter available: it already encodes what the user considers noise — build artifacts, dependencies, editor files — without requiring per-language configuration in the app.
3. Ignore directories. Only file changes count.
4. Record the current simulated time as `last_active_at` for that project, subject to the same monotonicity rule as `last_commit_at`.
5. Re-derive state and update the UI.

Working-tree events are debounced the same 250ms as reflog events. A single save burst collapses to one update.

> **Corrected 28 August 2026, after three defects found in the build already on the store.** Step 2 as written above is not enough, and step 3 was not implemented at all.
>
> **Step 2 also filters a built-in list of operating system metadata**: `.DS_Store`, `._*`, `.Spotlight-V100`, `.Trashes`, `.fseventsd`, `Thumbs.db`, `desktop.ini`. The reasoning above is sound about a `.gitignore` encoding what the user considers noise, and wrong that it is the only place they encode it: `.DS_Store` belongs in a *global* ignore file, and under App Sandbox `$HOME` is the container, so the shipped build cannot read the one file that would have covered this. Untreated, opening a dormant project in Finder woke the mascot and could spend a comeback. The list is prepended to the project's own file rather than appended, so a `!` line in the repository still wins, which is the precedence git itself uses.
>
> **Step 2 matches parent directories too.** It asked whether the changed path was ignored, not whether anything above it was, so `target/debug/app` read as unignored with `target/` in the ignore file. Almost every line in a real ignore file names a directory, which left this filter mostly decorative.
>
> **Step 3 now exists.** Directory events are rejected by `notify`'s own `Create(Folder)` and `Remove(Folder)` labels and by `is_dir`; a removed folder is gone by the time the path is examined, so the label is the only evidence there.

---

## 10. Tech Stack

**Tauri v2: a Rust backend with a webview UI, in a single codebase.** Rust owns file watching (`notify`), reflog parsing, state persistence, and the tray. The webview owns the room rendering, popover layout, and the share canvas. Storage is one local JSON file. There is no network layer.

### 10.1 Why Tauri

- **One codebase, three platforms**, with native tray on all of them, and **menu-bar-only on macOS** via `ActivationPolicy::Accessory`, the direct equivalent of `LSUIElement`.
- **Both hard art jobs are trivial in a webview.** Pixel rendering is `image-rendering: pixelated` plus a CSS `steps()` animation; the share image is a canvas with smoothing disabled and `toBlob()`. In SwiftUI these are fighting the framework's smoothing, hand-rolling frame timing, and manual `NSImage` and pasteboard work.
- **Rust keeps what suits Rust.** `notify` is a mature cross-platform watcher, and the parsing and persistence work is small, fast, and testable.

Since roughly 90% of the value is art and feel, the correct optimisation is making art and feel cheap to iterate on. A webview does that better than any native toolkit available here.

### 10.2 What was rejected

**Rejected: a Rust CLI plus a separate Swift menu bar app, communicating through a JSON file and a `keepgoing://` custom URL scheme** (the previous spec's architecture).

The Swift app was about 95% of the product and macOS-only. That architecture pays for two languages, two build systems, and an IPC protocol **now**, and still forces a complete UI rewrite at the Windows boundary. It buys nothing toward the cross-platform goal, and the two hardest jobs here, pixel art and share-image generation, are the two jobs SwiftUI is worst at.

The IPC existed only because the process split existed. Deleting the split deletes the JSON handshake, the URL scheme, the `Info.plist` registration, the "did the ping arrive" failure mode, and the app-not-running-when-hook-fires failure mode, all at once. **The CLI binary, the IPC layer, and the custom URL scheme are deleted from the design.**

Cross-platform reach is a real goal, because reach is how the character finds the people who will love it. A single portable codebase is the entire point.

### 10.3 The one platform-specific exception

Section 10.1 claims one codebase and no native escape hatches. There is exactly one exception, and it is better named here than discovered in a diff.

**The desktop pet's window must be converted to an `NSPanel` on macOS**, using `object_setClass` plus three setters, roughly fifteen lines of `objc2` behind `#[cfg(target_os = "macos")]`. Section 11, wall 1 has the recipe and the evidence. Without it the pet is invisible over fullscreen applications, which removes the product's primary surface, so this is not optional and not deferrable.

**Hand-rolled rather than a dependency.** `tauri-nspanel` does the same thing, and taking it would mean a dependency, its Tauri-version coupling, and its plugin surface in exchange for about fifteen lines that are already written and verified. In a project whose stated failure mode is sprawl, that is the wrong trade. The block is small, commented with why each call exists, and pinned by the spike's findings.

Two consequences to hold onto:

- **Set it once at window creation.** Never adjust level or `collectionBehavior` at runtime. Section 11 explains why: reconfiguring a live window produced history-dependent results.
- **`transparent: true` requires Tauri's `macos-private-api` feature**, which makes the app ineligible for the Mac App Store. Distribution is direct, so this costs nothing today, but it forecloses that option silently.
- **The bundle is universal, `LSUIElement`, and ad-hoc signed.** Three things the first packaged build settled, each for a stated reason:
  - **Universal rather than native-only**, because the default was worse than either: an older standalone Rust install in `/usr/local/bin` shadowed rustup's, so every build in the project until packaging was x86_64 running under Rosetta on an Apple Silicon machine, reporting no problem anywhere. Worth knowing generally: a toolchain silently building for the wrong architecture looks exactly like a toolchain building correctly.
  - **`LSUIElement` in `src-tauri/Info.plist`**, not only `ActivationPolicy::Accessory` at runtime. They do the same job at different moments, and the runtime call cannot take effect until the process is up, so without the key a bundled build shows a Dock icon for a fraction of a second on every launch.
  - **Ad-hoc signed and not notarized**, which is a real limit rather than a detail: Gatekeeper refuses the app on any machine other than the one that built it, so anyone receiving the disk image has to right-click and Open once. Removing that needs a paid Developer ID. Recorded because it is the first thing that will be reported as a bug by whoever is handed the demo.
- **The application icon is derived from the pack; the tray icon is not.** They are different jobs at different sizes (section 6.2), and the split has a licensing consequence worth being explicit about. The tray mark is drawn by hand at 16px, is covered by nothing in the pack's licence, and is committed. The app icon is the character at 1024px inside the popover's own mat and mount colours, so it falls under section 4.2 exactly as the rooms do: shipped compiled into the binary, never committed as an asset. `tools/make-app-icon.sh` builds it, and `tauri build` therefore needs the licensed pack, which is already true for every other reason.

> **Superseded, 2026-08-22, and settled 2026-08-27.** The trade above was correct while direct
> distribution was the only target. It is no longer the plan: see
> `docs/superpowers/specs/2026-08-22-mac-app-store-design.md`, and
> `docs/app-store-listing.md` for what actually happened at the store.
>
> The prediction was closer to right than the first draft of that design document was. The pet
> does **not** have to be an opaque square. But the reason the first draft gave, that window
> transparency is public API, does not hold through Tauri: `tauri-runtime-wry`'s
> `window.transparent(...)` and `WindowBuilder::transparent()` are both gated on the
> `macos-private-api` feature, and with it off the only complaint is an `eprintln!` gated on
> `debug_assertions`. So the pet keeps its alpha only because the app makes the `setOpaque:` and
> `setBackgroundColor:` calls itself, in `src-tauri/src/appkit.rs`.
>
> Two private API strings also remain in the shipped binary and are not removable without forking
> wry and tao: `allowsPictureInPictureMediaPlayback` and `_wantsKeyDownForEvent`. Neither is
> reachable from this codebase, and precedent is the whole justification for shipping them.
>
> **0.3.1 was approved for the Mac App Store on 27 August 2026**, at
> `apps.apple.com/app/momentum-mascot/id6804925509`, with the pet's alpha intact. The one
> rejection on the way was guideline 2.1 Information Needed, a request for a screen recording and
> seven answers about the app, and nothing to do with private API, transparency, or the art
> licence.

> **Corrected 28 August 2026, after the section 9 test list was run by eye on 0.3.1.** This
> section says "the desktop pet's window", and the popover needs the same conversion. It shipped
> as an ordinary window with `alwaysOnTop: true`, which is a *level*, and the exception exists
> precisely because no level is enough: over a fullscreen app the pet was visible and the popover
> was not. That combination is worse than neither working, because the pet stays clickable and
> clicking it looks like the app is broken.
>
> So the recipe now lives in `appkit::show_over_fullscreen` and both windows call it, once each,
> before either is first shown. The two disagree on one thing: the pet must never take the
> keyboard, and the popover must, because Escape dismisses it.
>
> That difference cost the one genuinely new piece of knowledge here. `object_setClass` to the
> stock `NSPanel` **discards tao's own `canBecomeKeyWindow` override**, and `NSWindow` answers NO
> for a borderless window, so a reclassed popover accepted no key events at all: measured
> `isKeyWindow` false for as long as the window was up, against true within a second before the
> reclass. The popover therefore gets a registered `NSPanel` subclass that answers YES. The count
> in this section goes from about fifteen macOS-only lines to about twenty-five, still in one
> place.

### 10.4 Repository layout

A single Tauri project, deliberately flat. No monorepo, no workspace, no shared packages.

```
momentum-mascot/
  src/                  # webview UI: popover markup, styles, room CSS, share canvas
  src/assets/           # composed room sheets per state, tray icons, embedded pixel font
  src-tauri/src/        # Rust: watcher, reflog parsing, state model, tray, commands
  docs/                 # this spec and the initial spec
```

The word `packages/` does not appear in this repository. If it ever does, something has gone wrong.

---

## 11. Platform Phasing

**macOS first.** The author is a Mac user and will dogfood it daily. This is where the product gets judged.

**Windows next, as a build target rather than a rewrite.** This is the payoff for the Tauri decision. Expected work: packaging, tray assets in the formats Windows expects, and verifying the folder picker, clipboard, and always-on-top paths.

**Linux last, and possibly never with the pet.** See the Wayland wall below.

The desktop pet introduces two platform walls that are real, and neither should be buried.

**Wall 1: macOS fullscreen. Solved, and the recipe is known.** This was the single biggest threat to the pet concept, because a developer who codes in fullscreen would have the pet invisible during exactly the hours they are working. It was therefore verified as a spike before any other pet work, and the spike is recorded in `spikes/always-on-top/RESULTS.md`.

An always-on-top `NSWindow` does not appear over a fullscreen application, because a fullscreen Space is its own space. **No window configuration fixes this.** Ten `collectionBehavior` values were tested across four window levels up to `kCGMaximumWindowLevel`, and every one was invisible over a fullscreen Chrome window.

What works is changing the kind of window it is. The verified recipe, applied **once at window creation** and never adjusted afterwards:

1. `object_setClass` the Tauri window to **`NSPanel`**, and add `NSWindowStyleMaskNonactivatingPanel`. This is the load-bearing step, and it is the same approach the `tauri-nspanel` community plugin takes.
2. Window **level 25** (`NSStatusWindowLevel`).
3. **`collectionBehavior` 273**: `canJoinAllSpaces | stationary | fullScreenAuxiliary`.

Verified against a fullscreen Chrome window: the pet is visible, clicking it does **not** leave the fullscreen Space or steal focus, and the fullscreen application remains fully interactive. Both halves of the gate pass, so section 6.1's "clickable, not click-through" stands.

**Applied once, never adjusted.** The spike found that reconfiguring a live window gives history-dependent results: the identical level and behaviour was invisible over fullscreen in one run and visible in another, decided by what had been applied minutes earlier. The product must therefore set this at creation and leave it alone. It is also why the minimal recipe is not pursued further; a configuration that measures differently depending on its past is not one to shave.

The cost is real and is named in section 10.3: roughly fifteen lines of macOS-only `objc2` sitting outside Tauri's cross-platform API, in a design that otherwise avoids paying for platform-specific code.

**Wall 2: Wayland cannot do this at all.** Wayland deliberately does not let applications position their own windows, so a pet pinned to a screen corner is essentially not implementable there. Linux already had flaky tray support (`StatusNotifierItem` varies by desktop environment; GNOME ships none by default and typically needs an extension such as AppIndicator Support), and this escalates the situation from a degraded tray to a **missing primary surface**. So the Linux position is not merely "last": Linux may ship without the pet, or not ship at all, and that should be **decided deliberately rather than discovered** during a port. Clipboard image writing is also less uniform under Wayland, and since Share Status is the growth mechanism, a Linux release is not worth shipping until that path is verified too.

Both walls are ecosystem constraints rather than Tauri limitations, and no framework choice avoids them. Wall 1 is solved at a documented cost; wall 2 is accepted rather than solved. Wall 2 belongs in the README rather than being found by a frustrated user.

---

## 12. Build Phases

**Phase 1: Refine the four room states.** No app code, no Tauri project, no Rust.

**The composition is done, and it animates.** The two weaknesses named in the prototype are fixed, three more were found by looking harder, and the room is regenerated from a script rather than placed by hand:

- **Awake.** The character now sits behind the desk with the computer in front of them and their face clear above it, which is the pack's own receptionist composition. It works because the screen points away from the viewer, so nothing has to face the wrong way. See section 4.1, constraints 3 and 4.
- **Dozing.** The character is away from the desk with a steaming coffee, and the `Z` is anchored to their head rather than floating over the furniture. Emote offsets are now defined relative to the character in every state.
- **Asleep**, which was not on the original list, was also wrong: the head sat on a double bed at half its width. It is now a single bed with the character's own hat on, under the bed's blanket.
- **The standing pose was a side view for the whole first pass**, which was not on any list because at 16px it looked like a small blob rather than like a mistake. Section 4.1, constraint 6.
- **The bookshelf was missing its right-hand cap**, so it stood in the room with a corner sliced off. Section 4.1, constraint 5. This one was caught by the author looking at the render, which is the argument for the week below.
- **Wall, floor, bed, lamp and rug** are deliberate picks from the Room Builder sheets rather than the placeholders sampled out of an assembled home.
- **Every state is a 12-frame loop** rather than a still, and there is a cat on the rug. Section 4.1 for the animation, section 4.4 for the cat.

**Composition is a script, not a document.** `tools/compose-rooms.sh` regenerates all four states from a local licensed copy of the pack, with every coordinate named and commented at the top. This replaces the "do it in Tiled or Aseprite" instruction in earlier drafts, and it earns its keep three times over: iterating on a placement is a one-line edit and a re-run, section 4.2 already requires that the app composite rooms at build time from a local pack rather than commit the art, and now that every state is a 12-frame loop, hand-placing would mean hand-placing 48 frames. The script is that compositor, written early. `docs/asset-picks.md` remains the human-readable manifest.

It emits, per state, the still PNG at 160x112, a 12-frame horizontal strip for the app to step through, and a GIF at that state's own rate for looking at. The GIFs are review artifacts and are never shipped.

What is left in this phase is the test that cannot be rushed. The author **lives with the four rooms for one week** before any application code: as a wallpaper, a lock screen, pinned in a window, wherever they are seen daily and unprompted. The test is whether the asleep room still feels comforting rather than sad on day seven, and whether the comeback room still causes a reaction. It tests whether the *states* feel right, not whether the pixels are well drawn. The art determines the outcome and is by far the cheapest thing to test: a week is recoverable, and discovering after the app is built that the rooms are not lovable is not.

**Phase 2: Design the share image. The composition is done.** `tools/compose-share.sh` builds all four cards from the Phase 1 stills, and the three findings in section 5.2 came out of building it. Settled here: the footer's three-part single baseline, the label outlined rather than shadowed, the wordmark merged into the URL so it is large enough to read, and Departure Mono as the embedded pixel font (section 6.4). Section 17's question about mat proportions is resolved: the spec's own 200px bars and 12px top margin are correct, and what had to move was the quote, not the mat.

**The phase closes here, and the instruction to "post it somewhere and see how it reads" moves out of it.** That instruction assumed a card could be posted cold and understood, and it cannot: see section 5, which was rewritten because of it. Posting a card before anyone knows what KeepGoing is tests nothing, because a failure would be indistinguishable from the audience simply having no idea what they were looking at. The card gets posted after the demo exists, and the demo needs the app, so that test now sits in Phase 5.

What that leaves settled: the composition, the footer's three-part single baseline, the label outlined rather than shadowed, the wordmark merged into the URL so it is large enough to read, and Departure Mono as the embedded pixel font (section 6.4). Section 17's question about mat proportions is resolved: the spec's own 200px bars and 12px top margin are correct, and what had to move was the quote, not the mat. The card is generated from **frame 0 of each state**, so it is a still of an animated room; whether it should ever move is section 17, question 3, and section 5.4's demo takes most of the pressure off that question by covering the case a still handles worst.

**Phase 3: Tauri app, macOS target.**

**The gate is already cleared.** Proving that a 64x64 always-on-top window can be visible over a fullscreen application on macOS was pulled out of this phase and run first as a spike, because the pet is the primary surface and a negative answer would have changed the art requirements. It passed: the window is visible over fullscreen Chrome and clicking it does not disturb the fullscreen application. The recipe is in section 11, wall 1; the evidence and the dead ends are in `spikes/always-on-top/RESULTS.md`. **Port that AppKit block into the pet's window setup rather than rediscovering it, and delete the spike, keeping its `RESULTS.md`.**

Then: the desktop pet window, the tray icon, the popover, `notify` on `.git/logs/HEAD` with the qualifying-commit scan, the state model, Add Project, and Share Status. The first phase producing a running application, and it produces a complete one.

**The injected clock is built here, not retrofitted.** Section 8.1 requires the current time to be a parameter rather than something read inside the derivation, and the debug scale factor rides on that. It is a few lines when the state model is first written and an invasive change afterwards, and both the demo and the transition tests below are blocked without it.

**Built. The application runs, and the spike is deleted with its findings kept.** What the phase produced, and what is still owed:

- **The AppKit block is ported into `src-tauri/src/pet.rs`** with the comments about why each call exists, which is the part that goes stale silently. One thing was added on top of the spike's finding: the pet takes the **intersection** of Tauri's work area and AppKit's own `visibleFrame` rather than subtracting a Dock-sized guess from one of them, so it clears the Dock whichever of the two accounts for it and cannot start double-counting if Tauri changes underneath it.
- **Five findings corrected this spec**, each recorded in the section it belongs to rather than here: the accelerated clock needed a whole timeline rather than a faster `now`, and a second clock for UI dwell time (section 8.1); the state file needed its own namespace inside `~/.keepgoing/` (section 8.3); the pet's emote had to move beside the head rather than above it (section 6.1); the tray icon had to be drawn rather than cropped (section 6.2); and the `HEAD` fallback shells out to `git`, which is the only process this app spawns (section 9.2).
- **Two more real defects came from the author running the built app**, and they are the reason this list is not the end of the phase. Both were invisible to the test suite and to every property the app can query about itself. The third entry is not a defect at all: it is the measurement trap that invented one, and it is recorded because it wasted more time than either of the real ones.
  - **The pet's window size has to be set in logical pixels, and setting it in physical pixels shrinks only the webview.** `resizable: false` clamps the window back to its configured size, so the two disagreed by the display's scale factor, the character was drawn at full size inside a viewport half as large, and what reached the screen was the hat alone. It took three wrong diagnoses to find, because the obvious measurement is the wrong one: `inner_size()` reports the window, and the window was correct the whole time. Only the webview knows its viewport, which is why `pet.js` is now the thing that measures it and sizes the character to what it finds. A stylesheet holding a second opinion about the size of its own window was the actual defect; the unit was just how it got in.
  - **`outer_position()` read straight after `set_position` returns the old position**, which is not a bug in the app but is worth a line here because it manufactured one. It reports the macOS default near the middle of the display, repeatedly and no matter how many times the call is retried, and a few hundred milliseconds later the same read returns the corner. Chasing it produced a diagnosis, a restructure of `main.rs` around `RunEvent::Ready`, and a fix, for a defect that never existed: placement was correct from the first run and the instrument was lying. All of that was reverted. **The lesson is not about Tauri.** Three of the four diagnoses in this list were wrong on the first attempt, every one of them because a value the app reported about itself was measuring something adjacent to the question, and the way each was finally settled was to reproduce the effect and compare it against a known-good render. A measurement that has not itself been checked against a case where it must fail is a hypothesis.
  - **The pet's dozing and asleep frames were the same drawing** (section 6.1), which is the one finding here that was already written down as a risk and shipped anyway.
- **What the app can check about itself and what it cannot is now a line worth drawing.** The state machine, the reflog scan, the clock and the card are all measurable from a test, and are. Window geometry and whether two drawings look alike are not, and no assertion inside the process would have caught either. **The manual pass in section 14 is not a formality, and treating it as one is exactly how these shipped.**
- **The share card is verified rather than assumed.** The webview's canvas version was diffed against the card `tools/compose-share.sh` produces, at 1200x630, and the result is **zero differing pixels**. So the app ships the composition that was designed and looked at in Phase 2, not an approximation of it. `tools/preview/` holds the harness that proves it, and it found a real bug on the way: the card composed silently in a fallback serif when the `@font-face` rule was not present. `share.js` now registers the font itself and refuses to compose if the advance is not 7px on an 11px cell.
- **The transitions are verified under the accelerated clock**, which is what the clock was built for. Driving the real state machine at 3600x crosses awake to dozing at 24.05h and dozing to asleep at 72.10h, ignores a `git checkout` and a `git pull`, fires the comeback on a real commit, and holds it. That is four lines of the definition of done checked in ninety seconds rather than three days.
- **What is still owed is the part a machine cannot check:** the manual pass in section 14 on a real desktop. The pet over a fullscreen app, clicking it, display changes, sleep and wake, the tray icon against both menu bars, and Share Status pasting into a real chat app and a real social composer.

**Phase 4: Record the demo.** The artifact that actually explains the product (section 5.4), and the first thing that can be made now that the app runs. A throwaway repository, a scaled clock, a clean desktop, and a 20 to 30 second timelapse ending on the comeback. Recording it against the real state machine doubles as the integration test that no unit test covers: real commits, real watcher, real derivation, only the clock scaled. Watch it back for leaked paths and tab titles before it goes anywhere (section 5.3).

This is deliberately its own phase rather than a task inside Phase 3. It is the highest-leverage artifact in the whole plan, it is the one most likely to be skipped under momentum, and giving it a phase number is the cheapest way to stop that happening.

**Phase 5: Post it, and only then post a card.** The demo goes out first and creates the context. Then a card can be posted and read for what it actually is, which is the test Phase 2 was originally asked to run and could not. If the demo lands and the cards do not, that is a real signal about the art rather than an ambiguous one about the audience.

**Phase 6: Windows build target.** Packaging, tray assets, and verification of the picker, clipboard, and watcher paths. No new features enter here.

**Phase 7: Linux, if and when someone asks**, with the caveats in section 11 understood before starting.

---

## 13. Data Schema

```json
{
  "version": "3.0",
  "last_displayed_state": "asleep",
  "character_id": "07",
  "pet_position": { "x": 1780, "y": 940 },
  "tracked_projects": [
    {
      "id": "a1b2c3d4-e5f6-7890",
      "path": "/Users/username/Projects/my-side-project",
      "name": "my-side-project",
      "added_at": "2026-08-01T08:00:00Z",
      "last_commit_at": "2026-08-12T08:00:00Z",
      "last_active_at": "2026-08-12T10:30:00Z",
      "operating": false
    }
  ]
}
```

- `version` allows a future migration. This release reads `"3.0"` and treats older files as best-effort, filling missing new fields with sane defaults.
- `last_displayed_state` is `awake`, `dozing`, or `asleep`. It exists solely so comeback detection survives a restart, and is never used as the current state.
- `character_id` is one of the three shipped characters. Missing, unknown, or malformed values fall back to the first character rather than erroring, per the resilient-parsing contract in section 8.3. A future release that ships more characters must not break on a value it does not recognise.
- `pet_position` is the desktop pet's top-left corner, in physical pixels. It starts unset, which places the pet bottom-right by default, and is written whenever the pet is dragged to a corner (section 6.1). A missing value, or one that falls outside the current display bounds (an unplugged monitor, a resolution change), resets to the bottom-right default rather than leaving the pet off screen.
- `last_commit_at` is nullable, for a repository with no commits yet, and is subject to the monotonicity rule in section 9.2.
- `last_active_at` is nullable, records the last non-ignored working-tree file change, and is subject to the same monotonicity rule.
- `operating` is a boolean defaulting to `false`. When `true`, the project is excluded from mood evaluation and shown with an `operating` label in the project list.
- `id` is a generated UUID, stable for the lifetime of the entry.
- **No per-project `status` field.** The previous schema stored one, and a derived value written to disk is a cache-invalidation bug waiting to happen. Per-project mood is not displayed anyway.

A tracked project whose path no longer exists is kept and shown as unavailable rather than silently deleted, because a disconnected external drive should not erase the user's list.

---

## 14. Testing Strategy

Effort follows risk, and the risk is concentrated in a small amount of logic.

**Unit tested in Rust:**

- **State derivation**, table-driven across the boundaries: just under and exactly 24h, just under and exactly 72h, far past 72h, empty list, all-null, mixed null and present, operating projects excluded, and activity timestamps treated equally with commit timestamps.
- **Comeback detection**, confirming `asleep -> awake` fires, `dozing -> awake` does not, and the restart case where the previous state comes from disk.
- **Reflog filtering**, on fixture lines: `checkout:` ignored, `pull:` ignored, fast-forward `merge <branch>:` ignored, `reset:` ignored, `commit:` counted, `commit (initial):` counted, `commit (amend):` counted, `commit (merge):` counted, a message containing tabs parsed correctly, a reflog whose last qualifying entry is many lines back found correctly, and a reflog with no qualifying entry within the bound falling through to `HEAD`.
- **Monotonicity**, confirming an older reading does not overwrite a newer stored value, for both commits and activity.
- **Operating mode**, confirming operating projects do not affect mood and that toggling updates the display.
- **State file resilience**: missing file, empty file, `{}`, empty array, missing fields, unknown fields, and invalid JSON all load without panicking, plus a write-then-read round trip, an atomic-write test, and backward compatibility for v2 files.
- **Repository validation**: `.git` as a directory, `.git` as a gitdir pointer file, a non-repo directory, a nonexistent path, and a repo with zero commits.

**Manually verified per platform:** tray icon appears and reads against light and dark menu bars; popover opens and closes and renders the room at crisp integer scale on standard and HiDPI displays; folder picker returns a usable path; Share Status pastes correctly into a real chat app and a real social composer; a real `git commit` updates the state within about a second; an edited file updates the state within about a second; a `git checkout` does not; marking a project operating excludes it from the mood.

**The pet needs its own manual pass**, because it is a window rather than logic and none of it is unit-testable:

- Appears above normal application windows, and stays there when other apps take focus.
- Appears (or provably does not appear) over a fullscreen application. This is the Phase 3 gate, and the answer is recorded either way.
- Clicking it opens the popover, and clicks near but not on it pass through to whatever is underneath.
- Shows the correct state, including the comeback celebration firing without the popover being opened.
- Survives display changes: unplugging an external monitor, a resolution change, and display rearrangement all leave it visible and on screen.
- Survives sleep and wake, and does not duplicate or vanish after either.

**Not tested:** the visual quality of the rooms, which is judged by the week in Phase 1. No end-to-end UI automation in v1; the surface is one popover with two buttons, and a harness would cost more than it returns.

---

## 15. Risks and Mitigations

| Risk | Impact | Mitigation |
| --- | --- | --- |
| **The character and rooms are not lovable.** No engineering fixes this. | Fatal | Phase 1 exists for this: a week of living with the rooms before any app code, with an explicit willingness to recompose or restart. |
| **Scope creep repeats the deprecated ecosystem.** | Fatal, slowly | The guardrail and out-of-scope table in section 3, the permanent non-goals, the flat layout, and the seams-not-extension-points rule in section 16. |
| **Using the wrong pack.** `Modern tiles_Free/` sits beside the paid pack and is non-commercial only, so a single asset pulled from it would silently breach the license. | High, legal | Source assets only from `moderninteriors-win/`. Named explicitly in sections 4.2 and 12 so it cannot be discovered by accident. |
| **Missing attribution.** Credit to `limezu.itch.io` is required by the license, and there is no about window to hide it in. | High, legal | Credit is specified as a functional requirement in three fixed places: the popover footer, the share image band, and the README. It is on the Definition of Done. |
| **Nobody shares the card**, so there is no growth and no signal. | High | Two mitigations, and the first was originally mistaken for the whole answer. The card is designed and looked at before any code generates one (Phase 2), which catches a card that is simply ugly. But a card nobody understands is the likelier failure, and the fix for that is the demo (section 5.4): discovery is its job, so the card is only ever asked to do the thing it can do. Phase 5 posts them in that order deliberately, so a poor result points at the art rather than at the audience. |
| **Nobody watches the demo either**, so the card never gets its context. | High | The genuinely unhedged risk in this plan, and worth naming as such rather than dressing up. Nothing in the design guarantees a launch post lands. What is under control is that the demo is cheap to remake, needs no code beyond the debug clock, and can be re-cut and re-posted as often as it takes without touching the product. |
| **Non-standard reflog messages** from GUI clients or libgit2-based tools are not matched by the `commit` prefix filter. | Medium | The `HEAD` fallback in section 9.2 covers it: the worst case is a slightly stale timestamp, never a false comeback. |
| **`notify` misses events** on network volumes, virtualised filesystems, or after sleep/wake. | Medium | The 60-second tick re-evaluates and startup re-reads all projects. Worst case the room updates within a minute instead of instantly, which is acceptable here. |
| ~~**The pet is invisible over macOS fullscreen apps.**~~ **Retired: solved.** Was rated fatal to the pet. | Closed | Spiked before any other pet work and cleared. The fix is an `NSPanel` conversion, not a window level; no `NSWindow` configuration works at all. Recipe in section 11, wall 1, cost accounted in section 10.3, evidence in `spikes/always-on-top/RESULTS.md`. |
| ~~**The pet is visible but hostile**~~, switching Spaces or stealing focus on click. **Retired: tested and does not happen.** | Closed | Verified as a separate verdict, because "visible" alone was not a pass: clicking the panel over fullscreen Chrome neither leaves the Space nor takes focus, and Chrome stays interactive. Section 6.1's "clickable, not click-through" stands. |
| **The AppKit block rots on a future macOS release.** It relies on `object_setClass` and an undocumented interaction, and the spike already saw the same configuration behave differently depending on history. | Medium, ongoing | Contained: about fifteen lines, one place, commented with why each call exists (section 10.3). `RESULTS.md` keeps the dead ends so a future break is re-diagnosed in minutes rather than re-explored from scratch. The failure mode is graceful, since a pet that stops appearing over fullscreen is degraded rather than broken. |
| ~~**Transparency depends on a private API**, so the app is ineligible for the Mac App Store.~~ **Reversed 2026-08-27.** | Closed | Eligible, and shipped: 0.3.1 was approved for the Mac App Store on 27 August 2026 with the pet's alpha intact. The route is App Sandbox applied at signing time plus security-scoped bookmarks, `macos-private-api` off, and the pet's transparency made by the app's own `setOpaque:` call. See section 10.3, `docs/superpowers/specs/2026-08-22-mac-app-store-design.md`, `docs/app-store.md`, and `docs/app-store-listing.md`. The prediction that the pet would have to become an opaque square was wrong. |
| **A pet that moves too much gets the app quit.** The always-visible surface is also the always-annoying one if it fidgets. | High | Motion is reserved by rule (section 6.1): asleep and dozing are still, awake is a slow idle, only comeback is loud. |
| **The pet cannot exist on Wayland**, which deliberately does not let applications position their own windows. | High, Linux only | Accepted rather than solved. Section 11, wall 2. Linux may ship without the pet, or not ship at all, and that is decided deliberately rather than discovered mid-port. |
| **Linux tray absent on GNOME; clipboard unreliable on Wayland.** | Medium | Stated honestly in section 11 and the README. Linux ships last and only after both paths are verified. |

---

## 16. Future Gates (not v1)

**Nothing in this section is planned, committed, or a reason to add abstraction now.** It exists so that deferred directions are named rather than rediscovered as "obvious" mid-implementation, and so that saying no to them is a decision already made.

The distinction that keeps this section from becoming the deprecated ecosystem:

- **Architectural seams are cheap and worth leaving.** The `version` field so a schema migration is possible; states defined as data (a manifest mapping state names to room sheets and quote pools) so a fifth state is content rather than code; the state-derivation function being pure and independently replaceable; Tauri already abstracting the platform. These cost nothing today and are just good structure.
- **Extension points built ahead of need are not.** No plugin system, no config file format, no abstraction layers for hypothetical futures, no interfaces with a single implementation. YAGNI applies, and it applies hardest to a project whose thesis is restraint.

Named, deferred, and undesigned: a fifth state such as a deep-work or on-a-streak room; per-project rooms rather than one room reflecting the newest activity anywhere; alternative characters or seasonal rooms; a light theme, if the dark-only room ever looks wrong on a light desktop; non-git signals, such as another version control system; and a community showcase of shared images, which is the only direction that could ever become the revenue path in section 1.

Any of these requires its own spec and its own argument against the guardrail in section 3.

---

## 17. Open Questions

Decisions the author still needs to settle. Each has a stated working default above, so implementation is never blocked.

Items that have been settled are deleted from this list and recorded in the phase that settled them (section 12) rather than kept here, because a resolved question left in an open-questions list is worse than no list. Removed so far: the awake perspective mismatch, the dozing pose, the placeholder wallpaper and floor, the share canvas proportions, and how the pet's dozing and asleep frames differ.

1. **Whether the cat's coat needs warming.** The pack's cat is drawn in a cool blue-grey (`#8b8bab` and neighbours) and the room is warm oak and beige, so the cat is the coldest thing in it and takes more attention than a cat lying on a rug should. A 10 to 15% warm colorize on that one layer would fix it and is one line in the compositor. Left alone for now, because recolouring pack art is a road that ends in hand-tuned assets, and the week below will say whether it actually bothers anyone.
2. **Comeback duration cap.** Default is "until the popover opens, capped at 30 minutes". Now less consequential than it was, since the pet delivers the moment regardless, so this only governs how long the popover keeps the celebration available. Decide after living with it.
3. **Whether the share artifact should move.** Every state is a 12-frame loop (section 4.1) and the card is built from frame 0, so the thing that travels is a still of a moving room. An animated card would carry the comeback far better than a still can, and the comeback is the moment most likely to be posted. Against it: a PNG is what the clipboard and every chat app accept without negotiation (section 5.1), an animated share path is a second renderer rather than a variation on the first, and a GIF of a dark room is a large file for a small gain in three of the four states. The honest split may be a still by default with the comeback as the single exception, which is also the most complexity for the least code. Not blocking: the still card is complete and shippable as specified. **Section 5.4's demo defuses most of this**, because the case a still handles worst is the transition, and the demo shows the transition properly. Revisit only if the demo lands and the cards visibly fail to.
4. **How much of the demo is reusable, and by whom.** Section 5.4's demo is recorded by the author against a throwaway repository. The obvious next thought is a "record your own" feature, and the answer is no for v1: it is a second product, it needs a settings screen, and it puts the privacy burden (section 5.3) on the user in the one place they are least able to audit it. The question worth keeping open is narrower, whether the demo should be re-recorded per release or shot once and left alone.
5. **Whether any project information ever appears in the shared image.** v1 says no, absolutely. The open sub-question is whether a non-identifying aggregate such as "3 projects" is acceptable later, or whether the image stays purely about the mood.
6. **Whether the comeback gets sound.** Currently no, because a tray app making noise is a fast route to being quit. Revisit only if it can be off by default with no settings screen to turn it on, which probably means the answer stays no.
7. **Quote pool size per state.** Four lines each are drafted. Whether that is enough before repetition grates is something Phase 3 dogfooding will answer.

---

## 18. v1 Definition of Done

v1 ships when all of these are true. Nothing else is required, and nothing else may be added to this list without deleting something from it.

- [ ] One 160x112 room background exists, plus the character frames and emotes the states need.
- [ ] Awake reads clearly as sitting at the desk, and dozing reads as resting rather than distracted.
- [ ] Wallpaper and floor are deliberate choices, not the sampled placeholders.
- [x] Each of the four states animates as a 12-frame loop at its own rate, from a single strip stepped in CSS, with no animation library and no runtime canvas loop.
- [ ] No multi-tile prop is missing its cap, checked by looking at the composed room rather than at the file listing.
- [x] Every asset came from `moderninteriors-win/`, none from `Modern tiles_Free/`.
- [x] `art: limezu.itch.io` appears in the popover, in the share image, and in the README.
- [ ] The author has lived with the four states for a week and still wants them.
- [ ] The desktop pet appears bottom-right at 64x64, above normal windows, showing the correct state, and its behaviour over a fullscreen app is known, recorded, and either working or explicitly accepted.
- [ ] Clicking the pet opens the popover.
- [ ] The pet's dozing and asleep loops are the lowest-amplitude ones in the app, and it does not fidget when awake.
- [ ] Tray icon is a monochrome template image that reads on both light and dark menu bars, and its right-click menu holds Open and Quit.
- [ ] The popover opens from the tray at 352px wide, shows the room at 320x224, a state-appropriate quote, the tracked-project list with relative times and operating toggles, two buttons, and the credit line.
- [ ] Clicking the character cycles through all three, and the choice survives a restart.
- [ ] Add Project opens a native folder picker, validates the repo, and starts watching it.
- [ ] Hover-`x` untracks a project.
- [ ] Each project row has a toggle that marks it operating and excludes it from the mascot's mood.
- [ ] A real `git commit` in a tracked repo updates the pet and the room within about a second.
- [ ] A real file edit in a tracked repo updates the pet and the room within about a second.
- [x] A `git checkout` or `git pull` in a tracked repo does **not** change the state. *Verified against a real repository, not only against fixture lines.*
- [x] Checking out an older branch does not move `last_commit_at` backwards.
- [x] The awake to dozing to asleep transitions happen from time passing alone, verified without restarting the app, under an accelerated clock rather than by waiting three days. *Driven at 3600x: 24.05h and 72.10h.*
- [ ] A demo recording exists that shows the full arc from commit to asleep to comeback, made against the real state machine with only the clock scaled, and watched back for leaked paths before posting.
- [ ] A commit landing while asleep produces the comeback room, which resolves when the popover is opened and survives an app restart.
- [ ] Share Status copies a 1200x630 image to the clipboard that pastes into a real chat app and a real social composer, contains no project name, path, hash, or timestamp, and matches the composition in section 5.2 with the room at exact 5x and the state still identifiable at a 280px feed width. *The composition half is done and measured: zero pixels differ from the compositor's card. The clipboard half is a manual check.*
- [x] State persists across restarts, and a corrupt or missing state file starts the app cleanly rather than failing.
- [ ] The app makes zero network requests, verified.
- [ ] macOS build runs with no dock icon and no app window.
- [x] The Rust unit tests in section 14 pass. *56 of them.*

---

## 19. Decision Summary

- **Purpose:** a fun retro mascot reflecting side-project momentum. Not a productivity tool. Success is users who love it, not revenue, and the non-goals (no monetization, accounts, telemetry, network calls, or hosted anything) are permanent.
- **Guardrail:** if it needs a second process, a second language, or a settings screen, it is not in v1. Section 3 states exactly what v1 ships and section 18 states when it is done.
- **The character is the product.** Roughly 90% of the value is art and personality; the git tracking exists only to give them a mood.
- **The character lives in a tiny room**, built from LimeZu's Modern Interiors, because a scene carries story a lone sprite cannot and makes the share image far stronger. Composed at the pack's 16x16 native grid, **10 by 7 tiles, so 160x112**, at 2x in the popover (320x224) and 5x in the share image (800x560). The room grew from 9 by 6 because at that size it read as empty once real furniture was placed, which is a tested finding. **The popover widened from 320px to 352px** to fit it.
- **The room never changes.** One background, with exactly three variables on top: character position (desk, standing, bed, rug), a flat colour multiply for lighting, and a `Z` or `!` emote. Four states therefore cost a handful of character frames plus two emotes, not four illustrated scenes. There is also a cat on the rug, which is in every state and is not a variable.
- **Every state animates, and amplitude is what separates them.** A 12-frame loop each, at 6 fps awake, 3 dozing, 2 asleep, 8 comeback, shipped as one strip per state and stepped in CSS. This reverses "the background is static": a room where nothing moves reads as a screenshot, and a screenshot is not a companion. The original concern was right about amplitude and it survives intact, aimed at the pet rather than the room, because the pet is the surface that sits in peripheral vision all day.
- **No original character art is needed.** The pack already ships the sleep, sitting, and idle animations plus emote sheets, which is why Phase 1 is composition rather than drawing. Exact source files and crops live in `docs/asset-picks.md`, deliberately kept out of this spec.
- **Seven constraints came from building it** (section 4.1), and four of them are the same lesson: **the pack's file listing and the pack's own transparency both lie about what a sprite is, so every sprite gets looked at pixel by pixel.** The bed must be a vertical single, because the `sleep` animation is a layered recipe of bed plus head plus blanket and side-view beds have no sleeping pose. The character is a separate composited layer, never baked into the room. The pack has no computer desk but does have a computer, inside `animated_receptionist`, seen from behind. Draw order is art: character before desk. A 32x48 bookshelf turned out to be two-thirds of a 48-wide one with the cap in the next file. And row 0 of a character sheet is one pose per facing, so the obvious `x=0` crop is a side view rather than an idle. Sprites are padded with their outline colour at alpha zero, so crop bounds have to be read off the alpha channel rather than judged by eye: the rug stood in the room with its bottom edge sliced off for exactly that reason.
- **Three characters ship in v1** (premades 07, 12, 20), cycled by clicking the character and persisted as `character_id`. This reverses the earlier one-character position: since the character must be a separate layer anyway and every premade sheet carries an identical animation set, three characters cost three PNGs. It is a swap, not a skin system, and it needs no picker UI, so the guardrail holds.
- **The license is settled.** Commercial use is permitted outright. Three binding consequences: **credit to `limezu.itch.io` is mandatory** and therefore a functional requirement (popover footer, share image, README); assets come from `moderninteriors-win/` only, never the non-commercial `Modern tiles_Free/`; and raw assets stay out of version control, which is why `docs/mockups/` is already gitignored, though shipping them compiled into a binary is ordinary permitted use.
- **The mascot never dies. It waits.** The previous spec's `Dead` state at 72h is rejected as guilt-ware aimed at the exact user being targeted. Four states: awake (<24h), dozing (24h to 72h), asleep (>=72h), and comeback on `asleep -> awake`.
- **The comeback is the emotional payload, and the pet solves it.** Earlier drafts had to accept that a popover-only design cannot guarantee the celebration is ever seen. The pet delivers it in peripheral vision the moment the commit lands, with no banner and no click. The 30 minute cap and popover-open resolution remain as the mechanism for settling back to `awake`, but nothing is owed if the popover is never opened.
- **Two real signals count: commits and working-tree edits.** The reflog is scanned backwards and filtered to messages beginning with `commit`; checkout, pull, fast-forward merge, reset, clone, and rebase replays are ignored, so a comeback can never fire because someone checked out a branch. Working-tree changes are filtered through the project's own `.gitignore`. `last_commit_at` and `last_active_at` never decrease. The reflog entry's timestamp is used rather than committer time, because amend and rebase rewrite committer time while the reflog records when the user acted.
- **Tone:** warm, encouraging, a little snarky, never guilt-inducing. No elapsed-time shaming anywhere.
- **There are two share artifacts, and conflating them was this spec's biggest mistake to date** (section 5). **The demo** is a 20 to 30 second timelapse the author records once, and it is what *explains* the product, because the product is a transition and no still frame can show one. **The card** is the 1200x630 image users copy from the app, and it carries mood and the URL to people who already know what they are looking at. A card posted cold explains nothing, which is why the discovery job could never have been its own. Validation stays with the card alone: a video the author posts once is a claim, and a thousand cards posted by users is evidence.
- **The card is composed** (section 5.2), on a 1200x630 canvas with the room letterboxed at integer scale, because a 10:7 room in a 1.91:1 crop gets a mat rather than fractional scaling. Building it produced a rule worth keeping: **anything drawn on the room matches the room's pixel unit, and anything drawn on the mat is free.** The label obeys the first half at 5x, the quote the second at 2x, and the quote had to move off the room entirely to get there, because the composed room has no quiet strip left in it. The typeface is **Departure Mono** under the OFL, chosen on measured widths rather than taste (section 6.4).
- **The app is built, and building it corrected this spec in five places** (section 12, Phase 3). Two of them are worth repeating here because they are the kind of mistake that looks like a detail and is not. **An accelerated clock has to be a timeline, not a faster `now`:** scaling only the current time left every incoming commit stranded on the real one, so a commit made during a timelapse was born seventy-two hours old and the comeback never fired. And **not everything measured in time belongs on that timeline:** the comeback's 30 minute cap is a dwell time on a piece of UI, and scaling it meant the celebration lasted half a real second, in the very recording made to show it. Both were found by running the real state machine and watching it, which is the entire argument for having built the clock in the first place.
- **The share card is verified rather than trusted.** The app's canvas card and the compositor's card differ by **zero pixels** at 1200x630, so Phase 2's composition is what ships rather than something that resembles it. The check found a real bug on the way: the card composed silently in a fallback serif when the `@font-face` rule was absent, which is hard-edged, correctly laid out, and entirely the wrong typeface. A module that draws type now owns the type it draws and refuses to compose in anything else.
- **The demo needs an accelerated clock, and so do the tests.** The real thresholds are 24 and 72 hours, so a timelapse is not optional and neither is verifying the transitions without waiting three days. Because section 8.1 already makes state derivation a pure function of injected time, this costs one parameter and a debug-only scale factor rather than a feature. Two unrelated needs landing on the same small seam is the sign the seam was in the right place.
- **Four surfaces, with distinct jobs.** The **desktop pet** (always visible, carries state ambiently), the **tray icon** (plumbing: opens the popover, holds Quit), the **popover room** (the reward, on demand), and the **share image** (the audience). The pet exists because 90% of the value is art, and a popover-only design leaves that art behind a click that happens twice a day.
- **The pet is a 64x64 always-on-top window**: the 32x32 character at 2x, character only and never the room, fixed bottom-right with its position persisted, and clickable rather than click-through so it becomes the primary way in. **Motion is reserved on the pet:** the lowest-amplitude loops when dozing and asleep, a slow idle when awake, loud only on comeback, because a sprite that moves constantly and largely in peripheral vision is the thing people quit.
- **The pet is a second window, and that out-of-scope row was reversed honestly.** The ban is now narrow: no second window *for UI chrome* (about, onboarding, stats, settings). The pet is not chrome, it is the primary ambient surface. The guardrail itself is unchanged: still no second process, no second language, no settings screen.
- **One art gap:** the pack has no sleeping character without a bed, so the pet uses a seated pose with a `Z` for both dozing and asleep, and **the bed stays exclusive to the room**. A deliberate split of registers: the pet is the character, the room is the scene.
- **Tray icons revert to monochrome template images**, reversing the earlier full-colour decision. That decision existed only because four states could not be told apart as one-bit silhouettes at 16x16. With the pet carrying state, the icon no longer encodes state at all, so the simpler and better-behaved option wins.
- **The macOS fullscreen wall was the biggest threat to the pet, and it is solved.** Spiked before any other pet work, because a negative answer would have changed the art requirements. No `NSWindow` configuration works, at any level; converting the window to a **non-activating `NSPanel`** does, and clicking it does not disturb the fullscreen app. This is the design's **one platform-specific exception** (section 10.3): about fifteen lines of macOS-only `objc2`, hand-rolled rather than taking `tauri-nspanel`, set once at window creation and never adjusted.
- **The remaining platform wall is Wayland**, where applications cannot position their own windows, so the pet is essentially not implementable and Linux may ship without it or not at all. Accepted rather than solved.
- **Stack:** Tauri v2, one codebase, Rust backend plus webview UI, one local JSON file with atomic writes and parsing resilient to missing fields and empty arrays.
- **Rejected:** the Rust CLI plus separate Swift menu bar app with a JSON handshake and a `keepgoing://` scheme, which pays for two languages now and still forces a rewrite at the Windows boundary. CLI, IPC, and URL scheme are deleted. Also rejected: git hooks in user repositories, which collide with husky and lefthook and leave an uninstall problem.
- **Future directions are named but undesigned.** Architectural seams stay; extension points built ahead of need do not.
- **Phasing:** compose and animate the four states (done), design the share card (composed), build the macOS app, **record the demo**, post the demo and then a card, add Windows as a build target, then Linux if asked with an honest `StatusNotifierItem` caveat. The demo gets its own phase number rather than being a task inside the app phase, because it is the highest-leverage artifact in the plan and the one most likely to be skipped under momentum. Phase 1 is gated on the author living with the rooms for a week, which is the only gate here that cannot be rushed.
