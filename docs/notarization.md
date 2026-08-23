# Notarization

Everything here is a one-time setup, done once when the Apple Developer Program membership
becomes active. After that `tools/release.sh patch` signs, notarizes and staples on its own,
and this document is only useful when something breaks.

> This document is the **direct download** channel: a Developer ID certificate, notarized,
> shipped as a `.dmg`. The Mac App Store channel uses different certificates and a
> different script, and is documented in [`docs/app-store.md`](app-store.md).

## Why bother

Before notarization, the first launch on any machine that did not build the app hits
*"Momentum Mascot cannot be opened because the developer cannot be verified"*.

The old escape from that dialog was Control-click then **Open**. macOS Sequoia removed it.
On Sequoia and later the user has to open System Settings, go to Privacy & Security, scroll to
a warning about a blocked app, and click **Open Anyway**. That is not a smaller version of the
same friction, it is a different order of it, and it happens to a person who has known about
this app for ninety seconds.

There is no measurement to fall back on either. The app makes no network requests by design,
so the only number available is the GitHub Releases download count, and a download count
cannot distinguish *nobody wanted it* from *nobody got past Gatekeeper*. Until the build is
notarized, every conclusion drawn from that number is unsafe.

## One-time setup

### 1. Confirm the membership is active

<https://developer.apple.com/account>. Enrolment takes up to two days. Nothing below works
until the account page stops saying pending.

### 2. Create a Developer ID Application certificate

This is the certificate for distributing outside the App Store. A Mac Development or Apple
Distribution certificate is a different thing and will not notarize.

Easiest path, through Xcode:

**Xcode → Settings → Accounts → (your Apple ID) → Manage Certificates → + → Developer ID
Application.**

Without Xcode: create a Certificate Signing Request in **Keychain Access → Certificate
Assistant → Request a Certificate From a Certificate Authority**, upload it at
<https://developer.apple.com/account/resources/certificates>, choose **Developer ID
Application**, then download and double-click the `.cer`.

Confirm it landed:

```sh
security find-identity -v -p codesigning
```

One line should read `Developer ID Application: Your Name (TEAMID1234)`. That whole quoted
string is `APPLE_SIGNING_IDENTITY`, and the ten characters in the parentheses are the Team ID.

### 3. Create an app-specific password

<https://account.apple.com> → **Sign-In and Security** → **App-Specific Passwords** → **+**.

This is not the Apple ID password. Apple's notary service refuses the real one. The format is
`xxxx-xxxx-xxxx-xxxx`, it is shown exactly once, and it is revocable from the same page if it
ever leaks.

### 4. Fill in the credentials file

```sh
cp tools/.release-env.example tools/.release-env
$EDITOR tools/.release-env
```

`tools/.release-env` is gitignored. Nothing else in the repository holds a secret, and it
should stay that way: no secret belongs in `tauri.conf.json`, in `release.sh`, or in shell
history, which is why the script sources a file rather than taking arguments.

### 5. Smoke-test before cutting a release

`tools/release.sh` commits and pushes a tag *before* it builds, so a signing failure during a
real release leaves a published tag to clean up. The preflight catches missing credentials, but
it cannot catch a certificate that exists and does not work. Prove the build end to end first:

Set `APPLE_SIGNING_IDENTITY` in `tools/.release-env` before doing this. `release.sh` falls back
to auto-detecting the certificate, but a bare `cargo tauri build` does not: with the variable
empty, Tauri does not complain, it silently ad-hoc signs and skips notarization, and the build
looks like it succeeded.

```sh
set -a; . tools/.release-env; set +a
[ -n "$APPLE_SIGNING_IDENTITY" ] || { echo "APPLE_SIGNING_IDENTITY is unset"; exit 1; }
(cd src-tauri && cargo tauri build --target universal-apple-darwin)
```

Then check the result:

```sh
APP="src-tauri/target/universal-apple-darwin/release/bundle/macos/Momentum Mascot.app"

codesign -dv --verbose=4 "$APP" 2>&1 | grep -E 'Authority|flags'
codesign --verify --deep --strict --verbose=2 "$APP"
xcrun stapler validate "$APP"
spctl --assess --type execute --verbose "$APP"
```

What each one is telling you:

- `Authority=Developer ID Application: ...` means it is signed with the right certificate.
- `flags=0x10000(runtime)` means the hardened runtime is on. Notarization is rejected without
  it.
- `stapler validate` succeeding means the ticket is attached to the app, so first launch works
  with no network.
- `spctl` printing `accepted` and `source=Notarized Developer ID` is the actual answer to the
  question this whole document exists for.

If instead `codesign` reports `Signature=adhoc` and `TeamIdentifier=not set`, the identity was
not in the environment and nothing was signed or notarized. Fix the variable and rebuild. Delete
`src-tauri/target/universal-apple-darwin/release/bundle` first, because Tauri reuses an existing
bundle and the stale unsigned one will otherwise pass straight through.

**Launch it once from that path before releasing.** The hardened runtime is the change most
likely to break something at runtime rather than at build time, and the way it breaks here is
specific: WKWebView needs to map executable memory for JavaScriptCore, so if the popover opens
blank or the pet never appears, the entitlement is the cause. `src-tauri/Entitlements.plist`
grants `com.apple.security.cs.allow-jit` for exactly that reason. If a blank webview survives
that, add `com.apple.security.cs.allow-unsigned-executable-memory` and try again.

### 6. Cut the release

```sh
tools/release.sh patch
```

Notarization adds a few minutes to the build while `notarytool --wait` sits on Apple's queue.
The script staples the `.dmg` afterwards and refuses to publish if verification fails.

## Escape hatch

```sh
MASCOT_SKIP_NOTARIZE=1 tools/release.sh patch
```

Builds ad-hoc signed, exactly like every release up to v0.3.0. The result only opens on the
machine that built it, so this is for testing the release plumbing, never for publishing.

## What is not covered

**Windows.** Authenticode signing is a separate certificate from a separate vendor, priced per
year rather than bundled, and an OV certificate still shows a SmartScreen warning until it
builds reputation. Nothing here helps with it.

**The certificate expiring.** Developer ID certificates last five years, and signatures made
with a secure timestamp keep validating after expiry. Notarization tickets do not expire
either. So a build shipped today keeps opening cleanly even if the membership lapses, and the
only thing a lapsed membership blocks is signing *new* builds.
