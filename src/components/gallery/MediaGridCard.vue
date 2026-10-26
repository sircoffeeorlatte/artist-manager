<script setup lang="ts">
import type { LibraryMediaItem } from '../../types/media'

const props = defineProps<{
  item: LibraryMediaItem
  index: number
  kind: 'photo' | 'video'
  source: string
}>()

const emit = defineEmits<{ open: [index: number] }>()

function playPreview(event: Event) {
  if (props.kind !== 'video' || props.item.extension.toLowerCase() === '.gif') return
  const video = (event.currentTarget as HTMLElement | null)?.querySelector('video')
  if (!video) return

  video.dataset.previewActive = 'true'
  video.muted = true
  video.defaultMuted = true
  video.playsInline = true
  video.loop = true
  video.preload = 'auto'

  const playWhenReady = () => {
    if (video.dataset.previewActive !== 'true') return
    if (video.readyState >= HTMLMediaElement.HAVE_CURRENT_DATA) {
      try {
        if (video.currentTime > 0) video.currentTime = 0
      } catch { /* Some streams cannot seek until metadata is ready. */ }
      void video.play().catch(() => {})
      return
    }

    if (video.dataset.previewLoading === 'true') return
    video.dataset.previewLoading = 'true'
    const finishLoading = () => { video.dataset.previewLoading = 'false' }
    const onLoadedData = () => {
      finishLoading()
      if (video.dataset.previewActive === 'true') void video.play().catch(() => {})
    }
    const onError = () => finishLoading()
    video.addEventListener('loadeddata', onLoadedData, { once: true })
    video.addEventListener('error', onError, { once: true })
    video.load()
  }

  playWhenReady()
}

function stopPreview(event: Event) {
  const video = (event.currentTarget as HTMLElement | null)?.querySelector('video')
  if (!video) return
  video.dataset.previewActive = 'false'
  video.pause()
  video.loop = false
  if (video.readyState >= HTMLMediaElement.HAVE_METADATA) {
    try { video.currentTime = 0 } catch { /* Some containers are not seekable. */ }
  }
}
</script>

<template>
  <button
    type="button"
    class="media-card"
    :aria-label="`Открыть ${kind === 'photo' ? 'фото' : 'видео'}: ${item.originalName}`"
    @click="emit('open', index)"
    @mouseenter="playPreview"
    @mouseleave="stopPreview"
    @focus="playPreview"
    @blur="stopPreview"
  >
    <div class="media-card-preview">
      <img v-if="kind === 'photo' || item.extension.toLowerCase() === '.gif'" :src="source" :alt="item.originalName" loading="lazy" />
      <video v-else :src="source" preload="metadata" muted playsinline loop aria-hidden="true" />
    </div>
  </button>
</template>
