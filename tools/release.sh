#!/bin/sh
#
# Builds a signed, notarized release locally and publishes it to GitHub.
#
# Because the LimeZu Modern Interiors pack cannot be redistributed, this runs on the machine
# that already has a licensed copy of the pack (or the composed assets from a previous build).
# It does not upload the pack or the composed assets anywhere except inside the final .dmg.
#
# Usage:
#
#   tools/release.sh 0.1.1              # bump to explicit version
#   tools/release.sh patch              # bump patch (0.1.0 -> 0.1.1)
#   tools/release.sh minor              # bump minor (0.1.0 -> 0.2.0)
#   tools/release.sh major              # bump major (0.1.0 -> 1.0.0)
#
# Signing credentials come from tools/.release-env, which is gitignored. Copy
# tools/.release-env.example and fill it in once. See docs/notarization.md for the
# one-time Apple setup.
#
# MASCOT_SKIP_NOTARIZE=1 builds ad-hoc signed instead, which is fine for a local smoke test
# and produces a .dmg that only opens on the machine that built it.
#
set -eu

ROOT=$(cd "$(dirname "$0")/.." && pwd)
APP="Momentum Mascot"

# Point cargo at rustup's toolchain, not the standalone install in /usr/local/bin.
export PATH="$HOME/.cargo/bin:$PATH"

cd "$ROOT"

# Local, gitignored signing credentials. Absent is fine; the preflight below explains what
# is missing rather than failing halfway through a build.
if [ -f "$ROOT/tools/.release-env" ]; then
  . "$ROOT/tools/.release-env"
fi

# ---------------------------------------------------------------- current version

CURRENT=$(grep -m1 '"version"' src-tauri/tauri.conf.json | sed 's/.*"version": "\([^"]*\)".*/\1/')
echo "current version: $CURRENT"

# ---------------------------------------------------------------- version argument

bump_version() {
  major=$(echo "$1" | cut -d. -f1)
  minor=$(echo "$1" | cut -d. -f2)
  patch=$(echo "$1" | cut -d. -f3)

  case "$2" in
    major) echo "$((major + 1)).0.0" ;;
    minor) echo "$major.$((minor + 1)).0" ;;
    patch) echo "$major.$minor.$((patch + 1))" ;;
    *) echo "$2" ;;
  esac
}

