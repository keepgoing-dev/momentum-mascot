#!/bin/sh
#
# Builds, signs, packages and uploads the Mac App Store build.
#
# A sibling of release.sh, deliberately not a modification of it: the DMG path works and must not
# be destabilised. Compared to that script, this one never tags, never touches CHANGELOG.md,
# never creates a GitHub release, and never notarizes. Apple: "you aren't required to notarize
# software that you distribute through the Mac App Store because the App Store submission process
# already includes equivalent security checks."
#
# Version bumping stays in release.sh, so the two channels cannot disagree about what a version
# is. The BUILD NUMBER is this script's own counter, because App Store Connect rejects a re-upload
# that reuses one and a first submission is very likely re-uploaded at least once.
#
# Usage:
#
#   tools/release-mas.sh              # build, sign, package, validate. Uploads nothing.
#   tools/release-mas.sh --upload     # ... and upload to App Store Connect
#
# Credentials come from tools/.release-env, which is gitignored. The one-time Apple setup is in
# docs/app-store.md.
#
# MASCOT_MAS_ALLOW_PRIVATE_API=1 downgrades the private-API gate to a warning so the pipeline can
# be rehearsed before the pet work lands. It also refuses to upload, which is the point: every
# step can be practised except the irreversible one.
set -eu

ROOT=$(cd "$(dirname "$0")/.." && pwd)
APP_NAME="Momentum Mascot"
BUNDLE_ID="dev.keepgoing.momentum-mascot"
BIN_NAME="momentum-mascot"

# rustup's toolchain, ahead of anything else on PATH. Measured, and the reason this line is not
# just tidiness: an x86_64 Homebrew Rust installed at /usr/local/bin (the Intel Homebrew prefix)
# shadows rustup on an Apple silicon Mac, and every build made during the pet work came out
# x86_64-only. macOS surfaced it as a "Support Ending for Intel-based Apps" notification. The
# architecture assertion after the build is the hard gate; this is the fix.
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
cd "$ROOT"

UPLOAD=""
if [ "${1:-}" = "--upload" ]; then
  UPLOAD=1
