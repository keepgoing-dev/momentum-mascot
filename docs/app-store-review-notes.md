# App Review Information: the 2.1 reply

**Rejected 27 August 2026, guideline 2.1 Information Needed**, on build 4 of 0.3.1. Nothing
about the binary, the art, or 4.2: App Review asked for a screen recording and seven pieces of
information, and the request is the standard new-app questionnaire rather than a finding about
this app. **No new build is needed.** The reply goes into Resolution Center in two messages,
an addendum goes into the Notes field of App Review Information so the next version is not
asked again, and build 4 continues in review.

**Reply to App Review holds 4000 characters, and the answer does not fit in one.** The seven
answers came to 5302 characters after being cut as far as they go without gutting point 7, which
is the one where wording carries weight. Resolution Center accepts more than one reply, so this
is two messages rather than a compressed one. Post them in order, back to back, with the
recording attached to the first.

Everything below is checked against the code rather than remembered; the section after it
records where each answer comes from. Character counts are exact and measured, not estimated.

## Reply 1 of 2: points 1 to 4 (3438 characters)

```
Momentum Mascot has no accounts, no purchases, no subscriptions, no user-generated content and makes no network requests, so it has no registration, login, deletion, purchase or content-reporting flows to show.

1. SCREEN RECORDING
Attached: one continuous take on a MacBook Pro, starting with launching the app and covering every feature. The app's only permission prompt is the folder picker at 4(d), which is included.

2. TESTED ON
MacBook Pro (Mac16,8), Apple M4 Pro, macOS 26.5.2 (25F84). Universal binary (arm64 and x86_64), minimum deployment target macOS 10.15.

3. WHAT IT DOES, AND FOR WHOM
An ambient desktop companion for developers with side projects. The user points it at git repositories on their own Mac. It reads one fact from each, the time of the last commit, and picks a state for a pixel character in a small animated room:
- awake, at their desk: a commit within 24 hours
- dozing: 24 to 72 hours
- asleep: over 72 hours
- comeback: a commit arrives after the character was asleep, and they leap out of bed
The character is a 64x64 pet in a corner of the desktop; the full room is in a popover from the menu bar icon.
The problem it solves: side projects go quiet for weeks when a day job gets busy, and existing tools answer that with streaks, scores and reminders. This app has none of those and never states how long it has been. The character waits, and the comeback is the moment of value. Audience: working developers with unfinished personal projects. There is no server, no account and no content outside the app bundle.

4. SETUP AND ACCESS
No credentials, and nothing to sign in to. The app needs a git repository on the review machine, and since its states are defined by commit age, a backdated commit is the fastest way to see them.
(a) Launch the app. The character appears in the bottom-right corner of the desktop and an icon appears in the menu bar.
(b) Click that icon. The popover holds the room, a character picker, a line of copy, the project list and two buttons.
(c) In Terminal, make a repository whose last commit is 100 hours old:
  mkdir -p ~/Documents/mascot-demo && cd ~/Documents/mascot-demo
  git init -q .
  GIT_COMMITTER_DATE="@$(( $(date +%s) - 100*3600 )) +0000" git -c user.email=demo@example.com -c user.name=Demo commit -q --allow-empty -m "work"
(d) Click "Add Project" and choose that folder. This is the folder picker, the app's only prompt. The character is ASLEEP: the last commit is older than 72 hours.
(e) For the COMEBACK, the app's central feature, commit again in that repository:
  git -C ~/Documents/mascot-demo -c user.email=demo@example.com -c user.name=Demo commit --allow-empty -m "back at it"
The pet leaps out of bed within a few seconds. Reopen the popover for the comeback room.
(f) Any repository with a recent commit shows AWAKE. Backdating 30 hours instead of 100 shows DOZING.
The rest is in the popover, and all of it is in the recording: a character head switches character, the open circle on a row marks a project "operating" so it keeps its place but stops affecting the character, the x on a hovered row stops tracking it, "Share Status" copies a 1200x630 image of the room to the clipboard, "privacy" opens the privacy policy in the default browser, and Escape closes the popover. Right-click the menu bar icon for Open and Quit. No sample files are needed.

Points 5, 6 and 7 follow in a second reply, because this field holds 4000 characters.
```

