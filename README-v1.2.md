# Media Library v1.2 — UI polish

Patch applies on top of the v1.1 project. It contains only the seven updated source files and this note.

## Changes
- Uses Oswald as the app-wide font, with Arial Narrow / Segoe UI fallbacks.
- Disables accidental text selection throughout the UI while preserving selection in editable inputs and explicitly marked path values.
- Unifies panel, toolbar, button and scrollbar styling; refines the gallery and TikTok stage visuals.
- Reworks the photo/video viewer layout, metadata panel, navigation and controls.
- Adds drag-to-pan for zoomed photos. Zoom is controlled by the wheel and +/- buttons, with reset by clicking the percentage.
- Replaces the mismatched TikTok sound glyphs with a single speaker icon that changes between waves and mute-cross states; adds a persistent vertical volume slider and saves volume in user configuration.
- Ties Video.js player accent colors to the user's current accent color.

## Apply
Extract this archive into the project root and allow it to replace the listed files. Keep the existing folder structure. No dependency/package changes are required.

Oswald is loaded from Google Fonts when a network connection is available. If it cannot be loaded, the interface uses the local Arial Narrow / Segoe UI fallback fonts.

The modified TypeScript blocks and CSS were syntax-parsed; Vue/Tauri production build was not run in this environment because the project dependencies and Rust/Cargo toolchain were not available here. Please run `npm run dev` and test photo dragging, volume persistence, and the Tauri build on Windows.