elif [ $# -gt 0 ]; then
  echo "usage: tools/release-mas.sh [--upload]" >&2
  exit 1
fi

if [ -f "$ROOT/tools/.release-env" ]; then
  . "$ROOT/tools/.release-env"
fi

# Tauri signs with APPLE_SIGNING_IDENTITY and notarizes with APPLE_ID/APPLE_PASSWORD during the
# build when they are set, and both are wrong here: this build is signed by this script with a
# different identity and a different entitlements file, and a store submission is not notarized.
unset APPLE_SIGNING_IDENTITY APPLE_ID APPLE_PASSWORD

VERSION=$(grep -m1 '"version"' src-tauri/tauri.conf.json | sed 's/.*"version": "\([^"]*\)".*/\1/')
echo "version: $VERSION"
echo "cargo:   $(command -v cargo) ($(cargo --version))"

# ---------------------------------------------------------------- signing preflight
#
# Runs before the build on purpose. A missing certificate should cost a second, not a ten minute
# build.
#
# Bare `-v`, never `-p codesigning`. Apple: "Don't use the -p codesigning option... Installer-
# signing identities are different from code-signing identities, so the -p codesigning option
# filters out installer-signing identities." release.sh:118 uses `-p codesigning` correctly for
# its own purpose, and copying that line here would make this step fail on a correctly configured
# machine.

IDENTITIES=$(security find-identity -v 2>/dev/null || true)

if [ -z "${MAS_APP_IDENTITY:-}" ]; then
  MAS_APP_IDENTITY=$(echo "$IDENTITIES" \
    | sed -n 's/.*"\(Apple Distribution: [^"]*\)".*/\1/p' | head -n 1)
fi
if [ -z "${MAS_INSTALLER_IDENTITY:-}" ]; then
  MAS_INSTALLER_IDENTITY=$(echo "$IDENTITIES" \
    | sed -n 's/.*"\(3rd Party Mac Developer Installer: [^"]*\)".*/\1/p' | head -n 1)
fi

if [ -z "$MAS_APP_IDENTITY" ]; then
  echo "error: no 'Apple Distribution' certificate in the keychain" >&2
  echo "" >&2
  echo "  This is the certificate that signs the .app. Create it at" >&2
  echo "  https://developer.apple.com/account/resources/certificates" >&2
  echo "  Its common name reads 'Apple Distribution: <name> (<team id>)'." >&2
  echo "  See docs/app-store.md." >&2
  exit 1
fi

if [ -z "$MAS_INSTALLER_IDENTITY" ]; then
  echo "error: no '3rd Party Mac Developer Installer' certificate in the keychain" >&2
  echo "" >&2
  echo "  This is the certificate that signs the .pkg. The developer portal calls it" >&2
  echo "  'Mac Installer Distribution', which is NOT its common name: no certificate's" >&2
  echo "  common name reads that. Same certificate, different label." >&2
  echo "  See docs/app-store.md." >&2
  exit 1
fi

echo "app identity:       $MAS_APP_IDENTITY"
echo "installer identity: $MAS_INSTALLER_IDENTITY"

if [ -n "$UPLOAD" ]; then
  MISSING=""
  [ -z "${ASC_API_KEY_ID:-}" ] && MISSING="$MISSING ASC_API_KEY_ID"
  [ -z "${ASC_API_ISSUER_ID:-}" ] && MISSING="$MISSING ASC_API_ISSUER_ID"
  if [ -n "$MISSING" ]; then
    echo "error: missing App Store Connect credentials:$MISSING" >&2
    echo "  Copy tools/.release-env.example to tools/.release-env and fill it in." >&2
    exit 1
  fi
  KEYFILE="$HOME/.appstoreconnect/private_keys/AuthKey_$ASC_API_KEY_ID.p8"
  if [ ! -f "$KEYFILE" ]; then
    echo "error: $KEYFILE not found" >&2
    echo "  altool searches ./private_keys, ~/private_keys, ~/.private_keys and" >&2
    echo "  ~/.appstoreconnect/private_keys for AuthKey_<key id>.p8." >&2
    exit 1
  fi
fi

# ---------------------------------------------------------------- assets

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

# ---------------------------------------------------------------- build number
#
# Burned before the build, not after the upload. A wasted number costs nothing; a reused one
# costs a rejected upload and a rebuild.

BUILD_FILE="$ROOT/tools/.mas-build"
if [ -f "$BUILD_FILE" ]; then
  PREV=$(cat "$BUILD_FILE")
else
  PREV=0
fi
BUILD=$((PREV + 1))
echo "$BUILD" > "$BUILD_FILE"
echo "build number: $BUILD"

# ---------------------------------------------------------------- build

echo "building universal macOS app..."

# Start from an empty bundle directory, for the same reason release.sh:223 does: Tauri reuses
# what is there, so a leftover ad-hoc signed .app can survive into a real submission.
rm -rf "$ROOT/src-tauri/target/universal-apple-darwin/release/bundle"

# --bundles app: no .dmg. The store channel does not want one and building it is a minute of
# nothing.
(cd src-tauri && cargo tauri build --target universal-apple-darwin --bundles app)

APP="$ROOT/src-tauri/target/universal-apple-darwin/release/bundle/macos/$APP_NAME.app"
if [ ! -d "$APP" ]; then
  echo "error: $APP not found after the build" >&2
  exit 1
fi

# ---------------------------------------------------------------- architecture gate
#
# Asserted, not printed. During the pet work every build was silently x86_64 on an arm64 Mac
# because of the PATH shadowing described at the top of this script, and nothing in the pipeline
# noticed: the app ran fine under Rosetta and the only complaint came from macOS itself, weeks
# later, as a "Support Ending for Intel-based Apps" notification. An x86_64-only bundle cannot be
# what ships, so a wrong architecture has to fail the build rather than scroll past in a log.

ARCHS=$(lipo -archs "$APP/Contents/MacOS/$BIN_NAME")
echo "architectures: $ARCHS"

for want in arm64 x86_64; do
  case " $ARCHS " in
    *" $want "*) ;;
    *)
      echo "error: the binary has no $want slice (lipo reports: $ARCHS)" >&2
      echo "" >&2
      echo "  The build asked for --target universal-apple-darwin, which must produce both." >&2
      echo "  A missing arm64 slice usually means an x86_64 Rust is shadowing rustup on PATH:" >&2
      echo "    command -v cargo   # expect ~/.cargo/bin/cargo, not /usr/local/bin/cargo" >&2
      echo "  A missing slice can also mean the target is not installed:" >&2
      echo "    rustup target add aarch64-apple-darwin x86_64-apple-darwin" >&2
      exit 1
      ;;
  esac
done

# ---------------------------------------------------------------- private API gate
#
# The two removable private KVC keys, from tauri's macos-private-api feature: drawsBackground
# (wry wkwebview/mod.rs:376, :382, :973) and fullScreenEnabled (:386-388). Both must be gone.
#
# Two OTHER private strings stay, and that is expected, not an oversight:
# allowsPictureInPictureMediaPlayback (wry, behind no feature gate) and _wantsKeyDownForEvent
# (tao, registered unconditionally). Removing those means forking wry and tao. Spec section 2.2.
# So this grep names exactly the two that are ours to remove, and nothing else.

PRIVATE=$(strings -a "$APP/Contents/MacOS/$BIN_NAME" \
  | grep -cE 'drawsBackground|fullScreenEnabled' || true)

if [ "$PRIVATE" -ne 0 ]; then
  if [ -n "${MASCOT_MAS_ALLOW_PRIVATE_API:-}" ]; then
    echo "" >&2
    echo "WARNING: $PRIVATE line(s) still carry drawsBackground / fullScreenEnabled." >&2
    echo "WARNING: rehearsal only. --upload is refused while this is set." >&2
    echo "" >&2
    UPLOAD=""
  else
    echo "error: the binary still carries the two removable private KVC keys" >&2
    echo "" >&2
    echo "  drawsBackground and fullScreenEnabled come from tauri's macos-private-api" >&2
    echo "  feature in src-tauri/Cargo.toml. Dropping it is the pet work in spec" >&2
    echo "  section 4, or the section 4.0 probe if that came out well." >&2
    echo "" >&2
    echo "  To rehearse this script without uploading:" >&2
    echo "    MASCOT_MAS_ALLOW_PRIVATE_API=1 tools/release-mas.sh" >&2
    exit 1
  fi
