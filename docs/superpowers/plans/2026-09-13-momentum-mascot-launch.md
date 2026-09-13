# Momentum Mascot Launch Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Put Momentum Mascot in front of the people who would want it, across Reddit, daily.dev, Show HN and a set of evergreen listings, over roughly three weeks.

**Architecture:** Four phases. Phase 0 does everything that has a lead time or must not be rushed, while nothing is public. Phase 1 spends the warm Reddit account one subreddit per day. Phase 2 publishes one article that feeds dev.to and daily.dev. Phase 3 spends the single Show HN, last, once the account has history. Phase 4 never ends.

**Tech Stack:** No code ships here. The tooling is `gh` for baselines, `bun run crosspost:devto` in the hoatrinh.dev repository, and a private browser window as the test harness.

**Spec:** `docs/superpowers/specs/2026-09-13-momentum-mascot-launch-design.md`

## Global Constraints

- **Nothing in this plan runs until `docs/superpowers/plans/2026-09-13-keepgoing-dev-site-split.md` is complete and `tools/verify-site.sh https://keepgoing.dev` passes.** Every link here points at `https://keepgoing.dev/momentum-mascot`.
- **macOS only, stated in the first line of every post.** It is a hard reach ceiling and the alternative is buying clicks from people who are then annoyed, in exactly the communities worth being in.
- **One Show HN.** There is no second attempt for this project.
- **Never the same text twice.** Reddit's sitewide heuristics fire on one URL across several subreddits in a short window, and the post does not fail loudly: it stops being visible to others while still looking normal to the person who posted it.
- **Every post gets a logged-out visibility check.** This is the test step, and it is the only way to catch a silent removal.
- **The article is first-person and about the author's own experience.** A subagent drafts structure and technical passages; it does not ghostwrite the personal ones. The health angle recorded in Commitropolis' spec is explicitly excluded here: that spec marks it the author's call alone.
- **No em dashes** in any copy. Use a hyphen or rewrite.
- **Answer every comment.** On Show HN the author in the thread matters more than the post, and across all channels the comments are the only free customer development this gets.

---

## Phase 0: before anything is public (days 1 to 7)

### Task 1: Record the baseline before anything moves

**Files:**
- Create: `docs/launch-log.md`

**Interfaces:**
- Consumes: nothing.
- Produces: `docs/launch-log.md`, the file every later task appends to.

Without a number written down before the first post, the launch produces a graph with no baseline and the whole point of notarizing first is lost.

- [ ] **Step 1: Pull the numbers that exist**

```bash
gh api repos/keepgoing-dev/momentum-mascot --jq '{stars: .stargazers_count, forks: .forks_count, watchers: .subscribers_count}'
gh api repos/keepgoing-dev/momentum-mascot/releases --jq '[.[] | {tag: .tag_name, downloads: [.assets[].download_count] | add}]'
```

- [ ] **Step 2: Pull the number that has been sitting unused**

In App Store Connect, open Analytics for Momentum Mascot and record **total units to date**, **impressions**, and the **traffic source** breakdown. The app sends nothing; the store counts downloads. This is a real per-channel signal and it has never been read.

- [ ] **Step 3: Write the log**

Create `docs/launch-log.md`:

```markdown
# Launch log

Three numbers and a list, weekly. The app has no telemetry and never will, so everything
here comes from stores, GitHub and the site's own server.

## Baseline, recorded before the first post

| Instrument | Value | Date |
|---|---|---|
| GitHub stars | | |
| GitHub release downloads, all tags | | |
| App Store units to date | | |
| App Store impressions to date | | |

## Weekly

| Week ending | Stars | Release downloads | App Store units | Site visits | itch follows | Notes |
|---|---|---|---|---|---|---|
```

Fill the baseline table from Steps 1 and 2.

- [ ] **Step 4: Commit**

```bash
git add docs/launch-log.md
git commit -m "Write down the numbers before they can move"
```

---

### Task 2: Submit hoatrinh.dev to daily.dev as a Source

**Files:**
- Modify: nothing. This is a submission on daily.dev.

**Interfaces:**
- Consumes: the existing feed at `https://hoatrinh.dev/rss.xml`.
- Produces: an approved Source, so Task 15's article syndicates without a cold link submission.

