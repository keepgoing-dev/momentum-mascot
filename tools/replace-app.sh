#!/bin/sh
#
# Installs a freshly built .app over the previous copy of it, and prints where it went.
#
# The App Store copy is never the one replaced: it is root-owned, tools/verify-store-copy.sh
# reads it from /Applications by default, and putting it back means a redownload. When it holds
# /Applications a local build goes to ~/Applications instead, where the two coexist (same bundle
# id, different state) as recorded in docs/app-store-listing.md.
#
# Usage: DST=$(tools/replace-app.sh "path/to/Built.app")   # notes on stderr, path on stdout
#        APP_DIR=/somewhere tools/replace-app.sh "path/to/Built.app"

set -eu

SRC=${1:?usage: replace-app.sh <path to .app>}
[ -d "$SRC" ] || { echo "error: $SRC not found" >&2; exit 1; }

NAME=$(basename "$SRC")
DIR=${APP_DIR:-/Applications}

is_store_copy() {
  [ -d "$1/Contents/_MASReceipt" ]
}

if [ -z "${APP_DIR:-}" ] && is_store_copy "$DIR/$NAME"; then
  DIR="$HOME/Applications"
  echo "the App Store copy holds /Applications/$NAME, so this build goes to $DIR" >&2
fi

DST="$DIR/$NAME"

if is_store_copy "$DST"; then
  echo "error: $DST is the App Store copy, which this script will not delete" >&2
  echo "remove it yourself, or leave APP_DIR unset to install alongside it" >&2
  exit 1
fi

pkill -x "${NAME%.app}" 2>/dev/null || true
pkill -x "momentum-mascot" 2>/dev/null || true
sleep 1

if [ -e "$DST" ] && [ ! -O "$DST" ]; then
  echo "$DST belongs to another user, removing it with sudo" >&2
  sudo rm -rf "$DST"
fi

mkdir -p "$DIR"
rm -rf "$DST"
ditto "$SRC" "$DST"

echo "$DST"
