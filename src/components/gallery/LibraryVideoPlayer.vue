<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import videojs from 'video.js'
import 'video.js/dist/video-js.css'

const props = defineProps<{
  src: string
  volume: number
  muted: boolean
  autoplay?: boolean
}>()

const emit = defineEmits<{ error: []; 'volume-change': [value: number]; 'muted-change': [value: boolean] }>()
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

onMounted(() => {
  if (!videoElement.value) return

  player = videojs(videoElement.value, {
    controls: true,
    autoplay: false,
    preload: 'metadata',
    responsive: true,
    fluid: false,
    inactivityTimeout: 0,
    playbackRates: [0.5, 0.75, 1, 1.25, 1.5, 2],
    controlBar: {
      volumePanel: { inline: false },
    },
  })

  player.addClass('library-videojs')
  player.on('error', () => emit('error'))
  player.on('volumechange', syncAudioSettings)
  // Keep the standard controls visible while the video is playing in the windowed viewer.
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

onBeforeUnmount(() => {
  if (player && !player.isDisposed()) player.dispose()
  player = null
})
</script>

<template>
  <div class="library-video-player-shell">
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
  border: 1px solid #252c37;
  border-radius: 10px;
  background: #05070a;
  box-shadow: 0 18px 70px #0005;
}

.library-video-player-shell .video-js {
  display: block;
  width: 100%;
  height: 100%;
  color: #eaf0f8;
  background: #05070a;
  font-family: Inter, "Segoe UI", system-ui, sans-serif;
  --vjs-theme-primary: #a3e635;
}

.library-video-player-shell .video-js .vjs-tech {
  object-fit: contain;
}

/* Video.js hides its bar after inactivity by default. Keep all controls reachable in-window too. */
.library-video-player-shell .video-js .vjs-control-bar,
.library-video-player-shell .video-js.vjs-user-inactive .vjs-control-bar,
.library-video-player-shell .video-js.vjs-paused .vjs-control-bar,
.library-video-player-shell .video-js.vjs-playing .vjs-control-bar {
  display: flex !important;
  visibility: visible !important;
  opacity: 1 !important;
  pointer-events: auto !important;
  background: linear-gradient(180deg, #090c12d9, #090c12f5);
  color: #f0f5e8;
  font-size: 13px;
  transition: none !important;
}

.library-video-player-shell .video-js .vjs-play-progress,
.library-video-player-shell .video-js .vjs-volume-level,
.library-video-player-shell .video-js .vjs-slider-bar {
  background: #a3e635;
}

.library-video-player-shell .video-js .vjs-load-progress,
.library-video-player-shell .video-js .vjs-load-progress div {
  background: #77839a;
}

.library-video-player-shell .video-js .vjs-big-play-button {
  top: 50%;
  left: 50%;
  width: 2.8em;
  height: 2.8em;
  transform: translate(-50%, -50%);
  border: 1px solid #d9f99d99;
  border-radius: 50%;
  background: #121a0fcc;
  color: #d9f99d;
  line-height: 2.65em;
  backdrop-filter: blur(8px);
}

.library-video-player-shell .video-js:hover .vjs-big-play-button,
.library-video-player-shell .video-js .vjs-big-play-button:focus {
  border-color: #d9f99d;
  background: #253418ef;
}

.library-video-player-shell .video-js .vjs-menu-button-popup .vjs-menu .vjs-menu-content {
  background: #141b25ed;
}

.library-video-player-shell .video-js .vjs-menu li {
  color: #d9e2ee;
}

.library-video-player-shell .video-js .vjs-menu li.vjs-selected,
.library-video-player-shell .video-js .vjs-menu li:hover {
  color: #19230c;
  background: #a3e635;
}

@media (max-height: 720px) {
  .library-video-player-shell {
    width: min(100%, calc((100vh - 145px) * 1.77778));
    max-height: calc(100vh - 120px);
  }
}

@media (max-width: 720px) {
  .library-video-player-shell {
    width: min(100%, calc((100vh - 205px) * 1.77778));
    max-height: calc(100vh - 175px);
    border-radius: 7px;
  }
}
</style>