**This is the first task with a clock on it.** Source review takes days. Everything it needs already exists and was verified: `apps/web/scripts/build-rss.ts` emits `<content:encoded>` with the full post body rather than an excerpt, `apps/web/index.html:7` declares the feed for auto-discovery, and sixteen posts give the review real history to screen. A feed with one item in it is what gets rejected.

- [ ] **Step 1: Confirm the feed is live and full-content**

```bash
curl -fsS https://hoatrinh.dev/rss.xml | grep -c "<item>"
curl -fsS https://hoatrinh.dev/rss.xml | grep -c "content:encoded"
```

Expected: both at least 16, and equal. If `content:encoded` is lower than `<item>`, some posts are syndicating as excerpts and daily.dev will render them badly.

- [ ] **Step 2: Submit**

Go to `daily.dev`, open Source submission, and submit `https://hoatrinh.dev`. Auto-discovery finds `/rss.xml` from the `<link rel="alternate">` tag, so the domain is enough.

- [ ] **Step 3: Record it**

Append to `docs/launch-log.md` under a new `## Channel status` heading: the submission date and the expected review window. Do not proceed to Phase 2 assuming approval; Task 15 has a fallback.

---

### Task 3: Warm the Hacker News account

**Files:**
- Modify: nothing.

**Interfaces:**
- Consumes: nothing.
- Produces: an account with comment history by day 14, when Task 16 spends the single Show HN.

A Show HN from a cold account is a coin flip: new accounts are ranked down and flagged quickly, and there is no reputation to absorb the "solution in search of a problem" reply that `docs/launch-copy.md` already predicts. This task runs in the background across the whole of Phase 0 and Phase 1.

- [ ] **Step 1: Create the account if it does not exist**

Register at `news.ycombinator.com`. Use a handle that will still be right in five years.

- [ ] **Step 2: Comment, honestly, with no links, most days**

Two or three substantive comments a day on threads about things actually known: Rust, Tauri, macOS behaviour, pixel art, side projects, licensing. Not on threads about this project or anything adjacent to promoting it.

**Never post a link to keepgoing.dev during this task.** A new account whose early history is link drops is the exact pattern the ranking penalises.

- [ ] **Step 3: Check the account has karma before Task 16**

Open `news.ycombinator.com/user?id=<handle>` and confirm karma is above 1 and the account is at least 14 days old. If it is not, Task 16 slips rather than proceeding.

---

### Task 4: Create the itch.io Follow surface

Covered by Task 6 of `docs/superpowers/plans/2026-09-13-keepgoing-dev-site-split.md`, which creates the account, publishes the public project page, and wires `/commitropolis` to it.

- [ ] **Step 1: Confirm it is done and public**

```bash
curl -fsS https://keepgoing.dev/commitropolis | grep -o 'https://[a-z0-9-]*\.itch\.io/commitropolis'
```

Expected: the itch URL, three times. Open it in a private window and confirm the Follow button is visible to a logged-out stranger. A restricted page has no Follow button and the whole point of publishing early is lost.

---

### Task 5: Rebuild and upload the GitHub social preview card

**Files:**
- Create: `docs/mockups/share-comeback-1280x640.png` (gitignored output)

**Interfaces:**
- Consumes: a licensed pack at `$MASCOT_PACK`.
- Produces: a social preview on the repository, so a shared repo link unfurls as art rather than as a wall of text.

`fcd4aaa` stopped committing this card, so it has to be regenerated. GitHub unfurls its own social preview rather than the site's card whenever the repo link is shared, which happens on r/rust and r/opensource by design.

- [ ] **Step 1: Compose the card**

```bash
cd ~/Workspace/KeepGoing.dev/momentum-mascot
MASCOT_CHAR=07 tools/compose-rooms.sh
MASCOT_CW=1280 MASCOT_CH=640 tools/compose-share.sh
```

Character 07 is deliberate: it matches the site's card, so the repository and the site unfurl the same person.

- [ ] **Step 2: Check the dimensions**

```bash
sips -g pixelWidth -g pixelHeight docs/mockups/share-comeback-1280x640.png
```

Expected: 1280 by 640. GitHub's minimum is 640 by 320, which is why the four 600x315 cards in the site's `assets/share/` cannot stand in.

- [ ] **Step 3: Upload it**

Repository Settings, General, Social preview, Edit, Upload. There is no API for this.

