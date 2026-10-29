# Media Library v1.7.3 — Open Folder Fix

Patch for v1.7.2. This fixes `opener.open_path not allowed` by no longer using the Tauri opener plugin for Explorer folder actions.

## Changes
- `fileLocations.ts` calls the native Tauri command `open_folder_in_explorer` after the existing in-app consent check.
- The Rust command validates that the requested absolute path exists and is a directory, then starts Windows Explorer directly.
- Registers the command in `src-tauri/src/lib.rs`.
- Removes the unnecessary `opener:allow-open-path` capability from the default capability; the application no longer calls `opener.open_path` for folder opening.
- The `permissions.openFolders` consent flow remains unchanged.

## Apply
Extract over the v1.7.2 project and preserve paths. Fully restart the Tauri process or rebuild the desktop app so the Rust command is compiled and registered. This patch is not a standalone application.

## Why this fixes it
The previous code called `@tauri-apps/plugin-opener`'s `openPath`, which is blocked by the runtime capability allowlist in the running build. The new flow uses an explicitly registered Rust command and launches `explorer.exe` directly, so it does not depend on `opener:allow-open-path` being applied by the current capability bundle. The in-app consent still gates calls from the UI.
