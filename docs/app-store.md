# Mac App Store: the one-time setup

`docs/notarization.md` covers the DMG channel. This covers the store, and the two use
**different certificates**. The machine currently holds exactly one identity:

```sh
security find-identity -v
#  1) ... "Developer ID Application: Hoa Trinh (3LM6674AC2)"
```

That one signs the DMG and cannot sign a store submission. Submitting with it yields App
Store Connect error 90034, "not signed using an Apple submission certificate".

Design and rationale: `docs/superpowers/specs/2026-08-22-mac-app-store-design.md`.
Implementation plan: `docs/superpowers/plans/2026-08-22-mac-app-store-submission.md`.

## 1. Register the App ID

<https://developer.apple.com/account/resources/identifiers> > Identifiers > App IDs >
App. Bundle ID `dev.keepgoing.momentum-mascot`, explicit, description "Momentum Mascot".
Enable no capabilities: App Sandbox and Hardened Runtime are not capabilities that need
registering, and this app uses no restricted entitlements.

## 2. Create the Apple Distribution certificate

This signs the `.app`.

<https://developer.apple.com/account/resources/certificates> > Certificates > + >
**Apple Distribution**. Generate a CSR from Keychain Access
(Certificate Assistant > Request a Certificate From a Certificate Authority, saved to
disk, 2048-bit RSA), upload it, download the `.cer`, double-click to install.

Its common name reads `Apple Distribution: Hoa Trinh (3LM6674AC2)`.

## 3. Create the Mac Installer Distribution certificate

This signs the `.pkg`, and this is the step with the trap in it.

Same page, + > **Mac Installer Distribution**. Same CSR flow.

**The portal label is not the certificate's common name.** The installed certificate reads
`3rd Party Mac Developer Installer: 3LM6674AC2`. No certificate's common name reads "Mac
Installer Distribution". `tools/release-mas.sh` looks for the real name.

Verify both, and note the flag:

```sh
security find-identity -v
```

Bare `-v`, not `-p codesigning`. Apple: "Don't use the `-p codesigning` option...
Installer-signing identities are different from code-signing identities, so the
`-p codesigning` option filters out installer-signing identities." `release.sh:118` uses
`-p codesigning` correctly for its own purpose; copying that line into a store script
makes it fail on a correctly configured machine.

Expected after this step: three identities, the Developer ID Application one plus these
two.

## 4. Create an App Store Connect API key

<https://appstoreconnect.apple.com/access/integrations/api> > Team Keys > +. Role
**App Manager** is enough for uploads. Download the `.p8` once; it cannot be downloaded
again.

```sh
mkdir -p ~/.appstoreconnect/private_keys
mv ~/Downloads/AuthKey_ABC123DEF4.p8 ~/.appstoreconnect/private_keys/
chmod 600 ~/.appstoreconnect/private_keys/AuthKey_ABC123DEF4.p8
```

Then put the key id and the issuer id into `tools/.release-env`, per
`tools/.release-env.example`. `tools/.release-env` currently holds an app-specific
password, which works, but an API key is the better auth for uploads.

Check it works before a build depends on it:

```sh
xcrun altool --list-providers --api-key "$ASC_API_KEY_ID" --api-issuer "$ASC_API_ISSUER_ID"
```

## 5. Optionally, a provisioning profile

**Not required.** TN3125: "A Mac app that uses no restricted entitlements doesn't need a
provisioning profile. This is true even if the app is distributed on the App Store. The
only exception to this rule is TestFlight, which always requires a profile." Apple's
unrestricted list explicitly includes "entitlements that enable and configure App Sandbox"
and "entitlements that configure the Hardened Runtime", which is every key in
`src-tauri/Entitlements.mas.plist`.

Do it anyway if you want to have walked it: Profiles > + > Mac App Store, pick the App ID
and the Apple Distribution certificate, download, and save it as
`tools/embedded.provisionprofile`. `release-mas.sh` copies it in **before** signing,
because "the profile is sealed by the code signature", and warns rather than failing when
it is absent.

## 6. Create the app record

<https://appstoreconnect.apple.com> > Apps > + > New App. Platform macOS, name
"Momentum Mascot", primary language English (UK or US), bundle ID from step 1, SKU
`momentum-mascot-1`.

The listing content itself is in `docs/app-store-listing.md`.

## Then

```sh
tools/release-mas.sh              # build, sign, package, validate
tools/release-mas.sh --upload     # and upload
```

The script never tags, never touches `CHANGELOG.md`, never creates a GitHub release, and
never notarizes. Version bumps stay in `tools/release.sh` so the two channels cannot
disagree about what a version is; the build number is `release-mas.sh`'s own counter in
`tools/.mas-build`, because App Store Connect rejects a re-upload that reuses one.

## What already passes without the certificates

Two of the script's gates run before signing, so they were verified before any of the setup
above existed, by handing the preflight placeholder identities:

```
architectures: x86_64 arm64
private API check: clean
CFBundleShortVersionString: 0.3.1
CFBundleVersion:            1
```

`private API check: clean` is the whole point of the native pet rewrite. The architecture
gate is there because every build made during that rewrite was silently x86_64 on an arm64
Mac: an Intel Homebrew Rust at `/usr/local/bin` shadows rustup on `PATH`, the app ran fine
under Rosetta, and the only complaint came from macOS weeks later as a "Support Ending for
Intel-based Apps" notification. `release-mas.sh` prepends rustup's `bin` to `PATH` and then
asserts on `lipo -archs` rather than printing it.

So the first run with real certificates should reach `codesign`, `productbuild` and
`altool --validate-app`, and those three are all that is left unrehearsed. A validation
failure there is worth more than anything else in this phase: it names exactly what App
Review's automated checks will object to, for free.
