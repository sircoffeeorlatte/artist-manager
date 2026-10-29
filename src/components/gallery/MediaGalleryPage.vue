<script setup lang="ts">
import { computed, onMounted, onBeforeUnmount, ref, watch } from 'vue'
import { convertFileSrc, isTauri } from '@tauri-apps/api/core'
import PageIntro from '../common/PageIntro.vue'
import MediaViewer from './MediaViewer.vue'
import MediaGridView from './MediaGridView.vue'
import TikTokGalleryView from './TikTokGalleryView.vue'
import { getSavedLibraryRoot, loadLibraryRoot } from '../../services/librarySettings'
import { mediaRepository } from '../../services/mediaRepository'
import { CATEGORY_LABELS, type ContentCategory, type LibraryMediaItem, type MediaKind } from '../../types/media'
import { beginActivity, errorText, finishActivity } from '../../services/appState'
import { getUserConfig, setGridColumns, updateUserConfig } from '../../services/userConfig'

const props = defineProps<{
  kind: 'photo' | 'video'
  title: string
  description: string
}>()

const items = ref<LibraryMediaItem[]>([])
const savedConfig = getUserConfig()
const columns = ref(savedConfig.gridColumns[props.kind])
const galleryMode = ref<'grid' | 'tiktok'>(props.kind === 'video' ? savedConfig.videoGalleryMode : 'grid')
const gridSizes = [2, 3, 4, 5, 6, 7]
const categoryFilter = ref<'all' | ContentCategory>('all')
const loading = ref(false)
const message = ref('')
const errorMessage = ref('')
const viewerIndex = ref<number | null>(null)
const rootPath = ref(getSavedLibraryRoot())
const previewUrls = new Set<string>()

const filteredItems = computed(() => items.value.filter((item) => categoryFilter.value === 'all' || item.category === categoryFilter.value))
const isDesktop = computed(() => isTauri())

function sourceFor(item: LibraryMediaItem) {
  if (item.previewOnly || item.absolutePath.startsWith('blob:')) return item.absolutePath
  return isTauri() ? convertFileSrc(item.absolutePath) : ''
}
function clearMessages() { message.value = ''; errorMessage.value = '' }

async function reloadItems(forceRefresh = false) {
  const activityId = beginActivity({
    kind: 'gallery',
    title: `Обновление: ${props.title}`,
    details: 'Чтение настроек и списка файлов',
  })
  loading.value = true
  clearMessages()
  try {
    rootPath.value = await loadLibraryRoot()
    if (!isTauri() || !rootPath.value) {
      items.value = items.value.filter((item) => item.previewOnly)
      finishActivity(activityId, 'success', !isTauri() ? 'Браузерный режим: показываются только временные превью.' : 'Папка медиатеки не выбрана.')
      return
    }
    const result = await mediaRepository.listByKind(rootPath.value, props.kind, forceRefresh)
    items.value = result
    finishActivity(activityId, 'success', `Найдено файлов: ${result.length}.`)
  } catch (error) {
    const message = errorText(error)
    errorMessage.value = message
    finishActivity(activityId, 'error', message)
  } finally {
    loading.value = false
  }
}

function addBrowserPreview(files: FileList | File[] | null, category: ContentCategory = 'irl') {
  if (!files?.length) return
  const accepted = Array.from(files).filter((file) => {
    const name = file.name.toLowerCase()
    const photoExt = /\.(jpe?g|png|webp|bmp|tiff?|avif|heic)$/i.test(name)
    const videoExt = /\.(mp4|mkv|avi|mov|webm|m4v|mpg|mpeg|wmv|flv|3gp|ogv|gif)$/i.test(name)
    return props.kind === 'photo' ? photoExt : videoExt
  })
  const previewItems = accepted.map((file, index) => {
    const url = URL.createObjectURL(file)
    previewUrls.add(url)
    const extension = file.name.includes('.') ? `.${file.name.split('.').pop()!.toLowerCase()}` : ''
    return {
      id: `preview-${Date.now()}-${index}-${file.name}`,
      kind: props.kind,
      originalName: file.name,
      relativePath: `Временное превью/${file.name}`,
      absolutePath: url,
      extension,
      sizeBytes: file.size,
      createdAt: file.lastModified || null,
      modifiedAt: file.lastModified || null,
      importedAt: new Date().toISOString(),
      category,
      tags: [],
      creatorTagId: null,
      isDemo: true,
      previewOnly: true,
    } satisfies LibraryMediaItem
  })
  items.value = [...previewItems, ...items.value]
  message.value = `${previewItems.length} файл(ов) добавлено только для браузерного превью. На диск они не копировались.`
  if (accepted.length !== files.length) errorMessage.value = 'Часть файлов пропущена: формат не подходит для этой галереи.'
}

