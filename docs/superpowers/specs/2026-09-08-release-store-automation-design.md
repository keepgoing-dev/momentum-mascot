# Automating the App Store Connect half of a release

**Date:** 2026-09-08
**Status:** design approved in brainstorming. Ready to plan.

**Extends** `docs/app-store-listing.md`, which is the field-by-field record of the listing and
will remain so. This document does not replace it. It moves four blocks of text out of it into
files a script can read, and leaves every word of the reasoning where it is.

**Completes** the split that `tools/release-mas.sh` started. That script ends at "uploaded 0.4.0
build 6. Processing takes a few minutes. Then set the build on the version in App Store
Connect." Everything after that sentence is currently done by hand in a browser. This is that
part.

**The justification is defect history, not convenience.** Three listing defects have shipped or
nearly shipped, and every one was caused by hand-editing and caught by reading the API:

- **0.3.1**: three commas typed straight into the panel. The doc was then wrong about them for
  weeks (`docs/app-store-listing.md:221`).
- **0.3.1**: the screenshot order went live wrong, because the media panel orders by drop
  position and six files dropped together do not land in name order. A released version's order
  cannot be changed afterwards.
- **0.4.0**: the App Review Notes field went in missing its entire addendum, 1936 characters
  live against 3886 expected. The panel showed a full, correctly formatted field. It was caught
  one step before Submit by grepping the API response for its own section headings.

Each one is the same shape: the local record and the live listing disagreed, and the panel could
not show it. A script that writes from the record and reads back to diff makes that class of
defect structurally impossible rather than caught by diligence.

---

## 1. Goal and non-goals

The goal is two commands per release instead of a panel session:

```sh
tools/release-store.sh            # converge the listing, print the diff, stop
tools/release-store.sh --submit   # verify again, submit, watch in the background
```

**Non-goals.**

- **One-time app-level setup.** App Privacy, age rating, categories, pricing. Answered on
  25 August 2026, carried forward by Apple, changed roughly never. Automating a thing done once
  is a loss.
- **Screenshot capture.** `tools/store-shots.sh` and `tools/hold-state.sh` stay as they are. The
  capture is inherently manual: it needs the popover opened from the menu bar icon by a person.
  Only the upload is automated.
- **Version bumping.** Stays in `tools/release.sh` and deliberately nowhere else, so the two
  channels cannot disagree about what a version is (`tools/release-mas.sh:11`).
- **Moving the existing release scripts.** See section 8.

## 2. Why not webhooks

App Store Connect gained webhooks at WWDC25. `POST /v1/webhooks` with `name`, `url`, `secret`,
`eventTypes` and an `app` relationship. The two relevant event types are the build upload state
change and `APP_STORE_VERSION_APP_VERSION_STATE_UPDATED`. Deliveries are signed
`x-apple-signature: hmacsha256=<hex>`, HMAC-SHA256 over the raw body.

They are the wrong tool for this problem, and the reason is where the work happens. A webhook
needs a public HTTPS listener. The thing that has to act on the event is a shell script on one
laptop holding `~/.appstoreconnect/private_keys/AuthKey_*.p8`. A Cloudflare Worker receiving
"build processing finished" cannot run that script. To let a webhook finalize anything, the ASC
key has to move into Cloudflare and this whole design has to be rebuilt as a Worker, which puts
a second copy of a credential that currently exists on exactly one machine somewhere that then
needs securing and rotating.

Polling from the laptop is simpler and adds no attack surface. For a release every few weeks by
one person, that is the whole analysis.

**The one case where a webhook would win**, recorded here so the option is not lost: a
notify-only Worker that receives the event, verifies the signature, pushes a phone notification
and stops. It needs no ASC key at all, because the payload carries the state. That is a separate
small project and is not part of this design.

## 3. What was measured

Everything below was probed against the live app on 2026-09-06, not assumed.

```
/usr/bin/jq                             present (system jq)
xcrun altool --generate-jwt             works. ES256, kid 52ZBN6J5ZA, aud appstoreconnect-v1,
                                        exp ~20 minutes
python3 -c "import cryptography"        ModuleNotFoundError
python3 -c "import jwt"                 ModuleNotFoundError
locales on appInfoLocalizations         en-US, and only en-US
appStoreVersions                        0.4.0, 0.3.2, 0.3.1, all READY_FOR_SALE
appVersionState on 0.4.0                READY_FOR_DISTRIBUTION
screenshotDisplayType                   APP_DESKTOP
releaseType                             AFTER_APPROVAL
```

