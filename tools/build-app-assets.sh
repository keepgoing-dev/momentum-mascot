#!/usr/bin/env bash
#
# Builds everything the app needs out of a local licensed copy of Modern Interiors.
#
# Run this once before the first `cargo run`, and again whenever a coordinate in
# compose-rooms.sh changes. It writes into two gitignored trees:
#
#   src/assets/rooms/<char>/<state>.png   12-frame room strip, 1920x112
#   src/assets/pet/<char>/<state>.png     12-frame pet strip, 384x32
#   src/assets/fonts/                     the embedded pixel font
#   src-tauri/icons/tray.png              the menu bar template image
#   src-tauri/icons/bundle/icon.icns      the application icon, for `tauri build`
#
# None of that is in version control, and that is the point: section 4.2 permits shipping
# these compiled into a binary and forbids redistributing them as assets. Anyone picking this
# project up gets the coordinates and needs their own licensed copy of the pack, which is the
# correct outcome. The scripts are the art's source form here; the PNGs are build output.
#
# Env:  MASCOT_PACK   root of moderninteriors-win  (default matches compose-rooms.sh)
#
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
APP_ASSETS="$ROOT/src/assets"
ICONS="$ROOT/src-tauri/icons"
FONT_SRC="$ROOT/assets/fonts/departure-mono"

# Section 6.3: v1 ships three characters, cycled by clicking the character in the popover.
# It is a swap, not a skin system: every premade sheet carries an identical animation set, so
# a character costs one PNG per state and no code at all. Adding a fourth is a line here.
CHARACTERS="07 12 20"

# The review artifacts (GIFs, contact sheets) still go to docs/mockups; the app tree gets
# only what ships. Keeping them apart is what stops a 4x preview GIF being shipped by
# accident.
REVIEW_OUT="${MASCOT_OUT:-$ROOT/docs/mockups}"

echo "composing app assets for characters: $CHARACTERS"
for char in $CHARACTERS; do
  echo "character $char"
  MASCOT_CHAR="$char" \
  MASCOT_OUT="$REVIEW_OUT" \
  MASCOT_APP_OUT="$APP_ASSETS" \
    "$ROOT/tools/compose-rooms.sh"
done

# ---------------------------------------------------------------- the tray icon
# A monochrome template image (section 6.2), drawn rather than cropped and committed rather
# than generated fresh, for the reasons in make-icons.sh. Re-run here anyway so that one
# command still produces everything the app needs.
mkdir -p "$ICONS"
"$ROOT/tools/make-icons.sh" | sed 's/^/  /'

# ---------------------------------------------------------------- the app icon
# The Dock, Finder and disk-image icon, which unlike the tray mark IS derived from the pack and
# is therefore build output. Bundling fails without it, which is correct: a distributable build
# needs the licensed pack anyway, because the rooms and the pet come from it.
"$ROOT/tools/make-app-icon.sh" | sed 's/^/  /'

# ---------------------------------------------------------------- the font
# Vendored under the SIL Open Font License 1.1 and committed at assets/fonts/. It is copied
# rather than referenced because the webview can only load what is inside the frontend root,
# and copied rather than moved so there is still exactly one place the licence text lives.
mkdir -p "$APP_ASSETS/fonts"
cp "$FONT_SRC/DepartureMono-Regular.woff2" "$APP_ASSETS/fonts/"
cp "$FONT_SRC/LICENSE" "$APP_ASSETS/fonts/DepartureMono-LICENSE.txt"
echo "  $APP_ASSETS/fonts/DepartureMono-Regular.woff2"

echo
echo "done. src/assets and src-tauri/icons/bundle are build output and stay out of git."