if [ $# -eq 0 ]; then
  printf 'new version (or patch/minor/major): '
  read -r ARG
else
  ARG=$1
fi

VERSION=$(bump_version "$CURRENT" "$ARG")

if ! echo "$VERSION" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
  echo "error: invalid version '$VERSION'" >&2
  exit 1
fi

echo "releasing version: $VERSION"

# ---------------------------------------------------------------- preflight checks

if ! git diff --quiet HEAD; then
  echo "error: working tree has uncommitted changes" >&2
  exit 1
fi

if ! command -v gh >/dev/null 2>&1; then
  echo "error: gh CLI is not installed" >&2
  exit 1
fi

if ! gh auth status >/dev/null 2>&1; then
  echo "error: gh CLI is not authenticated" >&2
  exit 1
fi

if [ -n "${MASCOT_PACK:-}" ]; then
  echo "recompositing assets from \$MASCOT_PACK..."
  "$ROOT/tools/build-app-assets.sh"
elif [ ! -d "$ROOT/src/assets/rooms" ]; then
  echo "error: src/assets is missing" >&2
  echo "set \$MASCOT_PACK to recomposite, or run tools/build-app-assets.sh first" >&2
  exit 1
else
  echo "using existing composed assets in src/assets/"
fi

# ---------------------------------------------------------------- signing preflight
#
# This runs before the tag is pushed on purpose. A missing certificate should cost a second,
# not a ten minute build that has already published a tag someone has to delete.

SKIP_NOTARIZE=${MASCOT_SKIP_NOTARIZE:-}

if [ -n "$SKIP_NOTARIZE" ]; then
  echo ""
  echo "MASCOT_SKIP_NOTARIZE is set."
  echo "building ad-hoc signed and NOT notarized: the .dmg will only open on this machine."
  echo ""
else
  # Fall back to whatever Developer ID cert is in the keychain if one was not named.
  if [ -z "${APPLE_SIGNING_IDENTITY:-}" ]; then
    APPLE_SIGNING_IDENTITY=$(security find-identity -v -p codesigning 2>/dev/null \
      | sed -n 's/.*"\(Developer ID Application: [^"]*\)".*/\1/p' | head -n 1) || true
  fi

  if [ -z "${APPLE_SIGNING_IDENTITY:-}" ]; then
    echo "error: no 'Developer ID Application' certificate found in the keychain" >&2
    echo "" >&2
    echo "  Create one at https://developer.apple.com/account/resources/certificates" >&2
    echo "  and download it into Keychain Access, then re-run." >&2
    echo "  See docs/notarization.md for the whole sequence." >&2
    echo "" >&2
    echo "  To build without signing: MASCOT_SKIP_NOTARIZE=1 tools/release.sh $ARG" >&2
    exit 1
  fi

  MISSING=""
  [ -z "${APPLE_ID:-}" ] && MISSING="$MISSING APPLE_ID"
  [ -z "${APPLE_PASSWORD:-}" ] && MISSING="$MISSING APPLE_PASSWORD"
  [ -z "${APPLE_TEAM_ID:-}" ] && MISSING="$MISSING APPLE_TEAM_ID"

  if [ -n "$MISSING" ]; then
    echo "error: missing notarization credentials:$MISSING" >&2
    echo "" >&2
    echo "  Copy tools/.release-env.example to tools/.release-env and fill it in." >&2
    echo "  APPLE_PASSWORD is an app-specific password from appleid.apple.com," >&2
    echo "  not your Apple ID password. See docs/notarization.md." >&2
    exit 1
  fi

  # Tauri reads all four of these during the build: it signs the .app with the identity, then
  # submits it to the notary service and staples the ticket before assembling the .dmg.
  export APPLE_SIGNING_IDENTITY APPLE_ID APPLE_PASSWORD APPLE_TEAM_ID

  echo ""
  echo "signing identity: $APPLE_SIGNING_IDENTITY"
  echo "notarizing as:    $APPLE_ID (team $APPLE_TEAM_ID)"
  echo ""
fi

# ---------------------------------------------------------------- update version files

sed -i '' "s/\"version\": \"$CURRENT\"/\"version\": \"$VERSION\"/" src-tauri/tauri.conf.json
sed -i '' "s/^version = \"$CURRENT\"/version = \"$VERSION\"/" src-tauri/Cargo.toml

# Keep Cargo.lock in sync.
cargo update --manifest-path src-tauri/Cargo.toml -p momentum-mascot >/dev/null

# ---------------------------------------------------------------- update changelog

DATE=$(date +%Y-%m-%d)

if grep -q "^## $VERSION$" CHANGELOG.md; then
  # Add release date to an existing unreleased section.
  sed -i '' "s/^## $VERSION$/## $VERSION ($DATE)/" CHANGELOG.md
elif ! grep -q "^## $VERSION (" CHANGELOG.md; then
  # Prepend a new section if none exists.
  {
    echo "# Changelog"
    echo ""
    echo "## $VERSION ($DATE)"
    echo ""
    echo "- Release $VERSION."
    echo ""
  } > CHANGELOG.md.new
  # Append the rest of the existing changelog, skipping its own title.
  tail -n +3 CHANGELOG.md >> CHANGELOG.md.new
  mv CHANGELOG.md.new CHANGELOG.md
fi

# ---------------------------------------------------------------- commit and tag

git add src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock CHANGELOG.md
git commit -m "Release v$VERSION"
git tag -a "v$VERSION" -m "Release v$VERSION"
git push origin "$(git branch --show-current)" "v$VERSION"

# ---------------------------------------------------------------- build

echo "building universal macOS app..."
(cd src-tauri && cargo tauri build --target universal-apple-darwin)

DMG=$(ls "$ROOT/src-tauri/target/universal-apple-darwin/release/bundle/dmg/"*.dmg | head -n 1)
if [ ! -f "$DMG" ]; then
  echo "error: .dmg not found after build" >&2
  exit 1
fi

echo "built: $DMG"

# ---------------------------------------------------------------- notarize the disk image
#
# Tauri already notarized and stapled the .app inside. The .dmg is a separate artifact and
# carries its own quarantine flag, so it needs its own ticket. Without the staple the first
# open of the .dmg is an online Gatekeeper check, which fails on a flaky connection and shows
# the "cannot be opened" dialog this whole exercise exists to remove.

if [ -n "$SKIP_NOTARIZE" ]; then
  echo "skipping notarization of the .dmg"
else
  echo "notarizing the .dmg (this takes a few minutes)..."
  xcrun notarytool submit "$DMG" \
    --apple-id "$APPLE_ID" \
    --team-id "$APPLE_TEAM_ID" \
    --password "$APPLE_PASSWORD" \
    --wait

  echo "stapling the ticket..."
  xcrun stapler staple "$DMG"

  echo "verifying..."
  xcrun stapler validate "$DMG"
  spctl --assess --type open --context context:primary-signature -v "$DMG"

  echo "notarized and stapled."
fi

# ---------------------------------------------------------------- publish

echo "creating GitHub release v$VERSION..."
gh release create "v$VERSION" \
  --title "$APP v$VERSION" \
  --generate-notes \
  "$DMG"

echo ""
echo "released v$VERSION"
echo "  tag:    v$VERSION"
echo "  asset:  $DMG"
if [ -n "$SKIP_NOTARIZE" ]; then
  echo "  NOTE:   not notarized, this build will not open on other machines"
fi