function openViewer(index: number) {
  document.querySelectorAll<HTMLVideoElement>('.media-card video').forEach((video) => video.pause())
  viewerIndex.value = index
}
function closeViewer() { viewerIndex.value = null }

function updateCategoryFilter(event: Event) {
  categoryFilter.value = (event.target as HTMLSelectElement).value as 'all' | ContentCategory
}

function onBrowserImport(event: Event) {
  const detail = (event as CustomEvent<{ kind: MediaKind; files: File[]; category: ContentCategory }>).detail
  if (!detail || detail.kind !== props.kind) return
  addBrowserPreview(detail.files, detail.category)
}
function onImportComplete() { void reloadItems() }

const TIKTOK_LAYOUT_CLASS = 'media-library-tiktok-layout'
function updateTikTokViewportSize() {
  if (typeof window === 'undefined' || typeof document === 'undefined') return
  const root = document.documentElement
  const cssScale = Number.parseFloat(getComputedStyle(root).getPropertyValue('--ui-scale'))
  const configScale = getUserConfig().appearance.scale / 100
  const scale = Number.isFinite(cssScale) && cssScale > 0 ? cssScale : configScale || 1
  // #app itself is zoomed for the user's global scale. Measure its CSS size in
  // inverse proportion so the scaled TikTok layout still fits the actual viewport.
  root.style.setProperty('--tiktok-layout-width', `${Math.max(320, window.innerWidth / scale)}px`)
  root.style.setProperty('--tiktok-layout-height', `${Math.max(320, window.innerHeight / scale)}px`)
}
function syncTikTokLayout() {
  if (typeof document === 'undefined') return
  const active = props.kind === 'video' && galleryMode.value === 'tiktok'
  const root = document.documentElement
  root.classList.toggle(TIKTOK_LAYOUT_CLASS, active)
  if (active) updateTikTokViewportSize()
  else {
    root.style.removeProperty('--tiktok-layout-width')
    root.style.removeProperty('--tiktok-layout-height')
  }
}
function onTikTokWindowResize() {
  if (props.kind === 'video' && galleryMode.value === 'tiktok') updateTikTokViewportSize()
}

watch(columns, (value) => setGridColumns(props.kind, value))
watch(galleryMode, (value) => {
  syncTikTokLayout()
  if (props.kind === 'video') updateUserConfig({ videoGalleryMode: value })
})
watch(() => props.kind, (kind) => {
  columns.value = getUserConfig().gridColumns[kind]
  galleryMode.value = kind === 'video' ? getUserConfig().videoGalleryMode : 'grid'
  syncTikTokLayout()
  void reloadItems()
})
onMounted(() => {
  syncTikTokLayout()
  void reloadItems()
  window.addEventListener('media-library:browser-import', onBrowserImport)
  window.addEventListener('media-library:import-complete', onImportComplete)
  window.addEventListener('resize', onTikTokWindowResize)
})
onBeforeUnmount(() => {
  document.documentElement.classList.remove(TIKTOK_LAYOUT_CLASS)
  document.documentElement.style.removeProperty('--tiktok-layout-width')
  document.documentElement.style.removeProperty('--tiktok-layout-height')
  window.removeEventListener('media-library:browser-import', onBrowserImport)
  window.removeEventListener('media-library:import-complete', onImportComplete)
  window.removeEventListener('resize', onTikTokWindowResize)
  for (const url of previewUrls) URL.revokeObjectURL(url)
})
</script>