## Reply 2 of 2: points 5 to 7 (1985 characters)

```
Continued from the previous reply.

5. EXTERNAL SERVICES
None: no data provider, authentication service, payment processor, analytics, crash reporter or AI service, and no network requests. Commit times come from a local file inside the folder the user chose (.git/logs/HEAD), with a rare fallback that runs the local "git" command. State is one JSON file in the app's own container. Artwork and fonts are inside the bundle. The only outbound action is opening the privacy policy URL in the default browser when the user clicks "privacy"; that URL is a compile-time constant and the app exposes no command that takes a URL.
The bundle does carry com.apple.security.network.client, which is not a contradiction. WKWebView needs it to reach its own networking process; without it a sandboxed webview never finishes navigation, so the popover renders blank with no violation logged. It grants WebKit that access. The app issues no requests, and the webview loads only local files from the bundle.

6. REGIONAL DIFFERENCES
None. Behaviour is identical in every region: no region-gated features or content, no server, no localization (English only).

7. THIRD-PARTY MATERIAL
Artwork: the rooms, furniture and characters are derived from "Modern Interiors" by LimeZu (limezu.itch.io), a commercial asset pack purchased under its full-version licence, which permits use and editing in any commercial project and forbids reselling or distributing the asset itself. The art ships composited into the application with no way to extract the source sprite sheets, so this is use rather than redistribution. The credit the licence requires is shown in the popover and in the app's copyright field. Receipt and licence text available on request.
Typeface: Departure Mono by Helena Zhang, under the SIL Open Font License 1.1, which permits bundling; its licence text ships in the app bundle.
The app is not in a regulated industry, handles no health, financial or personal data, and collects nothing.
```

## The Notes field: the review notes plus this addendum (2025 characters)

Apple's closing line asks for this information in the Notes field of App Review Information so
the next version is not asked again. The version is editable while it sits rejected, so this can
be done now, and it should be: the questionnaire arrives per app, but a reviewer who cannot
reach the four states arrives per version.

**The field must keep the reviewer instructions.** Those are in `docs/app-store-listing.md` under
"Review notes", they are 1971 characters, and they are what tells a reviewer where to click in an
app with no window. This addendum goes below them and covers only what a reviewer cannot work out
by clicking: what it was tested on, who it is for, how to reach the states, and what is licensed
from whom.

**The pair lands at 3886 characters against a 4000 limit, which is not a comfortable fit.** The
limit on this field is assumed rather than measured: 4000 is what Reply to App Review states, and
the Notes field is treated the same here.

**TARGET AUDIENCE AND VALUE was dropped for 0.4.0, and the rewritten THIRD-PARTY MATERIAL is why.**
The pair stood at 3998 on 0.3.2 with 319 characters of audience copy in it. The third-party
paragraph had to grow by 207 to stay true (below), which put it at 4205, and the audience
paragraph was already named here as the first thing to go: it is the one answer that also exists
in the Description, on the page the reviewer is already reading. If more room is ever needed after
that, the next cut is the REGIONAL DIFFERENCES line.

```
TESTED ON: MacBook Pro (Mac16,8), Apple M4 Pro, macOS 26.5.2 (25F84). Universal binary (arm64 and x86_64), minimum deployment target macOS 10.15.

SEEING ALL FOUR STATES: the thresholds are 24 and 72 hours, so backdate a commit. In Terminal:

  mkdir -p ~/Documents/mascot-demo && cd ~/Documents/mascot-demo
  git init -q .
  GIT_COMMITTER_DATE="@$(( $(date +%s) - 100*3600 )) +0000" git -c user.email=demo@example.com -c user.name=Demo commit -q --allow-empty -m "work"

Add that folder with "Add Project" and the character is asleep. Commit again in it, without the date, and the character leaps out of bed: that is the comeback. Backdate 30 hours instead of 100 for dozing.

EXTERNAL SERVICES: none, as above: no data provider, authentication service, payment processor, analytics, crash reporter or AI service.

REGIONAL DIFFERENCES: none. No region-gated features or content, and no localization (English only).

THIRD-PARTY MATERIAL: the rooms, furniture and characters are derived from "Modern Interiors" by LimeZu (limezu.itch.io), a commercial asset pack purchased under its full-version licence, which permits use and editing in any commercial project and forbids reselling or distributing the asset itself. The art ships inside the application as its own resources: composited rooms, plus the curated character layers the in-app mascot builder composites at runtime. That is a derived subset cut to this app's frame geometry rather than the pack as sold, and the app offers no way to export any of it, so this is use rather than redistribution. The required credit is shown in the popover and in the copyright field; receipt and licence text available on request. The typeface is Departure Mono by Helena Zhang under the SIL Open Font License 1.1, which permits bundling, and its licence text ships in the bundle.

Not a regulated industry: no health, financial or personal data, and nothing collected.
```

