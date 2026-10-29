# Media Library v1.7.2 — Folder Access Consent

Patch for Media Library v1.7.1.

## What changed

- Adds `opener:allow-open-path` to `src-tauri/capabilities/default.json`, fixing the Tauri error `opener.open_path not allowed`.
- Adds an app-level consent dialog on startup for Windows Explorer folder opening.
- Stores consent in `user-config.json` as `permissions.openFolders`.
- Gates folder opening behind the consent service. If the user chooses “Не сейчас”, opening paths is cancelled and the prompt appears again at the next launch or next path-open attempt.
- Older config files without `permissions` are migrated with consent defaulting to `false`.

## Apply

Extract the contents over the project v1.7.1 and preserve the directory structure. Then fully restart the Tauri dev process or rebuild the desktop application (`npm run tauri:dev` / the project's desktop build command). Tauri capabilities are applied by the desktop build; replacing frontend files alone does not update an already-running executable.

## Important distinction

The dialog is an application-level consent gate and preference. Tauri v2's `opener:allow-open-path` is a build-time capability declared in `src-tauri/capabilities/default.json`; it is not a Windows runtime permission prompt. The app only calls `openPath` after the user has accepted this consent.
