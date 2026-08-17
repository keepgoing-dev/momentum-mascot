#!/bin/sh
#
# Analyzes commits since the last tag and suggests a semantic version bump.
# Intended for use with the momentum-mascot release-assistant skill.
#
set -eu

LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || true)

if [ -z "$LAST_TAG" ]; then
  echo "No previous tag found."
  echo "Suggested bump: minor"
  exit 0
fi

echo "Last tag: $LAST_TAG"
echo ""
echo "Commits since $LAST_TAG:"
git log "$LAST_TAG..HEAD" --oneline || true
echo ""

major=0
minor=0
patch=0

COMMITS=$(git log "$LAST_TAG..HEAD" --pretty=format:"%s" 2>/dev/null || true)

if [ -z "$COMMITS" ]; then
  echo "No commits since $LAST_TAG."
  echo "Suggested bump: none"
  exit 0
fi

while IFS= read -r msg; do
  [ -z "$msg" ] && continue

  lower=$(echo "$msg" | tr '[:upper:]' '[:lower:]')

  case "$lower" in
    *breaking* | *"breaking change"* | *"breaking:"* | *"!:"* )
      major=1
      ;;
  esac

  case "$lower" in
    feat* | feature* | "add "* | "adds "* | "added "* | "implement"* | "support"* )
      minor=1
      ;;
    fix* | bugfix* | "fix:"* | "bug fix"* | "patch"* )
      patch=1
      ;;
    *)
      patch=1
      ;;
  esac
done <<EOF
$COMMITS
EOF

if [ "$major" -eq 1 ]; then
  BUMP="major"
elif [ "$minor" -eq 1 ]; then
  BUMP="minor"
else
  BUMP="patch"
fi

echo "Suggested bump: $BUMP"