### 3.1 altool mints the JWT, so shell is viable

ES256 was the only hard part of talking to this API from `sh`, and Apple's own binary removes it:

```sh
xcrun altool --generate-jwt --apiKey "$ASC_API_KEY_ID" --apiIssuer "$ASC_API_ISSUER_ID"
```

**The token goes to stderr, with a banner line before it.** `2>/dev/null` silently discards it,
which reads as an empty token rather than an error. The extraction is `2>&1 | awk 'END{print $NF}'`.

The alternatives were `openssl dgst -sha256 -sign` plus a DER to raw R||S conversion by hand, or a
`pip install cryptography` that none of the other 24 tools in `tools/` need. Neither is worth it
when a required dependency already does the job.

### 3.2 sourceFileChecksum reads back, so screenshots are content-addressed

```
live  slot 1  1-comeback.png  sourceFileChecksum  eb9342faff154bdfea0bde3bc96dc2ca  4879370 bytes
local         1-comeback.png  md5 -q              eb9342faff154bdfea0bde3bc96dc2ca  4879370 bytes
```

This decides the whole screenshot design, because the live listing is already carrying stale
names:

```
slot 1  1-comeback.png      slot 5  4-asleep.png
slot 2  2-builder.png       slot 6  5-pet.png
slot 3  2-awake.png         slot 7  6-card.png
slot 4  3-dozing.png
```

Slots 3 to 7 kept their 0.3.2 names when the builder was inserted at slot 2. Locally those files
are `3-awake` through `7-card`. **A name-based verification would fail today against a listing
whose order is correct**, and would pass on a listing whose order is wrong but whose names happen
to line up. A checksum comparison is right in both cases.

## 4. Components

```
tools/lib/asc.sh          talk to the API. Nothing else.
tools/release-store.sh    converge the listing to the repo. --submit.
tools/release-watch.sh    poll appVersionState until it settles. Report.
docs/store/fields/*.txt   the field text
```

### 4.1 tools/lib/asc.sh

Sourced, not executed, following `tools/lib/tints.sh`. Exposes `asc get|post|patch|delete <path>
[json]`.

**It re-mints the JWT at fifteen minutes.** The token lives twenty, and a single run can wait
thirty minutes for a build and then upload 34MB of screenshots. Without this the run dies partway
through phase 4, which is the one state that needs a second run to heal.

**It checks the status code itself.** The API answers a bad request with a well-formed JSON error
body and a non-2xx code. `curl -sS` succeeds, `set -eu` sees nothing wrong, and `jq` returns
`null` for the field that was asked for. Every call captures `%{http_code}` separately and
surfaces `.errors[].detail`:

```
PATCH appStoreVersionLocalizations/bb5ab5c3 -> 409
  The attribute 'whatsNew' cannot be changed while the version is in review.
```

Silent `null` is the failure mode that lets the script write nothing and report success, which is
worse than any crash and is the same shape as the three defects in the preamble.

### 4.2 The field files

One file per field, content only:

```
docs/store/fields/promotional-text.txt       162    limit 170
docs/store/fields/description.txt           1859    limit 4000
docs/store/fields/keywords.txt                     limit 100
docs/store/fields/review-notes.txt          3856    limit 4000, assumed
docs/store/whats-new/0.4.0.txt              1424    limit 4000
```

What's New is per-version because it is the only field whose superseded value stays true.
`docs/app-store-listing.md` already keeps 0.3.2's as a record, and that instinct is right.

**Review Notes becomes one file.** It is currently split between
`docs/app-store-listing.md:441` and the whole of `docs/app-store-review-notes.md`, and that split
is precisely how 0.4.0 shipped to the review queue with half the field missing. The prose in both
documents stays; the field content is assembled into one file that is the only thing anything
reads.