- [ ] **Step 4: Verify the unfurl**

Paste `https://github.com/keepgoing-dev/momentum-mascot` into a private Slack or Discord message and confirm the comeback room appears rather than the README text.

---

### Task 6: Prove the install is clean from a machine that never built it

**Files:**
- Modify: nothing.

**Interfaces:**
- Consumes: the published v0.4.0 DMG.
- Produces: certainty that the download does not dead-end.

Every post in this plan sends people to a download, and an install that dead-ends in a Gatekeeper dialog spends attention that cannot be recovered. On macOS Sequoia and later, Control-click then Open no longer bypasses Gatekeeper, so a signing problem is not something the user can work around.

- [ ] **Step 1: Download and install on a different Mac**

Use a machine that has never run `tools/release.sh` and has no developer certificates. Download `Momentum Mascot.dmg` from the Releases page, open it, drag to Applications, and launch.

Expected: it opens with **no Gatekeeper warning and nothing to run in Terminal**.

- [ ] **Step 2: Verify the notarization ticket is stapled**

```bash
spctl -a -vvv -t install "/Applications/Momentum Mascot.app"
```

Expected: `accepted` and `source=Notarized Developer ID`.

- [ ] **Step 3: Open the Support menu item**

Right-click the menu bar icon, choose Support. Confirm the browser lands on `keepgoing.dev/#support` and that the support section is visible. This is the same check as the site plan's Task 7 Step 6 and it is worth doing twice, from a real install, because nothing in the app reports it being broken.

---

### Task 7: Email LimeZu

**Files:**
- Modify: nothing.

**Interfaces:**
- Consumes: the live site.
- Produces: a message to the artist whose pack the project is built on.

Highest ratio of likely return to effort in this plan, and it appears on no channel list anywhere else. Their audience is precisely the people who would enjoy this, and the licence already requires crediting them.

- [ ] **Step 1: Send it**

To the contact address on `limezu.itch.io`. Subject: `Modern Interiors shipped in a notarized Mac app`.

```
Hi,

I built a small macOS app called Momentum Mascot on top of Modern Interiors. It is a pixel
character who lives in a room on your desktop and reflects how your side projects are going:
at their desk when you have been committing, dozing after a day, asleep after three, and
leaping out of bed when you come back.

It is free, it is on the Mac App Store, and the credit to limezu.itch.io is on the site, in
the app and in the repository.

On the licence: the composed art is not in version control at all. The repository holds the
coordinates that composite the rooms and the builder layers, and anyone building it brings
their own copy of the pack. That seemed like the arrangement that most clearly respected
"ship it compiled, do not redistribute the assets".

https://keepgoing.dev/momentum-mascot

Thank you for the pack. It is the reason this thing looks like anything at all.

Hoa
```

- [ ] **Step 2: Record it**

Append the date to the `## Channel status` section of `docs/launch-log.md`.

---

### Task 8: Write the article

**Files:**
- Create: `~/Workspace/Personal/hoatrinh.dev/packages/content/markdown/blog/the-mascot-never-dies-it-waits.md`

**Interfaces:**
- Consumes: nothing.
- Produces: the article Task 15 publishes and Task 16 points at in the Show HN thread.

This is the single highest-leverage asset in the plan. It is daily.dev's payload, dev.to's payload, and the thing that makes a two-week-old HN account look like a person rather than a drive-by.

**The spine is: the feeling produced the constraints.** Refusing to guilt the user is why there are no streaks. Refusing to leak is why there is no network layer. Refusing to redistribute is why the art is not in the repository. One argument, and a reader who came for either end arrives at the same place. A post that is only the feeling reads as a toy; a post that is only the engineering sells a build system to people who wanted a mascot.

- [ ] **Step 1: Write it, roughly 1200 to 1800 words, in this shape**

**Opening, the problem, first person.** Side projects going quiet, and the guilt about the quiet being worse than the quiet. Every tool tightening that loop: streaks to break, graphs going grey, a number counting up. The author's own words; a subagent does not draft this paragraph.

**The refusal, stated as a rule.** No streaks, no scores, no notifications, and nothing that will ever say how long it has been. The mascot does not die, it waits.

**Then three constraints, each shown to follow from the refusal:**

