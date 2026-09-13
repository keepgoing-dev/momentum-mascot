# Momentum Mascot: launch and distribution

**Date:** 2026-09-13
**Status:** design approved in brainstorming. Ready to plan.

**Depends on** `2026-09-13-keepgoing-dev-site-split-design.md`. Every link here points at
`keepgoing.dev/momentum-mascot`, which does not exist yet. Nothing in this document may be
executed before that one is finished and verified.

**Extends** `docs/launch-copy.md`, which holds drafted copy for Show HN, r/macapps and an X
thread, plus an order of operations gated on "once the notarized build is live". That gate has
passed: 0.4.0 is notarized on GitHub Releases and the app has been on the Mac App Store since
31 August 2026. The copy survives. The order of operations in it does not, for the reason in
section 3.

## 1. What this launch is for

Two goals, chosen deliberately and in tension:

- **Installs.** The largest number of Macs running it.
- **An audience for what comes next.** People who follow the author rather than the product,
  because Commitropolis ships after this and a third thing after that.

Revenue is not a goal. The app is free on both channels
(`docs/app-store-listing.md:23`) and will stay free.

## 2. The constraints that shape everything

- **Reddit is warm. Everything else is cold.** The Reddit account has real history outside the
  promotional subs. The Hacker News and X accounts do not. This inverts the conventional launch
  order and is argued in section 3.
- **macOS only.** Stated in the first line of every post, every time. It is a hard ceiling on
  reach, and the alternative to stating it is buying clicks from people who are then annoyed,
  which costs goodwill in exactly the communities worth being in.
- **The app has no telemetry and never will.** That is a product promise, not a gap to be
  closed. It shapes measurement, in section 7, rather than preventing it.
- **One Show HN.** There is no second attempt for the same project.

## 3. The hook is split by channel, and the article carries both

Two hooks are available and they do not reach the same person.

**The feeling.** Side projects go quiet, every tool makes that worse, this one refuses to.
Streaks to break, graphs going grey, a number counting up since the last commit: none of it is
here, by construction. This is the strongest sentence the project has and it works on Reddit and
on X.

**The engineering.** A repository that commits the *coordinates* that composite the art rather
than the art, because LimeZu's licence permits shipping it compiled into an application and
forbids redistributing it as assets. No HTTP client compiled in, which is a different claim from
"no telemetry yet". A share card built so that a project name cannot be *expressed*, rather than
relying on remembering not to type one. This is credible and it is HN-shaped.

Neither is used everywhere. The feeling leads the consumer channels, the engineering leads the
article and Show HN, and they are one argument rather than two messages: **the feeling produced
the constraints.** Refusing to guilt the user is why there are no streaks. Refusing to leak is
why there is no network layer. Refusing to redistribute is why the art is not in the repository.
Entered from either end, it arrives at the same place.

The reason this matters more than it sounds: the two goals in section 1 pull apart. Reach lives
on Reddit, where the feeling wins outright. An audience for what comes next lives with
developers who follow people whose reasoning they respect, and that is the engineering. A single
hook serves one goal and abandons the other.

## 4. Show HN goes last

`docs/launch-copy.md` says "Post. Show HN first". That was written before the account situation
was known and it is now wrong.

A Show HN from a cold account is a coin flip. New accounts are ranked down and flagged quickly,
there is no reputation to absorb the "solution in search of a problem" reply the existing copy
already predicts, and the shot is not repeatable. Going last buys three things: an account with
real comment history, knowledge of which hook actually landed on Reddit, and a published article
to point at in the thread instead of a landing page.

The cost is that a large Reddit thread may reach HN readers first. That is acceptable. HN
regularly rewards things its readers have already seen elsewhere, and the alternative risks the
one irreplaceable shot to save a week.

## 5. Channels

### Reddit, one sub per day, never the same text twice

| Sub | Angle | Link |
|---|---|---|
| r/macapps | Feeling. `[Free]` flair. macOS-only in line one. | `/momentum-mascot` |
| r/SideProject | Same, first person: "I kept abandoning side projects" | `/momentum-mascot` |
| r/rust | Tauri 2, the NSPanel class swap that got the pet over fullscreen apps, the binary size | Repository |
| r/pixelart | The rooms alone, LimeZu credited, **no download link in the body** | None |
| r/opensource | The licensing arrangement as the lede | Repository |
| r/desktops | One screenshot, the pet in situ | `/momentum-mascot` |

Bodies are rewritten, not reworded. Reddit's sitewide heuristics fire on one URL hitting several
subreddits in a short window, and the post does not fail loudly: it stops being visible while
still looking fine to the person who posted it. Spacing is the mitigation and it is the reason
this phase takes four days rather than an afternoon.

`docs/launch-copy.md` already holds the r/macapps body. r/pixelart is deliberately a post about
art with nothing to sell, because that subreddit reacts badly to promotion and well to craft.

### The article, written once

