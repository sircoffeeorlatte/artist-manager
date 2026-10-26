<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, watch } from 'vue'
import type { LibraryMediaItem } from '../../types/media'

const props = defineProps<{
  item: LibraryMediaItem
  source: string
  label: string
  position: 'prev' | 'next'
  active: boolean
}>()

const emit = defineEmits<{ select: [] }>()
const videoElement = ref<HTMLVideoElement | null>(null)

watch(() => [props.source, props.active] as const, async ([source, active], previous) => {
  await nextTick()
  const video = videoElement.value
  if (!video) return
  video.muted = true
  video.defaultMuted = true
  video.playsInline = true
  video.loop = true

  if (previous && source !== previous[0]) video.load()
  if (active) void video.play().catch(() => {})
  else video.pause()
}, { immediate: true, flush: 'post' })

onBeforeUnmount(() => videoElement.value?.pause())
</script>

<template>
  <button
    type="button"
    class="tiktok-neighbour"
    :class="`tiktok-neighbour-${position}`"
    :aria-label="label"
    :title="label"
    @click="emit('select')"
  >
    <img
      v-if="item.extension.toLowerCase() === '.gif'"
      class="tiktok-neighbour-media"
      :src="source"
      alt=""
    />
    <video
      v-else
      ref="videoElement"
      class="tiktok-neighbour-media"
      :src="source"
      muted
      loop
      playsinline
      :preload="active ? 'auto' : 'metadata'"
      tabindex="-1"
      aria-hidden="true"
    />
    <span class="tiktok-neighbour-label">{{ position === 'prev' ? '↑' : '↓' }} {{ label }}</span>
  </button>
</template>