1. *No network layer.* Not "no telemetry yet". There is no HTTP client compiled in, so the promise is a property of the binary rather than of the author's intentions. State lives in one readable JSON file at `~/.keepgoing/mascot/state.json`.
2. *A share card that cannot leak.* Drawn in a canvas at 1200x630 and copied to the clipboard. It is built so a project name, path, commit message, hash or timestamp **cannot be expressed on it**, rather than relying on remembering not to put them there. That distinction is the interesting part.
3. *The art is not in the repository.* LimeZu's Modern Interiors permits shipping the art compiled into an application and forbids redistributing it as assets. So the repository commits the coordinates that composite the rooms, and you bring your own copy of the pack. This turned out tidier than expected: the MIT licence on the code then cannot accidentally speak for art that is not the author's to give away.

**What it reads, precisely.** The reflog of the repositories you point it at, and exactly one thing from each: when you last actually committed. Four states, 24 and 72 hours. Checking out a branch or pulling does not count and cannot trigger the comeback.

**Limitations, before anyone else raises them.** macOS only. Does not start at login yet. Linux is genuinely uncertain because Wayland does not let an application position its own window, which makes the desktop pet close to unimplementable there.

**Close.** The mascot never dies, it waits, and every other decision followed from that.

- [ ] **Step 2: Add the frontmatter, matching the schema exactly**

`packages/content/src/schema.ts:52` defines it. `excerpt` is capped at **160 characters** and `tag` is a single existing value (`ai`, `dev`, `habits`, `life`, `tools`). `tags` is what dev.to receives and is capped at four.

```markdown
---
slug: the-mascot-never-dies-it-waits
title: The mascot never dies. It waits.
date: 2026-09-24
excerpt: I built a desktop pet that refuses to guilt you. That refusal is why it has no network layer and no art in its repository.
tag: dev
tags: [macos, rust, tauri, opensource]
crosspost: true
---
```

Set `date` to the actual publication date. `crosspost: true` is what opts the post into Task 15's dev.to run.

- [ ] **Step 3: Check the excerpt length before the build rejects it**

```bash
cd ~/Workspace/Personal/hoatrinh.dev
awk -F': ' '/^excerpt: /{print length($2)}' packages/content/markdown/blog/the-mascot-never-dies-it-waits.md
```

Expected: 160 or below.

- [ ] **Step 4: Build and look at it**

```bash
bun run typecheck && bun run build && bun run preview
```

Expected: a clean build, and the post rendering at `/post/the-mascot-never-dies-it-waits`. A Zod validation failure here means the frontmatter is wrong, and the message names the field.

- [ ] **Step 5: Commit, but do not deploy yet**

```bash
git add packages/content/markdown/blog/the-mascot-never-dies-it-waits.md
git commit -m "Write up why the mascot has no network layer"
```

Publication is Task 15, in Phase 2, deliberately after Reddit.

---

## Phase 1: Reddit, one subreddit per day (days 8 to 11)

Each task below is one day. They share a shape, and each has its own body because posting the same text twice is what triggers the filter.

**Start on a Tuesday or Wednesday, around 9am US Eastern.**

### Task 9: r/macapps

**Interfaces:**
- Consumes: the live site.
- Produces: the first public post.

- [ ] **Step 1: Read the subreddit's rules page and note the required flair**

`[Free]` is the flair this post needs. Rules change; read them the morning of.

- [ ] **Step 2: Post the body already drafted**

The body is in `docs/launch-copy.md` under the `## r/macapps` heading, and is used from there rather than duplicated into this plan, so that there is one copy of it to keep correct.

Two substitutions before posting:

- `Download: <link>` becomes `Download: https://keepgoing.dev/momentum-mascot`
- `Source: <link>` becomes `Source: https://github.com/keepgoing-dev/momentum-mascot`

Title, which differs from `docs/launch-copy.md` by one character: the en dash there becomes a
hyphen, for consistency with every other title in this plan.

```
[Free] Momentum Mascot - a pixel character who lives on your desktop and dozes off when your side projects go quiet
```

- [ ] **Step 3: The test. Check it is visible to somebody who is not you**

Open a private browser window, logged out, and load both:

1. The post's own permalink.
2. `https://old.reddit.com/r/macapps/new/`

Expected: the post appears in **both**. A removed or filtered post still renders on its permalink for its author while being absent from `/new` for everyone else, which is why one check is not enough.

