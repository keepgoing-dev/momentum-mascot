#!/usr/bin/env bash
#
# Asserts src/baker.js and tools/assemble-frame.sh agree on where every character frame goes.
#
# The pixels need a browser, but the arithmetic does not, and the arithmetic is where the bugs
# are: an off-by-one in the hop index or a frame range read the wrong way produces a character
# standing still in a room that is breathing. Both sides read the manifest, so this checks they
# read it the same way.

set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
M="$ROOT/src/assets/character-layout.json"
[ -f "$M" ] || { echo "no manifest: $M" >&2; exit 1; }
command -v node >/dev/null || { echo "node not found; skipping" >&2; exit 0; }

WORK=$(mktemp -d -t verify-baker)
trap 'rm -rf "$WORK"' EXIT

cat > "$WORK/check.mjs" <<JS
import { frameIndex, framePlacement } from "$ROOT/src/baker.js";
import { readFileSync } from "node:fs";
const m = JSON.parse(readFileSync("$M", "utf8"));
const rows = [];
for (const [state, s] of Object.entries(m.states)) {
  for (const surface of ["room", "pet"]) {
    if (!s[surface]) continue;
    for (let i = 0; i < m.frames; i++) {
      const p = framePlacement(m, state, surface, i);
      rows.push(\`\${state} \${surface} \${i} k=\${frameIndex(m, state, surface, i)} x=\${p.x} y=\${p.y}\`);
    }
  }
}
console.log(rows.join("\n"));
JS
node "$WORK/check.mjs" | sort > "$WORK/baker.txt"

q() { jq -r "$1" "$M"; }
F=$(q '.frames')
{
  for state in $(q '.states | keys[]'); do
    for surface in room pet; do
      [ "$(q ".states.\"$state\".$surface // empty")" = "" ] && continue
      cx=$(q ".states.\"$state\".$surface.char.x")
      cy=$(q ".states.\"$state\".$surface.char.y")
      range=$(q ".states.\"$state\".$surface.char.range")
      lo=$(q ".layerStrip.ranges.$range[0]"); hi=$(q ".layerStrip.ranges.$range[1]")
      single=$(q ".states.\"$state\".$surface.char.frame // empty")
      n=$((hi - lo))
      i=0
      while [ "$i" -lt "$F" ]; do
        hop=$(q ".states.\"$state\".$surface.char.hop[$i]")
        if [ -n "$single" ]; then k=$((lo + single)); else k=$((lo + i % n)); fi
        echo "$state $surface $i k=$k x=$cx y=$((cy + hop))"
        i=$((i + 1))
      done
    done
  done
} | sort > "$WORK/oracle.txt"

if diff -q "$WORK/baker.txt" "$WORK/oracle.txt" >/dev/null; then
  echo "ok    baker and oracle agree on $(wc -l < "$WORK/baker.txt" | tr -d ' ') placements"
else
  echo "FAIL  baker and oracle disagree:"
  diff "$WORK/baker.txt" "$WORK/oracle.txt" | head -20
  exit 1
fi
