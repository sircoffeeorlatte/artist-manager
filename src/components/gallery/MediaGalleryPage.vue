<script setup lang="ts">
import { computed, onMounted, onBeforeUnmount, ref, watch } from 'vue'
import { convertFileSrc, isTauri } from '@tauri-apps/api/core'
import PageIntro from '../common/PageIntro.vue'
import MediaViewer from './MediaViewer.vue'
import GalleryImportPanel from './GalleryImportPanel.vue'
import MediaGridView from './MediaGridView.vue'
import TikTokGalleryView from './TikTokGalleryView.vue'
import { selectSourceFiles } from '../../services/nativeDialogs'
import { getSavedLibraryRoot, loadLibraryRoot } from '../../services/librarySettings'
import { importMediaFiles, listLibraryMedia } from '../../services/filesystem'
import { CATEGORY_LABELS, type ContentCategory, type ImportResult, type LibraryMediaItem } from '../../types/media'

const props = defineProps<{
  kind: 'photo' | 'video'
  title: string
  description: string
}>()

const items = ref<LibraryMediaItem[]>([])
const columns = ref(3)
const galleryMode = ref<'grid' | 'tiktok'>('grid')
const gridSizes = [2, 3, 4, 5, 6, 7]
const categoryFilter = ref<'all' | ContentCategory>('all')
const destinationCategory = ref<ContentCategory>('irl')
const importMode = ref<'copy' | 'move'>('copy')
const busy = ref(false)
const loading = ref(false)
const message = ref('')
const errorMessage = ref('')
const viewerIndex = ref<number | null>(null)
const browserFileInput = ref<HTMLInputElement | null>(null)
const rootPath = ref(getSavedLibraryRoot())
const previewUrls = new Set<string>()

const filteredItems = computed(() => items.value.filter((item) => categoryFilter.value === 'all' || item.category === categoryFilter.value))
const isDesktop = computed(() => isTauri())
const acceptedExtensions = computed(() => props.kind === 'photo' ? 'image/*' : 'video/*,image/gif')

function sourceFor(item: LibraryMediaItem) {
  if (item.previewOnly || item.absolutePath.startsWith('blob:')) return item.absolutePath
  return isTauri() ? convertFileSrc(item.absolutePath) : ''
}
function clearMessages() { message.value = ''; errorMessage.value = '' }

async function reloadItems() {
  rootPath.value = await loadLibraryRoot()
  if (!isTauri() || !rootPath.value) {
    items.value = items.value.filter((item) => item.previewOnly)
    return
  }
  loading.value = true
  clearMessages()
  try {
    items.value = await listLibraryMedia(rootPath.value, props.kind)
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  } finally {
    loading.value = false
  }
}

function onBrowserFilesChange(event: Event) {
  const input = event.target as HTMLInputElement
  addBrowserPreview(input.files)
  input.value = ''
}

function addBrowserPreview(files: FileList | null) {
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
      category: destinationCategory.value,
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

async function addFiles() {
  clearMessages()
  if (!isTauri()) {
    browserFileInput.value?.click()
    return
  }
  if (!rootPath.value) {
    errorMessage.value = 'Сначала откройте «Настройки» и создайте папку медиатеки.'
    return
  }
  if (importMode.value === 'move' && !window.confirm('Файлы будут скопированы в медиатеку и удалены из исходного места только после проверки копии. Продолжить?')) return
  busy.value = true
  try {
    const sourcePaths = await selectSourceFiles(props.kind)
    if (!sourcePaths.length) return
    const result: ImportResult = await importMediaFiles({
      sourcePaths,
      rootPath: rootPath.value,
      category: destinationCategory.value,
      kind: props.kind,
      mode: importMode.value,
    })
    await reloadItems()
    message.value = `Импортировано: ${result.imported.length}`
    if (result.warnings.length) errorMessage.value = result.warnings.join('\n')
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  } finally {
    busy.value = false
  }
}

function openViewer(index: number) {
  document.querySelectorAll<HTMLVideoElement>('.media-card video').forEach((video) => video.pause())
  viewerIndex.value = index
}
function closeViewer() { viewerIndex.value = null }

function onDrop(event: DragEvent) {
  event.preventDefault()
  if (!isDesktop.value && event.dataTransfer?.files.length) addBrowserPreview(event.dataTransfer.files)
}
function onDragOver(event: DragEvent) { event.preventDefault() }

watch(() => props.kind, () => { void reloadItems() })
onMounted(() => { void reloadItems() })
onBeforeUnmount(() => {
  for (const url of previewUrls) URL.revokeObjectURL(url)
})
</script>

<template>
  <div class="gallery-page" :class="{ 'is-tiktok-mode': props.kind === 'video' && galleryMode === 'tiktok' }" @drop="onDrop" @dragover="onDragOver">
    <PageIntro
      v-if="galleryMode === 'grid'"
      :title="title"
      :description="description"
      :eyebrow="props.kind === 'photo' ? 'LIBRARY / IMAGES' : 'LIBRARY / VIDEO'"
      :icon="props.kind === 'photo' ? '▧' : '▶'"
    />

    <GalleryImportPanel
      v-if="galleryMode === 'grid'"
      :kind="props.kind"
      :is-desktop="isDesktop"
      :destination-category="destinationCategory"
      :category-filter="categoryFilter"
      :import-mode="importMode"
      :busy="busy"
      @update:destination-category="destinationCategory = $event"
      @update:category-filter="categoryFilter = $event"
      @update:import-mode="importMode = $event"
      @add-files="addFiles"
    />
    <input ref="browserFileInput" class="hidden" type="file" multiple :accept="acceptedExtensions" @change="onBrowserFilesChange" />

    <div v-if="galleryMode === 'grid' && message" class="notice notice-success" style="margin-top: 12px">{{ message }}</div>
    <div v-if="galleryMode === 'grid' && errorMessage" class="notice notice-error" style="margin-top: 12px; white-space: pre-line">{{ errorMessage }}</div>

    <div class="gallery-toolbar">
      <div class="gallery-toolbar-left">
        <span class="gallery-toolbar-count">{{ loading ? 'Сканирование…' : `${filteredItems.length} файлов` }}</span>
        <span class="gallery-toolbar-hint">{{ categoryFilter === 'all' ? 'Все категории' : CATEGORY_LABELS[categoryFilter] }}</span>
      </div>
      <div class="gallery-toolbar-actions">
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
        <button type="button" class="btn btn-secondary gallery-refresh-button" :disabled="loading" aria-label="Обновить галерею" title="Обновить галерею" @click="reloadItems"><span aria-hidden="true">↻</span></button>
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
      <p>{{ isDesktop ? (rootPath ? 'Добавь файлы с помощью кнопки выше. Медиатека просканирует папки выбранного хранилища.' : 'Сначала зайди в настройки, выбери диск и создай корневую папку медиатеки.') : 'Выбери файлы для временного предпросмотра — они появятся в сетке, но не будут скопированы на диск.' }}</p>
      <button v-if="!isDesktop" type="button" class="btn btn-primary" style="margin-top:16px" @click="addFiles">Выбрать файлы</button>
    </section>

    <MediaViewer v-if="viewerIndex !== null" :items="filteredItems" :initial-index="viewerIndex ?? 0" :media-kind="props.kind" @close="closeViewer" />
  </div>
</template>
