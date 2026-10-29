# Media Library v1.6 — Liquid Glass + Light Theme

Patch based on v1.5. Contains only changed/new files.

## Changes
- Folder tree uses a single vertical category list; count chips stay on one line.
- Category and settings dropdowns use theme-aware control styling (native popup rendering remains partly controlled by WebView2/Windows).
- Adds `appearance.visualMode`: `liquid-glass` (default) or `standard`.
- Adds `appearance.theme`: `dark` or `light`; dark remains the initial theme, while Liquid Glass is the default surface mode.
- Preferences are saved in the existing `user-config.json`; older configs are normalized with defaults.
- Rust configuration model includes the new values and validates them.

## Apply
Unzip over the v1.5 project root, keeping the folder structure. Rebuild the Tauri app if you need the native JSON config path.

## Checks
TypeScript syntax and CSS delimiter checks were performed. A complete Vue/Tauri build was not performed in this environment.
