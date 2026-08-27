#!/usr/bin/env bash
#
# The mechanical half of the section 9 test list, run against an installed App Store copy.
#
# Everything in spec section 9 that a script can answer is here; everything that needs eyes on a
# screen is deliberately not, and the list is printed at the end so the two halves stay together.
#
# Why an installed store copy rather than the build directory: the pkg that `release-mas.sh`
# uploads is signed `3rd Party Mac Developer Installer` and carries no provisioning profile, so it
# cannot be installed and launched here at all. The Developer ID build from `install-local.sh` runs
# but is NOT sandboxed (`Entitlements.plist` against `Entitlements.mas.plist`), so it cannot answer
# the sandbox questions. Once a version is live, the store copy is the only artifact that is both
# the shipped bits and runnable.
#
# Usage:  tools/verify-store-copy.sh ["/Applications/Momentum Mascot.app"]
#
# Read-only: nothing here launches, modifies, or signs anything.

set -uo pipefail

APP="${1:-/Applications/Momentum Mascot.app}"
BIN_NAME="momentum-mascot"
BIN="$APP/Contents/MacOS/$BIN_NAME"
CONTAINER="$HOME/Library/Containers/dev.keepgoing.momentum-mascot"

FAILED=0

pass() { printf '  \033[32mok\033[0m    %s\n' "$1"; }
fail() { printf '  \033[31mFAIL\033[0m  %s\n' "$1"; FAILED=$((FAILED + 1)); }
info() { printf '  --    %s\n' "$1"; }

[ -d "$APP" ] || { echo "not found: $APP" >&2; exit 1; }
[ -f "$BIN" ] || { echo "no binary at $BIN" >&2; exit 1; }

echo ""
echo "$APP"
echo ""

echo "provenance"
IS_STORE=
if [ -f "$APP/Contents/_MASReceipt/receipt" ]; then
  IS_STORE=1
  pass "_MASReceipt present: this is a Mac App Store copy"
else
  fail "_MASReceipt absent: this is NOT a store copy, so the sandbox answers below are not the shipped ones"
fi
# Authority lines only. The full codesign dump carries certificate hashes that have no business
# in a terminal someone might paste from.
codesign -dv --verbose=2 "$APP" 2>&1 | grep '^Authority=' | while read -r line; do
  info "$line"
done
if codesign --verify --deep --strict "$APP" 2>/dev/null; then
  pass "signature verifies"
elif [ -z "$IS_STORE" ]; then
  # `install-local.sh` passes no identity, so its bundle is ad-hoc and --deep --strict reports
  # "code has no resources but signature indicates they must be present". Expected there, and
  # not worth a red line on a build that was never meant to leave the machine.
  info "signature does not verify, which is what an ad-hoc local build looks like"
else
  fail "signature does not verify"
fi

echo ""
echo "sandbox"
ENT=$(codesign -d --entitlements - --xml "$APP" 2>/dev/null | plutil -convert xml1 -o - - 2>/dev/null)
for key in com.apple.security.app-sandbox \
           com.apple.security.files.user-selected.read-only \
           com.apple.security.files.bookmarks.app-scope; do
  if printf '%s' "$ENT" | grep -q "$key"; then
    pass "$key"
  else
    fail "$key missing"
  fi
done
if [ -d "$CONTAINER" ] && [ -z "$IS_STORE" ]; then
  # The container is keyed on the bundle id, not on the copy, so an unsandboxed build sits beside
  # a container left behind by some earlier sandboxed one. Saying "ok" here would credit this
  # copy with a file it cannot even reach.
  info "a container exists, but this copy is not sandboxed: it belongs to some earlier sandboxed run"
elif [ -d "$CONTAINER" ]; then
  pass "container exists: ~/Library/Containers/dev.keepgoing.momentum-mascot"
  STATE="$CONTAINER/Data/.keepgoing/mascot/state.json"
  if [ -f "$STATE" ]; then
    pass "state file is inside the container ($(wc -c <"$STATE" | tr -d ' ') bytes)"
    if command -v python3 >/dev/null; then
      python3 - "$STATE" <<'PY'
