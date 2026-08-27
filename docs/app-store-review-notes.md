# App Review Information: the 2.1 reply

**Rejected 27 August 2026, guideline 2.1 Information Needed**, on build 4 of 0.3.1. Nothing
about the binary, the art, or 4.2: App Review asked for a screen recording and seven pieces of
information, and the request is the standard new-app questionnaire rather than a finding about
this app. **No new build is needed.** The reply goes into Resolution Center, the same seven
answers go into the Notes field of App Review Information so the next version does not get
asked again, and build 4 continues in review.

The text below is meant to be pasted. Everything in it is checked against the code rather than
remembered; the section after it records where each answer comes from.

**It goes below the review notes in `docs/app-store-listing.md`, not instead of them.** That
block is written for a reviewer sitting in front of the app and says where to click; this one
answers a questionnaire. Both belong in the Notes field, in that order, and the one paragraph
they share on purpose is the network entitlement.

## The reply, verbatim

```
Thank you for the review. Momentum Mascot has no accounts, no in-app purchases, no
subscriptions, no user-generated content, and makes no network requests. Answers to each
point below, in order.

1. SCREEN RECORDING

Attached. It is one continuous take on a physical Mac, beginning with launching the app,
and it covers every feature the app has. There are no registration, login, deletion,
purchase, subscription, or content-reporting flows to show, because the app has none. The
only permission prompt in the app is the standard folder picker in step 3, which is
included.

2. DEVICES AND OPERATING SYSTEMS TESTED

- MacBook Pro (Mac16,8), Apple M4 Pro, macOS 26.5.2 (25F84)

The app is a universal binary (arm64 and x86_64) and its minimum deployment target is
macOS 10.15.

3. WHAT THE APP DOES, AND FOR WHOM

Momentum Mascot is an ambient desktop companion for people who work on side projects in
their spare time. The user points it at git repositories on their own Mac. It reads exactly
one fact from each: the timestamp of the last commit. From that it picks one of four states
for a pixel character who lives in a small animated room:

- awake, at their desk: committed within the last 24 hours
- dozing: 24 to 72 hours
- asleep: more than 72 hours
- comeback: a commit arrives after the character was asleep, and the character leaps out
  of bed

The character appears as a 64x64 pet in a corner of the desktop, and the full room is in a
popover from the menu bar icon.

The problem it solves is that side projects go quiet for weeks when a day job gets busy,
and every existing tool responds to that with streaks, scores, and reminders that punish
the gap. This app deliberately does the opposite: no streaks, no scores, no notifications,
no leaderboards, and it never states how long it has been. The character simply waits, and
the moment of value is the comeback animation when the user returns. The target audience is
working developers with unfinished personal projects.

There is no server, no account, and no content beyond what is in the app bundle.

4. SETTING UP AND ACCESSING THE MAIN FEATURES

No credentials are needed and there is nothing to sign in to. The app needs at least one
git repository on the review machine, and because the states are defined by how long ago a
commit happened, the fastest way to see all of them is to create a repository with a
backdated commit.

  a. Launch the app. The character appears in the bottom-right corner of the desktop, and
     a small icon appears in the menu bar.

  b. Click the menu bar icon to open the popover: the animated room, a character picker,
     one line of copy, the project list, and two buttons.

  c. In Terminal, create a repository whose last commit is 100 hours old:

       mkdir -p ~/Documents/mascot-demo && cd ~/Documents/mascot-demo
       git init -q .
       GIT_COMMITTER_DATE="@$(( $(date +%s) - 100*3600 )) +0000" \
         git -c user.email=demo@example.com -c user.name=Demo \
         commit -q --allow-empty -m "work"

  d. In the popover, click "Add Project" and choose ~/Documents/mascot-demo. This is the
     folder picker, and it is the only permission prompt the app shows. The project appears
     in the list and the character is ASLEEP, because the last commit is older than 72
     hours.

  e. To see the COMEBACK, which is the app's central feature, commit again in that same
     repository:

       cd ~/Documents/mascot-demo
       git -c user.email=demo@example.com -c user.name=Demo \
         commit --allow-empty -m "back at it"

     Within a few seconds the desktop character leaps out of bed, and reopening the
     popover shows the comeback room and a line of copy for it.

  f. Any ordinary repository with a recent commit shows the AWAKE state. Backdating the
     commit in step (c) by 30 hours instead of 100 shows DOZING.

Everything else is in the popover: click a character head to switch character, click the
open circle on a project row to mark it as "operating" so it keeps its place in the list but
no longer affects the character, hover a row and click the x to stop tracking it, and click
"Share Status" to copy a 1200x630 image of the room to the clipboard, which can then be
pasted into any app that accepts an image. The "privacy" link under the buttons opens the
privacy policy in the default browser. Pressing Escape closes the popover. Right-clicking
the menu bar icon gives an Open and Quit menu, which is where the app is quit.

No sample files are required beyond the repository created above.

5. EXTERNAL SERVICES, TOOLS, OR PLATFORMS USED

None. The app makes no network requests of any kind. There is no data provider, no
authentication service, no payment processor, no analytics, no crash reporter, and no AI
service. Specifically:

- Repository timestamps are read from a local file inside the folder the user chose
  (.git/logs/HEAD), and in a rare fallback case by running the local "git" command.
- All state is one JSON file inside the app's own sandbox container.
- All artwork, animation, and fonts are inside the app bundle.
- The only outbound action in the app is opening the privacy policy URL in the user's
  default browser, which happens only when the user clicks the "privacy" link. The app
  cannot open any other address: that URL is a compile-time constant and the app exposes no
  command that takes a URL.

The bundle does carry com.apple.security.network.client, and that is not a contradiction of
the above. The entitlement is required for WKWebView to reach its own networking process:
without it, a sandboxed webview never finishes navigation, so the popover renders blank and
the app appears broken, with no sandbox violation logged. It grants WebKit that access. The
app itself issues no requests, and the content the webview loads is local files inside the
app bundle.

6. REGIONAL DIFFERENCES

There are none. The app behaves identically in every region. It has no region-gated
features, no region-specific content, no server, and no localization: the interface is
English only.

7. THIRD-PARTY MATERIAL

Two items, both licensed for this use.

- ARTWORK. The rooms, furniture, and characters are derived from "Modern Interiors" by
  LimeZu (limezu.itch.io), a commercial asset pack purchased under its full-version
  licence. That licence permits "Edit and use the asset in any commercial or non commercial
  project", with no restriction on distribution channel, and forbids reselling or
  distributing the asset itself. This app ships the art composited into the application
  binary and exposes no way to extract or export the source sprite sheets, so it uses the
  art rather than redistributing it. The licence requires a credit to limezu.itch.io, which
  is displayed in the app under the buttons in the popover, and in the app's copyright
  field. The purchase receipt and the full licence text can be provided on request.

- TYPEFACE. Departure Mono by Helena Zhang, under the SIL Open Font License 1.1, which
  permits bundling in an application. The licence text ships with the font in the app
  bundle.

The app is not in a regulated industry. It handles no health, financial, or personal data,
and collects nothing.
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