If it is missing from `/new`, message the moderators rather than deleting and reposting. A repost compounds the problem.

- [ ] **Step 4: Stay in the thread**

Answer every comment for the next few hours.

- [ ] **Step 5: Log it**

Append the post URL and the day's upvote count to `docs/launch-log.md`.

---

### Task 10: r/SideProject

- [ ] **Step 1: Post**

Title:

```
I kept abandoning side projects, so I built a desktop pet that refuses to guilt me about it
```

Body:

```
macOS only, free, open source, no accounts and no network requests.

The abandoning was never really the problem. The feeling bad about the abandoning was the
problem, and every tool I tried made that loop tighter: streaks to break, graphs going grey,
a number counting up since my last commit.

So this one refuses to do any of that. It watches the git repos you point it at and reads
exactly one thing from each: when you last actually committed. A pixel character sits in a
small room on your desktop and is at their desk if you have been working, dozing after a day,
asleep after three, and leaps out of bed when a real commit lands after a sleep.

No streaks, no scores, no notifications, and nothing in it will ever tell you how long it has
been. The mascot does not die. It waits.

There is a 64x64 desktop pet, the full room in a menu bar popover, a builder for making your
own character, and a share card it copies to your clipboard that carries the mood and nothing
that identifies a project.

It has been on the Mac App Store since the end of August. Happy to answer anything.

https://keepgoing.dev/momentum-mascot
```

- [ ] **Step 2: The test**

Private window, logged out: the permalink **and** `https://old.reddit.com/r/SideProject/new/`. Both must show it.

- [ ] **Step 3: Answer comments, then log the URL and score**

---

### Task 11: r/rust

The engineering hook leads here, and the link is the repository rather than the site.

- [ ] **Step 1: Post**

Title:

```
Momentum Mascot: a Tauri 2 menu bar app with a desktop pet that had to become an NSPanel to work over fullscreen apps
```

Body:

```
macOS only. Free, MIT, and on the Mac App Store.

It is a desktop pet that reflects how your side projects are going by reading the reflog of
the repos you point it at. That part is straightforward. Three things were not:

**Fullscreen.** No NSWindow level makes a window visible over another app's fullscreen space.
The pet is a 64x64 always-on-top window and it simply was not there when you were in a
fullscreen editor. Swapping the class to NSPanel with the right collection behaviour is what
cleared it, and no amount of level tuning would have.

**No network layer at all.** Not "no telemetry", there is no HTTP client compiled in, which
means the privacy claim is a property of the binary rather than of my intentions. State is one
JSON file you can read.

**Art the licence will not let me commit.** LimeZu's Modern Interiors permits shipping the art
compiled into an application and forbids redistributing it as assets, so the repo commits the
coordinates that composite the rooms and you bring your own copy of the pack. The MIT licence
on the code then cannot accidentally speak for art that is not mine to give away.

Universal build, around 10MB. Two things it does not do: start at login, and run on Linux,
because Wayland does not let an app position its own window and the pet is close to
unimplementable there.

https://github.com/keepgoing-dev/momentum-mascot
```

- [ ] **Step 2: The test**

Private window: permalink **and** `https://old.reddit.com/r/rust/new/`.

- [ ] **Step 3: Answer comments, then log**

---

### Task 12: r/opensource

The licensing arrangement is the lede.

- [ ] **Step 1: Post**

Title:

```
My repo commits the coordinates that generate the art, not the art, because that is what the asset licence actually permits
```

Body:

```
Momentum Mascot is a small free macOS app: a pixel character on your desktop who reflects how
your side projects are going. MIT, no network layer, no accounts.

The part worth posting here is the licensing. The art is LimeZu's Modern Interiors, whose
licence permits shipping it compiled into an application and forbids redistributing it as
assets. Committing the composed rooms to a public repo would be redistribution.

So the repository holds the coordinates and the compositing scripts, and the art is generated
at build time from a pack you supply yourself. Nothing under the art directories is in version
control. A separate doc records exactly where the line sits: composited rooms and finished
cards may go on a public URL, layer strips and colour swatches may not.

The effect I did not expect is that it makes the MIT licence on the code honest. The permissive
licence now covers only things I actually have the right to give away, instead of quietly
appearing to cover art that is not mine.

https://github.com/keepgoing-dev/momentum-mascot
```

