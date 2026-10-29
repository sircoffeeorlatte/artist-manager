<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { convertFileSrc, isTauri } from '@tauri-apps/api/core'
import { openPath } from '@tauri-apps/plugin-opener'
import type { LibraryMediaItem } from '../../types/media'
import { CATEGORY_LABELS } from '../../types/media'
import { formatBytes, formatDate } from '../../services/formatters'
import LibraryVideoPlayer from './LibraryVideoPlayer.vue'
import { notify } from '../../services/appState'
import { getUserConfig, updateUserConfig } from '../../services/userConfig'

const props = defineProps<{
  items: LibraryMediaItem[]
  initialIndex: number
  mediaKind: 'photo' | 'video'
}>()
const emit = defineEmits<{ close: [] }>()

const currentIndex = ref(props.initialIndex)
const zoom = ref(1)
const pan = ref({ x: 0, y: 0 })
const isPanning = ref(false)
const showInfo = ref(true)
const volume = ref(getUserConfig().video.volume)
const muted = ref(getUserConfig().video.muted)
const playbackError = ref(false)
const imageLoadError = ref(false)
const externalOpenError = ref('')
const viewerImage = ref<HTMLImageElement | null>(null)
const current = computed(() => props.items[currentIndex.value])
const isGif = computed(() => current.value?.extension.toLowerCase() === '.gif')
const canPan = computed(() => props.mediaKind === 'photo' && zoom.value > 1.001 && !isGif.value)

watch(() => props.initialIndex, (index) => {
  currentIndex.value = index
  resetView()
})
watch(currentIndex, () => {
  resetView()
  playbackError.value = false
  imageLoadError.value = false
  externalOpenError.value = ''
})

function resetView() {
  zoom.value = 1
  pan.value = { x: 0, y: 0 }
  isPanning.value = false
}

function sourceFor(item: LibraryMediaItem): string {
  if (item.previewOnly || item.absolutePath.startsWith('blob:')) return item.absolutePath
  return isTauri() ? convertFileSrc(item.absolutePath) : ''
}
function next(step: number) {
  if (!props.items.length) return
  currentIndex.value = (currentIndex.value + step + props.items.length) % props.items.length
}
function handleVideoError() {
  playbackError.value = true
  if (current.value) notify('error', 'Не удалось воспроизвести видео', `${current.value.originalName}: формат может не поддерживаться или файл больше недоступен. Путь: ${current.value.absolutePath}`)
}
function handleImageError() {
  if (imageLoadError.value || !current.value) return
  imageLoadError.value = true
  notify('error', 'Не удалось загрузить изображение', `${current.value.originalName}: файл мог быть удалён, диск отключён или путь стал недоступен. Путь: ${current.value.absolutePath}`)
}
function updatePlayerVolume(value: number) {
  volume.value = Math.max(0, Math.min(1, value))
  updateUserConfig({ video: { ...getUserConfig().video, volume: volume.value } })
}
function updatePlayerMuted(value: boolean) {
  muted.value = value
  updateUserConfig({ video: { ...getUserConfig().video, muted: value } })
}
function onVolumeInput(event: Event) {
  const target = event.target
  if (target instanceof HTMLInputElement) updatePlayerVolume(Number(target.value))
}
async function openExternally() {
  if (!current.value || !isTauri()) return
  externalOpenError.value = ''
  try { await openPath(current.value.absolutePath) }
  catch (error) {
    externalOpenError.value = error instanceof Error ? error.message : String(error)
    notify('error', 'Не удалось открыть файл внешним плеером', externalOpenError.value)
  }
}
function changeZoom(delta: number) {
  zoom.value = Math.min(5, Math.max(0.25, Math.round((zoom.value + delta) * 100) / 100))
  if (zoom.value <= 1) pan.value = { x: 0, y: 0 }
  else void nextTick(clampPan)
}
function onWheel(event: WheelEvent) {
  if (props.mediaKind !== 'photo') return
  changeZoom(event.deltaY < 0 ? 0.1 : -0.1)
}
function clampPan() {
  const image = viewerImage.value
  if (!image || zoom.value <= 1) {
    pan.value = { x: 0, y: 0 }
    return
  }
  const maxX = Math.max(0, image.clientWidth * (zoom.value - 1) / 2)
  const maxY = Math.max(0, image.clientHeight * (zoom.value - 1) / 2)
  pan.value = {
    x: Math.max(-maxX, Math.min(maxX, pan.value.x)),
    y: Math.max(-maxY, Math.min(maxY, pan.value.y)),
  }
}
let dragOrigin = { x: 0, y: 0, panX: 0, panY: 0 }
function startPan(event: PointerEvent) {
  if (!canPan.value || event.button !== 0) return
  event.preventDefault()
  isPanning.value = true
  dragOrigin = { x: event.clientX, y: event.clientY, panX: pan.value.x, panY: pan.value.y }
  const image = event.currentTarget
  if (image instanceof HTMLImageElement) image.setPointerCapture(event.pointerId)
}
function movePan(event: PointerEvent) {
  if (!isPanning.value) return
  pan.value = {
    x: dragOrigin.panX + event.clientX - dragOrigin.x,
    y: dragOrigin.panY + event.clientY - dragOrigin.y,
  }
  clampPan()
}
function endPan(event: PointerEvent) {
  isPanning.value = false
  const image = event.currentTarget
  if (image instanceof HTMLImageElement && image.hasPointerCapture(event.pointerId)) {
    image.releasePointerCapture(event.pointerId)
  }
}
function onKeydown(event: KeyboardEvent) {
  const target = event.target
  if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement || (target instanceof HTMLElement && target.isContentEditable)) return
  if (event.key === 'Escape') emit('close')
  if (event.key === 'ArrowLeft') next(-1)
  if (event.key === 'ArrowRight') next(1)
}
onMounted(() => window.addEventListener('keydown', onKeydown))
onBeforeUnmount(() => window.removeEventListener('keydown', onKeydown))
</script>

