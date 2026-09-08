# App Store Connect release automation: implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the hand-driven App Store Connect panel session at the end of every release with `tools/release-store.sh`, which converges the live listing to the repository, proves it by reading back, and submits only under an explicit flag.

**Architecture:** Stateless convergence. No local state file and no resume flag: every run reads App Store Connect, compares against files in the repository, and writes only the difference. Transport lives in `tools/lib/asc.sh`, pure comparison logic in `tools/lib/listing.sh` so it is testable with no network, and orchestration in `tools/release-store.sh`. The review wait detaches into `tools/release-watch.sh`.

**Tech Stack:** POSIX `sh`, `curl`, `/usr/bin/jq`, `magick identify`, and `xcrun altool --generate-jwt` for the ES256 JWT. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-09-08-release-store-automation-design.md`

## Global Constraints

- **Shell dialect.** Entry-point tools are `#!/bin/sh` with `set -eu`, matching `tools/release.sh` and `tools/release-mas.sh`. `verify-*.sh` scripts may be `#!/usr/bin/env bash` with `set -uo pipefail`, matching `tools/verify-baker.sh` and `tools/verify-store-copy.sh`.
- **No new dependencies.** `jq` is `/usr/bin/jq`. `magick` is already required by `tools/compose-rooms.sh`. `xcrun altool` is already required by `tools/release-mas.sh`. Nothing else may be added, and in particular no `pip install`: `python3` on this machine has neither `jwt` nor `cryptography`.
- **Comments: no comment may exceed two lines, ever.** The default number of comments in a diff is zero. Write one only for a non-obvious constraint a reader cannot recover from the code. Never write section banners, narration of obvious flow, changelog or attribution notes, incident narratives with dates and version numbers, or rejected alternatives. Long reasoning goes in the spec or the commit message, and the code gets at most a two-line pointer.
- **Never use em dashes** in code, comments, documentation, or commit messages. Use a hyphen or rewrite.
- **Credentials** come from `tools/.release-env`, which is gitignored, and the private key from `~/.appstoreconnect/private_keys/AuthKey_$ASC_API_KEY_ID.p8`. Never echo a JWT, never put one on a command line, never commit one.
- **`xcrun altool --generate-jwt` writes the token to stderr behind a banner line.** The extraction is `2>&1 | awk 'END{print $NF}'`. Redirecting stderr to `/dev/null` discards the token and yields an empty string rather than an error.
- **App id** is `6804925509`. **Bundle id** is `dev.keepgoing.momentum-mascot`. **Platform** is `MAC_OS`. **Locale** is `en-US` and only `en-US`. **screenshotDisplayType** is `APP_DESKTOP`.
- **App Store Connect filter parameters need URL-encoded brackets:** `filter%5Bapp%5D=6804925509`. Unencoded brackets return a body that is not JSON.
- **Field files hold the exact API value plus one trailing newline.** Every comparison strips that one newline before comparing. This is the single most likely source of a spurious mismatch.

---

## File structure

| File | Responsibility |
|---|---|
| `tools/lib/asc.sh` | Transport only. Mint and refresh the JWT, issue requests, turn a non-2xx into a readable failure. Knows nothing about listings. |
| `tools/lib/listing.sh` | Pure logic. Field limits, field comparison, screenshot manifest and slot comparison. No network, so it is testable offline. |
| `tools/release-store.sh` | Orchestration. The six phases, the flags, the output table. |
| `tools/release-watch.sh` | Poll `appVersionState` until it settles, report to stdout and to Notification Center. |
| `tools/verify-release-store.sh` | The offline test suite over `tools/testdata/`. |
| `tools/testdata/asc/*.json` | Captured real API responses, used as fixtures. |
| `docs/store/fields/*.txt` | The field text. The only input the script reads for copy. |

`asc.sh` and `listing.sh` are sourced, never executed, following `tools/lib/tints.sh`.

---

## Task 1: Verify the rehearsal loop before building on it

The whole test strategy in spec section 9 rests on being able to create a throwaway version and delete it. That has not been verified against this key. Find out before anything is built on the assumption.

**Files:**
- Create: none. This task produces a recorded answer.
- Modify: `docs/superpowers/specs/2026-09-08-release-store-automation-design.md` (section 9, the unverified-assumption paragraph)

**Interfaces:**
- Consumes: nothing.
- Produces: a verified yes or no on `DELETE /v1/appStoreVersions/{id}`, which decides whether later tasks rehearse against a throwaway version or against the real next version.

- [ ] **Step 1: Mint a token and create a throwaway version**

```sh
. tools/.release-env
T=$(xcrun altool --generate-jwt --apiKey "$ASC_API_KEY_ID" --apiIssuer "$ASC_API_ISSUER_ID" 2>&1 | awk 'END{print $NF}')
API=https://api.appstoreconnect.apple.com/v1

curl -sS -X POST "$API/appStoreVersions" \
  -H "Authorization: Bearer $T" -H "Content-Type: application/json" \
  -d '{"data":{"type":"appStoreVersions",
       "attributes":{"platform":"MAC_OS","versionString":"0.4.1"},
       "relationships":{"app":{"data":{"type":"apps","id":"6804925509"}}}}}' \
  | jq '{id: .data.id, state: .data.attributes.appVersionState, errors: .errors}'
```

Expected: an id and `PREPARE_FOR_SUBMISSION`. If it returns an error about the version string, try `0.5.0`: App Store Connect requires a version greater than the last one.

- [ ] **Step 2: Delete it**

```sh
VID=<the id from step 1>
curl -sS -o /dev/null -w '%{http_code}\n' -X DELETE "$API/appStoreVersions/$VID" \
  -H "Authorization: Bearer $T"
```

Expected: `204`.

- [ ] **Step 3: Confirm it is gone**

```sh
curl -sS -H "Authorization: Bearer $T" "$API/apps/6804925509/appStoreVersions?limit=5" \
  | jq -r '.data[].attributes.versionString'
```

Expected: `0.4.0 0.3.2 0.3.1` and no `0.4.1`.

If step 2 returned anything but `204`, the draft version is harmless but must be removed by hand in the panel, and the answer to record is no.

- [ ] **Step 4: Record the answer in the spec**

Replace the paragraph in section 9 beginning "**This rehearsal depends on**" with what actually happened, including the status code. If the answer is no, also change spec section 9's "Full rehearsal against a throwaway version" paragraph to rehearse against the real next version instead, and note that the rehearsal is then not disposable.

- [ ] **Step 5: Commit**

```sh
git add docs/superpowers/specs/2026-09-08-release-store-automation-design.md
git commit -m "Record whether a draft version can be deleted

The rehearsal strategy in section 9 assumed it. Measured instead."
```

---

## Task 2: tools/lib/asc.sh, the transport

**Files:**
- Create: `tools/lib/asc.sh`
- Create: `tools/verify-release-store.sh`

**Interfaces:**
- Consumes: `ASC_API_KEY_ID` and `ASC_API_ISSUER_ID` from `tools/.release-env`.
- Produces:
  - `asc_get <path>` prints the response body to stdout, returns 0 on 2xx.
  - `asc_post <path> <json>`, `asc_patch <path> <json>` same contract.
  - `asc_delete <path>` returns 0 on 2xx, prints nothing.
  - All four return 1 on non-2xx and print `<METHOD> <path> -> <code>` plus each `.errors[].detail` to stderr.
  - `$ASC_API` is `https://api.appstoreconnect.apple.com/v1`.

- [ ] **Step 1: Write the failing test**

Create `tools/verify-release-store.sh`:

