# Mac App Store: the one-time setup

`docs/notarization.md` covers the DMG channel. This covers the store, and the two use
**different certificates**. The Developer ID Application certificate that signs the DMG
cannot sign a store submission: doing so yields App Store Connect error 90034, "not signed
using an Apple submission certificate".

Sections 1 to 3 were completed on 25 August 2026, so `security find-identity -v` now
reports three identities. The steps are kept because they are the record of how, and
section 3a is the hour this cost.

Design and rationale: `docs/superpowers/specs/2026-08-22-mac-app-store-design.md`.
Implementation plan: `docs/superpowers/plans/2026-08-22-mac-app-store-submission.md`.

## 1. Register the App ID

<https://developer.apple.com/account/resources/identifiers> > Identifiers > App IDs >
App. Bundle ID `dev.keepgoing.momentum-mascot`, explicit, description "Momentum Mascot".
Enable no capabilities: App Sandbox and Hardened Runtime are not capabilities that need
registering, and this app uses no restricted entitlements.

## 2. Create the Apple Distribution certificate

This signs the `.app`.

**With Xcode installed, skip the portal and the CSR entirely.** Xcode > Settings >
Accounts > select the team > Manage Certificates > **+** offers both this certificate and
the one in section 3, generates the key pair, files the request and installs the result.
This is what was actually done, and it takes about ten seconds each.

Otherwise: <https://developer.apple.com/account/resources/certificates> > Certificates >
+ > **Apple Distribution**. Generate a CSR from Keychain Access (Certificate Assistant >
Request a Certificate From a Certificate Authority, saved to disk, 2048-bit RSA), upload
it, download the `.cer`, double-click to install.

Its common name reads `Apple Distribution: Hoa Trinh (3LM6674AC2)`.

## 3. Create the Mac Installer Distribution certificate

This signs the `.pkg`, and this is the step with the trap in it.

Same **+** menu in Xcode, or the same portal page and CSR flow.

**The portal label is not the certificate's common name.** Measured, the installed
certificate reads `3rd Party Mac Developer Installer: Hoa Trinh (3LM6674AC2)`. No
certificate's common name reads "Mac Installer Distribution". `tools/release-mas.sh`
matches on the prefix and takes whatever follows, so either form works.

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
two. If you get one, read the next section before doing anything else.

## 3a. When the new certificates are "not trusted"

Both certificates can install correctly, with their private keys, and still not appear:

```
$ security find-identity -v
  1) ... "Developer ID Application: Hoa Trinh (3LM6674AC2)"
     1 valid identities found
```

Drop the `-v` to see what is actually wrong. That flag filters to *valid* identities, so a
chain that does not evaluate looks like a certificate that does not exist:

```
$ security find-identity
  2) ... "Apple Distribution: ..." (CSSMERR_TP_NOT_TRUSTED)
  3) ... "3rd Party Mac Developer Installer: ..." (CSSMERR_TP_NOT_TRUSTED)
```

The cause is a missing intermediate. These two are issued by **WWDR G3**, and this machine
had only the **G1**, which expired on 7 February 2023. The Developer ID certificate chains
through a different CA, which is why it kept working and made the problem look like it was
about the new certificates rather than about the CA under them.

Check the issuer against what is installed:

```sh
security find-certificate -c "Apple Distribution" -p | openssl x509 -noout -issuer
security find-certificate -a -c "Apple Worldwide Developer Relations" -p \
  | openssl x509 -noout -subject -dates
```

Fix it by installing the generation the issuer line names:

```sh
curl -fsSLO https://www.apple.com/certificateauthority/AppleWWDRCAG3.cer
security add-certificates -k ~/Library/Keychains/login.keychain-db AppleWWDRCAG3.cer
```

`security find-identity -v` reports three immediately afterwards, with no restart, no
re-issued certificates, and nothing to redo in the portal.

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

Check it works before a build depends on it. **Not with `--list-providers`**, which is
what an earlier draft of this document said: altool refuses it outright with
`AuthenticationFailure("list-providers does not support APIKey authentication.")`. That
command is username-and-password only, so it can never verify an API key.

The check that does work needs no network at all, and note the flags: `--generate-jwt`
takes camelCase `--apiKey` and `--apiIssuer`, where every other altool command takes
`--api-key` and `--api-issuer`.

```sh
set -a; . tools/.release-env; set +a
xcrun altool --generate-jwt --apiKey "$ASC_API_KEY_ID" --apiIssuer "$ASC_API_ISSUER_ID"
```

Exit 0 and a token proves the key id, the issuer id and the `.p8` are mutually consistent
and that altool found the file. The token is a live credential for twenty minutes, so do
not paste it anywhere.

Server-side, the proof arrives for free during the first `--validate-app`: an
authentication failure and a *rejection about the app* are different errors, and getting
the latter means the key was accepted.

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

**This is a prerequisite for validating anything, not a later step.** Without an app
record, `altool --validate-app` fails at once with:

```
ERROR: Cannot determine the Apple ID from Bundle ID 'dev.keepgoing.momentum-mascot'
       and platform 'MAC_OS'. (19)
```

That is App Store Connect answering a query rather than refusing a credential, so it also
happens to be the server-side confirmation that the API key from section 4 works.

<https://appstoreconnect.apple.com> > Apps > + > New App. Platform macOS, name
"Momentum Mascot", primary language English (UK or US), bundle ID from step 1, SKU
`momentum-mascot-1`.

The bundle ID appears in that dropdown only once section 1 is done, and the name is
reserved account-wide the moment the record is created.

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
