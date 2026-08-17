#!/bin/sh
#
# Builds a release locally and publishes it to GitHub.
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
set -eu

ROOT=$(cd "$(dirname "$0")/.." && pwd)
APP="Momentum Mascot"

# Point cargo at rustup's toolchain, not the standalone install in /usr/local/bin.
export PATH="$HOME/.cargo/bin:$PATH"

cd "$ROOT"

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

# ---------------------------------------------------------------- build

echo "building universal macOS app..."
(cd src-tauri && cargo tauri build --target universal-apple-darwin)

DMG=$(ls "$ROOT/src-tauri/target/universal-apple-darwin/release/bundle/dmg/"*.dmg | head -n 1)
if [ ! -f "$DMG" ]; then
  echo "error: .dmg not found after build" >&2
  exit 1
fi

echo "built: $DMG"

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
