<script setup lang="ts">
import { computed, onMounted, onBeforeUnmount, ref, watch } from 'vue'
import { convertFileSrc, isTauri } from '@tauri-apps/api/core'
import PageIntro from '../common/PageIntro.vue'
import MediaViewer from './MediaViewer.vue'
import { selectSourceFiles } from '../../services/nativeDialogs'
import { getSavedLibraryRoot, loadLibraryRoot } from '../../services/librarySettings'
import { importMediaFiles, listLibraryMedia } from '../../services/filesystem'
import { CATEGORY_LABELS, CATEGORY_OPTIONS, type ContentCategory, type ImportResult, type LibraryMediaItem } from '../../types/media'
import { formatBytes } from '../../services/formatters'

const props = defineProps<{
  kind: 'photo' | 'video'
  title: string
  description: string
}>()

const items = ref<LibraryMediaItem[]>([])
const columns = ref(3)
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

function openViewer(index: number) { viewerIndex.value = index }
function closeViewer() { viewerIndex.value = null }
function onDrop(event: DragEvent) {
  event.preventDefault()
  if (!isDesktop.value && event.dataTransfer?.files.length) addBrowserPreview(event.dataTransfer.files)
}
function onDragOver(event: DragEvent) { event.preventDefault() }

watch(() => props.kind, () => { void reloadItems() })
onMounted(() => { void reloadItems() })
onBeforeUnmount(() => { for (const url of previewUrls) URL.revokeObjectURL(url) })
</script>

