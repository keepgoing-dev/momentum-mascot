# Launch copy

Drafts for the first real push, once the notarized build is live. Nothing here should go out
before then: every one of these posts sends people to a download, and an install that dead-ends
in a Gatekeeper dialog spends attention you cannot get back.

## Order of operations

1. Notarized `v0.4.0` published and downloaded once **from a different machine** to prove the
   install is clean.
2. Site deployed with the updated install steps.
3. Post. Show HN first, in the morning US Eastern on a weekday. Reddit and X the same day.
4. Stay at the keyboard for the next four hours. On Show HN the author answering questions in
   the thread matters more than the post.

## What to link

The site, not the repo. The site has the hero GIF above the fold and the download button;
GitHub opens on a wall of text. `site/hero.gif` and `docs/mockups/states-four.gif` are the two
assets that do the actual selling.

---

## Show HN

**Title**

```
Show HN: A desktop pet that dozes off when your side project does
```

Alternates, if that one reads too cute:

```
Show HN: Momentum Mascot – a pixel room that reflects your git activity, with no streaks
Show HN: I built a desktop pet for side projects that never guilts you
```

**URL:** `https://keepgoing.dev` (or wherever the mascot page lives)

**First comment, posted immediately after submitting:**

```
I kept abandoning side projects and then feeling bad about the abandoning, which is a worse
problem than the abandoning. Every tool I found made that loop tighter: streaks to break,
graphs going grey, a number counting up since my last commit.

So this one refuses to do that. It watches the reflog of repositories you point it at and reads
exactly one thing from each, which is when you last actually committed. Four states: awake under
24 hours, dozing to 72, asleep past that, and a comeback animation when a real commit lands
after a sleep. Checking out a branch or pulling does not count and cannot trigger it.

There are no streaks, no scores, no notifications, and it will never tell you how long it has
been. The mascot does not die, it waits. That constraint drove most of the design.

Technical bits that might be interesting:

- Tauri 2 and Rust, ~10MB, sits in the menu bar with an optional 64x64 pet floating on the
  desktop.
- No network layer at all. Not "no telemetry yet", there is no HTTP client compiled in. State is
  one readable JSON file at ~/.keepgoing/mascot/state.json.
- The share card is drawn in a canvas at 1200x630 and copied to the clipboard. It is built so
  that a project name, path, commit message, hash or timestamp cannot be expressed on it, rather
  than relying on remembering not to put them there.
- The art is LimeZu's Modern Interiors, whose licence permits shipping compiled into an app and
  forbids redistributing the assets. So the repo commits the *coordinates* that composite the
  rooms and not the art itself, and you bring your own copy of the pack to build it. That turned
  out to be a tidier arrangement than I expected.

Limitations, honestly: macOS only right now. It does not start on login yet. Linux is genuinely
uncertain because Wayland does not let an app position its own window, which makes the pet close
to unimplementable there.

Code is MIT. Happy to answer anything.
```

**On the HN thread**

The objection to expect is "this is a solution in search of a problem" or "just look at your
commit history." Do not argue the premise. The honest answer is that it is a toy that happens to
be kind, and the reason it exists is the guilt loop rather than the information.

The other thing that will come up is the closed-source art arrangement. Answer it plainly, the
licensing setup is genuinely the most interesting engineering decision in the repo and it reads
well when explained rather than defended.

---

## r/macapps

**Title**

```
[Free] Momentum Mascot – a pixel character who lives on your desktop and dozes off when your side projects go quiet
```

**Body**

```
Free, open source, no accounts, no network requests, macOS 10.15+.

It watches the git repos you point it at and reads one thing: when you last committed. The
mascot is at their desk if you have been working, dozing after a day, asleep after three, and
leaps out of bed when you come back.

Deliberately not a productivity tool. No streaks, no scores, no notifications, and nothing in
it will ever tell you how long it has been since you last committed.

There is a 64x64 desktop pet, a full animated room in a menu bar popover, and a share card it
copies to your clipboard.

Zero network layer, so nothing leaves the machine. Everything it knows is in one JSON file you
can read.

Download: <link>
Source: <link>
```

`r/SideProject` takes roughly the same body with a first-person framing. `r/pixelart` is worth a
separate post about the rooms alone, credited to LimeZu, with no download link, because that
subreddit reacts badly to promotion and well to art.

---

## X thread

**1/**

```
my side projects kept going quiet and every tool I tried made me feel worse about it

so I built one that doesn't

a pixel character who lives on your desktop and dozes off when your repos do. no streaks,
no scores, never tells you how long it's been

free, macOS
```
*(attach `states-four.gif`)*

**2/**

```
four states, driven by one number: when you last actually committed

awake under 24h
dozing to 72
asleep past that
and a comeback when a real commit lands after a sleep

checking out a branch doesn't count. pulling doesn't count.
```
*(attach `pet-comeback.gif`)*

**3/**

```
no network layer. not "no telemetry yet" — there's no HTTP client compiled into it

everything it knows is one JSON file at ~/.keepgoing/mascot/state.json that you can read
or delete

the share card can't physically express a project name, path, commit message or timestamp
```
*(attach a share card PNG)*

**4/**

```
the mascot never dies. it waits.

that's the whole design constraint and everything else followed from it

<link>
```

---

## After

The only instruments available are the Releases download count and whatever shows up in search,
because there is no telemetry by design and there never will be. So write down the download
count immediately before posting. Without that number the launch produces a graph with no
baseline, and the whole point of notarizing first was to make the number mean something.
