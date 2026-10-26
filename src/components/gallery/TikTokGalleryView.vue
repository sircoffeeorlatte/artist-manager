<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import TikTokNeighbour from './TikTokNeighbour.vue'
import type { ContentCategory, LibraryMediaItem } from '../../types/media'

const props = defineProps<{
  items: LibraryMediaItem[]
  sourceFor: (item: LibraryMediaItem) => string
  categoryLabels: Record<ContentCategory, string>
  active: boolean
}>()

const emit = defineEmits<{ open: [index: number] }>()

const currentIndex = ref(0)
const muted = ref(true)
const currentVideo = ref<HTMLVideoElement | null>(null)
const currentItem = computed(() => props.items[currentIndex.value] ?? null)
const previousItem = computed(() => props.items[currentIndex.value - 1] ?? null)
const nextItem = computed(() => props.items[currentIndex.value + 1] ?? null)

function move(step: number) {
  if (!props.items.length) return
  currentIndex.value = Math.max(0, Math.min(props.items.length - 1, currentIndex.value + step))
}

let scrollLocked = false
let scrollUnlockTimer: ReturnType<typeof setTimeout> | undefined
function onWheel(event: WheelEvent) {
  if (!props.active || Math.abs(event.deltaY) < 12 || scrollLocked) return
  const before = currentIndex.value
  move(event.deltaY > 0 ? 1 : -1)
  if (before === currentIndex.value) return
  scrollLocked = true
  if (scrollUnlockTimer) clearTimeout(scrollUnlockTimer)
  scrollUnlockTimer = setTimeout(() => { scrollLocked = false }, 420)
}

function togglePlayback() {
  const video = currentVideo.value
  if (!video) return
  if (video.paused) void video.play().catch(() => {})
  else video.pause()
}

function onKeydown(event: KeyboardEvent) {
  if (!props.active) return
  const target = event.target
  if (target instanceof HTMLInputElement || target instanceof HTMLSelectElement || target instanceof HTMLTextAreaElement) return
  if (event.key === 'ArrowUp') { event.preventDefault(); move(-1) }
  if (event.key === 'ArrowDown') { event.preventDefault(); move(1) }
}

watch(() => props.items.map((item) => item.id).join('\u001f'), () => { currentIndex.value = 0 })
watch([() => props.active, currentItem], async ([active, item]) => {
  await nextTick()
  const video = currentVideo.value
  if (!video) return
  video.muted = muted.value
  video.defaultMuted = muted.value
  if (active && item) void video.play().catch(() => {})
  else video.pause()
}, { flush: 'post' })
watch(muted, (value) => {
  if (currentVideo.value) {
    currentVideo.value.muted = value
    currentVideo.value.defaultMuted = value
  }
})

onMounted(() => window.addEventListener('keydown', onKeydown))
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown)
  if (scrollUnlockTimer) clearTimeout(scrollUnlockTimer)
  currentVideo.value?.pause()
})
</script>

<template>
  <section class="tiktok-viewer" aria-label="Просмотр видео в стиле TikTok" @wheel.prevent="onWheel">
    <div v-if="currentItem" class="tiktok-feed-rail">
      <TikTokNeighbour
        v-if="previousItem"
        :key="previousItem.id"
        :item="previousItem"
        :source="sourceFor(previousItem)"
        label="Предыдущее видео"
        position="prev"
        :active="active"
        @select="move(-1)"
      />

      <div class="tiktok-stage">
        <img
          v-if="currentItem.extension.toLowerCase() === '.gif'"
          :key="currentItem.id"
          class="tiktok-media"
          :src="sourceFor(currentItem)"
          :alt="currentItem.originalName"
        />
        <video
          v-else
          :key="currentItem.id"
          ref="currentVideo"
          class="tiktok-media"
          :src="sourceFor(currentItem)"
          :autoplay="active"
          :muted="muted"
          playsinline
          preload="auto"
          @click="togglePlayback"
        />
        <div class="tiktok-gradient" aria-hidden="true"></div>
        <div class="tiktok-caption">
          <span class="tiktok-index">{{ currentIndex + 1 }} / {{ items.length }}</span>
          <span class="tiktok-category">{{ currentItem.category ? categoryLabels[currentItem.category] : 'Без категории' }}</span>
        </div>
        <div class="tiktok-actions" aria-label="Управление видео">
          <button type="button" class="tiktok-action-button" :disabled="currentIndex <= 0" aria-label="Предыдущее видео" title="Предыдущее видео (↑)" @click="move(-1)">↑</button>
          <button type="button" class="tiktok-action-button" :disabled="currentIndex >= items.length - 1" aria-label="Следующее видео" title="Следующее видео (↓)" @click="move(1)">↓</button>
          <button type="button" class="tiktok-action-button" :aria-label="muted ? 'Включить звук' : 'Выключить звук'" :title="muted ? 'Включить звук' : 'Выключить звук'" @click="muted = !muted">{{ muted ? '◖' : '♫' }}</button>
          <button type="button" class="tiktok-action-button" aria-label="Открыть видео в просмотрщике" title="Открыть видео" @click="emit('open', currentIndex)">↗</button>
        </div>
      </div>

      <TikTokNeighbour
        v-if="nextItem"
        :key="nextItem.id"
        :item="nextItem"
        :source="sourceFor(nextItem)"
        label="Следующее видео"
        position="next"
        :active="active"
        @select="move(1)"
      />
    </div>

    <div v-else class="tiktok-empty">
      <div class="gallery-empty-icon">▶</div>
      <h2>Пока нет видео для просмотра</h2>
      <p>Добавь видео в сетке — они появятся здесь, в вертикальной ленте.</p>
    </div>

    <p v-if="currentItem" class="tiktok-help">Колёсико мыши или ↑ / ↓ — переключить видео · Нажми на ролик, чтобы поставить на паузу</p>
  </section>
</template>
