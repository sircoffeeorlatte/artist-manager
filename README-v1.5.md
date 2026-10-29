# Media Library v1.5 — TikTok viewport + wallpaper library

Patch for the existing **Media Library v1.4** project. This archive contains only changed and newly added source files, not a complete project or installer.

## Changes

- Lock the document and app content to the viewport while the video gallery is in TikTok mode. The gallery page itself no longer scrolls; the wheel switches videos.
- Remove the `line-height: 1` override from the Video.js icon rule and explicitly restore the Video.js icon font after the global Roboto reset.
- Hide the Video.js control bar in TikTok mode only. The same Video.js component is used for the current TikTok clip, and the side rail provides previous/next, mute, a vertical volume slider, fullscreen and open-in-standard-viewer controls. The normal viewer retains its standard Video.js controls.
- Stop routine video playback/loading and gallery-scan tasks from flashing in the compact global status. Errors and meaningful background work remain available.
- Restyle the gallery category dropdown to fit the Quiet Graphite controls; native dropdown styling still depends partly on WebView2/Windows.
- Add a reusable wallpaper library component. Images and videos are copied (not moved) to `MediaLibrary/wallpapers`; existing wallpaper files are retained when another wallpaper is selected. The grid shows photo/video cards, and muted video previews play on hover.
- Persist the selected wallpaper as `appearance.wallpaperPath` in `user-config.json` using a relative path such as `wallpapers/desk.mp4`. A blank path uses the solid background color configured in settings.
- Create the `wallpapers` folder when a new media-library structure is created.

## Installation

1. Make a backup of the project or commit your current changes.
2. Extract this ZIP over the project directory for v1.4, preserving the folder structure and allowing the listed files to be replaced.
3. Start the app in desktop/Tauri mode. Wallpaper importing and background display use native file dialogs and the media-library folder, so those features are not available in browser-only preview mode.
4. Open **Settings → Appearance**, add an image/video wallpaper and select it from the grid. Previously copied wallpapers remain in `MediaLibrary/wallpapers` and can be reselected later.

The wallpaper source file outside the library is never moved or deleted. WebView2 video-codec support determines whether a particular video can play as a wallpaper or hover preview.

## Validation status

The changed TypeScript script blocks passed a syntax parse; CSS delimiters and key wiring checks passed. A full Vue/Tauri/Rust build was not available in the authoring environment because the Vue compiler dependencies and Cargo toolchain were absent. Please verify `npm run dev` and the Windows Tauri build before relying on this patch.

The white theme is intentionally not part of this update.