<template>
  <div @drop="onDrop" @dragover="onDragOver">
    <PageIntro :title="title" :description="description" />

    <section class="mb-5 rounded-xl border border-slate-200 bg-white p-4">
      <div class="flex flex-wrap items-end justify-between gap-4">
        <div class="grid flex-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
          <label class="block">
            <span class="mb-1.5 block text-xs font-medium text-slate-600">Категория для загрузки</span>
            <select v-model="destinationCategory" class="w-full rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm">
              <option v-for="category in CATEGORY_OPTIONS" :key="category.value" :value="category.value">{{ category.label }}</option>
            </select>
          </label>
          <label class="block">
            <span class="mb-1.5 block text-xs font-medium text-slate-600">Показывать категорию</span>
            <select v-model="categoryFilter" class="w-full rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm">
              <option value="all">Все категории</option>
              <option v-for="category in CATEGORY_OPTIONS" :key="category.value" :value="category.value">{{ category.label }}</option>
            </select>
          </label>
          <label v-if="isDesktop" class="block">
            <span class="mb-1.5 block text-xs font-medium text-slate-600">Импортировать как</span>
            <select v-model="importMode" class="w-full rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm">
              <option value="copy">Копировать (оригинал останется)</option>
              <option value="move">Переместить (удалить оригинал после проверки)</option>
            </select>
          </label>
        </div>
        <button type="button" class="inline-flex items-center justify-center rounded-lg bg-slate-900 px-4 py-2.5 text-sm font-semibold text-white hover:bg-slate-700 disabled:cursor-wait disabled:opacity-60" :disabled="busy" @click="addFiles">
          {{ busy ? 'Загрузка…' : (props.kind === 'photo' ? '+ Добавить фото' : '+ Добавить видео') }}
        </button>
      </div>
      <input ref="browserFileInput" class="hidden" type="file" multiple :accept="acceptedExtensions" @change="onBrowserFilesChange" />
      <p class="mt-3 text-xs leading-5 text-slate-500">
        <template v-if="isDesktop">Файлы попадут в папку категории «{{ CATEGORY_LABELS[destinationCategory] }}». При перемещении оригинал удаляется только после успешного копирования и проверки размера.</template>
        <template v-else>Браузерный режим: выбери файлы или перетащи их сюда, чтобы проверить сетку и просмотрщик. Файлы останутся временными и на диск медиатеки не записываются.</template>
      </p>
    </section>

    <div v-if="message" class="mb-3 whitespace-pre-line rounded-lg border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-900">{{ message }}</div>
    <div v-if="errorMessage" class="mb-3 whitespace-pre-line rounded-lg border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-900">{{ errorMessage }}</div>

    <div class="mb-3 flex flex-wrap items-center justify-between gap-3">
      <p class="text-sm text-slate-600">{{ loading ? 'Сканирование медиатеки…' : `Файлов: ${filteredItems.length}` }}</p>
      <div class="flex items-center gap-2 text-sm">
        <label for="grid-columns" class="text-slate-500">Сетка</label>
        <select id="grid-columns" v-model.number="columns" class="rounded-lg border border-slate-300 bg-white px-2.5 py-1.5">
          <option :value="2">2 колонки</option>
          <option :value="3">3 колонки</option>
          <option :value="4">4 колонки</option>
          <option :value="5">5 колонок</option>
          <option :value="6">6 колонок</option>
        </select>
        <button type="button" class="rounded-lg border border-slate-300 px-3 py-1.5 text-slate-700 hover:bg-white" @click="reloadItems">Обновить</button>
      </div>
    </div>

    <section v-if="filteredItems.length" class="grid gap-3" :style="{ gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))` }">
      <button v-for="(item, index) in filteredItems" :key="item.id" type="button" class="group min-w-0 overflow-hidden rounded-xl border border-slate-200 bg-white text-left transition hover:-translate-y-0.5 hover:border-slate-400 hover:shadow-sm" @click="openViewer(index)">
        <div class="relative flex aspect-[4/3] items-center justify-center overflow-hidden bg-slate-100">
          <img v-if="props.kind === 'photo' || item.extension === '.gif'" :src="sourceFor(item)" :alt="item.originalName" class="h-full w-full object-contain" loading="lazy" />
          <video v-else :src="sourceFor(item)" class="h-full w-full object-contain" preload="metadata" muted playsinline />
          <span v-if="props.kind === 'video'" class="absolute bottom-2 right-2 rounded bg-black/70 px-2 py-1 text-[10px] font-semibold uppercase text-white">{{ item.extension === '.gif' ? 'GIF' : 'VIDEO' }}</span>
          <span v-if="item.previewOnly" class="absolute left-2 top-2 rounded bg-amber-100 px-2 py-1 text-[10px] font-semibold text-amber-900">Временное превью</span>
        </div>
        <div class="p-3">
          <p class="truncate text-sm font-medium text-slate-900" :title="item.originalName">{{ item.originalName }}</p>
          <p class="mt-1 flex flex-wrap gap-x-2 text-xs text-slate-500"><span>{{ formatBytes(item.sizeBytes) }}</span><span>{{ item.category ? CATEGORY_LABELS[item.category] : 'Без категории' }}</span></p>
          <p class="mt-2 truncate text-[11px] text-slate-400" :title="item.relativePath">{{ item.relativePath }}</p>
        </div>
      </button>
    </section>

    <section v-else class="rounded-xl border border-dashed border-slate-300 bg-white px-5 py-16 text-center">
      <div class="mx-auto flex h-12 w-12 items-center justify-center rounded-2xl bg-slate-100 text-xl text-slate-500">{{ props.kind === 'photo' ? 'IMG' : 'VID' }}</div>
      <h2 class="mt-4 text-base font-semibold text-slate-900">{{ loading ? 'Загружаем файлы…' : 'Пока здесь пусто' }}</h2>
      <p class="mx-auto mt-2 max-w-md text-sm leading-6 text-slate-500">
        {{ isDesktop ? (rootPath ? 'Добавь файлы кнопкой выше. Галерея будет сканировать папки выбранной медиатеки.' : 'Сначала открой «Настройки», выбери место и создай папку медиатеки.') : 'Добавь локальные файлы кнопкой выше — они появятся только для проверки интерфейса в браузере.' }}
      </p>
      <button v-if="!isDesktop" type="button" class="mt-4 rounded-lg bg-slate-900 px-4 py-2 text-sm font-semibold text-white" @click="addFiles">Выбрать файлы для превью</button>
    </section>

    <MediaViewer v-if="viewerIndex !== null" :items="filteredItems" :initial-index="viewerIndex ?? 0" :media-kind="props.kind" @close="closeViewer" />
  </div>
</template>