Canonical on **hoatrinh.dev**, cross-posted to dev.to and Hashnode with canonical tags, and
syndicated to daily.dev by registering hoatrinh.dev as a Source.

hoatrinh.dev rather than a blog on keepgoing.dev for two reasons. It is already designated as
the writing home in Commitropolis' own launch spec, and consistency between the two launches is
worth more than tidiness. And an audience that accrues to the author rather than to a product
domain is the goal in section 1, which a product blog serves worse.

daily.dev by Source registration rather than by link submission. Its feed is article-driven, a
bare landing-page link from a cold account sinks, and Source review takes days, which is why it
is the first task in Phase 0 rather than a Phase 2 one.

### Show HN

Title and first comment are already drafted in `docs/launch-copy.md` and stand. The URL becomes
`/momentum-mascot`. Tuesday to Thursday, 8-10am US Eastern, and four hours at the keyboard
afterwards, because on Show HN the author answering in the thread matters more than the post.

### X, Mastodon, Bluesky

The X thread in `docs/launch-copy.md` stands, but it is an artifact of record rather than a
growth lever: a cold account's thread does not travel. It exists to be linkable and to catch
people arriving from elsewhere. Mastodon (fosstodon) and Bluesky are worth the same content and
gate on account age less severely.

### Evergreen, and where a free macOS app quietly gets most of its lifetime installs

Pull requests to `awesome-mac`, `awesome-tauri` and `awesome-macos-apps`. An AlternativeTo
listing. Tauri's own showcase and Discord, which actively welcome shipped applications. One item
a week, indefinitely, long after the launch has stopped being a launch.

### LimeZu

Email the artist. Their audience is precisely the people who would enjoy this, the licence
already requires crediting them, and a pixel artist seeing their pack shipped in a notarized Mac
application is a good message to receive. Highest ratio of likely return to effort in this
document, and it is not on any channel list anywhere else.

### Product Hunt

Excluded. It rewards prearranged networks and hunters, a cold free macOS-only launch routinely
finishes outside the top twenty, and it consumes a day of attention doing so.

## 6. Phases

**Phase 0, days 1-7. Nothing is public.**
Record the baseline Releases download count and star count before anything moves, because
without it the launch produces a graph with no baseline. Execute the site split spec end to end
and verify it. Confirm hoatrinh.dev has a blog and an RSS feed at all, which this document
assumes and has not checked, then submit the daily.dev Source. Write the article. Warm the HN
account by commenting honestly on unrelated threads, with no links. Rebuild and upload the
GitHub social preview card, which `fcd4aaa` removed from git, per the recipe in
`docs/launch-copy.md`. Install once from a different machine to prove the download is clean.
Email LimeZu.

**Phase 1, days 8-11. Reddit.** r/macapps on a Tuesday or Wednesday around 9am US Eastern, then
one subreddit per day in the order in section 5.

**Phase 2, days 12-13. The article.** Publish on hoatrinh.dev, cross-post with canonicals,
daily.dev picks it up from the Source, X and Mastodon point at it.

**Phase 3, days 14-15. Show HN.**

**Phase 4, ongoing.** The evergreen list, one item a week.

## 7. Measurement

`docs/launch-copy.md` says the Releases download count is nearly the only instrument available.
That undersells it. Four instruments exist and none of them costs the app's no-network promise
anything:

- **App Store Connect** reports units, impressions and traffic sources. It is a real per-channel
  signal and it has been sitting unused. The app sends nothing; the store counts downloads.
- **GitHub Insights traffic** gives referrers, on a rolling 14-day window, so it has to be read
  *during* the launch and not after it. This is easy to lose by forgetting.
- **Cloudflare Web Analytics** per path on the new site, which separates hub from
  `/momentum-mascot` from `/commitropolis` and therefore measures routing as well as arrival.
  Be ready to answer for it honestly: it is the website, not the application, and the promise in
  the README is about the application.
- **itch.io page views and followers** for the Commitropolis teaser, which is also the first
  real datum that spec will get.

The qualitative instrument is comments and email. Commitropolis' launch spec calls the comments
the research and the only customer development available for free, and that is equally true
here.

Record all of it weekly in a plain file. Three numbers and a list.

## 8. Risks

- **Reddit's cross-sub filter, which fails silently.** Mitigated by spacing and by rewriting.
- **The single Show HN.** Mitigated by going last, section 4.
- **macOS only.** Not mitigable. Stated everywhere, up front.
- **The cutover window.** Mitigated by doing it in Phase 0 with nothing pointing at the domain.
- **A `#support` regression** would break the Support menu item in every shipped binary,
  including App Store installs, and nothing would report it. It has an explicit check in the
  site spec.
- **The Commitropolis teaser overpromising** a game whose own spec reserves the right not to
  ship. Mitigated by the no-date rule in that spec's section 6.
- **Article-first discovery burning the Show HN.** If someone else submits the article to HN
  during Phase 2, the Show HN is spent. Accepted: the odds are low and the alternative is
  worse.
