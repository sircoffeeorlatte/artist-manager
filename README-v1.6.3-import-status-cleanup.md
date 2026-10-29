# Media Library v1.6.3 — Import & Status Cleanup

Patch for the v1.6.2 project. Copy the contents of this archive over the project folder, preserving the directory structure.

## Changes

- Removed the background, border, shadow, and blur from the global status trigger in the top bar.
- Made the expanded `app-status-panel` an opaque, theme-aware surface with no Liquid Glass blur, so the status content remains readable.
- Added Liquid Glass surfaces to the Import page and its queue, progress, destination, log, history, header, and footer areas. Standard mode keeps solid surfaces.
- Added a full-window drag-and-drop target that is active only while the `/import` route is mounted.
  - Browser mode adds actual `File` objects to the queue.
  - Tauri desktop mode uses native dropped file paths from the Webview drag/drop API.
  - Supported extensions are validated against the currently selected import type.
  - Leaving the Import page removes the route-scoped handlers.
- Removed drag-and-drop importing from gallery pages; drag-and-drop import is now offered only on the Import page.
- Replaced the card-based folder illustration with a compact text tree.
- Removed the accent-color preview samples and the `LIVE PREVIEW` badge from Settings.

## Files included

- `src/components/import/ImportWindow.vue`
- `src/pages/ImportPage.vue`
- `src/pages/SettingsPage.vue`
- `src/components/gallery/MediaGalleryPage.vue`
- `src/styles.css`

## Validation

TypeScript script sections were syntax-transpiled, CSS was parsed with no syntax errors, and feature sanity checks passed. A full Vue/Tauri build was not run in this environment.
