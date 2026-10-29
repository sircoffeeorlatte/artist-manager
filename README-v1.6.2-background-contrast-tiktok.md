# Media Library v1.6.2 — Background Contrast & TikTok Stability

Patch base: **Media Library v1.6.1 Theme Audit**.

## Files included

- `src/styles.css`
- `src/services/appearanceSettings.ts`
- `src/components/gallery/MediaGalleryPage.vue`
- `src/components/gallery/TikTokGalleryView.vue`
- `src/pages/SettingsPage.vue`

## Changes

- The selected background HEX remains saved as-is in `user-config.json`. At runtime, the displayed canvas lightness is adjusted to keep the active theme's text readable. Bright backgrounds in dark mode are darkened while retaining hue where possible; very dark backgrounds in light mode are lightened. This prevents dark-theme light text from disappearing on a white user background.
- When a wallpaper image/video is enabled, page-intro copy and section-heading get a theme-aware surface. Their text uses the current theme's foreground colors, and Liquid Glass adds blur/transparency; Standard mode uses a solid panel.
- TikTok viewer surface is transparent so the selected wallpaper remains visible behind the video stage.
- TikTok layout dimensions are recalculated from window size and global UI scale, including on resize. This addresses the vertical offset/overflow caused by applying `zoom` to `#app` while using raw `100vh/100vw` dimensions.
- The TikTok mute/unmute button and volume slider are separate blocks.
- The Video.js big-play overlay is hidden in TikTok mode to prevent the brief overlay flash between clips. Ordinary Video.js player mode is unchanged.
- TikTok category chip uses theme-aware colors and surfaces in Light/Dark and Liquid Glass/Standard modes.

## Installation

Unzip this archive over your current v1.6.1 project root, preserving folder structure and replacing the listed files. This is a source patch, not a full project or EXE installer.

## Verification notes

CSS was parsed with PostCSS. Modified TypeScript script blocks passed TypeScript transpilation/syntax checks, and changed Vue templates passed tag-balance checks. A full Vue/Tauri build was not run in this environment.