```bash
#!/usr/bin/env bash
#
# Offline assertions for the release-store tooling, plus two live read-only checks of the
# transport. Nothing here writes to App Store Connect.

set -uo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
FAILED=0

ok()   { printf '  ok    %s\n' "$1"; }
fail() { printf '  FAIL  %s\n' "$1"; FAILED=1; }

check() {
  local name=$1; shift
  if "$@" >/dev/null 2>&1; then ok "$name"; else fail "$name"; fi
}

check_fails() {
  local name=$1; shift
  if "$@" >/dev/null 2>&1; then fail "$name (expected failure, got success)"; else ok "$name"; fi
}

echo "asc.sh"
. "$ROOT/tools/.release-env"
. "$ROOT/tools/lib/asc.sh"

check       "a known app reads back"        asc_get "apps/6804925509"
check_fails "a bad path fails"              asc_get "apps/0000000000"

MSG=$(asc_get "apps/0000000000" 2>&1 >/dev/null)
case "$MSG" in
  *"-> 404"*) ok "the failure names the status code" ;;
  *)          fail "the failure names the status code (got: $MSG)" ;;
esac

exit "$FAILED"
```

- [ ] **Step 2: Run it to verify it fails**

```sh
chmod +x tools/verify-release-store.sh && tools/verify-release-store.sh
```

Expected: fails, because `tools/lib/asc.sh` does not exist.

- [ ] **Step 3: Write tools/lib/asc.sh**

```sh
# Sourced, not executed. Transport for the App Store Connect API.

ASC_API="https://api.appstoreconnect.apple.com/v1"
_ASC_TOKEN=""
_ASC_TOKEN_AT=0

# Re-mints at 15 minutes: the token lives 20, and one run can wait out a build and then upload
# 34MB of screenshots.
asc_token() {
  _now=$(date +%s)
  if [ -z "$_ASC_TOKEN" ] || [ $((_now - _ASC_TOKEN_AT)) -ge 900 ]; then
    _ASC_TOKEN=$(xcrun altool --generate-jwt \
      --apiKey "$ASC_API_KEY_ID" --apiIssuer "$ASC_API_ISSUER_ID" 2>&1 | awk 'END{print $NF}')
    case "$_ASC_TOKEN" in
      ey*.*.*) ;;
      *) echo "asc: could not mint a JWT. Check ASC_API_KEY_ID and the .p8 in ~/.appstoreconnect/private_keys" >&2
         _ASC_TOKEN=""; return 1 ;;
    esac
    _ASC_TOKEN_AT=$_now
  fi
  printf '%s' "$_ASC_TOKEN"
}

# The API answers a bad request with a well-formed JSON body, so the status code is the only
# signal that anything went wrong.
asc() {
  _method=$1; _path=$2; _body=${3:-}
  _tok=$(asc_token) || return 1
  _out=$(mktemp -t asc)
  if [ -n "$_body" ]; then
    _code=$(printf '%s' "$_body" | curl -sS -o "$_out" -w '%{http_code}' \
      -X "$_method" -H "Authorization: Bearer $_tok" \
      -H "Content-Type: application/json" --data-binary @- "$ASC_API/$_path")
  else
    _code=$(curl -sS -o "$_out" -w '%{http_code}' \
      -X "$_method" -H "Authorization: Bearer $_tok" "$ASC_API/$_path")
  fi
  case "$_code" in
    2*) [ "$_method" = DELETE ] || cat "$_out"; rm -f "$_out"; return 0 ;;
  esac
  echo "$_method $_path -> $_code" >&2
  jq -r '.errors[]? | "  \(.title): \(.detail)"' < "$_out" >&2 2>/dev/null || cat "$_out" >&2
  rm -f "$_out"
  return 1
}

asc_get()    { asc GET    "$1"; }
asc_post()   { asc POST   "$1" "$2"; }
asc_patch()  { asc PATCH  "$1" "$2"; }
asc_delete() { asc DELETE "$1"; }
```

- [ ] **Step 4: Run the test to verify it passes**

```sh
tools/verify-release-store.sh
```

Expected: three `ok` lines, exit 0.

- [ ] **Step 5: Confirm the token refresh path**

Refreshing cannot be waited out in a test, so exercise it directly:

```sh
sh -c '. tools/.release-env; . tools/lib/asc.sh
  asc_get apps/6804925509 >/dev/null && echo "first: ok"
  _ASC_TOKEN_AT=0
  asc_get apps/6804925509 >/dev/null && echo "after forced expiry: ok"'
```

Expected: both lines print.

- [ ] **Step 6: Commit**

```sh
git add tools/lib/asc.sh tools/verify-release-store.sh
git commit -m "Add an App Store Connect transport that fails loudly

altool mints the ES256 JWT, which is the only reason this can be shell.
The status code is checked separately because the API answers a bad
request with a well-formed JSON body that jq reads as null."
```

---

## Task 3: Move the field text into files