`docs/app-store-listing.md` keeps every word of its prose, history, character counts and
reasoning. Each field section loses only its fenced block, replaced by a pointer line. The
document stops being an input and returns to what it is good at, which is the argument for why
the text says what it says.

### 4.3 No screenshot manifest

`docs/store-shots/` is already named `1-comeback.png` through `7-card.png` in slot order. The
leading number is the slot. The script asserts the numbers are 1..N with no gaps and no
duplicates, which is exactly the "a set where two files claim slot 2" failure
`docs/app-store-listing.md:486` warns about.

This changes those names from decorative to load-bearing. `docs/app-store-listing.md:493` says
"the numbering is a note to the person uploading and nothing more". That line becomes false and
should be rewritten when this ships.

### 4.4 App-level fields are printed, not asserted

Name, subtitle and the three URLs live on `appInfoLocalizations`, not on the version. Preflight
prints them and checks the URLs resolve. It does not diff them, because there is no second copy
of "Momentum Mascot" to diff against that would not be invented here, and inventing one
recreates the drift problem this design exists to remove.

## 5. Convergence

**The script is stateless and idempotent.** No local state file, no resume flag. Every run reads
App Store Connect and works out what is left to do. Killed halfway, it converges on the next run.

The argument is this listing's own history rather than general principle. A remembered position
is one more thing that can disagree with Apple, and the three defects in the preamble were all
disagreements between a local record and the live listing. The API is the state, which is the
rule `docs/app-store-listing.md` already tells a person to follow by hand on every submission.

It also makes the diff gate free: if writing a field means "write unless it already matches",
then the second run of the same command is the verification pass.

### 5.1 Phase 0, preflight

No network writes. Version from `src-tauri/tauri.conf.json`, build number from
`tools/.mas-build`, so the two values the script could get wrong are read from the same files the
build used rather than typed. Field files exist and fit their limits. Screenshots are 1..N
contiguous with no duplicates, all exactly 2560x1600.

The three URLs are checked **by content, not by status code**, because keepgoing.dev serves a
page for unknown paths:

```sh
curl -sS https://keepgoing.dev/privacy | grep -q "<title>Privacy Policy"
```

### 5.2 Phase 1, the version

Filter `appStoreVersions` by `versionString`.

| Live state | Action |
|---|---|
| absent | `POST` create: `platform MAC_OS`, `versionString`, `copyright`, `releaseType AFTER_APPROVAL` |
| `PREPARE_FOR_SUBMISSION` | reuse |
| anything else | refuse, naming the state |

```
0.4.0 exists in READY_FOR_DISTRIBUTION. Nothing to do here.
0.4.0 exists in WAITING_FOR_REVIEW. Cancel the submission first.
```

**`--dry-run` warns here instead of refusing**, and carries on into the read-only comparison.
Refusing would make the flag useless on the only listing that exists today: `tauri.conf.json`
reads 0.4.0, and 0.4.0 is `READY_FOR_DISTRIBUTION`. A dry run against a released version is the
cheapest way to ask "does the repo still describe what is live", which is a question worth being
able to ask at any time and not only during a release.

### 5.3 Phase 2, the build and the first wait

**This is where the gate cannot help, so the check lives here.** The chosen human gate catches
wrong text. It cannot catch right text written against the wrong build, because the diff would
be clean. So: filter `builds` by `version` equal to `tools/.mas-build`, then assert its
`preReleaseVersion.version` equals the version string from `tauri.conf.json`. Both must agree
before anything is attached.

Poll `processingState` every 15 seconds, cap 30 minutes, print elapsed. Build 6 took 63 seconds
(`ABSENT` at 23:34:13, `VALID` at 23:35:16). Earlier builds took considerably longer, so the cap
is generous and the elapsed time is printed rather than guessed at.

Then `PATCH /v1/appStoreVersions/{id}/relationships/build`.

### 5.4 Phase 3, the fields

`GET` the en-US `appStoreVersionLocalizations`. Compare `promotionalText`, `description`,
`keywords` and `whatsNew` against their files. `PATCH` only what differs.

Then `appStoreReviewDetail` on the version: `POST` if it does not exist yet, `PATCH` `notes` if
it does.

