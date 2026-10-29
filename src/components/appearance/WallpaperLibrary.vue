<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { convertFileSrc, isTauri } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { errorText, notify } from '../../services/appState'
import { importWallpaper, listWallpapers, type WallpaperItem } from '../../services/wallpapers'

const props = defineProps<{
  modelValue: string
  rootPath: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const items = ref<WallpaperItem[]>([])
const busy = ref(false)
const loading = ref(false)
const errorMessage = ref('')
const successMessage = ref('')
const isDesktop = computed(() => isTauri())
const hasRoot = computed(() => Boolean(props.rootPath.trim()))

function sourceFor(item: WallpaperItem): string {
  return isDesktop.value ? convertFileSrc(item.absolutePath) : ''
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} Б`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} КБ`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} МБ`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} ГБ`
}

async function refresh(showError = true): Promise<void> {
  errorMessage.value = ''
  if (!isDesktop.value || !hasRoot.value) {
    items.value = []
    return
  }
  loading.value = true
  try {
    items.value = await listWallpapers(props.rootPath)
  } catch (error) {
    if (showError) errorMessage.value = errorText(error)
    items.value = []
  } finally {
    loading.value = false
  }
}

async function addWallpaper(): Promise<void> {
  if (!hasRoot.value || busy.value) return
  errorMessage.value = ''
  successMessage.value = ''
  try {
    const selected = await open({
      multiple: false,
      title: 'Выбери изображение или видео для фона',
      filters: [{
        name: 'Изображения и видео',
        extensions: ['jpg', 'jpeg', 'png', 'webp', 'bmp', 'avif', 'gif', 'mp4', 'mkv', 'avi', 'mov', 'webm', 'm4v', 'mpg', 'mpeg', 'wmv', 'flv', '3gp', 'ogv'],
      }],
    })
    if (typeof selected !== 'string') return

    busy.value = true
    const imported = await importWallpaper(selected, props.rootPath)
    await refresh()
    emit('update:modelValue', imported.relativePath)
    successMessage.value = `Добавлено в wallpapers: ${imported.name}`
  } catch (error) {
    errorMessage.value = errorText(error)
    notify('error', 'Не удалось добавить обои', errorMessage.value)
  } finally {
    busy.value = false
  }
}

function selectWallpaper(item: WallpaperItem): void {
  emit('update:modelValue', item.relativePath)
  successMessage.value = `Выбраны обои: ${item.name}`
  errorMessage.value = ''
}

function useColorBackground(): void {
  emit('update:modelValue', '')
  successMessage.value = 'Используется цвет фона из настроек.'
  errorMessage.value = ''
}

function playPreview(event: MouseEvent): void {
  const video = (event.currentTarget as HTMLElement).querySelector('video')
  if (!video) return
  video.currentTime = 0
  void video.play().catch(() => {})
}

function pausePreview(event: MouseEvent): void {
  const video = (event.currentTarget as HTMLElement).querySelector('video')
  if (!video) return
  video.pause()
  try { video.currentTime = 0 } catch { /* WebView may not seek until metadata arrives. */ }
}

watch(() => props.rootPath, () => { void refresh() })
onMounted(() => { void refresh() })
</script>

<template>
  <section class="wallpaper-library" aria-labelledby="wallpaper-library-title">
    <div class="wallpaper-library-heading">
      <div>
        <h3 id="wallpaper-library-title" class="appearance-control-title">Фото- и видеообои</h3>
        <p class="appearance-control-hint">Обои копируются в папку <code>wallpapers</code> внутри медиатеки. Старые файлы сохраняются, а выбирать можно любой из списка.</p>
      </div>
      <div class="wallpaper-library-actions">
        <button type="button" class="btn btn-secondary" :class="{ 'is-selected': !modelValue }" @click="useColorBackground">Цвет фона</button>
        <button type="button" class="btn btn-primary" :disabled="!isDesktop || !hasRoot || busy" @click="addWallpaper">
          <span aria-hidden="true">{{ busy ? '…' : '+' }}</span>{{ busy ? 'Добавление…' : 'Добавить обои' }}
        </button>
      </div>
    </div>

    <div v-if="!isDesktop" class="notice notice-warning wallpaper-library-notice">Добавление обоев доступно в настольной версии Tauri.</div>
    <div v-else-if="!hasRoot" class="notice notice-warning wallpaper-library-notice">Сначала выбери и создай корневую папку медиатеки — внутри неё будет создан каталог wallpapers.</div>

    <div v-if="errorMessage" class="notice notice-error wallpaper-library-notice" role="alert">{{ errorMessage }}</div>
    <div v-else-if="successMessage" class="wallpaper-library-message" role="status">{{ successMessage }}</div>

    <div v-if="loading" class="wallpaper-library-empty">Читаем сохранённые обои…</div>
    <div v-else-if="!items.length && isDesktop && hasRoot" class="wallpaper-library-empty">
      <span class="wallpaper-library-empty-icon" aria-hidden="true">▧</span>
      <strong>Пока нет обоев</strong>
      <span>Добавь изображение или видео — оно останется в медиатеке и появится здесь.</span>
    </div>

    <div v-else-if="items.length" class="wallpaper-library-grid">
      <button
        v-for="item in items"
        :key="item.relativePath"
        type="button"
        class="wallpaper-library-card"
        :class="{ 'is-selected': modelValue === item.relativePath }"
        :aria-pressed="modelValue === item.relativePath"
        :title="`${item.name} · ${formatSize(item.sizeBytes)}`"
        @click="selectWallpaper(item)"
        @mouseenter="playPreview"
        @mouseleave="pausePreview"
      >
        <span class="wallpaper-library-preview">
          <video v-if="item.kind === 'video'" :src="sourceFor(item)" muted loop playsinline preload="metadata" />
          <img v-else :src="sourceFor(item)" :alt="item.name" loading="lazy" />
          <span class="wallpaper-library-kind">{{ item.kind === 'video' ? 'ВИДЕО' : 'ФОТО' }}</span>
          <span v-if="modelValue === item.relativePath" class="wallpaper-library-check" aria-label="Текущие обои">✓</span>
        </span>
        <span class="wallpaper-library-card-copy">
          <strong>{{ item.name }}</strong>
          <small>{{ formatSize(item.sizeBytes) }}</small>
        </span>
      </button>
    </div>
  </section>
</template>
