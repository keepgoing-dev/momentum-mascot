#!/bin/sh
#
# Rebuilds the app and reinstalls it, for the machine you are on.
#
# This is the fast loop for testing your own build. It deliberately does NOT do the
# universal build or the .dmg from README "Packaging it": those are for handing to other
# people, and a universal build compiles every crate twice for an architecture the machine
# you are sitting at cannot run. Here it builds natively (arm64 on Apple Silicon) and
# bundles only the .app, which is all an install needs.
#
# Two traps this script exists to avoid, both documented in the README:
#
#   1. The standalone Rust in /usr/local/bin shadows rustup's, which silently produces an
#      x86_64-only build that runs under Rosetta and reports no problem at all. The PATH
#      export below points cargo at rustup's toolchain instead.
#
#   2. The composited art (src/assets, src-tauri/icons/bundle) is not in version control
#      because the licence forbids redistributing it. Code changes don't touch the art, so
#      this script skips recompositing unless $MASCOT_PACK is set — which is also the only
#      time it is even possible.
#
# Usage:
#
#   tools/install-local.sh                     # rebuild + reinstall using art already on disk
#   MASCOT_PACK=/path/to/pack tools/install-local.sh   # also recomposite the art first
#   APP_DIR=/Applications tools/install-local.sh       # force the folder it installs into

set -eu

ROOT=$(cd "$(dirname "$0")/.." && pwd)
APP="Momentum Mascot"

# Point cargo at rustup's toolchain, not the standalone install in /usr/local/bin.
export PATH="$HOME/.cargo/bin:$PATH"

# Recomposite the art only when the pack is available. Otherwise the art must already be
# on disk from an earlier build, and a missing pack is not an error.
if [ -n "${MASCOT_PACK:-}" ]; then
  "$ROOT/tools/build-app-assets.sh"
elif [ ! -d "$ROOT/src/assets/rooms" ]; then
  echo "error: src/assets is missing and \$MASCOT_PACK is not set" >&2
  echo "run: MASCOT_PACK=/path/to/pack tools/install-local.sh" >&2
  exit 1
fi

(cd "$ROOT" && cargo tauri build --bundles app)

SRC="$ROOT/src-tauri/target/release/bundle/macos/$APP.app"

# Quits a running copy, then installs. It picks the folder: /Applications normally, and
# ~/Applications when the App Store copy is sitting in /Applications.
DST=$("$ROOT/tools/replace-app.sh" "$SRC")

echo "installed $DST"