**Promotional Text is re-entered on every version.** It is not copied forward, and an empty one
is a legal listing that warns about nothing (`docs/app-store-listing.md:75`). Because the script
writes it unconditionally from the file, this stops being something to remember.

### 5.5 Phase 4, the screenshots

Fetch the `APP_DESKTOP` set for the localization, creating it if absent, and its screenshots,
which the API returns in slot order.

Compare the ordered list of live `sourceFileChecksum` against the ordered list of local
`md5 -q`. Equal, skip the phase. Different anywhere, replace the whole set:

1. `DELETE` every live screenshot in the set.
2. Per slot in order: `POST /v1/appScreenshots` with `fileName` and `fileSize` to reserve, which
   returns `uploadOperations`.
3. `PUT` each operation's byte range with its `requestHeaders`.
4. `PATCH` the screenshot with `sourceFileChecksum` (MD5 hex) and `uploaded: true`.
5. `PATCH /v1/appScreenshotSets/{id}/relationships/appScreenshots` with the ordered id list,
   rather than trusting creation order the way the panel trusts drop order.
6. Poll `assetDeliveryState.state` until every one reads `COMPLETE`.

**Whole-set replacement rather than a minimal edit**, because inserting the builder at slot 2
shifted five files down, so "minimal" is only minimal in the case that does not happen.

The cost is a window where a crash mid-upload leaves the version short of screenshots. That
version cannot be submitted in that state, and the next run converges it, so the failure is loud
and self-healing. Step 5 is separate from step 2 for the same reason: creation order is not a
contract, and the 0.3.1 order defect came from believing it was.

### 5.6 Phase 5, read-back

Re-fetch everything written and print it. This is the gate:

```
promotional-text   162  match
description       1859  match
whats-new         1424  match
review-notes      3856  match
screenshots        7/7  slots match by checksum
NOT submitted. Re-run with --submit.
```

Any mismatch is a non-zero exit and no submission.

### 5.7 Phase 6, submit, only under --submit

Re-runs phase 5 and refuses on any mismatch. Then:

```
POST  /v1/reviewSubmissions       relationships: app
POST  /v1/reviewSubmissionItems   relationships: reviewSubmission, appStoreVersion
PATCH /v1/reviewSubmissions/{id}  attributes: { submitted: true }
```

An existing submission in a reusable state is reused rather than duplicated. Filters need URL
encoding: `filter%5Bapp%5D=6804925509&filter%5Bplatform%5D=MAC_OS`. Unencoded brackets return a
body that is not JSON, which surfaces as a parse error rather than an API error.

Then hands off to `tools/release-watch.sh`.

## 6. The two waits

| Wait | Measured | Handling |
|---|---|---|
| Build processing | build 6: 63 seconds. Earlier builds much longer. | Inline poll, 15s, cap 30 min |
| Screenshot delivery | seconds | Inline poll, 5s, cap 5 min |
| Review | 0.4.0: 1h19m. 0.3.2: about 16 hours. | `release-watch.sh`, detached |

`tools/release-watch.sh` polls `appVersionState` every five minutes and exits on a terminal
state: `READY_FOR_DISTRIBUTION`, or any rejection state. It reports twice, a line to stdout and
`osascript -e 'display notification'`, so a sixteen-hour wait reaches a person who is not looking
at the terminal. No dependency, no listener, no credential leaving the machine.

Called with no arguments it prints the current state once and exits, which makes "where is it"
a command rather than a browser tab.

## 7. Refusals and retries

Every refusal names what to do about it.

- A version in a state that cannot be edited (section 5.2).
- A build whose `preReleaseVersion.version` disagrees with `tauri.conf.json`.
- A field over its limit, or a field file missing.
- A screenshot set with a gap, a duplicate slot, or a file that is not 2560x1600.
- `Missing Compliance`. This should no longer occur: `ITSAppUsesNonExemptEncryption` in
  `src-tauri/Info.plist` answers export compliance at build time. It blocked build 4, there is no
  API to answer it (`buildBundles` returns `403 FORBIDDEN` to this key), and without an explicit
  check it surfaces as an unexplained submit failure.

**Retries only where they are safe.** The screenshot `PUT` goes to Apple's blob store rather than
the API, so a flake there is a genuine flake: three attempts with backoff. Nothing else retries.
A `POST` that creates a review submission must never be retried into two submissions.

