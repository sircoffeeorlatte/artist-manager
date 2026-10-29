<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import videojs from 'video.js'
import 'video.js/dist/video-js.css'

const props = withDefaults(defineProps<{
  src: string
  volume: number
  muted: boolean
  autoplay?: boolean
  variant?: 'standard' | 'tiktok'
}>(), {
  autoplay: false,
  variant: 'standard',
})

const emit = defineEmits<{
  error: []
  'volume-change': [value: number]
  'muted-change': [value: boolean]
  loadstart: []
  waiting: []
  stalled: []
  canplay: []
  playing: []
}>()

const videoElement = ref<HTMLVideoElement | null>(null)
let player: ReturnType<typeof videojs> | null = null

function applyAudioSettings() {
  if (!player) return
  player.volume(Math.max(0, Math.min(1, props.volume)))
  player.muted(props.muted)
}

function syncAudioSettings() {
  if (!player) return
  emit('volume-change', player.volume())
  emit('muted-change', player.muted())
}

function tryPlay() {
  if (!player || !props.autoplay) return
  const result = player.play()
  if (result && typeof result.catch === 'function') void result.catch(() => {})
}

function syncPlaybackState(autoplay: boolean) {
  if (!player) return
  if (autoplay) tryPlay()
  else player.pause()
}

/** Called by the TikTok side rail so its player can hide the normal control bar. */
function toggleFullscreen(): void {
  if (!player || player.isDisposed()) return
  try {
    if (player.isFullscreen()) player.exitFullscreen()
    else player.requestFullscreen()
  } catch {
    const element = player.el()
    if (element?.requestFullscreen) void element.requestFullscreen().catch(() => {})
  }
}

defineExpose({ toggleFullscreen })

onMounted(() => {
  if (!videoElement.value) return

  player = videojs(videoElement.value, {
    controls: true,
    autoplay: false,
    preload: 'metadata',
    responsive: true,
    fluid: false,
    inactivityTimeout: 0,
    playsinline: true,
    playbackRates: [0.5, 0.75, 1, 1.25, 1.5, 2],
    controlBar: {
      volumePanel: { inline: false },
      fullscreenToggle: true,
      pictureInPictureToggle: false,
    },
  })

  player.addClass('library-videojs')
  player.on('error', () => emit('error'))
  player.on('loadstart', () => emit('loadstart'))
  player.on('waiting', () => emit('waiting'))
  player.on('stalled', () => emit('stalled'))
  player.on('canplay', () => emit('canplay'))
  player.on('playing', () => emit('playing'))
  player.on('volumechange', syncAudioSettings)
  // Keep controls easy to reach while not fullscreen. Video.js still owns fullscreen behavior.
  player.on('play', () => player?.userActive(true))
  player.on('pause', () => player?.userActive(true))
  player.on('ended', () => player?.userActive(true))
  player.ready(() => {
    if (!player) return
    if (props.src) player.src(props.src)
    applyAudioSettings()
    tryPlay()
  })
})

watch(() => props.src, (src) => {
  if (!player || !src) return
  player.src(src)
  tryPlay()
})

watch(() => [props.volume, props.muted] as const, applyAudioSettings)
watch(() => props.autoplay, syncPlaybackState)

onBeforeUnmount(() => {
  if (player && !player.isDisposed()) player.dispose()
  player = null
})
</script>

<template>
  <div :class="['library-video-player-shell', { 'is-tiktok': variant === 'tiktok' }]">
    <video ref="videoElement" class="video-js vjs-big-play-centered" playsinline preload="metadata" />
  </div>
</template>