else
  echo "private API check: clean"
fi

# ---------------------------------------------------------------- build number and profile
#
# Tauri writes tauri.conf.json's version into BOTH CFBundleShortVersionString and CFBundleVersion,
# so without this every upload of 0.3.1 would claim build 0.3.1 and the second one would be
# rejected.

/usr/libexec/PlistBuddy -c "Set :CFBundleVersion $BUILD" "$APP/Contents/Info.plist"
echo "CFBundleShortVersionString: $(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$APP/Contents/Info.plist")"
echo "CFBundleVersion:            $(/usr/libexec/PlistBuddy -c 'Print :CFBundleVersion' "$APP/Contents/Info.plist")"

# A provisioning profile is NOT required. TN3125: "A Mac app that uses no restricted entitlements
# doesn't need a provisioning profile. This is true even if the app is distributed on the App
# Store. The only exception to this rule is TestFlight, which always requires a profile." App
# Sandbox and Hardened Runtime entitlements are both on Apple's unrestricted list. So this is a
# warning, never an error. If one IS present it is copied in BEFORE signing, because "the profile
# is sealed by the code signature".
PROFILE="$ROOT/tools/embedded.provisionprofile"
if [ -f "$PROFILE" ]; then
  cp "$PROFILE" "$APP/Contents/embedded.provisionprofile"
  echo "embedded the provisioning profile"
else
  echo "note: no tools/embedded.provisionprofile. Not required (TN3125); TestFlight would need one."
fi

# ---------------------------------------------------------------- sign
#
# Apple, both current: "Sign code from the inside out" and "Don't pass the --deep option to
# codesign when you sign code."
#
# Measured: this bundle has no nested code at all. Contents holds Info.plist, MacOS/, Resources/
# and the signature. So "inside out" is one call. If nested code ever appears, this refuses rather
# than signing an outer bundle over unsigned inner code.

if [ -d "$APP/Contents/Frameworks" ] || [ -d "$APP/Contents/PlugIns" ] \
  || [ -d "$APP/Contents/XPCServices" ] || [ -d "$APP/Contents/Library" ]; then
  echo "error: nested code appeared in the bundle" >&2
  echo "  Sign the nested code first, without entitlements, then the app. Do not use --deep." >&2
  exit 1
fi

codesign --force --timestamp --options runtime \
  --sign "$MAS_APP_IDENTITY" \
  --entitlements "$ROOT/src-tauri/Entitlements.mas.plist" \
  "$APP"

codesign --verify --strict --verbose=2 "$APP"
echo "sealed entitlements:"
codesign -d --entitlements - "$APP" 2>/dev/null || true

# ---------------------------------------------------------------- package
#
# Verbatim Apple's own recipe: "The following is the simplest use of productbuild, sufficient for
# submitting your app to the Mac App Store: productbuild --sign <Identity> --component
# <PathToApp> /Applications <PathToPackage>".

PKG="$ROOT/src-tauri/target/universal-apple-darwin/release/bundle/$APP_NAME-$VERSION-$BUILD.pkg"
rm -f "$PKG"
productbuild --sign "$MAS_INSTALLER_IDENTITY" --component "$APP" /Applications "$PKG"
echo "packaged: $PKG"

# ---------------------------------------------------------------- validate and upload
#
# altool, not notarytool. TN3147: "Apple has deprecated altool for the purposes of notarization...
# However, altool is still a good way to perform other tasks, like submitting an app to the App
# Store." notarytool is not a store-upload tool. Measured against altool 26.40.1: --upload-package
# and --validate-app both take a path plus authentication and nothing else, and the API key flags
# are --api-key / --api-issuer.

if [ -z "${ASC_API_KEY_ID:-}" ] || [ -z "${ASC_API_ISSUER_ID:-}" ]; then
  echo ""
  echo "no App Store Connect API key configured, so skipping validation."
  echo "package is at: $PKG"
  exit 0
fi

echo "validating the package (this catches signing and entitlement errors before an upload)..."
xcrun altool --validate-app "$PKG" \
  --api-key "$ASC_API_KEY_ID" \
  --api-issuer "$ASC_API_ISSUER_ID"

if [ -n "$UPLOAD" ]; then
  echo "uploading..."
  xcrun altool --upload-package "$PKG" \
    --api-key "$ASC_API_KEY_ID" \
    --api-issuer "$ASC_API_ISSUER_ID" \
    --show-progress
  echo ""
  echo "uploaded $VERSION build $BUILD"
  echo "processing takes a few minutes. Then set the build on the version in App Store Connect."
else
  echo ""
  echo "validated but NOT uploaded. Re-run with --upload when the validation is clean:"
  echo "  tools/release-mas.sh --upload"
  echo ""
  echo "note: that rebuilds and burns build number $((BUILD + 1)), which is fine and expected."
fi