## Where each answer comes from

**The recording is the whole reply.** The other six answers are cheap; the recording is the
one Apple actually asked twice for, once in the numbered list and again under "Bugs and
crashes". The shot list is below.

**Point 2 lists one machine, and that is the complete answer to the question asked.** The
question is which device models and operating systems the app was tested on, and the honest
list has one row on it. The universal-binary and deployment-target lines are added because
"tested on an M4 Pro" invites the follow-up about Intel and 10.15, and stating what the binary
supports is better than being asked.

**Point 7 was rewritten for 0.4.0 because 0.3.2's wording stopped being true.** It said the art
ships "composited into the application with no way to extract the source sheets". The mascot
builder composites at runtime, so `src/assets/layers/` and `src/assets/swatches/` now ship as
PNG files in the bundle's Resources, and anyone who opens the app in Finder can copy them out.
The old sentence would have been a false statement to App Review about the one subject the
questionnaire asks about by name, and it would have been falsifiable in about four clicks. The
replacement says what is actually in there and rests the "use rather than redistribution"
conclusion on the two facts that survive: it is a curated derived subset cut to this app's frame
geometry, and the app offers no export. The full reasoning, including where it is weakest, is in
`docs/app-store-licence-check.md` under the 0.4.0 addendum.

**Point 4 is the part worth writing carefully.** A reviewer on a fresh Mac has no git
repositories, and an app whose entire behaviour is a function of commit age shows them a
sleeping character and an empty list. Worse, the states are 24 and 72 hours apart, so no amount
of clicking reveals them. The backdated-commit snippet is the only way a reviewer sees the
product in the time they have, and it is verified: `GIT_COMMITTER_DATE` sets the reflog entry's
timestamp, which is the timestamp the app reads (`reflog.rs`), and the entry it writes is
`commit (initial):`, which passes `reflog::qualifies`. The comeback in step (e) works live
because `evaluate` compares the newly awake state against `last_displayed_state`, which is
already `asleep` by then.

**Point 5 is stronger than "we do not collect data" and it is measured.** There is no HTTP
client in the dependency tree: `src-tauri/Cargo.toml` has `serde`, `serde_json`, `notify`,
`ignore`, `uuid`, the two Tauri plugins, and the objc2 crates, and nothing that opens a socket.
The one shell-out is `repo::head_commit_time`, documented in place as the fallback when the
reflog read fails. The claim about arbitrary URLs is the same one the age-rating answer rests
on: `PRIVACY_POLICY_URL` in `commands.rs` is a constant, there is deliberately no
`open_url(url)` command, and `capabilities/default.json` grants no shell and no opener plugin.

**Point 7 restates `docs/app-store-licence-check.md`** rather than reasoning about the licence
again. The offer of the receipt is deliberate: point 7 asks for documentation, and attaching a
purchase receipt unasked publishes an order number to a ticket for no gain. If App Review asks,
send it.

**Point 5 states the network entitlement rather than hoping nobody diffs the bundle.** The
paragraph is lifted from the review notes already on the listing, where it is load-bearing for
the reason recorded there: an earlier draft claimed the app "makes no network requests of any
kind" while the signed bundle shipped `com.apple.security.network.client`, which a reviewer is
entitled to read as a false statement. Probe 1 measured that the entitlement is mandatory, so
the answer is to explain it every time the subject comes up, in the reply as well as in the
notes. Do not drop this paragraph to make point 5 read more cleanly.

