<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { convertFileSrc, isTauri } from '@tauri-apps/api/core'
import { openPath } from '@tauri-apps/plugin-opener'
import type { LibraryMediaItem } from '../../types/media'
import { CATEGORY_LABELS } from '../../types/media'
import { formatBytes, formatDate } from '../../services/formatters'
import LibraryVideoPlayer from './LibraryVideoPlayer.vue'

const props = defineProps<{
  items: LibraryMediaItem[]
  initialIndex: number
  mediaKind: 'photo' | 'video'
}>()
const emit = defineEmits<{ close: [] }>()

const currentIndex = ref(props.initialIndex)
const zoom = ref(1)
const showInfo = ref(true)
const volume = ref(1)
const muted = ref(false)
const playbackError = ref(false)
const externalOpenError = ref('')
const current = computed(() => props.items[currentIndex.value])
const isGif = computed(() => current.value?.extension.toLowerCase() === '.gif')

watch(() => props.initialIndex, (index) => { currentIndex.value = index; zoom.value = 1 })
watch(currentIndex, () => { zoom.value = 1; playbackError.value = false; externalOpenError.value = '' })
function sourceFor(item: LibraryMediaItem): string {
  if (item.previewOnly || item.absolutePath.startsWith('blob:')) return item.absolutePath
  return isTauri() ? convertFileSrc(item.absolutePath) : ''
}
function next(step: number) {
  if (!props.items.length) return
  currentIndex.value = (currentIndex.value + step + props.items.length) % props.items.length
}
function handleVideoError() { playbackError.value = true }
function updatePlayerVolume(value: number) { volume.value = value }
function updatePlayerMuted(value: boolean) { muted.value = value }
async function openExternally() {
  if (!current.value || !isTauri()) return
  externalOpenError.value = ''
  try { await openPath(current.value.absolutePath) }
  catch (error) { externalOpenError.value = error instanceof Error ? error.message : String(error) }
}
function onWheel(event: WheelEvent) {
  if (props.mediaKind !== 'photo') return
  zoom.value = Math.min(5, Math.max(0.25, zoom.value + (event.deltaY < 0 ? 0.1 : -0.1)))
}
function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') emit('close')
  if (event.key === 'ArrowLeft') next(-1)
  if (event.key === 'ArrowRight') next(1)
}
onMounted(() => window.addEventListener('keydown', onKeydown))
onBeforeUnmount(() => window.removeEventListener('keydown', onKeydown))
</script>