<style>
.library-video-player-shell {
  position: relative;
  width: min(100%, 1080px, calc((100vh - 175px) * 1.77778));
  aspect-ratio: 16 / 9;
  max-height: calc(100vh - 150px);
  margin: auto;
  overflow: hidden;
  border: 1px solid var(--border, #252c37);
  border-radius: 9px;
  background: #05070a;
  box-shadow: none;
}

.library-video-player-shell .video-js {
  display: block;
  width: 100%;
  height: 100%;
  color: #eaf0f8;
  background: #05070a;
  font-family: "Roboto", "Segoe UI", Arial, sans-serif;
  --vjs-theme-primary: var(--accent);
}

.library-video-player-shell .video-js .vjs-tech { object-fit: contain; }

/* Explicitly restore Video.js's icon font after the app-wide typography reset. */
.library-video-player-shell .video-js .vjs-icon-placeholder,
.library-video-player-shell .video-js .vjs-icon-placeholder::before,
.library-video-player-shell .video-js .vjs-control::before,
.library-video-player-shell .video-js .vjs-button > .vjs-icon-placeholder::before {
  font-family: VideoJS !important;
  font-style: normal !important;
  font-weight: normal !important;
}

.library-video-player-shell .video-js .vjs-control-bar,
.library-video-player-shell .video-js.vjs-user-inactive .vjs-control-bar,
.library-video-player-shell .video-js.vjs-paused .vjs-control-bar,
.library-video-player-shell .video-js.vjs-playing .vjs-control-bar {
  display: flex !important;
  visibility: visible !important;
  opacity: 1 !important;
  pointer-events: auto !important;
  background: linear-gradient(180deg, #090c12dc, #090c12f5);
  color: #f0f2f6;
  font-size: 13px;
  transition: none !important;
}

.library-video-player-shell .video-js .vjs-play-progress,
.library-video-player-shell .video-js .vjs-volume-level,
.library-video-player-shell .video-js .vjs-slider-bar { background: var(--accent); }

.library-video-player-shell .video-js .vjs-load-progress,
.library-video-player-shell .video-js .vjs-load-progress div { background: #77839a; }

.library-video-player-shell .video-js .vjs-big-play-button {
  top: 50%;
  left: 50%;
  width: 2.8em;
  height: 2.8em;
  transform: translate(-50%, -50%);
  border: 1px solid color-mix(in srgb, var(--accent) 60%, transparent);
  border-radius: 50%;
  background: color-mix(in srgb, var(--accent) 12%, #12151b);
  color: var(--accent);
  line-height: 2.65em;
  backdrop-filter: blur(8px);
}

.library-video-player-shell .video-js:hover .vjs-big-play-button,
.library-video-player-shell .video-js .vjs-big-play-button:focus {
  border-color: var(--accent-strong);
  background: var(--accent-alpha-20);
}

.library-video-player-shell .video-js .vjs-menu-button-popup .vjs-menu .vjs-menu-content { background: #191b20f5; }
.library-video-player-shell .video-js .vjs-menu li { color: #d9dce2; }
.library-video-player-shell .video-js .vjs-menu li.vjs-selected,
.library-video-player-shell .video-js .vjs-menu li:hover { color: var(--accent-text); background: var(--accent); }

/* TikTok uses the same Video.js player and control bar, fitted into the portrait stage. */
.library-video-player-shell.is-tiktok {
  position: absolute;
  inset: 0;
  z-index: 3;
  width: 100%;
  height: 100%;
  max-height: none;
  aspect-ratio: auto;
  margin: 0;
  overflow: visible;
  border: 0;
  border-radius: inherit;
  box-shadow: none;
}
.library-video-player-shell.is-tiktok .video-js { width: 100%; height: 100%; border-radius: inherit; }
.library-video-player-shell.is-tiktok .video-js .vjs-control-bar { display: none !important; visibility: hidden !important; opacity: 0 !important; pointer-events: none !important; }
.library-video-player-shell.is-tiktok .video-js .vjs-fullscreen-control { display: block !important; }
.library-video-player-shell.is-tiktok .video-js .vjs-menu { z-index: 10; }

@media (max-height: 720px) {
  .library-video-player-shell:not(.is-tiktok) {
    width: min(100%, calc((100vh - 145px) * 1.77778));
    max-height: calc(100vh - 120px);
  }
}

@media (max-width: 720px) {
  .library-video-player-shell:not(.is-tiktok) {
    width: min(100%, calc((100vh - 205px) * 1.77778));
    max-height: calc(100vh - 175px);
    border-radius: 7px;
  }
}
</style>
