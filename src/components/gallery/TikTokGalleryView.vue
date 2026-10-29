<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import TikTokNeighbour from './TikTokNeighbour.vue'
import LibraryVideoPlayer from './LibraryVideoPlayer.vue'
import type { ContentCategory, LibraryMediaItem } from '../../types/media'
import { getUserConfig, updateUserConfig } from '../../services/userConfig'
import { notify } from '../../services/appState'

const props = defineProps<{
  items: LibraryMediaItem[]
  sourceFor: (item: LibraryMediaItem) => string
  categoryLabels: Record<ContentCategory, string>
  active: boolean
}>()

const emit = defineEmits<{ open: [index: number] }>()

const currentIndex = ref(0)
const muted = ref(getUserConfig().video.tiktokMuted)
const volume = ref(getUserConfig().video.volume)
const soundIsOff = computed(() => muted.value || volume.value <= 0.001)
const currentPlayer = ref<{ toggleFullscreen: () => void } | null>(null)
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
  if (event.target instanceof HTMLElement && event.target.closest('input, button, select, textarea, .vjs-control-bar, .vjs-menu')) return
  if (!props.active || Math.abs(event.deltaY) < 12 || scrollLocked) return
  event.preventDefault()
  const before = currentIndex.value
  move(event.deltaY > 0 ? 1 : -1)
  if (before === currentIndex.value) return
  scrollLocked = true
  if (scrollUnlockTimer) clearTimeout(scrollUnlockTimer)
  scrollUnlockTimer = setTimeout(() => { scrollLocked = false }, 420)
}

function onKeydown(event: KeyboardEvent) {
  if (!props.active) return
  const target = event.target
  if (target instanceof HTMLInputElement || target instanceof HTMLSelectElement || target instanceof HTMLTextAreaElement || (target instanceof HTMLElement && target.closest('.vjs-control-bar, .vjs-menu'))) return
  if (event.key === 'ArrowUp') { event.preventDefault(); move(-1) }
  if (event.key === 'ArrowDown') { event.preventDefault(); move(1) }
}

function onPlayerVolumeChange(value: number) {
  volume.value = Math.max(0, Math.min(1, value))
}

function toggleMute() {
  if (soundIsOff.value) {
    if (volume.value <= 0.001) volume.value = 0.5
    muted.value = false
  } else {
    muted.value = true
  }
}

function toggleFullscreen() {
  currentPlayer.value?.toggleFullscreen()
}

function notifyVideoError() {
  notify('error', 'Не удалось открыть видео', `${currentItem.value?.originalName ?? 'Видео'}: файл недоступен, повреждён или его формат не поддерживается.`)
}

function onPlayerMutedChange(value: boolean) {
  muted.value = value
}

watch(() => props.items.map((item) => item.id).join('\u001f'), () => {
  currentIndex.value = 0
})
watch(muted, (value) => {
  const savedVideo = getUserConfig().video
  if (savedVideo.tiktokMuted !== value) updateUserConfig({ video: { ...savedVideo, tiktokMuted: value } })
})
watch(volume, (value) => {
  const safeVolume = Math.max(0, Math.min(1, value))
  if (safeVolume !== value) { volume.value = safeVolume; return }
  const savedVideo = getUserConfig().video
  if (Math.abs(savedVideo.volume - safeVolume) > 0.001) updateUserConfig({ video: { ...savedVideo, volume: safeVolume } })
})

onMounted(() => window.addEventListener('keydown', onKeydown))
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown)
  if (scrollUnlockTimer) clearTimeout(scrollUnlockTimer)
})
</script>

<template>
  <section class="tiktok-viewer" aria-label="Просмотр видео в стиле TikTok" @wheel="onWheel">
    <div v-if="currentItem" class="tiktok-feed-layout">
      <div class="tiktok-feed-rail">
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
          <LibraryVideoPlayer
            v-else
            ref="currentPlayer"
            :key="currentItem.id"
            variant="tiktok"
            :src="sourceFor(currentItem)"
            :volume="volume"
            :muted="muted"
            :autoplay="active"
            @volume-change="onPlayerVolumeChange"
            @muted-change="onPlayerMutedChange"
            @error="notifyVideoError"
          />
          <div class="tiktok-gradient" aria-hidden="true"></div>
          <div class="tiktok-caption">
            <span class="tiktok-index">{{ currentIndex + 1 }} / {{ items.length }}</span>
            <span class="tiktok-category">{{ currentItem.category ? categoryLabels[currentItem.category] : 'Без категории' }}</span>
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

      <div class="tiktok-actions" aria-label="Управление лентой">
        <button type="button" class="tiktok-action-button" :disabled="currentIndex <= 0" aria-label="Предыдущее видео" title="Предыдущее видео (↑)" @click="move(-1)">
          <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m6 14 6-6 6 6"/></svg>
        </button>
        <button type="button" class="tiktok-action-button" :disabled="currentIndex >= items.length - 1" aria-label="Следующее видео" title="Следующее видео (↓)" @click="move(1)">
          <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m6 10 6 6 6-6"/></svg>
        </button>
        <div class="tiktok-mute-block" title="Звук видео">
          <button type="button" class="tiktok-action-button tiktok-mute-button" :aria-label="soundIsOff ? 'Включить звук' : 'Выключить звук'" :title="soundIsOff ? 'Включить звук' : 'Выключить звук'" @click="toggleMute">
            <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
              <path d="M11 5 6 9H3v6h3l5 4z" />
              <path v-if="soundIsOff" d="m16 9 5 6m0-6-5 6" />
              <g v-else><path d="M15 9a5 5 0 0 1 0 6" /><path d="M18 6a9 9 0 0 1 0 12" /></g>
            </svg>
          </button>
        </div>
        <div class="tiktok-volume-control" title="Громкость видео" @wheel.stop.prevent>
          <input v-model.number="volume" class="tiktok-volume-slider" type="range" min="0" max="1" step="0.05" aria-label="Громкость видео" :aria-valuetext="`${Math.round(volume * 100)} процентов`" @wheel.stop.prevent />
          <span class="tiktok-volume-value">{{ Math.round(volume * 100) }}%</span>
        </div>
        <button type="button" class="tiktok-action-button" aria-label="Полноэкранный режим" title="Полноэкранный режим" @click="toggleFullscreen">
          <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M8 3H5a2 2 0 0 0-2 2v3M16 3h3a2 2 0 0 1 2 2v3M3 16v3a2 2 0 0 0 2 2h3M16 21h3a2 2 0 0 0 2-2v-3"/></svg>
        </button>
        <button type="button" class="tiktok-action-button" aria-label="Открыть видео в обычном просмотрщике" title="Открыть обычный просмотрщик" @click="emit('open', currentIndex)">
          <svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M14 4h6v6M20 4l-9 9"/><path d="M18 13v5a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h5"/></svg>
        </button>
      </div>
    </div>

    <div v-else class="tiktok-empty">
      <div class="gallery-empty-icon">▶</div>
      <h2>Пока нет видео для просмотра</h2>
      <p>Добавь видео в сетке — они появятся здесь, в вертикальной ленте.</p>
    </div>

    <p v-if="currentItem" class="tiktok-help">Колёсико мыши или ↑ / ↓ — переключить видео · Громкость и полноэкранный режим — справа от ролика</p>
  </section>
</template>