- [ ] **Step 2: The test**

Private window: permalink **and** `https://old.reddit.com/r/opensource/new/`.

- [ ] **Step 3: Answer comments, then log**

---

### Task 13: r/pixelart, and it sells nothing

**No download link in the body.** This subreddit reacts badly to promotion and well to craft, and a link is the difference between the two.

- [ ] **Step 1: Post the rooms as art**

Title:

```
Four times of day in one tiny room, built from LimeZu's Modern Interiors
```

Body:

```
Art is LimeZu's Modern Interiors (limezu.itch.io), composited into four moods of the same
room: awake at the desk, standing away with coffee, asleep, and leaping out of bed.

Everything but the awake frame applies a colour pass over the whole finished frame, character
included, which is what sells the time of day more than the furniture does. The character hops
position per frame rather than per mood, so the coffee and the emotes had to hop with them.

Type is Departure Mono by Helena Zhang.
```

Attach the four room PNGs directly as an image post.

- [ ] **Step 2: The test**

Private window: permalink **and** `https://old.reddit.com/r/pixelart/new/`.

- [ ] **Step 3: If asked what it is for, answer in a comment**

Only then does the link belong in the thread, as a reply to a real question.

---

### Task 14: r/desktops

- [ ] **Step 1: Post a screenshot**

Title:

```
A 64x64 pixel roommate who dozes off when my side projects do
```

Body:

```
macOS, free. It reads when I last committed to the repos I have pointed it at and picks one of
four moods. That is the whole thing: no streaks, no counter, nothing that tells me how long it
has been.

https://keepgoing.dev/momentum-mascot
```

Attach one screenshot of the pet in situ on a real desktop.

- [ ] **Step 2: The test**

Private window: permalink **and** `https://old.reddit.com/r/desktops/new/`.

- [ ] **Step 3: Answer comments, then log**

---

## Phase 2: the article (days 12 to 13)

### Task 15: Publish, cross-post, and syndicate

**Files:**
- Modify: `~/Workspace/Personal/hoatrinh.dev/packages/content/markdown/blog/the-mascot-never-dies-it-waits.md`

**Interfaces:**
- Consumes: the article from Task 8 and the Source from Task 2.
- Produces: the canonical URL Task 16 points at from the Show HN thread.

- [ ] **Step 1: Deploy the post**

```bash
cd ~/Workspace/Personal/hoatrinh.dev
git push
```

- [ ] **Step 2: Confirm it is live and in the feed**

```bash
curl -fsS https://hoatrinh.dev/post/the-mascot-never-dies-it-waits | grep -q "<title>" && echo "post live"
curl -fsS https://hoatrinh.dev/rss.xml | grep -c "the-mascot-never-dies-it-waits"
```

Expected: `post live`, and a count of at least 1. If the feed does not carry it, daily.dev will never see it and Step 4 is pointless.

- [ ] **Step 3: Cross-post to dev.to, dry run first**

```bash
bun run crosspost:devto --dry-run
```

Expected: a summary showing one `create` for this slug. If it shows `skip:opt-out`, `crosspost: true` is missing from the frontmatter.

Then for real:

```bash
bun run crosspost:devto
```

- [ ] **Step 4: Verify the canonical actually rendered on dev.to**

```bash
curl -fsS https://dev.to/<handle>/the-mascot-never-dies-it-waits | grep -o '<link rel="canonical" href="[^"]*"'
```

Expected: `https://hoatrinh.dev/post/the-mascot-never-dies-it-waits`. Without it, dev.to outranks the original for its own article and the audience accrues there instead.

- [ ] **Step 5: Check daily.dev picked it up**

If the Source from Task 2 was approved, the post appears in the feed within a few hours of the RSS update. Search daily.dev for the title.

**Fallback if the Source is still under review:** submit the post URL as a community link. It performs worse, and it is still better than nothing.

- [ ] **Step 6: Post the X thread and the Mastodon version**

The four-part thread is drafted in `docs/launch-copy.md` under `## X thread`, used from there rather than duplicated. Change the closing `<link>` to `https://keepgoing.dev/momentum-mascot` and add a fifth post linking the article.

The same content goes to Mastodon (fosstodon) and Bluesky as single posts with the GIF attached. Neither gates on account age the way X's ranking effectively does, so for a cold account they are the better of the three.