import json,sys
d=json.load(open(sys.argv[1]))
ps=d.get("tracked_projects",[])
booked=sum(1 for p in ps if p.get("bookmark"))
print(f"  --    {len(ps)} tracked project(s), {booked} with a security-scoped bookmark")
PY
    fi
  else
    info "no state file yet: launch it and add a project, then re-run for the persistence check"
  fi
else
  info "no container yet: the app has not been launched"
fi

echo ""
echo "private API gate"
# The same grep as tools/release-mas.sh, on the delivered binary rather than the one we built.
PRIVATE=$(strings -a "$BIN" | grep -cE 'drawsBackground|fullScreenEnabled')
if [ "$PRIVATE" -eq 0 ]; then
  pass "drawsBackground / fullScreenEnabled: 0 occurrences"
else
  fail "$PRIVATE occurrence(s) of drawsBackground / fullScreenEnabled"
fi
# The debug-only escape hatches must not exist in a shipped binary either.
for v in KEEPGOING_CLOCK_SCALE KEEPGOING_MASCOT_STATE KEEPGOING_PIN_POPOVER KEEPGOING_HOLD_COMEBACK; do
  n=$(strings -a "$BIN" | grep -c "$v")
  if [ "$n" -eq 0 ]; then pass "$v absent"; else fail "$v present ($n)"; fi
done

echo ""
echo "bundle"
info "version $(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$APP/Contents/Info.plist" 2>/dev/null), build $(/usr/libexec/PlistBuddy -c 'Print :CFBundleVersion' "$APP/Contents/Info.plist" 2>/dev/null)"
info "architectures: $(lipo -archs "$BIN" 2>/dev/null)"
if [ "$(/usr/libexec/PlistBuddy -c 'Print :LSUIElement' "$APP/Contents/Info.plist" 2>/dev/null)" = "true" ]; then
  pass "LSUIElement: no Dock icon on launch"
else
  fail "LSUIElement is not true, so a Dock icon flashes on every launch"
fi
CAT=$(/usr/libexec/PlistBuddy -c 'Print :LSApplicationCategoryType' "$APP/Contents/Info.plist" 2>/dev/null)
if [ "$CAT" = "public.app-category.developer-tools" ]; then
  pass "category matches the listing: $CAT"
else
  fail "category is $CAT, and the listing says Developer Tools"
fi

echo ""
if [ "$FAILED" -eq 0 ]; then
  echo "all mechanical checks passed."
else
  echo "$FAILED check(s) failed."
fi

cat <<'LIST'

What is left needs a person at the screen. Spec section 9, in the order that fails fastest:

  1. The pet is visible and non-hostile over a fullscreen app: it appears, clicking it does not
     switch Space or steal focus, and the app underneath stays interactive.
  2. The pet drags to all four corners and snaps, and a click opens the popover.
  3. The popover: add a project, cycle a character, toggle operating, untrack, copy the share
     card and paste it somewhere, dismiss with Escape.
  4. Sandbox persistence: add a repository, quit from the tray, relaunch, and it is still
     readable. Re-run this script afterwards to see the bookmark count.
  5. A tracked `git worktree` checkout shows its own message; an ordinary clone is unaffected.
  6. The privacy link opens the hosted page.
  7. The popover's rounded corners read correctly on a light and a dark desktop.
  8. Pixel art stays crisp when the pet is dragged to a display of a different density.

The comeback room is the ninth and cannot be staged in a release build: it needs
`KEEPGOING_HOLD_COMEBACK`, which the gate above proves is absent. Stage it with
`tools/hold-state.sh comeback` on a debug build, or reach it in a store copy the slow way, by
tracking a repository whose last commit is over 72 hours old and then committing to it.
LIST