## The recording

**Record the Developer ID build, not the store build.** The submitted pkg is signed
`3rd Party Mac Developer Installer` and the app inside carries no
`Contents/embedded.provisionprofile`, so it cannot be installed and launched on this machine at
all. `tools/install-local.sh` builds and installs the same source. The visible difference is
nothing: the sandbox changes where the state file lives and adds the security-scoped bookmark
behind the folder picker, and neither of those is on camera.

Build it first, because the copy in `/Applications` is 0.2.0:

```sh
tools/install-local.sh
```

**Before recording anything, clear the screen.** A full-screen capture of this desktop has
already caught a work chat once. Quit every messaging and mail app, close every window that is
not part of the shot, turn on Do Not Disturb, and either quit the default browser first or
accept that step 9 will show whatever it restores. The recording goes to Apple and cannot be
recalled.

QuickTime Player, File > New Screen Recording, whole screen, no microphone. The pet lives in a
corner of the desktop and the popover hangs off the menu bar, so a window-only capture cannot
show the product.

Stage the two repositories first, in a folder with nothing else in it:

```sh
D=~/Documents/mascot-demo
mkdir -p "$D/pixel-diary" "$D/tiny-synth"
git init -q "$D/pixel-diary"
GIT_COMMITTER_DATE="@$(( $(date +%s) - 100*3600 )) +0000" git -C "$D/pixel-diary" \
  -c user.email=demo@example.com -c user.name=Demo commit -q --allow-empty -m "work"
git init -q "$D/tiny-synth"
GIT_COMMITTER_DATE="@$(( $(date +%s) - 30*3600 )) +0000" git -C "$D/tiny-synth" \
  -c user.email=demo@example.com -c user.name=Demo commit -q --allow-empty -m "work"
```

The shot list, in order. Two to three minutes end to end.

1. Launch from `/Applications` in Finder, so the take begins with launching the app. The pet
   appears in the corner and the icon appears in the menu bar.
2. Click the menu bar icon. The popover opens: room, character picker, copy line, and
   "Nothing tracked yet."
3. Click "Add Project" and choose `pixel-diary`. This is the folder picker, and it is the one
   prompt Apple's list asks to see. The row appears and the character is asleep.
4. Click "Add Project" again and choose `tiny-synth`. The mood follows the newest commit
   across all projects, so this moves the character to dozing, which is a state the recording
   would otherwise not contain.
5. Click each of the three character heads. The room and the pet both change.
6. Click the open circle on the `tiny-synth` row to mark it operating. It fills in, and the
   character falls back to asleep because the only project still counting is `pixel-diary`.
   Click it again to undo, and the character returns to dozing.
7. Hover `tiny-synth` and click the x to stop tracking it. The character goes back to asleep,
   with `pixel-diary` alone in the list.
8. Click "Share Status". The button reads "Art copied" for a moment. Paste into a new blank
   TextEdit document to show the 1200x630 card, then close it without saving.
9. Click "privacy". The policy opens in the default browser. Skip this step if the browser
   cannot be trusted to open clean.
10. Drag the pet from one corner to another, and again to a third.
11. Open any app fullscreen and show the pet still visible over it. Pick the app for this
    deliberately: TextEdit with an empty document is safe, a code editor is not.
12. The comeback, which is the payload. With the character asleep, run in Terminal:

    ```sh
    git -C ~/Documents/mascot-demo/pixel-diary \
      -c user.email=demo@example.com -c user.name=Demo commit --allow-empty -m "back at it"
    ```

    The pet leaps out of bed within a few seconds. Open the popover to show the comeback room
    and its line of copy, which also ends the celebration. Do not open the popover before the
    commit: closing it resolves the comeback, and the cap is 30 minutes.
13. Right-click the menu bar icon and choose Quit, so the take ends where it started.

Then delete `~/Documents/mascot-demo`.

**If the file is too large for a Resolution Center attachment**, host it somewhere unlisted and
paste the link in the reply instead. Do not compress the pixel art into mush to fit: this is an
app whose whole case is that it looks good, and the reviewer is about to judge that.