**Files:**
- Create: `docs/store/fields/promotional-text.txt`
- Create: `docs/store/fields/description.txt`
- Create: `docs/store/fields/keywords.txt`
- Create: `docs/store/fields/review-notes.txt`
- Create: `docs/store/whats-new/0.4.0.txt`
- Modify: `docs/app-store-listing.md` (the fenced blocks under Promotional Text, Keywords, Description, What's New in This Version, Review notes)
- Modify: `docs/app-store-review-notes.md` (the addendum fenced block)
- Modify: `tools/verify-release-store.sh`

**Interfaces:**
- Consumes: `asc_get` from Task 2.
- Produces: five files, each holding the exact live API value plus one trailing newline. Later tasks read these and strip that one newline before comparing.

**The files are extracted from the live API, not copied from the documents.** 0.4.0 is live and its text is correct. The documents have been wrong about this text twice. Extracting from Apple and then checking the documents against the extraction is the only ordering that cannot introduce a third error.

- [ ] **Step 1: Extract the four version-level fields**

```sh
. tools/.release-env; . tools/lib/asc.sh
mkdir -p docs/store/fields docs/store/whats-new

VID=$(asc_get "apps/6804925509/appStoreVersions?limit=1" | jq -r '.data[0].id')
LID=$(asc_get "appStoreVersions/$VID/appStoreVersionLocalizations" | jq -r '.data[0].id')
LOC=$(asc_get "appStoreVersionLocalizations/$LID")

printf '%s\n' "$(printf '%s' "$LOC" | jq -r '.data.attributes.promotionalText')" > docs/store/fields/promotional-text.txt
printf '%s\n' "$(printf '%s' "$LOC" | jq -r '.data.attributes.description')"      > docs/store/fields/description.txt
printf '%s\n' "$(printf '%s' "$LOC" | jq -r '.data.attributes.keywords // ""')"   > docs/store/fields/keywords.txt
printf '%s\n' "$(printf '%s' "$LOC" | jq -r '.data.attributes.whatsNew')"         > docs/store/whats-new/0.4.0.txt

RID=$(asc_get "appStoreVersions/$VID/appStoreReviewDetail" | jq -r '.data.id')
printf '%s\n' "$(asc_get "appStoreReviewDetails/$RID" | jq -r '.data.attributes.notes')" > docs/store/fields/review-notes.txt

wc -c docs/store/fields/*.txt docs/store/whats-new/*.txt
```

`$(...)` strips every trailing newline and `printf '%s\n'` adds exactly one back, which is the normalisation the whole comparison depends on.

- [ ] **Step 2: Check the extraction against the counts the documents claim**

```sh
for f in docs/store/fields/promotional-text.txt docs/store/fields/description.txt \
         docs/store/whats-new/0.4.0.txt docs/store/fields/review-notes.txt; do
  printf '%-46s %s\n' "$f" "$(( $(wc -c < "$f") - 1 ))"
done
```

Expected, from `docs/app-store-listing.md`: promotional text 162, description 1859, what's new 1424, review notes 3856. **A disagreement here is a finding, not a problem with this step.** It means the documents drifted again. Record the real number, and fix the document rather than the file.

- [ ] **Step 3: Verify review-notes.txt contains the whole field, not half of it**

0.4.0 reached the review queue once with this field missing its entire addendum, 1936 characters against 3886 expected, and the panel showed a full-looking field. Check for content, not for emptiness:

```sh
grep -c "^" docs/store/fields/review-notes.txt
for h in $(grep -o '^#\+ .*' docs/app-store-review-notes.md | head -20); do :; done
grep -F -f <(grep -oE '^\*\*[A-Z][^*]{10,60}\*\*' docs/app-store-review-notes.md | head) \
  docs/store/fields/review-notes.txt | head
```

Expected: every section heading that `docs/app-store-review-notes.md` says belongs in the field is present in the file. If any is missing, stop: the live listing is wrong, not the extraction.

- [ ] **Step 4: Replace the fenced blocks in the documents with pointers**

In `docs/app-store-listing.md`, under each of Promotional Text, Keywords, Description, What's New in This Version, and Review notes, delete the fenced block holding the field content and put a pointer line in its place. Leave every word of surrounding prose, every character count, and every historical note exactly as it is. Leave the ```sh blocks alone: they are commands, not content.

```markdown
Field content: `docs/store/fields/description.txt`. Read it there and edit it there;
`tools/release-store.sh` writes the listing from that file and nothing else.
```

Do the same for the addendum block in `docs/app-store-review-notes.md`, pointing at `docs/store/fields/review-notes.txt` and noting that the two halves are now one file.

- [ ] **Step 5: Add the round-trip assertion to the verify script**

Append to `tools/verify-release-store.sh`, before the final `exit`:

```bash
echo
echo "field files round-trip against the live listing"

VID=$(asc_get "apps/6804925509/appStoreVersions?limit=1" | jq -r '.data[0].id')
LID=$(asc_get "appStoreVersions/$VID/appStoreVersionLocalizations" | jq -r '.data[0].id')
LOC=$(asc_get "appStoreVersionLocalizations/$LID")

roundtrip() {
  local name=$1 file=$2 live=$3
  if [ "$(cat "$file")" = "$live" ]; then ok "$name"; else fail "$name differs from the live listing"; fi
}

roundtrip "promotional-text" "$ROOT/docs/store/fields/promotional-text.txt" \
  "$(printf '%s' "$LOC" | jq -r '.data.attributes.promotionalText')"
roundtrip "description"      "$ROOT/docs/store/fields/description.txt" \
  "$(printf '%s' "$LOC" | jq -r '.data.attributes.description')"
roundtrip "whats-new"        "$ROOT/docs/store/whats-new/0.4.0.txt" \
  "$(printf '%s' "$LOC" | jq -r '.data.attributes.whatsNew')"

RID=$(asc_get "appStoreVersions/$VID/appStoreReviewDetail" | jq -r '.data.id')
roundtrip "review-notes"     "$ROOT/docs/store/fields/review-notes.txt" \
  "$(asc_get "appStoreReviewDetails/$RID" | jq -r '.data.attributes.notes')"
```

`$(cat file)` strips the trailing newline, so this compares the same way the script will.

- [ ] **Step 6: Run it**

```sh
tools/verify-release-store.sh
```

Expected: four more `ok` lines, exit 0.

- [ ] **Step 7: Commit**

```sh
git add docs/store docs/app-store-listing.md docs/app-store-review-notes.md tools/verify-release-store.sh
git commit -m "Give the listing fields files a script can read

Extracted from the live API rather than copied out of the documents,
because the documents have been wrong about this text twice and the API
has not. The review notes were split across two files, which is how
0.4.0 reached the queue missing half the field; they are one file now."
```

---

## Task 4: tools/lib/listing.sh, the pure logic

**Files:**
- Create: `tools/lib/listing.sh`
- Create: `tools/testdata/asc/screenshots-live.json`
- Create: `tools/testdata/asc/screenshots-swapped.json`
- Modify: `tools/verify-release-store.sh`

**Interfaces:**
- Consumes: nothing. No network, no `asc_*`.
- Produces:
  - `field_limit <name>` prints the character limit for `promotional-text`, `description`, `keywords`, `whats-new`, `review-notes`; returns 1 on an unknown name.
  - `check_field <name> <file>` returns 0 if the file exists and its content, minus the one trailing newline, is within the limit. Prints the count and limit on failure.
  - `shot_manifest <dir>` prints one `slot<TAB>path<TAB>md5` line per screenshot, sorted by slot. Returns 1 on a gap, a duplicate slot, a name with no slot number, or a file that is not 2560x1600.
  - `live_slots <json-file>` prints one `slot<TAB>filename<TAB>checksum` line per live screenshot, in the order the API returned them.
  - `slots_match <manifest-file> <live-file>` returns 0 if the checksum columns agree in order. On failure prints each differing slot.

- [ ] **Step 1: Write the failing tests**

Append to `tools/verify-release-store.sh`, before the final `exit`:

```bash
echo
echo "listing.sh, offline"
. "$ROOT/tools/lib/listing.sh"

# A sourced function is not visible to `bash -c`, so assert on its output directly.
if [ "$(field_limit promotional-text)" = 170 ]; then ok "promotional-text is 170"; else fail "promotional-text is 170"; fi
if [ "$(field_limit description)" = 4000 ]; then ok "description is 4000"; else fail "description is 4000"; fi
check_fails "field_limit rejects an unknown field" field_limit nonsense

WORK=$(mktemp -d -t verify-release-store)
trap 'rm -rf "$WORK"' EXIT

printf 'x%.0s' $(seq 169) > "$WORK/ok.txt";  printf '\n' >> "$WORK/ok.txt"
printf 'x%.0s' $(seq 171) > "$WORK/big.txt"; printf '\n' >> "$WORK/big.txt"
check       "a 169-character promotional text passes" check_field promotional-text "$WORK/ok.txt"
check_fails "a 171-character promotional text fails"  check_field promotional-text "$WORK/big.txt"
check_fails "a missing field file fails"              check_field description "$WORK/absent.txt"

mkshots() {
  local dir=$1; shift
  mkdir -p "$dir"
  for n in "$@"; do magick -size 2560x1600 "xc:#$(printf '%06x' $((n * 111111)))" "$dir/$n-shot.png"; done
}

mkshots "$WORK/good" 1 2 3
check       "1..3 with no gaps passes" shot_manifest "$WORK/good"

mkshots "$WORK/gap" 1 3
check_fails "a gap at slot 2 fails"    shot_manifest "$WORK/gap"

mkdir -p "$WORK/dup" && magick -size 2560x1600 xc:red "$WORK/dup/1-a.png" \
  && magick -size 2560x1600 xc:blue "$WORK/dup/2-b.png" \
  && magick -size 2560x1600 xc:green "$WORK/dup/2-c.png"
check_fails "two files claiming slot 2 fails" shot_manifest "$WORK/dup"

mkdir -p "$WORK/small" && magick -size 100x100 xc:red "$WORK/small/1-a.png"
check_fails "a shot that is not 2560x1600 fails" shot_manifest "$WORK/small"

shot_manifest "$WORK/good" > "$WORK/manifest.tsv"
live_slots "$ROOT/tools/testdata/asc/screenshots-live.json" > "$WORK/live.tsv"
if [ "$(wc -l < "$WORK/live.tsv" | tr -d ' ')" = 7 ]; then
  ok "live_slots reads seven slots in order"
else
  fail "live_slots reads seven slots in order"
fi
```

- [ ] **Step 2: Capture the two fixtures**

```sh
. tools/.release-env; . tools/lib/asc.sh
mkdir -p tools/testdata/asc
asc_get "appScreenshotSets/bd84280b-c323-423b-9d1c-e36a3034c5c4/appScreenshots" \
  | jq '{data: [.data[] | {id, attributes: {fileName: .attributes.fileName,
        sourceFileChecksum: .attributes.sourceFileChecksum,
        assetDeliveryState: .attributes.assetDeliveryState,
        imageAsset: {width: .attributes.imageAsset.width, height: .attributes.imageAsset.height}}}]}' \
  > tools/testdata/asc/screenshots-live.json

jq '.data |= (. as $d | [$d[0], $d[1], $d[3], $d[2]] + $d[4:])' \
  tools/testdata/asc/screenshots-live.json > tools/testdata/asc/screenshots-swapped.json
```

The first fixture is the most valuable one in the suite: it carries the stale 0.3.2 names (`2-awake`, `3-dozing`, `4-asleep`, `5-pet`, `6-card`) against a correct slot order, so a name-based implementation fails it and a checksum-based one passes. The second swaps slots 3 and 4.

- [ ] **Step 3: Run the tests to verify they fail**

```sh
tools/verify-release-store.sh
```

Expected: fails, because `tools/lib/listing.sh` does not exist.

- [ ] **Step 4: Write tools/lib/listing.sh**

```sh
# Sourced, not executed. Pure comparison logic: no network, so it can be tested offline.

field_limit() {
  case $1 in
    promotional-text) echo 170 ;;
    keywords) echo 100 ;;
    description|whats-new|review-notes) echo 4000 ;;
    *) echo "listing: unknown field '$1'" >&2; return 1 ;;
  esac
}

# Field files carry the API value plus one trailing newline, which $(cat) strips.
check_field() {
  _name=$1; _file=$2
  [ -f "$_file" ] || { echo "listing: $_name: no such file: $_file" >&2; return 1; }
  _limit=$(field_limit "$_name") || return 1
  _n=$(printf '%s' "$(cat "$_file")" | wc -c | tr -d ' ')
  [ "$_n" -le "$_limit" ] || { echo "listing: $_name is $_n characters, limit $_limit" >&2; return 1; }
  return 0
}

shot_manifest() {
  _dir=$1
  _prev=0
  _any=0
  for _base in $(ls "$_dir" 2>/dev/null | grep -E '^[0-9]+-.*\.png$' | sort -t- -k1,1n); do
    _slot=${_base%%-*}
    _slot=$((_slot))
    if [ "$_slot" -ne $((_prev + 1)) ]; then
      echo "listing: $_base claims slot $_slot after slot $_prev; slots must run 1..N with no gaps or duplicates" >&2
      return 1
    fi
    _size=$(magick identify -format '%wx%h' "$_dir/$_base")
    if [ "$_size" != "2560x1600" ]; then
      echo "listing: $_base is $_size, must be 2560x1600" >&2
      return 1
    fi
    printf '%s\t%s\t%s\n' "$_slot" "$_dir/$_base" "$(md5 -q "$_dir/$_base")"
    _prev=$_slot
    _any=1
  done
  [ "$_any" = 1 ] || { echo "listing: no screenshots found in $_dir" >&2; return 1; }
  return 0
}

live_slots() {
  jq -r '.data | to_entries[]
         | "\(.key + 1)\t\(.value.attributes.fileName)\t\(.value.attributes.sourceFileChecksum)"' "$1"
}

# Compares the checksum column in order. File names are deliberately ignored: the live listing
# carries names from the version the shot was first uploaded under.
slots_match() {
  _mine=$1; _live=$2
  _differs=0
  _n=$(wc -l < "$_mine" | tr -d ' ')
  _ln=$(wc -l < "$_live" | tr -d ' ')
  if [ "$_n" != "$_ln" ]; then
    echo "listing: $_n screenshots locally, $_ln live" >&2
    return 1
  fi
  while IFS="$(printf '\t')" read -r _slot _path _sum; do
    _livesum=$(awk -F'\t' -v s="$_slot" '$1 == s { print $3 }' "$_live")
    if [ "$_sum" != "$_livesum" ]; then
      echo "listing: slot $_slot differs. local ${_path##*/} $_sum, live $_livesum" >&2
      _differs=1
    fi
  done < "$_mine"
  return $_differs
}
```

- [ ] **Step 5: Run the tests to verify they pass**

```sh
tools/verify-release-store.sh
```

Expected: every check `ok`, exit 0.

- [ ] **Step 6: Add the two fixture assertions that are the point of the whole task**

Append to `tools/verify-release-store.sh`, before the final `exit`:

```bash
live_slots "$ROOT/tools/testdata/asc/screenshots-live.json"    > "$WORK/live.tsv"
live_slots "$ROOT/tools/testdata/asc/screenshots-swapped.json" > "$WORK/swapped.tsv"

# The left side carries the local names (3-awake ... 7-card) against the live checksums, which
# is exactly the shape of the real comparison: right order, stale names on the other side.
awk -F'\t' 'BEGIN { split("1-comeback 2-builder 3-awake 4-dozing 5-asleep 6-pet 7-card", n, " ") }
            { printf "%s\tdocs/store-shots/%s.png\t%s\n", $1, n[$1], $3 }' \
    "$WORK/live.tsv" > "$WORK/mine.tsv"

check       "stale live names still match by checksum" slots_match "$WORK/mine.tsv" "$WORK/live.tsv"
check_fails "slots 3 and 4 swapped fails"              slots_match "$WORK/mine.tsv" "$WORK/swapped.tsv"

MSG=$(slots_match "$WORK/mine.tsv" "$WORK/swapped.tsv" 2>&1 >/dev/null)
case "$MSG" in
  *"slot 3 differs"*) ok "the mismatch names the slot" ;;
  *)                  fail "the mismatch names the slot (got: $MSG)" ;;
esac
```

The first check is the one that matters. `slots_match` is handed a left side whose names are the
current local ones and a right side whose names are the 0.3.2 ones, with identical checksums. A
name-based implementation fails it; a checksum-based one passes.

- [ ] **Step 7: Run and commit**

```sh
tools/verify-release-store.sh
git add tools/lib/listing.sh tools/testdata tools/verify-release-store.sh
git commit -m "Compare screenshot slots by checksum, not by file name

The live set carries 0.3.2 names on slots 3 to 7 because the builder was
inserted at slot 2 and nothing renames an uploaded file. A name-based
check fails on a listing whose order is right and passes on one whose
order is wrong. The captured fixture is that exact case."
```

---

## Task 5: release-store.sh, preflight through build attach

**Files:**
- Create: `tools/release-store.sh`

**Interfaces:**
- Consumes: `asc_get`/`asc_post`/`asc_patch` from Task 2, everything from Task 4, the field files from Task 3.
- Produces: a script accepting `--dry-run`, `--submit`, or no arguments. After this task it stops after attaching the build.

- [ ] **Step 1: Write the script**

```sh
#!/bin/sh
#
# Converges the App Store Connect listing to what this repository says, then proves it by
# reading back. Submits only under --submit.
#
# Stateless: every run reads Apple and writes only the difference, so a killed run heals on the
# next one. The design is docs/superpowers/specs/2026-09-08-release-store-automation-design.md.
#
# Usage:
#
#   tools/release-store.sh              # converge, diff, stop
#   tools/release-store.sh --dry-run    # compare and print, write nothing
#   tools/release-store.sh --submit     # verify, then Add for Review and Submit
set -eu

ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"

APP_ID=6804925509
PLATFORM=MAC_OS
LOCALE=en-US

DRY=""
SUBMIT=""
case "${1:-}" in
  --dry-run) DRY=1 ;;
  --submit)  SUBMIT=1 ;;
  "")        ;;
  *) echo "usage: tools/release-store.sh [--dry-run|--submit]" >&2; exit 1 ;;
esac

[ -f tools/.release-env ] && . tools/.release-env
. tools/lib/asc.sh
. tools/lib/listing.sh

WORK=$(mktemp -d -t release-store)
trap 'rm -rf "$WORK"' EXIT

VERSION=$(grep -m1 '"version"' src-tauri/tauri.conf.json | sed 's/.*"version": "\([^"]*\)".*/\1/')
BUILD=$(tr -d '[:space:]' < tools/.mas-build)

echo "version: $VERSION   build: $BUILD"

# Phase 0
echo
echo "preflight"
for pair in "promotional-text:docs/store/fields/promotional-text.txt" \
            "description:docs/store/fields/description.txt" \
            "keywords:docs/store/fields/keywords.txt" \
            "review-notes:docs/store/fields/review-notes.txt" \
            "whats-new:docs/store/whats-new/$VERSION.txt"; do
  name=${pair%%:*}; file=${pair#*:}
  check_field "$name" "$file"
  printf '  %-18s %5s\n' "$name" "$(printf '%s' "$(cat "$file")" | wc -c | tr -d ' ')"
done

shot_manifest docs/store-shots > "$WORK/shots.tsv"
printf '  %-18s %5s\n' "screenshots" "$(wc -l < "$WORK/shots.tsv" | tr -d ' ')"

for url in https://keepgoing.dev https://keepgoing.dev/privacy; do
  curl -sS "$url" | grep -q "<title" || { echo "  $url served no page" >&2; exit 1; }
done
printf '  %-18s ok\n' "urls"

# Printed, never diffed: there is no second copy of these to diff against that would not be
# invented here. Spec section 4.4.
AID=$(asc_get "apps/$APP_ID/appInfos" | jq -r '.data[0].id')
asc_get "appInfos/$AID/appInfoLocalizations" \
  | jq -r --arg l "$LOCALE" '.data[] | select(.attributes.locale == $l)
          | "  app-level         \(.attributes.name) / \(.attributes.subtitle)"'

# Phase 1
echo
echo "version"
VID=$(asc_get "apps/$APP_ID/appStoreVersions?filter%5BversionString%5D=$VERSION&filter%5Bplatform%5D=$PLATFORM" \
      | jq -r '.data[0].id // ""')
if [ -n "$VID" ]; then
  STATE=$(asc_get "appStoreVersions/$VID" | jq -r '.data.attributes.appVersionState')
  case "$STATE" in
    PREPARE_FOR_SUBMISSION) echo "  $VERSION exists, $STATE, reusing" ;;
    *)
      if [ -n "$DRY" ]; then
        echo "  $VERSION is $STATE. A real run would refuse here; carrying on read-only."
      else
        echo "  $VERSION is $STATE, which cannot be edited." >&2
        case "$STATE" in
          WAITING_FOR_REVIEW|IN_REVIEW) echo "  Cancel the submission first." >&2 ;;
          READY_FOR_DISTRIBUTION)       echo "  Nothing to do here." >&2 ;;
        esac
        exit 1
      fi ;;
  esac
elif [ -n "$DRY" ]; then
  echo "  $VERSION does not exist. A real run would create it."
  exit 0
else
  VID=$(asc_post appStoreVersions "$(jq -n --arg v "$VERSION" --arg a "$APP_ID" --arg p "$PLATFORM" \
    '{data:{type:"appStoreVersions",attributes:{platform:$p,versionString:$v},
      relationships:{app:{data:{type:"apps",id:$a}}}}}')" | jq -r '.data.id')
  echo "  $VERSION created"
fi

# Phase 2
echo
echo "build"
WAITED=0
while :; do
  B=$(asc_get "builds?filter%5Bapp%5D=$APP_ID&filter%5Bversion%5D=$BUILD&include=preReleaseVersion")
  BID=$(printf '%s' "$B" | jq -r '.data[0].id // ""')
  BSTATE=$(printf '%s' "$B" | jq -r '.data[0].attributes.processingState // "ABSENT"')
  BVER=$(printf '%s' "$B" | jq -r '.included[]? | select(.type=="preReleaseVersions") | .attributes.version' | head -1)
  [ "$BSTATE" = VALID ] && break
  case "$BSTATE" in
    INVALID|FAILED) echo "  build $BUILD is $BSTATE" >&2; exit 1 ;;
  esac
  [ "$WAITED" -ge 1800 ] && { echo "  build $BUILD still $BSTATE after 30 minutes" >&2; exit 1; }
  printf '\r  build %s: %s (%ss)' "$BUILD" "$BSTATE" "$WAITED"
  sleep 15
  WAITED=$((WAITED + 15))
done
echo "  build $BUILD: VALID after ${WAITED}s"

if [ -n "$BVER" ] && [ "$BVER" != "$VERSION" ]; then
  echo "  build $BUILD belongs to $BVER, not $VERSION. Refusing to attach it." >&2
  exit 1
fi

# ITSAppUsesNonExemptEncryption in src-tauri/Info.plist should answer this at build time. It
# blocked build 4, there is no API to answer it, and unchecked it surfaces as a submit failure.
if [ "$(printf '%s' "$B" | jq -r '.data[0].attributes.usesNonExemptEncryption')" = null ]; then
  echo "  build $BUILD is Missing Compliance. Check ITSAppUsesNonExemptEncryption in" >&2
  echo "  src-tauri/Info.plist, or answer it by hand in App Store Connect." >&2
  exit 1
fi

if [ -z "$DRY" ]; then
  asc_patch "appStoreVersions/$VID/relationships/build" \
    "$(jq -n --arg b "$BID" '{data:{type:"builds",id:$b}}')" >/dev/null
  echo "  attached"
fi
```

- [ ] **Step 2: Run it in dry-run against the current listing**

```sh
chmod +x tools/release-store.sh && tools/release-store.sh --dry-run
```

Expected: preflight prints five field counts and `screenshots 7`; the version phase warns that 0.4.0 is `READY_FOR_DISTRIBUTION` and carries on; the build phase finds build 6 `VALID` immediately and reports it belongs to 0.4.0.

- [ ] **Step 3: Run it for real and confirm it refuses**

```sh
tools/release-store.sh; echo "exit: $?"
```

Expected: refuses at the version phase with `0.4.0 is READY_FOR_DISTRIBUTION, which cannot be edited.` and `Nothing to do here.`, exit 1.

- [ ] **Step 4: Confirm the wrong-build guard**

```sh
cp tools/.mas-build /tmp/mas-build.bak && echo 5 > tools/.mas-build
tools/release-store.sh --dry-run; echo "exit: $?"
cp /tmp/mas-build.bak tools/.mas-build
```

Expected: build 5 is found `VALID` and the script refuses with `build 5 belongs to 0.3.2, not 0.4.0. Refusing to attach it.`, exit 1. Confirm `tools/.mas-build` reads 6 again afterwards.

- [ ] **Step 5: Confirm preflight catches a bad field**

```sh
cp docs/store/fields/promotional-text.txt /tmp/pt.bak
printf 'x%.0s' $(seq 171) > docs/store/fields/promotional-text.txt
tools/release-store.sh --dry-run; echo "exit: $?"
cp /tmp/pt.bak docs/store/fields/promotional-text.txt
```

Expected: `listing: promotional-text is 171 characters, limit 170`, exit 1, and nothing after preflight runs.

- [ ] **Step 6: Commit**

```sh
git add tools/release-store.sh
git commit -m "Converge the version and the build

The chosen human gate catches wrong text but cannot catch right text
written against the wrong build, because that diff is clean. So the
build's preReleaseVersion is asserted against tauri.conf.json before
anything is attached."
```

---

## Task 6: Phase 3, the fields

**Files:**
- Modify: `tools/release-store.sh` (append after the build phase)

**Interfaces:**
- Consumes: `$VID` and `$DRY` from Task 5.
- Produces: `$LID`, the en-US `appStoreVersionLocalizations` id, used by Task 7.

- [ ] **Step 1: Append the phase**

```sh
# Phase 3
echo
echo "fields"
LID=$(asc_get "appStoreVersions/$VID/appStoreVersionLocalizations" \
      | jq -r --arg l "$LOCALE" '.data[] | select(.attributes.locale == $l) | .id')
[ -n "$LID" ] || { echo "  no $LOCALE localization on $VERSION" >&2; exit 1; }
LOC=$(asc_get "appStoreVersionLocalizations/$LID")

write_field() {
  attr=$1; file=$2
  want=$(cat "$file")
  have=$(printf '%s' "$LOC" | jq -r --arg a "$attr" '.data.attributes[$a] // ""')
  if [ "$want" = "$have" ]; then
    printf '  %-18s match\n' "$attr"
    return 0
  fi
  if [ -n "$DRY" ]; then
    printf '  %-18s WOULD WRITE (%s -> %s chars)\n' "$attr" "$(printf '%s' "$have" | wc -c | tr -d ' ')" "$(printf '%s' "$want" | wc -c | tr -d ' ')"
    return 0
  fi
  asc_patch "appStoreVersionLocalizations/$LID" \
    "$(jq -n --arg i "$LID" --arg a "$attr" --arg v "$want" \
       '{data:{type:"appStoreVersionLocalizations",id:$i,attributes:{($a):$v}}}')" >/dev/null
  printf '  %-18s written\n' "$attr"
}

write_field promotionalText docs/store/fields/promotional-text.txt
write_field description     docs/store/fields/description.txt
write_field keywords        docs/store/fields/keywords.txt
write_field whatsNew        "docs/store/whats-new/$VERSION.txt"

RID=$(asc_get "appStoreVersions/$VID/appStoreReviewDetail" | jq -r '.data.id // ""')
NOTES=$(cat docs/store/fields/review-notes.txt)
if [ -z "$RID" ]; then
  if [ -n "$DRY" ]; then
    printf '  %-18s WOULD CREATE\n' "notes"
  else
    asc_post appStoreReviewDetails "$(jq -n --arg v "$VID" --arg n "$NOTES" \
      '{data:{type:"appStoreReviewDetails",attributes:{notes:$n},
        relationships:{appStoreVersion:{data:{type:"appStoreVersions",id:$v}}}}}')" >/dev/null
    printf '  %-18s created\n' "notes"
  fi
else
  HAVE=$(asc_get "appStoreReviewDetails/$RID" | jq -r '.data.attributes.notes // ""')
  if [ "$NOTES" = "$HAVE" ]; then
    printf '  %-18s match\n' "notes"
  elif [ -n "$DRY" ]; then
    printf '  %-18s WOULD WRITE (%s -> %s chars)\n' "notes" "$(printf '%s' "$HAVE" | wc -c | tr -d ' ')" "$(printf '%s' "$NOTES" | wc -c | tr -d ' ')"
  else
    asc_patch "appStoreReviewDetails/$RID" \
      "$(jq -n --arg i "$RID" --arg n "$NOTES" \
         '{data:{type:"appStoreReviewDetails",id:$i,attributes:{notes:$n}}}')" >/dev/null
    printf '  %-18s written\n' "notes"
  fi
fi
```

- [ ] **Step 2: Run it in dry-run**

```sh
tools/release-store.sh --dry-run
```

Expected: all five fields report `match`, because Task 3 extracted them from this exact listing. Any `WOULD WRITE` here means the extraction and the comparison disagree about trailing newlines: fix that before going further, because every later phase depends on this comparison being exact.

- [ ] **Step 3: Prove the mismatch path reports rather than lies**

```sh
cp docs/store/fields/description.txt /tmp/desc.bak
printf 'Changed.\n' >> docs/store/fields/description.txt
tools/release-store.sh --dry-run | grep description
cp /tmp/desc.bak docs/store/fields/description.txt
```

Expected: `description   WOULD WRITE (1859 -> 1868 chars)`. Confirm the file is restored and dry-run reports `match` again.

- [ ] **Step 4: Commit**

```sh
git add tools/release-store.sh
git commit -m "Write the listing fields from files, only where they differ

Because a write happens only on a difference, a second run of the same
command is the verification pass."
```

---

## Task 7: Phase 4, the screenshots

**Files:**
- Modify: `tools/release-store.sh` (append after the fields phase)

**Interfaces:**
- Consumes: `$LID` from Task 6, `$DRY`, and `$WORK/shots.tsv` from Task 5.
- Produces: nothing later tasks need.

- [ ] **Step 1: Append the phase**

```sh
# Phase 4
echo
echo "screenshots"
SETID=$(asc_get "appStoreVersionLocalizations/$LID/appScreenshotSets" \
        | jq -r '.data[] | select(.attributes.screenshotDisplayType == "APP_DESKTOP") | .id')
if [ -z "$SETID" ]; then
  if [ -n "$DRY" ]; then
    echo "  no APP_DESKTOP set. A real run would create one."
  else
    SETID=$(asc_post appScreenshotSets "$(jq -n --arg l "$LID" \
      '{data:{type:"appScreenshotSets",attributes:{screenshotDisplayType:"APP_DESKTOP"},
        relationships:{appStoreVersionLocalization:{data:{type:"appStoreVersionLocalizations",id:$l}}}}}')" \
      | jq -r '.data.id')
  fi
fi

SHOTS_JSON=$(mktemp -t shots)
LIVE_TSV=$(mktemp -t livetsv)
asc_get "appScreenshotSets/$SETID/appScreenshots" > "$SHOTS_JSON"
live_slots "$SHOTS_JSON" > "$LIVE_TSV"

if slots_match "$WORK/shots.tsv" "$LIVE_TSV" 2>/dev/null; then
  echo "  $(wc -l < "$LIVE_TSV" | tr -d ' ')/$(wc -l < "$WORK/shots.tsv" | tr -d ' ') slots match by checksum"
else
  slots_match "$WORK/shots.tsv" "$LIVE_TSV" 2>&1 >/dev/null | sed 's/^listing: /  /' || true
  if [ -n "$DRY" ]; then
    echo "  WOULD REPLACE the whole set"
  else
    jq -r '.data[].id' "$SHOTS_JSON" | while read -r sid; do
      asc_delete "appScreenshots/$sid"
    done
    IDS=""
    while IFS="$(printf '\t')" read -r slot path sum; do
      name=${path##*/}
      size=$(wc -c < "$path" | tr -d ' ')
      RES=$(asc_post appScreenshots "$(jq -n --arg s "$SETID" --arg n "$name" --argjson z "$size" \
        '{data:{type:"appScreenshots",attributes:{fileName:$n,fileSize:$z},
          relationships:{appScreenshotSet:{data:{type:"appScreenshotSets",id:$s}}}}}')")
      SID=$(printf '%s' "$RES" | jq -r '.data.id')
      printf '%s' "$RES" | jq -c '.data.attributes.uploadOperations[]' | while read -r op; do
        cfg=$(mktemp -t uop)
        printf '%s' "$op" | jq -r '"request = \"" + .method + "\"", "url = \"" + .url + "\"",
                                   (.requestHeaders[] | "header = \"" + .name + ": " + .value + "\"")' > "$cfg"
        off=$(printf '%s' "$op" | jq -r '.offset')
        len=$(printf '%s' "$op" | jq -r '.length')
        n=0
        until tail -c "+$((off + 1))" "$path" | head -c "$len" \
              | curl -sS -K "$cfg" --data-binary @- ; do
          n=$((n + 1))
          [ "$n" -ge 3 ] && { echo "  slot $slot: upload failed after 3 attempts" >&2; exit 1; }
          sleep $((n * 5))
        done
        rm -f "$cfg"
      done
      asc_patch "appScreenshots/$SID" "$(jq -n --arg i "$SID" --arg c "$sum" \
        '{data:{type:"appScreenshots",id:$i,attributes:{sourceFileChecksum:$c,uploaded:true}}}')" >/dev/null
      IDS="$IDS $SID"
      printf '  slot %s uploaded (%s)\n' "$slot" "$name"
    done < "$WORK/shots.tsv"

    ORDER=$(printf '%s\n' $IDS | jq -R '{type:"appScreenshots",id:.}' | jq -sc '{data:.}')
    asc_patch "appScreenshotSets/$SETID/relationships/appScreenshots" "$ORDER" >/dev/null
    echo "  order set explicitly"

    WAITED=0
    until [ "$(asc_get "appScreenshotSets/$SETID/appScreenshots" \
               | jq -r '[.data[].attributes.assetDeliveryState.state] | unique | join(",")')" = COMPLETE ]; do
      [ "$WAITED" -ge 300 ] && { echo "  screenshots still processing after 5 minutes" >&2; exit 1; }
      sleep 5; WAITED=$((WAITED + 5))
    done
    echo "  all COMPLETE"
  fi
fi
rm -f "$SHOTS_JSON" "$LIVE_TSV"
```

The `IDS` accumulation runs inside a `while` reading from a file rather than a pipe, so it is in the parent shell and survives the loop. Do not restructure that into a pipeline.

- [ ] **Step 2: Run in dry-run against the live listing**

```sh
tools/release-store.sh --dry-run
```

Expected: `7/7 slots match by checksum`, despite the live names being the 0.3.2 ones. If it reports a mismatch, the checksum comparison is wrong, not the listing.

- [ ] **Step 3: Prove the mismatch path**

```sh
cp docs/store-shots/4-dozing.png /tmp/4-dozing.bak
magick docs/store-shots/4-dozing.png -modulate 101 docs/store-shots/4-dozing.png
tools/release-store.sh --dry-run | sed -n '/screenshots/,$p'
cp /tmp/4-dozing.bak docs/store-shots/4-dozing.png
```

Expected: `slot 4 differs. local 4-dozing.png <new md5>, live <old md5>` and `WOULD REPLACE the whole set`. Confirm dry-run is clean again after restoring.

- [ ] **Step 4: Rehearse the upload end to end against a throwaway version**

Only if Task 1 confirmed `DELETE` works. Create a draft version by hand or with the Task 1 snippet, point the script at it by temporarily setting the version in `src-tauri/tauri.conf.json`, run without `--dry-run` up to and including this phase, then read back:

```sh
. tools/.release-env; . tools/lib/asc.sh; . tools/lib/listing.sh
VID=<throwaway version id>
LID=$(asc_get "appStoreVersions/$VID/appStoreVersionLocalizations" | jq -r '.data[0].id')
SETID=$(asc_get "appStoreVersionLocalizations/$LID/appScreenshotSets" | jq -r '.data[0].id')
asc_get "appScreenshotSets/$SETID/appScreenshots" > /tmp/rehearsal.json
live_slots /tmp/rehearsal.json
shot_manifest docs/store-shots > /tmp/mine.tsv
live_slots /tmp/rehearsal.json > /tmp/live.tsv
slots_match /tmp/mine.tsv /tmp/live.tsv && echo "rehearsal: slots correct"
```

Expected: seven rows in slot order, every checksum matching the local file, `rehearsal: slots correct`. Then delete the throwaway version and restore `tauri.conf.json`.

- [ ] **Step 5: Prove it heals after being killed**

Re-run the previous step's upload against a fresh throwaway version and interrupt it with ctrl-C partway through the uploads. Re-run it. Expected: the second run replaces the partial set and reads back correct. Delete the throwaway version.

- [ ] **Step 6: Commit**

```sh
git add tools/release-store.sh
git commit -m "Upload the whole screenshot set and set its order explicitly

Order is set with a relationship PATCH rather than by upload order. The
panel orders by drop position, which is how the 0.3.1 order went live
wrong, and a released version's order cannot be changed afterwards."
```

---

## Task 8: Phase 5, the gate

**Files:**
- Modify: `tools/release-store.sh` (append)

**Interfaces:**
- Consumes: everything above.
- Produces: exit 0 with a clean table, or exit 1 naming what differs. Task 9 depends on this exit code.

- [ ] **Step 1: Append the phase**

```sh
# Phase 5
echo
echo "read-back"
LOC=$(asc_get "appStoreVersionLocalizations/$LID")
CLEAN=0

verify_field() {
  attr=$1; file=$2; label=$3
  want=$(cat "$file")
  have=$(printf '%s' "$LOC" | jq -r --arg a "$attr" '.data.attributes[$a] // ""')
  n=$(printf '%s' "$want" | wc -c | tr -d ' ')
  if [ "$want" = "$have" ]; then
    printf '  %-18s %5s  match\n' "$label" "$n"
  else
    printf '  %-18s %5s  DIFFERS (live %s)\n' "$label" "$n" "$(printf '%s' "$have" | wc -c | tr -d ' ')"
    CLEAN=1
  fi
}

verify_field promotionalText docs/store/fields/promotional-text.txt promotional-text
verify_field description     docs/store/fields/description.txt      description
verify_field keywords        docs/store/fields/keywords.txt         keywords
verify_field whatsNew        "docs/store/whats-new/$VERSION.txt"     whats-new

RID=$(asc_get "appStoreVersions/$VID/appStoreReviewDetail" | jq -r '.data.id // ""')
HAVE=$(asc_get "appStoreReviewDetails/$RID" | jq -r '.data.attributes.notes // ""')
WANT=$(cat docs/store/fields/review-notes.txt)
if [ "$WANT" = "$HAVE" ]; then
  printf '  %-18s %5s  match\n' "review-notes" "$(printf '%s' "$WANT" | wc -c | tr -d ' ')"
else
  printf '  %-18s %5s  DIFFERS (live %s)\n' "review-notes" "$(printf '%s' "$WANT" | wc -c | tr -d ' ')" "$(printf '%s' "$HAVE" | wc -c | tr -d ' ')"
  CLEAN=1
fi

LIVE_TSV=$(mktemp -t livetsv)
asc_get "appScreenshotSets/$SETID/appScreenshots" > "$WORK/shots-back.json"
live_slots "$WORK/shots-back.json" > "$LIVE_TSV"
if slots_match "$WORK/shots.tsv" "$LIVE_TSV" 2>/dev/null; then
  printf '  %-18s %5s  slots match by checksum\n' "screenshots" "$(wc -l < "$LIVE_TSV" | tr -d ' ')"
else
  printf '  %-18s        SLOTS DIFFER\n' "screenshots"
  CLEAN=1
fi
rm -f "$LIVE_TSV"

if [ "$CLEAN" != 0 ]; then
  echo
  echo "the listing does not match the repository. Not submitting." >&2
  exit 1
fi
```

- [ ] **Step 2: Add the closing message for a run without --submit**

Append:

```sh
if [ -z "$SUBMIT" ]; then
  echo
  [ -n "$DRY" ] && echo "dry run. Nothing was written." || echo "NOT submitted. Re-run with --submit."
  exit 0
fi
```

- [ ] **Step 3: Run and read the table**

```sh
tools/release-store.sh --dry-run
```

Expected, at the end:

```
read-back
  promotional-text     162  match
  description         1859  match
  keywords               0  match
  whats-new           1424  match
  review-notes        3856  match
  screenshots            7  slots match by checksum

dry run. Nothing was written.
```

- [ ] **Step 4: Prove a mismatch stops it**

```sh
cp docs/store/fields/review-notes.txt /tmp/notes.bak
printf 'Extra.\n' >> docs/store/fields/review-notes.txt
tools/release-store.sh --dry-run; echo "exit: $?"
cp /tmp/notes.bak docs/store/fields/review-notes.txt
```

Expected: `review-notes 3863 DIFFERS (live 3856)`, `the listing does not match the repository. Not submitting.`, exit 1.

- [ ] **Step 5: Commit**

```sh
git add tools/release-store.sh
git commit -m "Read the whole listing back and make the diff the gate

Every listing defect this project has shipped was the local record and
the live listing disagreeing where the panel could not show it."
```

---

## Task 9: Phase 6, submit

**Files:**
- Modify: `tools/release-store.sh` (append)

**Interfaces:**
- Consumes: `$VID`, `$APP_ID`, and a clean phase 5.
- Produces: a submitted review submission, then hands off to `tools/release-watch.sh`.

- [ ] **Step 1: Append the phase**

```sh
# Phase 6
echo
echo "submit"
SUBID=$(asc_get "reviewSubmissions?filter%5Bapp%5D=$APP_ID&filter%5Bplatform%5D=$PLATFORM&filter%5Bstate%5D=READY_FOR_REVIEW" \
        | jq -r '.data[0].id // ""')
if [ -z "$SUBID" ]; then
  SUBID=$(asc_post reviewSubmissions "$(jq -n --arg a "$APP_ID" --arg p "$PLATFORM" \
    '{data:{type:"reviewSubmissions",attributes:{platform:$p},
      relationships:{app:{data:{type:"apps",id:$a}}}}}')" | jq -r '.data.id')
  echo "  submission $SUBID created"
else
  echo "  reusing submission $SUBID"
fi

HAS=$(asc_get "reviewSubmissions/$SUBID/items" | jq -r --arg v "$VID" \
      '[.data[] | select(.relationships.appStoreVersion.data.id == $v)] | length')
if [ "$HAS" = 0 ]; then
  asc_post reviewSubmissionItems "$(jq -n --arg s "$SUBID" --arg v "$VID" \
    '{data:{type:"reviewSubmissionItems",
      relationships:{reviewSubmission:{data:{type:"reviewSubmissions",id:$s}},
                     appStoreVersion:{data:{type:"appStoreVersions",id:$v}}}}}')" >/dev/null
  echo "  $VERSION added to the submission"
fi

asc_patch "reviewSubmissions/$SUBID" \
  "$(jq -n --arg i "$SUBID" '{data:{type:"reviewSubmissions",id:$i,attributes:{submitted:true}}}')" >/dev/null
echo "  submitted"

echo
exec tools/release-watch.sh "$VERSION"
```

- [ ] **Step 2: Confirm --submit refuses on a dirty read-back**

Do not submit anything real to test this. Force the gate to fail and confirm phase 6 is never reached:

```sh
cp docs/store/fields/description.txt /tmp/desc.bak
printf 'Changed.\n' >> docs/store/fields/description.txt
tools/release-store.sh --submit 2>&1 | tail -5; echo "exit: ${PIPESTATUS[0]}"
cp /tmp/desc.bak docs/store/fields/description.txt
```

Expected: the run refuses at phase 1 (0.4.0 is `READY_FOR_DISTRIBUTION`) before reaching the gate at all. That is the correct behaviour and confirms `--submit` cannot act on a released version. Note in the commit that the gate's own refusal is exercised in Task 8 step 4.

- [ ] **Step 3: Commit**

```sh
git add tools/release-store.sh
git commit -m "Submit for review behind an explicit flag

An existing READY_FOR_REVIEW submission is reused rather than
duplicated, and the POST that creates one never retries: a retried
create is two submissions."
```

---

## Task 10: tools/release-watch.sh

**Files:**
- Create: `tools/release-watch.sh`

**Interfaces:**
- Consumes: `asc_get`. Takes an optional version string; defaults to the version in `src-tauri/tauri.conf.json`.
- Produces: exit 0 on `READY_FOR_DISTRIBUTION`, exit 1 on any rejection state.

- [ ] **Step 1: Write the script**

```sh
#!/bin/sh
#
# Polls a version's review state until it settles, and says so twice: once to stdout and once to
# Notification Center, because a sixteen-hour wait outlives anyone's attention on a terminal.
#
# Usage:
#
#   tools/release-watch.sh            # print the current state once and exit
#   tools/release-watch.sh 0.4.0      # poll until it settles
set -eu

ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"

APP_ID=6804925509
PLATFORM=MAC_OS

[ -f tools/.release-env ] && . tools/.release-env
. tools/lib/asc.sh

WATCH=""
if [ $# -gt 0 ]; then
  VERSION=$1
  WATCH=1
else
  VERSION=$(grep -m1 '"version"' src-tauri/tauri.conf.json | sed 's/.*"version": "\([^"]*\)".*/\1/')
fi

state() {
  asc_get "apps/$APP_ID/appStoreVersions?filter%5BversionString%5D=$VERSION&filter%5Bplatform%5D=$PLATFORM" \
    | jq -r '.data[0].attributes.appVersionState // "ABSENT"'
}

notify() {
  osascript -e "display notification \"$1\" with title \"Momentum Mascot $VERSION\"" 2>/dev/null || true
}

S=$(state)
echo "$(date '+%H:%M:%S')  $VERSION: $S"
[ -n "$WATCH" ] || exit 0

PREV=$S
while :; do
  case "$S" in
    READY_FOR_DISTRIBUTION)
      notify "Approved and released."; exit 0 ;;
    PENDING_DEVELOPER_RELEASE)
      notify "Approved. Waiting for you to release it."; exit 0 ;;
    REJECTED|DEVELOPER_REJECTED|METADATA_REJECTED|INVALID_BINARY)
      notify "$S. Open App Store Connect."; exit 1 ;;
  esac
  sleep 300
  S=$(state)
  if [ "$S" != "$PREV" ]; then
    echo "$(date '+%H:%M:%S')  $VERSION: $PREV -> $S"
    notify "$S"
    PREV=$S
  fi
done
```

- [ ] **Step 2: Confirm the one-shot form**

```sh
chmod +x tools/release-watch.sh && tools/release-watch.sh; echo "exit: $?"
```

Expected: `HH:MM:SS  0.4.0: READY_FOR_DISTRIBUTION`, exit 0.

- [ ] **Step 3: Confirm the watch form exits immediately on a terminal state**

```sh
tools/release-watch.sh 0.4.0; echo "exit: $?"
```

Expected: one line, a notification banner, exit 0, and no five-minute wait.

- [ ] **Step 4: Confirm an unknown version reports rather than hangs**

```sh
tools/release-watch.sh 9.9.9; echo "exit: $?"
```

Expected: `9.9.9: ABSENT`, then it enters the poll loop. Interrupt with ctrl-C. This is acceptable: an absent version is not a terminal state and a person watching one has mistyped it. Add nothing for it.

- [ ] **Step 5: Add the one-shot check to the verify script**

Append to `tools/verify-release-store.sh` before the final `exit`:

```bash
echo
echo "release-watch.sh"
check "the one-shot form exits zero on a released version" \
      "$ROOT/tools/release-watch.sh"
```

- [ ] **Step 6: Run everything and commit**

```sh
tools/verify-release-store.sh
git add tools/release-watch.sh tools/verify-release-store.sh
git commit -m "Watch a review to its conclusion without holding a terminal

Reports to stdout and to Notification Center. 0.3.2 took about sixteen
hours; nobody watches a terminal for that."
```

---

## Task 11: Documentation

**Files:**
- Modify: `docs/app-store-listing.md`
- Modify: `docs/app-store.md`
- Modify: `README.md`
- Modify: `.opencode/skills/release-assistant/SKILL.md`

**Interfaces:**
- Consumes: the finished tooling.
- Produces: a runbook a person can follow without reading any script.

- [ ] **Step 1: Rewrite the lines this work made false**

`docs/app-store-listing.md:493` says "the numbering is a note to the person uploading and nothing more". That is no longer true: `tools/release-store.sh` reads the leading number as the slot and refuses a gap or a duplicate. Rewrite that paragraph. Keep the whole history of how the 0.3.1 order went wrong; only the conclusion changes.

Check the same file for any other line that assumes a person is typing into the panel, in particular under Promotional Text and Attaching the build.

- [ ] **Step 2: Replace the "In App Store Connect" checklist with the two commands**

The queue section of `docs/app-store-listing.md` lists eight manual panel steps. Replace them with the two commands and what each one guarantees. Leave the 0.4.0 queue as the historical record it is: add the new procedure as the process for the next release rather than editing what was done for 0.4.0.

- [ ] **Step 3: Add the runbook to docs/app-store.md**

The full release now reads:

```sh
tools/release.sh minor            # version, DMG, notarize, GitHub release
tools/release-mas.sh --upload     # build, sign, package, upload. Burns a build number.
tools/release-store.sh            # listing, screenshots, read-back diff
tools/release-store.sh --submit   # submit, then watch
```

Document that `--dry-run` is safe at any time and answers "does the repo still describe what is live", and that `tools/release-watch.sh` with no arguments prints the current state.

- [ ] **Step 4: Update the release assistant skill**

`.opencode/skills/release-assistant/SKILL.md` names the release scripts. Add the two new ones and remove any instruction to drive the panel by hand.

- [ ] **Step 5: Add the new tools to README.md**

Follow the existing table or list format in that file. One line each.

- [ ] **Step 6: Commit**

```sh
git add docs/app-store-listing.md docs/app-store.md README.md .opencode/skills/release-assistant/SKILL.md
git commit -m "Document the store release as two commands

The screenshot numbering is load-bearing now, so the line calling it a
note to the person uploading had to go."
```
