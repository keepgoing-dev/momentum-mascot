---
name: release-assistant
description: Release the Momentum Mascot desktop app. Use this skill whenever the user wants to cut a release, publish a new version, bump the version, build and release the app, or check what changed since the last release. This skill is project-local and only works for the momentum-mascot repository.
---

# Release Assistant — Momentum Mascot

This skill automates the release workflow for the Momentum Mascot Tauri app.

## Project context

- Source repo: `keepgoing-dev/momentum-mascot`
- Release script: `tools/release.sh`
- Version files: `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`
- Changelog: `CHANGELOG.md`
- Output artifact: universal macOS `.dmg`

The app assets (`src/assets/rooms/`, `src/assets/pet/`, `src-tauri/icons/bundle/`) are
derived from a licensed copy of LimeZu's Modern Interiors and are not committed to git. The
release script uses existing composed assets on disk, or recomposites them if `MASCOT_PACK` is
set.

## Workflow

1. **Check the latest released version**
   - Run `gh release list --limit 1 --json tagName`
   - Fall back to `git tag --sort=-v:refname` if no GitHub release exists
   - Report the last released tag to the user

2. **Show what changed since the last release**
   - Determine the default branch with `git branch --show-current`
   - Run `git log <last-tag>..HEAD --oneline`
   - Summarize the commits for the user

3. **Suggest a semantic version bump**
   - Run `scripts/suggest-version.sh` to analyze commits and suggest `patch`, `minor`, or `major`
   - Explain the rationale based on the changes
   - Ask the user to confirm or override the suggestion

4. **Run the release**
   - If the user confirms, run `tools/release.sh <suggestion>`
   - If the user provides an explicit version, run `tools/release.sh <version>`
   - If `src/assets/` is missing, remind the user to set `MASCOT_PACK` first

5. **Report the result**
   - On success, report the new tag, the `.dmg` path, and the GitHub release URL
   - On failure, report the exact error and do not claim success
   - If the tag was created but the release failed, warn the user about the partial state

## Pre-flight checks

Before running `tools/release.sh`, verify:

- Working tree is clean (`git status --short` is empty)
- `gh` CLI is installed and authenticated (`gh auth status`)
- The `tools/release.sh` script exists and is executable
- Either `src/assets/` is present or `MASCOT_PACK` is set

## Important rules

- Never commit or push manually; `tools/release.sh` handles commit, tag, push, build, and release.
- Do not suggest creating a release if the working tree has uncommitted changes.
- Do not override the user's explicit version choice.
- If the user says "release" without specifying a version, always analyze changes and suggest a bump first.