<template>
  <Teleport to="body">
    <div v-if="current" class="fixed inset-0 z-50 flex flex-col bg-slate-950/95 text-white" @click.self="emit('close')">
      <header class="flex min-h-16 items-center justify-between gap-3 border-b border-white/10 px-4 py-3 md:px-6">
        <div class="min-w-0">
          <p class="truncate text-sm font-semibold">{{ current.originalName }}</p>
          <p class="mt-0.5 text-xs text-slate-400">{{ currentIndex + 1 }} / {{ items.length }}</p>
        </div>
        <div class="flex shrink-0 items-center gap-2">
          <button v-if="mediaKind === 'photo'" type="button" class="rounded-lg bg-white/10 px-3 py-2 text-sm hover:bg-white/20" @click="zoom = 1">Масштаб {{ Math.round(zoom * 100) }}%</button>
          <button type="button" class="rounded-lg bg-white/10 px-3 py-2 text-sm hover:bg-white/20" @click="showInfo = !showInfo">{{ showInfo ? 'Скрыть информацию' : 'Информация' }}</button>
          <button type="button" aria-label="Закрыть просмотрщик" class="rounded-lg bg-white/10 px-3 py-2 text-sm hover:bg-white/20" @click="emit('close')">✕</button>
        </div>
      </header>

      <div class="relative flex min-h-0 flex-1 items-stretch">
        <button type="button" aria-label="Предыдущий файл" class="w-12 shrink-0 text-3xl text-slate-300 hover:bg-white/5 md:w-16" @click="next(-1)">‹</button>
        <div class="flex min-w-0 flex-1 items-center justify-center overflow-hidden py-3" @wheel.prevent="onWheel">
          <img
            v-if="mediaKind === 'photo' || isGif"
            :src="sourceFor(current)"
            :alt="current.originalName"
            class="max-h-full max-w-full object-contain transition-transform duration-100"
            :style="{ transform: `scale(${zoom})` }"
            draggable="false"
          />
          <div v-if="mediaKind === 'video' && !isGif && playbackError" class="max-w-md rounded-xl border border-white/15 bg-slate-900 p-5 text-center">
            <p class="font-semibold">Не удалось воспроизвести файл встроенно</p>
            <p class="mt-2 text-sm leading-6 text-slate-400">Формат или кодек может не поддерживаться WebView2. Можно попробовать открыть файл во внешнем плеере Windows.</p>
            <button v-if="isTauri()" type="button" class="mt-4 rounded-lg bg-white px-4 py-2 text-sm font-semibold text-slate-900 hover:bg-slate-200" @click="openExternally">Открыть внешним плеером</button>
            <p v-if="externalOpenError" class="mt-2 break-words text-xs text-rose-300">{{ externalOpenError }}</p>
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
        <button type="button" aria-label="Следующий файл" class="w-12 shrink-0 text-3xl text-slate-300 hover:bg-white/5 md:w-16" @click="next(1)">›</button>

        <aside v-if="showInfo" class="absolute bottom-3 left-3 right-3 z-10 max-h-48 overflow-y-auto rounded-xl border border-white/10 bg-slate-900/95 p-4 lg:static lg:z-auto lg:max-h-none lg:w-72 lg:shrink-0 lg:rounded-none lg:border-l lg:bg-slate-900/70">
          <h2 class="text-sm font-semibold">Информация о файле</h2>
          <dl class="mt-4 space-y-3 text-sm">
            <div><dt class="text-xs text-slate-400">Имя</dt><dd class="mt-1 break-words">{{ current.originalName }}</dd></div>
            <div><dt class="text-xs text-slate-400">Путь в медиатеке</dt><dd class="mt-1 break-all text-slate-300">{{ current.relativePath }}</dd></div>
            <div><dt class="text-xs text-slate-400">Размер</dt><dd class="mt-1">{{ formatBytes(current.sizeBytes) }}</dd></div>
            <div><dt class="text-xs text-slate-400">Дата изменения</dt><dd class="mt-1">{{ formatDate(current.modifiedAt) }}</dd></div>
            <div><dt class="text-xs text-slate-400">Категория</dt><dd class="mt-1">{{ current.category ? CATEGORY_LABELS[current.category] : 'Не указана' }}</dd></div>
            <div><dt class="text-xs text-slate-400">Теги</dt><dd class="mt-1 flex flex-wrap gap-1"><span v-if="!current.tags.length" class="text-slate-500">Теги пока не назначены</span><span v-for="tag in current.tags" :key="tag" class="rounded bg-white/10 px-2 py-1 text-xs">{{ tag }}</span></dd></div>
          </dl>
          <div v-if="mediaKind === 'video' && !isGif" class="mt-5 border-t border-white/10 pt-4">
            <div class="flex items-center justify-between text-xs"><label for="viewer-volume">Громкость</label><span>{{ muted ? 'Без звука' : `${Math.round(volume * 100)}%` }}</span></div>
            <div class="mt-2 flex items-center gap-2">
              <button type="button" class="rounded bg-white/10 px-2 py-1.5 text-xs" @click="muted = !muted">{{ muted ? 'Включить звук' : 'Заглушить' }}</button>
              <input id="viewer-volume" v-model.number="volume" class="w-full" type="range" min="0" max="1" step="0.05" aria-label="Громкость видео" />
            </div>
          </div>
          <p class="mt-5 border-t border-white/10 pt-3 text-xs leading-5 text-slate-500">Автор/создатель будет оформлен тегом в будущей версии.</p>
        </aside>
      </div>
      <div class="flex items-center justify-center gap-3 border-t border-white/10 px-4 py-3 text-xs text-slate-400">
        <span>← / → — предыдущий или следующий файл</span><span>·</span><span>Esc — закрыть</span><span v-if="mediaKind === 'photo'">· Колесо мыши — масштаб</span>
      </div>
    </div>
  </Teleport>
</template>
