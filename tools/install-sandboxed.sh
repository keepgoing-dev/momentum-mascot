#!/bin/sh
#
# Builds the app with the STORE's entitlements, signs it so it will actually launch, and
# installs it into /Applications. This is the build the manual test list of spec section 9 wants.
#
# It exists because neither sibling script can do this job:
#
#   install-local.sh  builds a copy that launches and is NOT sandboxed, so nothing it does
#                     proves anything about the sandbox: no container, no bookmarks, no denials.
#   release-mas.sh    builds a copy that IS sandboxed and cannot be launched at all. It is signed
#                     for the store, and macOS only trusts that signature on a bundle the store
#                     installed, so testing behaviour with it means uploading, waiting for review,
#                     and downloading your own app.
#
# The difference here is one flag: the same Entitlements.mas.plist, signed with the Developer ID
# certificate the disk-image channel already uses. App Sandbox is switched on by the entitlement,
# not by the certificate, so this copy is genuinely sandboxed - same container, same $HOME
# redirect, same bookmark requirement - while still being a bundle this Mac will run.
#
# What it is NOT: the exact bits the store ships. Nothing signed on this machine can be. The
# mechanical half of that check is tools/verify-store-copy.sh against a real store install; this
# script is for the half a person has to look at.
#
# Usage:
#
#   tools/install-sandboxed.sh
#   MASCOT_PACK=/path/to/pack tools/install-sandboxed.sh   # recomposite the art first
set -eu

ROOT=$(cd "$(dirname "$0")/.." && pwd)
APP_NAME="Momentum Mascot"
BUNDLE_ID="dev.keepgoing.momentum-mascot"

export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
cd "$ROOT"

if [ -f "$ROOT/tools/.release-env" ]; then
  . "$ROOT/tools/.release-env"
fi

# Tauri signs and notarizes during the build when these are set. Both are wrong here: this script
# signs with its own entitlements afterwards, and a local test build is not notarized.
unset APPLE_SIGNING_IDENTITY APPLE_ID APPLE_PASSWORD

IDENTITY=$(security find-identity -v -p codesigning 2>/dev/null \
  | sed -n 's/.*"\(Developer ID Application: [^"]*\)".*/\1/p' | head -n 1) || true
if [ -z "$IDENTITY" ]; then
  # Ad-hoc still sandboxes: the kernel reads the entitlement, not the certificate. The copy is
  # then bound to this machine, which for a test build on this machine is no loss.
  IDENTITY="-"
  echo "no Developer ID certificate found, signing ad-hoc"
fi

if [ -n "${MASCOT_PACK:-}" ]; then
  "$ROOT/tools/build-app-assets.sh"
elif [ ! -d "$ROOT/src/assets/rooms" ]; then
  echo "error: src/assets is missing and \$MASCOT_PACK is not set" >&2
  exit 1
fi

cargo tauri build --bundles app

APP="$ROOT/src-tauri/target/release/bundle/macos/$APP_NAME.app"
[ -d "$APP" ] || { echo "error: $APP not found after build" >&2; exit 1; }

codesign --force --timestamp --options runtime \
  --sign "$IDENTITY" \
  --entitlements "$ROOT/src-tauri/Entitlements.mas.plist" \
  "$APP"

# The whole point of this build, so it is asserted rather than assumed. A bundle that quietly
# came out unsandboxed would pass every item on the manual list and prove none of them.
# The dots in the key are escaped because plutil reads an unescaped dot as a key-path
# separator, so the plain key name silently resolves to nothing and this check would fail on a
# bundle that is perfectly sandboxed.
if ! codesign -d --entitlements - --xml "$APP" 2>/dev/null \
  | plutil -extract 'com\.apple\.security\.app-sandbox' raw - -o - 2>/dev/null | grep -q '^true$'; then
  echo "error: the signed bundle does not carry com.apple.security.app-sandbox" >&2
  exit 1
fi

pkill -x "$APP_NAME" 2>/dev/null || true
pkill -x "momentum-mascot" 2>/dev/null || true
sleep 1

DST="/Applications/$APP_NAME.app"
rm -rf "$DST"
ditto "$APP" "$DST"

echo ""
echo "installed $DST"
echo "sandboxed: yes. container: ~/Library/Containers/$BUNDLE_ID/Data"
echo "state file: ~/Library/Containers/$BUNDLE_ID/Data/.keepgoing/mascot/state.json"
echo ""
echo "This replaced whatever was at that path. tools/install-local.sh puts the unsandboxed"
echo "development copy back."