<template>
  <div class="gallery-page" :class="{ 'is-tiktok-mode': props.kind === 'video' && galleryMode === 'tiktok' }">
    <PageIntro
      v-if="galleryMode === 'grid'"
      :title="title"
      :description="description"
      :eyebrow="props.kind === 'photo' ? 'LIBRARY / IMAGES' : 'LIBRARY / VIDEO'"
      :icon="props.kind === 'photo' ? '▧' : '▶'"
    />


    <div v-if="galleryMode === 'grid' && message" class="notice notice-success" style="margin-top: 12px">{{ message }}</div>
    <div v-if="galleryMode === 'grid' && errorMessage" class="notice notice-error" style="margin-top: 12px; white-space: pre-line">{{ errorMessage }}</div>

    <div class="gallery-toolbar">
      <div class="gallery-toolbar-left">
        <span class="gallery-toolbar-count">{{ loading ? 'Сканирование…' : `${filteredItems.length} файлов` }}</span>
        <span class="gallery-toolbar-hint">{{ categoryFilter === 'all' ? 'Все категории' : CATEGORY_LABELS[categoryFilter] }}</span>
      </div>
      <div class="gallery-toolbar-actions">
        <label class="gallery-category-filter-wrap">
          <span class="sr-only">Фильтр по категории</span>
          <select class="field-control gallery-category-filter" :value="categoryFilter" aria-label="Фильтр по категории" @change="updateCategoryFilter">
            <option value="all">Все категории</option>
            <option value="irl">IRL</option>
            <option value="anime">Аниме</option>
            <option value="3d">3D</option>
          </select>
        </label>
        <div v-if="props.kind === 'video'" class="gallery-mode-switch" role="group" aria-label="Режим галереи">
          <button type="button" class="gallery-mode-button" :class="{ 'is-active': galleryMode === 'grid' }" :aria-pressed="galleryMode === 'grid'" @click="galleryMode = 'grid'">Сетка</button>
          <button type="button" class="gallery-mode-button" :class="{ 'is-active': galleryMode === 'tiktok' }" :aria-pressed="galleryMode === 'tiktok'" @click="galleryMode = 'tiktok'">TikTok</button>
        </div>
        <div v-if="galleryMode === 'grid'" class="gallery-density" role="group" aria-label="Размер сетки">
          <button
            v-for="size in gridSizes"
            :key="size"
            type="button"
            class="gallery-density-button"
            :class="{ 'is-active': columns === size }"
            :aria-pressed="columns === size"
            :aria-label="`Сетка ${size} на ${size}`"
            :title="`Сетка ${size} × ${size}`"
            @click="columns = size"
          >{{ size }}×{{ size }}</button>
        </div>
        <button type="button" class="btn btn-secondary gallery-refresh-button" :disabled="loading" aria-label="Переиндексировать файлы в SQLite и обновить галерею" title="Переиндексировать файлы в SQLite и перечитать каталог" @click="reloadItems(true)"><span aria-hidden="true">↻</span></button>
      </div>
    </div>

    <TikTokGalleryView
      v-if="props.kind === 'video'"
      v-show="galleryMode === 'tiktok'"
      :items="filteredItems"
      :source-for="sourceFor"
      :category-labels="CATEGORY_LABELS"
      :active="galleryMode === 'tiktok' && viewerIndex === null"
      @open="openViewer"
    />

    <MediaGridView
      v-if="galleryMode === 'grid' && filteredItems.length"
      :items="filteredItems"
      :kind="props.kind"
      :columns="columns"
      :source-for="sourceFor"
      :title="title"
      @open="openViewer"
    />

    <section v-if="galleryMode === 'grid' && !filteredItems.length" class="gallery-empty">
      <div class="gallery-empty-icon">{{ props.kind === 'photo' ? '▧' : '▶' }}</div>
      <h2>{{ loading ? 'Сканируем медиатеку…' : (categoryFilter === 'all' ? 'Здесь пока пусто' : `В категории «${CATEGORY_LABELS[categoryFilter]}» пока нет файлов`) }}</h2>
      <p>{{ isDesktop ? (rootPath ? 'Нажми «Импорт» в верхней панели, чтобы добавить файлы в выбранную категорию.' : 'Сначала зайди в настройки, выбери диск и создай корневую папку медиатеки.') : 'Нажми «Импорт» в верхней панели для временного предпросмотра; файлы не будут скопированы на диск.' }}</p>
    </section>

    <MediaViewer v-if="viewerIndex !== null" :items="filteredItems" :initial-index="viewerIndex ?? 0" :media-kind="props.kind" @close="closeViewer" />
  </div>
</template>