## 8. Layout: nothing moves

The four release scripts stay flat in `tools/`, sharing the `release-` prefix.

```
tools/release.sh         version, DMG, notarize, GitHub release
tools/release-mas.sh     build, sign, package, upload
tools/release-store.sh   listing, screenshots, submit          new
tools/release-watch.sh   poll review state, report             new
tools/lib/asc.sh         API client                            new
```

Three reasons a `tools/release/` directory was rejected:

- **The repo already has a structuring convention, and it is prefixes.** `assemble-*`,
  `compose-*`, `verify-*`, `make-*`, `install-*`, `release-*`. Six groups doing the job a
  directory would do, at zero cost. A directory for four files introduces a second competing
  convention and leaves the other twenty in the first one.
- **Some references are history, not documentation.** There are roughly 148 references to
  `tools/release*` in the tree. `docs/superpowers/plans/2026-08-22-mac-app-store-submission.md`
  alone names those paths 24 times, as a record of what was run in August. Rewriting them makes
  the record describe something that did not happen; leaving them makes it wrong differently.
- **`tools/lib/` is where this repo puts structure**, and it already exists. `asc.sh` goes there
  either way, which is the structuring that was actually wanted.

Every tool resolves `ROOT=$(cd "$(dirname "$0")/.." && pwd)`, one level, in 18 places. A move
breaks that in each file it touches.

## 9. Testing

The repo's convention is `verify-*.sh`: self-contained, assert, exit non-zero.

**Preflight is pure and gets fixture directories.** Field lengths, slot contiguity, duplicate
slots, wrong dimensions, missing files. Most of the logic that can be wrong lives here and none
of it needs a network.

**Convergence gets captured API responses.** The probes on 2026-09-06 produced real ones: a
version in `READY_FOR_DISTRIBUTION`, an en-US localization, and a seven-screenshot set with live
checksums and the stale 0.3.2 names. The stale-names response is the most valuable fixture in the
set, because a name-based implementation passes every other test and fails only that one.

**`--dry-run` against the live app.** Prints every write it would make and touches nothing.
Runnable today: it must refuse 0.4.0 by state, and must report the current listing as converged.

**Full rehearsal against a throwaway version.** A version in `PREPARE_FOR_SUBMISSION` costs
nothing. Create one, run every phase against it including the whole screenshot upload, then
delete it. Unlike `tools/release-mas.sh`, where any run burns a build number before the build
(`release-mas.sh:138-151`), this pipeline is rehearsable in full. The only step that is not is
Submit, and even that can be cancelled before review begins.

**This rehearsal depends on `DELETE /v1/appStoreVersions/{id}` succeeding for a version in
`PREPARE_FOR_SUBMISSION` with this key, which has not been verified.** It is step one of the
implementation plan, before anything is built on the assumption. If it fails, the rehearsal
target becomes the real next version instead, which is still safe but no longer disposable.

## 10. Acceptance tests

1. `tools/release-store.sh --dry-run` against the current listing warns that 0.4.0 is
   `READY_FOR_DISTRIBUTION`, then reports every field and all seven screenshot slots as
   converged, and writes nothing.
2. `tools/release-store.sh` without `--dry-run`, against that same released 0.4.0, refuses at
   phase 1 naming the state, and reaches no later phase.
3. A build number in `tools/.mas-build` that belongs to a different version string is refused
   before anything is attached.
4. A `docs/store/fields/promotional-text.txt` of 171 characters is refused in preflight, with the
   count and the limit.
5. A `docs/store-shots/` with two files claiming slot 2 is refused in preflight.
6. Against the stale-names fixture, convergence reports the screenshot set as **matching**. A
   name-based implementation fails this test.
7. Against a fixture where slots 3 and 4 are swapped, convergence reports a mismatch and names
   both slots.
8. A full rehearsal on a throwaway version: seven screenshots uploaded, all `COMPLETE`, read back
   in the correct slots by checksum, then the version deleted.
9. Killing the script during phase 4 and re-running it converges the set without manual repair.
10. `tools/release-watch.sh` with no arguments prints the current state of the latest version and
    exits zero.
