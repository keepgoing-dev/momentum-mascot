# Design Specification: KeepGoing MVP (Native macOS + Rust)

## 1. Project Overview
KeepGoing is a local-first, minimal developer tool designed to maintain momentum on side projects. It acts as an 8-bit "mascot/cheerleader" that lives in the macOS Menu Bar. It monitors specific, user-selected local Git repositories (opt-in tracking) and updates the mascot's emotional state based on commit frequency.

**Core Philosophy:** Strict MVP. Zero network calls, zero complex backend, zero multi-tab UI. Do not add features outside this spec.

## 2. Tech Stack & Architecture
*   **UI/Frontend:** Native macOS App built with Swift & SwiftUI. Runs exclusively in the Menu Bar (System Tray).
*   **Backend/Watcher:** CLI Tool and Git Hook handler built with Rust.
*   **Inter-Process Communication (IPC):** 
    1.  **State Storage:** Rust writes state to a local JSON file (`~/.keepgoing/state.json`).
    2.  **Trigger:** Rust triggers a Custom URL Scheme (e.g., `keepgoing://ping`) to wake up/force-refresh the Swift Menu Bar app immediately after a commit.

## 3. Core Workflows

### A. Opt-in Project Tracking
1. User clicks "Add Project" in the Swift UI.
2. Swift opens a native file picker to select a local directory.
3. Swift executes the Rust CLI binary (e.g., `keepgoing-cli track <path>`).
4. Rust CLI validates the Git repo, installs a local `post-commit` hook in that specific repo, and registers the path in `state.json`.

### B. Commit & State Evaluation
1. User runs `git commit` in a tracked project.
2. The `post-commit` hook executes the Rust CLI: `keepgoing-cli hook-trigger <path>`.
3. Rust CLI updates the `last_commit_at` timestamp for that project in `state.json`.
4. Rust CLI evaluates the overall "Global State" based on timestamps.
5. Rust CLI pings the Swift app via `open keepgoing://ping`.
6. Swift app re-reads `state.json` and updates the Menu Bar icon and Mascot UI.

## 4. State Definitions & Logic
The Mascot represents the overall health of the user's active side projects. 
*   **Fire (Active):** The most recent commit across *all* tracked projects is `< 24 hours` old. (Mascot is happy/dancing).
*   **Idle (Warning):** The most recent commit is `>= 24 hours` AND `< 72 hours` old. (Mascot is bored/waiting).
*   **Dead (Neglected):** The most recent commit is `>= 72 hours` old. (Mascot is dead/ghost/crying).

## 5. Data Model (Local JSON Schema)
All state is stored locally at `~/.keepgoing/state.json`. 

```json
{
  "version": "1.0",
  "global_state": "idle", 
  "tracked_projects": [
    {
      "id": "a1b2c3d4-e5f6-7890",
      "path": "/Users/username/Projects/my-side-project",
      "name": "my-side-project",
      "added_at": "2026-08-12T08:00:00Z",
      "last_commit_at": "2026-08-12T08:00:00Z",
      "status": "fire"
    }
  ]
}
```

* Note for Claude: Ensure parsing logic in both Swift and Rust is resilient to missing fields or empty arrays.

## 6. UI/UX Requirements (SwiftUI)
Menu Bar Icon: Dynamic icon (e.g., a pixel heart or flame) that changes color based on global_state (Red = Dead, Yellow = Idle, Green/Blue = Fire).

### Popover View:

* Top Section: A 150x150 pixel area displaying the Mascot animation/image corresponding to global_state. Includes a short, hardcoded snarky/encouraging quote.

* Middle Section: A minimal list of tracked_projects showing the project name and time elapsed since last_commit_at (e.g., "2 hours ago").

### Bottom Section: Two buttons:

* "Add Project" (Triggers file picker).

* "Share Status" (Generates a static 8-bit style image of the Mascot and copies it to the clipboard).

* Aesthetics: Dark mode by default. Use monospace fonts (e.g., standard macOS monospace or an embedded pixel font).

## 7. Implementation Phases (Instructions for Claude Code)
### Phase 1: Rust Core MVP

* Initialize Rust binary.

* Implement state.json read/write operations.

* Implement the track <path> command (installs bash post-commit hook).

* Implement the hook-trigger <path> command (updates JSON and executes macOS open keepgoing://ping).

### Phase 2: Swift UI Skeleton

* Initialize macOS SwiftUI App.

* Configure Info.plist for Menu Bar only (LSUIElement = true) and URL Scheme (keepgoing).

* Build the base Popover UI and dynamic Menu Bar icon.

### Phase 3: Integration & Assets

* Connect Swift UI to read from ~/.keepgoing/state.json on launch and on URL Scheme trigger.

* Wire up the "Add Project" button to execute the compiled Rust binary.

* Implement the "Share Status" clipboard logic.