- [ ] **Step 7: Log the URLs**

---

## Phase 3: the single Show HN (days 14 to 15)

### Task 16: Show HN

**Interfaces:**
- Consumes: the warmed account from Task 3, the article from Task 15, and whatever Phase 1 taught about which hook landed.
- Produces: the one Show HN this project gets.

- [ ] **Step 1: Check the gate before posting**

Open `news.ycombinator.com/user?id=<handle>`. The account must be at least 14 days old with karma above 1. **If it is not, this task slips.** It does not proceed at reduced odds, because there is no second attempt.

- [ ] **Step 2: Post, Tuesday to Thursday, 8 to 10am US Eastern**

Title and first comment are drafted in `docs/launch-copy.md` under `## Show HN`, used from there so there is one copy to keep correct. Pick the title based on what Phase 1 showed: the cute framing if r/macapps and r/SideProject outperformed, the technical framing if r/rust and r/opensource did.

**URL:** `https://keepgoing.dev/momentum-mascot`

Post the first comment immediately after submitting, and add one line to it linking the article as the long version.

- [ ] **Step 3: The test. Check it is not dead**

Logged out, in a private window:

1. `https://news.ycombinator.com/newest` - the story should be there.
2. `https://news.ycombinator.com/show` - it should appear here too.
3. The item page itself, checking for a `[flagged]` or `[dead]` marker next to the title.

A killed submission is invisible to logged-out users while looking completely normal to its author. If it is dead, email `hn@ycombinator.com` rather than resubmitting.

- [ ] **Step 4: Four hours at the keyboard**

On Show HN the author answering in the thread matters more than the post. Two objections are expected and both are answerable rather than arguable:

- *"Solution in search of a problem"* or *"just look at your commit history"*. Do not argue the premise. The honest answer is that it is a toy that happens to be kind, and it exists because of the guilt loop rather than because of the information.
- *The closed-source art arrangement.* Answer plainly. It is the most interesting engineering decision in the repository and it reads well explained rather than defended.

- [ ] **Step 5: Log the URL, the peak rank and the final score**

---

## Phase 4: evergreen, one item a week, indefinitely

### Task 17: The listings that compound

**Interfaces:**
- Consumes: the live site.
- Produces: durable traffic long after the launch stops being a launch.

This is where a free macOS app quietly gets most of its lifetime installs. None of it is urgent and all of it is permanent.

- [ ] **Step 1: `awesome-mac`**

Fork `jaywcjlove/awesome-mac`, add an entry under the developer tools section, open a pull request. Read `CONTRIBUTING.md` first: the entry format and the ordering rules are enforced and a malformed entry is closed rather than fixed.

- [ ] **Step 2: `awesome-tauri`**

Fork `tauri-apps/awesome-tauri`, add it to the applications list, open a pull request.

- [ ] **Step 3: `awesome-macos-apps`**

Same shape. Check the repository is still maintained before spending the time; abandoned awesome lists send nothing.

- [ ] **Step 4: AlternativeTo**

Create the listing. Fill in the macOS platform, the free licence, and the tags. Add it as an alternative to the streak-and-graph tools it is deliberately not, which is how people actually find it.

- [ ] **Step 5: Tauri showcase and Discord**

Submit to Tauri's showcase and post once in their Discord's showcase channel. They actively welcome shipped applications and this is a shipped, signed, store-approved one.

- [ ] **Step 6: Verify each landed**

A pull request is not a listing until it is merged. Check each one a week later and record in `docs/launch-log.md` which are live.

---

### Task 18: Keep the log

- [ ] **Step 1: Every week, fill one row**

Stars, release downloads, App Store units, site visits by path, itch follows. Plus every comment or email from a human, which is the only qualitative instrument there is and the only free customer development this gets.

- [ ] **Step 2: Read GitHub's referrers during the launch, not after**

```bash
gh api repos/keepgoing-dev/momentum-mascot/traffic/popular/referrers
gh api repos/keepgoing-dev/momentum-mascot/traffic/views
```

**GitHub keeps only a rolling 14-day window.** Run this during Phase 1 and again during Phase 3. Waiting until the launch is over loses the data permanently, and this is the single easiest thing in the plan to forget.

- [ ] **Step 3: Commit the log**

```bash
git add docs/launch-log.md
git commit -m "Record week N"
```