<template>
  <Teleport to="body">
    <div v-if="current" class="media-viewer-overlay" @click.self="emit('close')">
      <header class="media-viewer-header">
        <div class="media-viewer-heading">
          <div class="media-viewer-type-icon" :class="`is-${mediaKind}`" aria-hidden="true">
            <svg v-if="mediaKind === 'photo'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="4"/><circle cx="8.5" cy="8.5" r="1.5"/><path d="m21 15-5-5L5 21"/></svg>
            <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="4" width="18" height="16" rx="4"/><path d="m10 8 6 4-6 4z"/></svg>
          </div>
          <div class="media-viewer-title-copy">
            <p class="media-viewer-file-title" :title="current.originalName">{{ current.originalName }}</p>
            <div class="media-viewer-meta">
              <span class="media-viewer-kind-chip">{{ mediaKind === 'photo' ? 'ФОТО' : 'ВИДЕО' }}</span>
              <span>{{ currentIndex + 1 }} / {{ items.length }}</span>
              <span v-if="current.category">{{ CATEGORY_LABELS[current.category] }}</span>
            </div>
          </div>
        </div>

        <div class="media-viewer-controls">
          <div v-if="mediaKind === 'photo'" class="media-viewer-zoom-controls" aria-label="Управление масштабом фото">
            <button type="button" class="media-viewer-tool-button" :disabled="zoom <= 0.25" aria-label="Уменьшить фото" title="Уменьшить" @click="changeZoom(-0.25)">−</button>
            <button type="button" class="media-viewer-zoom-value" title="Сбросить масштаб" @click="resetView">{{ Math.round(zoom * 100) }}%</button>
            <button type="button" class="media-viewer-tool-button" :disabled="zoom >= 5" aria-label="Увеличить фото" title="Увеличить" @click="changeZoom(0.25)">+</button>
          </div>
          <button type="button" class="media-viewer-tool-button media-viewer-info-toggle" :aria-pressed="showInfo" @click="showInfo = !showInfo">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"><circle cx="12" cy="12" r="9"/><path d="M12 11v5m0-8h.01"/></svg>
            <span>{{ showInfo ? 'Скрыть данные' : 'Данные файла' }}</span>
          </button>
          <button type="button" aria-label="Закрыть просмотрщик" title="Закрыть (Esc)" class="media-viewer-close" @click="emit('close')">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="m6 6 12 12M18 6 6 18"/></svg>
          </button>
        </div>
      </header>

      <div class="media-viewer-main">
        <button type="button" aria-label="Предыдущий файл" title="Предыдущий файл (←)" class="media-viewer-nav" @click="next(-1)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m15 18-6-6 6-6"/></svg>
        </button>

        <div class="media-viewer-canvas" :class="{ 'is-photo': mediaKind === 'photo', 'is-zoomed': canPan, 'is-panning': isPanning }" @wheel.prevent="onWheel">
          <img
            v-if="(mediaKind === 'photo' || isGif) && !imageLoadError"
            :key="current.id"
            ref="viewerImage"
            :src="sourceFor(current)"
            :alt="current.originalName"
            class="media-viewer-image"
            :class="{ 'is-grabbable': canPan, 'is-grabbing': isPanning }"
            :style="{ transform: `translate3d(${pan.x}px, ${pan.y}px, 0) scale(${zoom})` }"
            draggable="false"
            @pointerdown="startPan"
            @pointermove="movePan"
            @pointerup="endPan"
            @pointercancel="endPan"
            @lostpointercapture="endPan"
            @error="handleImageError"
          />

          <div v-else-if="(mediaKind === 'photo' || isGif) && imageLoadError" class="media-viewer-error-card">
            <div class="media-viewer-error-icon">!</div>
            <h2>Не удалось открыть изображение</h2>
            <p>Файл мог быть удалён, диск отключён или путь стал недоступен.</p>
            <p class="selectable-path media-viewer-error-path">{{ current.absolutePath }}</p>
          </div>

          <div v-else-if="mediaKind === 'video' && !isGif && playbackError" class="media-viewer-error-card">
            <div class="media-viewer-error-icon">!</div>
            <h2>Не удалось воспроизвести видео</h2>
            <p>Формат или кодек может не поддерживаться WebView2. Попробуй открыть файл во внешнем плеере.</p>
            <button v-if="isTauri()" type="button" class="btn btn-primary" @click="openExternally">Открыть внешним плеером</button>
            <p v-if="externalOpenError" class="media-viewer-error-path">{{ externalOpenError }}</p>
            <p class="selectable-path media-viewer-error-path">{{ current.absolutePath }}</p>
          </div>

          <LibraryVideoPlayer
            v-else-if="mediaKind === 'video' && !isGif"
            :key="current.id"
            :src="sourceFor(current)"
            :volume="volume"
            :muted="muted"
            autoplay
            @error="handleVideoError"
            @volume-change="updatePlayerVolume"
            @muted-change="updatePlayerMuted"
          />
        </div>

        <button type="button" aria-label="Следующий файл" title="Следующий файл (→)" class="media-viewer-nav" @click="next(1)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m9 18 6-6-6-6"/></svg>
        </button>

        <aside v-if="showInfo" class="media-viewer-info-panel">
          <div class="media-viewer-info-heading">
            <div><span class="page-eyebrow">ИНФОРМАЦИЯ</span><h2>О файле</h2></div>
            <span class="media-viewer-info-index">{{ currentIndex + 1 }} / {{ items.length }}</span>
          </div>
          <dl class="media-viewer-details">
            <div><dt>Имя файла</dt><dd>{{ current.originalName }}</dd></div>
            <div><dt>Путь в медиатеке</dt><dd class="selectable-path">{{ current.relativePath }}</dd></div>
            <div><dt>Полный путь</dt><dd class="selectable-path">{{ current.absolutePath }}</dd></div>
            <div><dt>Размер</dt><dd>{{ formatBytes(current.sizeBytes) }}</dd></div>
            <div><dt>Дата изменения</dt><dd>{{ formatDate(current.modifiedAt) }}</dd></div>
            <div><dt>Категория</dt><dd>{{ current.category ? CATEGORY_LABELS[current.category] : 'Не указана' }}</dd></div>
            <div><dt>Теги</dt><dd class="media-viewer-tags"><span v-if="!current.tags.length" class="dim">Теги пока не назначены</span><span v-for="tag in current.tags" :key="tag">{{ tag }}</span></dd></div>
          </dl>
          <div v-if="mediaKind === 'video' && !isGif" class="media-viewer-volume-panel">
            <div class="media-viewer-volume-heading"><span>Громкость</span><strong>{{ muted ? 'Выкл.' : `${Math.round(volume * 100)}%` }}</strong></div>
            <div class="media-viewer-volume-controls">
              <button type="button" class="media-viewer-tool-button" :aria-label="muted ? 'Включить звук' : 'Выключить звук'" @click="updatePlayerMuted(!muted)">
                <svg v-if="muted" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M11 5 6 9H3v6h3l5 4zM16 9l5 6m0-6-5 6"/></svg>
                <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M11 5 6 9H3v6h3l5 4zM15 9a5 5 0 0 1 0 6m3-9a9 9 0 0 1 0 12"/></svg>
              </button>
              <input id="viewer-volume" :value="volume" @input="onVolumeInput" class="media-viewer-volume-slider" type="range" min="0" max="1" step="0.05" aria-label="Громкость видео" />
            </div>
          </div>
          <p class="media-viewer-info-note">Совет: зажми и перетаскивай увеличенное фото, чтобы осмотреть детали.</p>
        </aside>
      </div>

      <footer class="media-viewer-footer">
        <span><kbd>←</kbd> <kbd>→</kbd> Переключение файлов</span>
        <span v-if="mediaKind === 'photo'"><kbd>Колесо</kbd> Масштаб <span class="media-viewer-footer-dot">·</span> <kbd>Grab</kbd> Перемещение</span>
        <span><kbd>Esc</kbd> Закрыть</span>
      </footer>
    </div>
  </Teleport>
</template>
