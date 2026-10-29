<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { useRouter } from 'vue-router'
import { CATEGORY_LABELS, CATEGORY_OPTIONS, type ContentCategory, type ImportProgress, type MediaKind } from '../../types/media'
import { importMediaFiles } from '../../services/filesystem'
import { loadLibraryRoot } from '../../services/librarySettings'
import { selectSourceFiles } from '../../services/nativeDialogs'
import { formatBytes } from '../../services/formatters'
import { addLog, beginActivity, errorText, finishActivity, getAppState, notify, updateActivity } from '../../services/appState'

interface QueuedFile {
  id: string
  name: string
  path?: string
  file?: File
  sizeBytes: number | null
}

const props = withDefaults(defineProps<{
  open: boolean
  initialKind: MediaKind
  variant?: 'modal' | 'page'
}>(), { variant: 'modal' })
const emit = defineEmits<{ close: []; completed: [] }>()
const router = useRouter()
const isDesktop = computed(() => isTauri())
const kind = ref<MediaKind>('photo')
const category = ref<ContentCategory>('irl')
const mode = ref<'copy' | 'move'>('copy')
const selectedFiles = ref<QueuedFile[]>([])
const rootPath = ref('')
const choosing = ref(false)
const importing = ref(false)
const progress = ref(0)
const progressDetails = ref<ImportProgress | null>(null)
const operationId = ref('')
const appState = getAppState()
const operationLogs = computed(() => appState.logs.filter((entry) => entry.activityId === operationId.value).slice(-80).reverse())
const warningMessage = ref('')
const resultMessage = ref('')
const errorMessage = ref('')
const browserFileInput = ref<HTMLInputElement | null>(null)
let unlistenProgress: UnlistenFn | null = null

const kindLabels: Record<MediaKind, string> = { photo: 'Фото', video: 'Видео', archive: 'Архивы' }
const kindIcons: Record<MediaKind, string> = { photo: '▧', video: '▶', archive: '▤' }
const kindOptions: MediaKind[] = ['photo', 'video', 'archive']
const accepted = computed(() => {
  if (kind.value === 'photo') return 'image/*,.jpg,.jpeg,.png,.webp,.bmp,.tif,.tiff,.avif,.heic'
  if (kind.value === 'video') return 'video/*,.mp4,.mkv,.avi,.mov,.webm,.m4v,.mpg,.mpeg,.wmv,.flv,.3gp,.ogv,.gif'
  return '.zip,.7z,.rar,.tar,.gz,.bz2,.xz'
})
const canStart = computed(() => selectedFiles.value.length > 0 && !importing.value && (isDesktop.value ? Boolean(rootPath.value) : kind.value !== 'archive'))
const progressLabel = computed(() => {
  if (progressDetails.value?.status === 'finished') return 'Импорт завершён'
  if (progressDetails.value?.status === 'copying') return `Копирование: ${progressDetails.value.currentFile}`
  if (progressDetails.value?.status === 'file-complete') return `Обработано файлов: ${progressDetails.value.completedFiles} из ${progressDetails.value.totalFiles}`
  if (progress.value >= 100) return 'Подготовка завершена'
  return 'Ожидание начала импорта'
})

function allowedExtension(name: string, selectedKind: MediaKind): boolean {
  const extension = name.split('.').pop()?.toLowerCase() ?? ''
  const groups: Record<MediaKind, string[]> = {
    photo: ['jpg', 'jpeg', 'png', 'webp', 'bmp', 'tif', 'tiff', 'avif', 'heic'],
    video: ['mp4', 'mkv', 'avi', 'mov', 'webm', 'm4v', 'mpg', 'mpeg', 'wmv', 'flv', '3gp', 'ogv', 'gif'],
    archive: ['zip', '7z', 'rar', 'tar', 'gz', 'bz2', 'xz'],
  }
  return groups[selectedKind].includes(extension)
}
function clearFeedback() { resultMessage.value = ''; errorMessage.value = ''; warningMessage.value = '' }
function logImport(level: 'info' | 'success' | 'warning' | 'error', message: string) {
  if (!operationId.value) return
  addLog(level, message, operationId.value)
}

async function prepareWindow() {
  // The dialog may be hidden while a native import continues; reopening must not reset its live state.
  if (importing.value) return
  clearFeedback()
  kind.value = props.initialKind
  category.value = 'irl'
  mode.value = 'copy'
  selectedFiles.value = []
  progress.value = 0
  progressDetails.value = null
  operationId.value = ''
  rootPath.value = ''
  if (isTauri()) {
    try { rootPath.value = await loadLibraryRoot() }
    catch (error) { errorMessage.value = errorText(error); notify('warning', 'Не удалось прочитать настройки медиатеки', errorMessage.value) }
  }
}
watch(() => props.open, (opened) => { if (opened) void prepareWindow() })
watch(() => props.initialKind, (value) => {
  if (props.open && !selectedFiles.value.length && !importing.value) kind.value = value
})

function openSettingsFromImport() {
  if (props.variant === 'page') void router.push('/settings')
  else {
    void router.push('/settings')
    closeWindow()
  }
}

function chooseFiles() {
  clearFeedback()
  if (!importing.value) { operationId.value = ''; progress.value = 0; progressDetails.value = null }
  if (isDesktop.value) void chooseNativeFiles()
  else browserFileInput.value?.click()
}
async function chooseNativeFiles() {
  choosing.value = true
  try {
    const paths = await selectSourceFiles(kind.value)
    const additions = paths.filter((path) => allowedExtension(path, kind.value)).map((path): QueuedFile => ({
      id: path,
      path,
      name: path.split(/[\\/]/).pop() || path,
      sizeBytes: null,
    }))
    const combined = new Map(selectedFiles.value.map((file) => [file.id, file]))
    additions.forEach((file) => combined.set(file.id, file))
    selectedFiles.value = [...combined.values()]
    if (paths.length && additions.length < paths.length) {
      errorMessage.value = 'Некоторые файлы пропущены: расширение не подходит выбранному типу импорта.'
      notify('warning', 'Часть файлов не добавлена', errorMessage.value)
    }
  } catch (error) {
    errorMessage.value = errorText(error)
    // File picker cancellation isn't an error; this branch only runs for rejected dialog operations.
    if (errorMessage.value) {
      addLog('error', `Не удалось открыть системный выбор файлов: ${errorMessage.value}`)
      notify('error', 'Не удалось выбрать файлы', errorMessage.value)
    }
  } finally { choosing.value = false }
}
function mergeQueue(additions: QueuedFile[]) {
  const combined = new Map(selectedFiles.value.map((file) => [file.id, file]))
  additions.forEach((file) => combined.set(file.id, file))
  selectedFiles.value = [...combined.values()]
}

function queueBrowserFiles(files: File[]): number {
  const supported = files.filter((file) => allowedExtension(file.name, kind.value))
  const additions = supported.map((file): QueuedFile => ({
    id: `browser:${file.webkitRelativePath || file.name}:${file.size}:${file.lastModified}`,
    name: file.name,
    file,
    sizeBytes: file.size,
  }))
  mergeQueue(additions)
  if (supported.length < files.length) {
    errorMessage.value = 'Некоторые файлы пропущены: расширение не подходит выбранному типу импорта.'
    notify('warning', 'Часть файлов не добавлена', errorMessage.value)
  }
  return additions.length
}

function acceptDroppedBrowserFiles(files: File[]): number {
  if (!props.open || importing.value || !files.length) return 0
  clearFeedback()
  if (!importing.value) { operationId.value = ''; progress.value = 0; progressDetails.value = null }
  const added = queueBrowserFiles(files)
  if (added > 0) notify('success', 'Файлы добавлены в очередь', `Добавлено файлов: ${added}. Тип импорта: ${kindLabels[kind.value]}.`)
  else if (files.length) notify('warning', 'Файлы не добавлены', `Поддерживаемый тип сейчас: ${kindLabels[kind.value]}. Выбери другой тип или добавь подходящие файлы.`)
  return added
}

function acceptDroppedPaths(paths: string[]): number {
  if (!props.open || importing.value || !paths.length) return 0
  clearFeedback()
  if (!importing.value) { operationId.value = ''; progress.value = 0; progressDetails.value = null }
  const supported = paths.filter((path) => allowedExtension(path, kind.value))
  const additions = supported.map((path): QueuedFile => ({
    id: path,
    path,
    name: path.split(/[\\/]/).pop() || path,
    sizeBytes: null,
  }))
  mergeQueue(additions)
  if (supported.length < paths.length) {
    errorMessage.value = 'Некоторые файлы пропущены: расширение не подходит выбранному типу импорта.'
    notify('warning', 'Часть файлов не добавлена', errorMessage.value)
  }
  if (additions.length > 0) notify('success', 'Файлы добавлены в очередь', `Добавлено файлов: ${additions.length}. Тип импорта: ${kindLabels[kind.value]}.`)
  else if (paths.length) notify('warning', 'Файлы не добавлены', `Поддерживаемый тип сейчас: ${kindLabels[kind.value]}. Выбери другой тип или добавь подходящие файлы.`)
  return additions.length
}

function onBrowserFilesChange(event: Event) {
  const input = event.target as HTMLInputElement
  const files = Array.from(input.files ?? [])
  queueBrowserFiles(files)
  input.value = ''
}

defineExpose({ acceptDroppedBrowserFiles, acceptDroppedPaths })
function selectKind(value: MediaKind) {
  if (importing.value) return
  kind.value = value
  selectedFiles.value = []
  progress.value = 0
  progressDetails.value = null
  operationId.value = ''
  clearFeedback()
}
function clearQueue() {
  if (importing.value) return
  selectedFiles.value = []
  progress.value = 0
  progressDetails.value = null
  operationId.value = ''
  clearFeedback()
}
function removeFile(id: string) {
  if (!importing.value) selectedFiles.value = selectedFiles.value.filter((file) => file.id !== id)
}
function onProgress(event: { payload: ImportProgress }) {
  const payload = event.payload
  if (operationId.value && payload.operationId && payload.operationId !== operationId.value) return
  progressDetails.value = payload
  progress.value = payload.overallPercent
}

async function startImport() {
  clearFeedback()
  if (!selectedFiles.value.length) { errorMessage.value = 'Сначала выбери хотя бы один файл.'; notify('warning', 'Импорт не запущен', errorMessage.value); return }
  if (!isDesktop.value) {
    if (kind.value === 'archive') {
      errorMessage.value = 'Архивы можно сохранить в медиатеку только в настольной версии Tauri.'
      notify('warning', 'Недоступный режим импорта', errorMessage.value)
      return
    }
    const browserFiles = selectedFiles.value.flatMap((file) => file.file ? [file.file] : [])
    if (!browserFiles.length) { errorMessage.value = 'Выбери файлы заново: браузер не сохранил доступ к исходным файлам.'; notify('error', 'Исходные файлы недоступны', errorMessage.value); return }
    const previewOperationId = `preview-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`
    operationId.value = previewOperationId
    beginActivity({ id: previewOperationId, kind: 'import', title: 'Подготовка превью', details: `${browserFiles.length} файлов · браузерный режим`, progress: 0 })
    logImport('info', 'Браузер не может записывать файлы в папку медиатеки; создаётся только временное превью.')
    progress.value = 100
    progressDetails.value = {
      operationId: previewOperationId,
      overallPercent: 100,
      fileIndex: browserFiles.length,
      totalFiles: browserFiles.length,
      currentFile: '',
      bytesCopied: browserFiles.reduce((total, file) => total + file.size, 0),
      fileSize: 0,
      completedFiles: browserFiles.length,
      status: 'finished',
    }
    try {
      await router.push(kind.value === 'photo' ? '/gallery/photos' : '/gallery/videos')
      await nextTick()
      window.dispatchEvent(new CustomEvent('media-library:browser-import', {
        detail: { kind: kind.value, files: browserFiles, category: category.value },
      }))
      resultMessage.value = `Подготовлено для временного превью: ${browserFiles.length}. Файлы не копировались на диск.`
      logImport('success', resultMessage.value)
      finishActivity(previewOperationId, 'success', resultMessage.value)
      selectedFiles.value = []
    } catch (error) {
      const message = errorText(error)
      errorMessage.value = message
      finishActivity(previewOperationId, 'error', message)
    }
    return
  }
  if (!rootPath.value) { errorMessage.value = 'Сначала открой настройки и выбери корневую папку медиатеки.'; notify('error', 'Не выбрана папка медиатеки', errorMessage.value); return }
  if (mode.value === 'move' && !window.confirm('Каждый файл будет скопирован и проверен по размеру. Исходник удалится только после успешной проверки. Продолжить?')) return
  const paths = selectedFiles.value.flatMap((file) => file.path ? [file.path] : [])
  if (!paths.length) { errorMessage.value = 'В списке нет доступных исходных путей. Выбери файлы заново.'; notify('error', 'Исходные файлы недоступны', errorMessage.value); return }

  importing.value = true
  progress.value = 0
  progressDetails.value = null
  warningMessage.value = ''
  const currentOperationId = `import-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`
  operationId.value = currentOperationId
  const destination = `${CATEGORY_LABELS[category.value]} / ${kindLabels[kind.value]}`
  beginActivity({
    id: currentOperationId,
    kind: 'import',
    title: 'Импорт файлов',
    details: `${paths.length} файлов → ${destination}`,
    progress: 0,
  })
  logImport('info', `Режим: ${mode.value === 'move' ? 'перемещение после проверки' : 'копирование с сохранением оригинала'}.`) 
  try {
    unlistenProgress = await listen<ImportProgress>('import-progress', onProgress)
    const result = await importMediaFiles({
      sourcePaths: paths,
      rootPath: rootPath.value,
      category: category.value,
      kind: kind.value,
      mode: mode.value,
      operationId: currentOperationId,
    })
    progress.value = 100
    resultMessage.value = `Импорт завершён. Успешно добавлено: ${result.imported.length} из ${paths.length}.`
    warningMessage.value = result.warnings.join('\n')
    if (result.warnings.length) {
      for (const warning of result.warnings) logImport('warning', warning)
    }
    logImport(result.warnings.length ? 'warning' : 'success', resultMessage.value)
    finishActivity(currentOperationId, result.warnings.length ? 'warning' : 'success', resultMessage.value + (result.warnings.length ? ` Предупреждений: ${result.warnings.length}.` : ''))
    selectedFiles.value = []
    emit('completed')
  } catch (error) {
    const message = errorText(error)
    errorMessage.value = message
    logImport('error', message)
    finishActivity(currentOperationId, 'error', message)
  } finally {
    importing.value = false
    if (unlistenProgress) { unlistenProgress(); unlistenProgress = null }
  }
}
function closeWindow() {
  if (choosing.value) return
  if (importing.value) logImport('info', 'Окно импорта свёрнуто пользователем; фоновая задача продолжает выполняться.')
  emit('close')
}
function onKeydown(event: KeyboardEvent) { if (props.open && event.key === 'Escape') closeWindow() }
onMounted(() => {
  window.addEventListener('keydown', onKeydown)
  if (props.open) void prepareWindow()
})
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown)
  if (unlistenProgress) {
    unlistenProgress()
    unlistenProgress = null
  }
})
</script>

<template>
  <Teleport to="body" :disabled="props.variant === 'page'">
    <div
      v-if="props.variant === 'page' || open"
      :class="props.variant === 'page' ? 'import-page-wrapper' : 'import-overlay'"
      @click.self="props.variant === 'modal' && closeWindow()"
    >
      <section
        class="import-dialog"
        :class="{ 'import-dialog-page': props.variant === 'page' }"
        role="dialog"
        :aria-modal="props.variant === 'modal' ? 'true' : undefined"
        aria-labelledby="import-window-title"
      >
        <header class="import-dialog-header">
          <div class="import-dialog-title-wrap">
            <span class="import-dialog-logo" aria-hidden="true">⇧</span>
            <div>
              <p class="import-dialog-eyebrow">MEDIA LIBRARY / INGEST</p>
              <h2 id="import-window-title" class="import-dialog-title">Импорт файлов</h2>
              <p class="import-dialog-subtitle">Добавь фото, видео или архивы в выбранную категорию.</p>
            </div>
          </div>
          <button type="button" class="import-close-button" aria-label="Закрыть окно импорта" :disabled="choosing" @click="closeWindow">{{ importing ? '−' : '✕' }}</button>
        </header>

        <div class="import-dialog-body">
          <div class="import-kind-options" role="group" aria-label="Тип импортируемых файлов">
            <button v-for="option in kindOptions" :key="option" type="button" class="import-kind-option" :class="{ 'is-active': kind === option }" :aria-pressed="kind === option" :disabled="importing" @click="selectKind(option)">
              <span class="import-kind-icon" aria-hidden="true">{{ kindIcons[option] }}</span>
              <span>{{ kindLabels[option] }}</span>
              <span v-if="kind === option" class="import-kind-check" aria-hidden="true">✓</span>
            </button>
          </div>

          <div class="import-settings-grid">
            <label>
              <span class="field-label">Категория хранения</span>
              <select v-model="category" class="field-control" :disabled="importing">
                <option v-for="option in CATEGORY_OPTIONS" :key="option.value" :value="option.value">{{ option.label }}</option>
              </select>
            </label>
            <label v-if="isDesktop">
              <span class="field-label">Способ импорта</span>
              <select v-model="mode" class="field-control" :disabled="importing">
                <option value="copy">Копировать — оставить оригинал</option>
                <option value="move">Переместить после проверки</option>
              </select>
            </label>
          </div>

          <p v-if="!isDesktop && kind === 'archive'" class="notice notice-warning import-feedback">Для реального импорта архивов нужно запустить настольную версию Tauri. В браузере файлы можно только выбрать.</p>

          <div class="import-destination-row">
            <div class="import-destination-icon" aria-hidden="true">⌁</div>
            <div class="import-destination-copy">
              <span class="import-meta-label">МЕСТО НАЗНАЧЕНИЯ</span>
              <strong>{{ CATEGORY_LABELS[category] }} / {{ kindLabels[kind] }}</strong>
              <p v-if="isDesktop">{{ rootPath || 'Папка медиатеки не выбрана' }}</p>
              <p v-else>Только временное превью в браузере; запись в папку Windows недоступна.</p>
            </div>
            <button v-if="isDesktop && !rootPath" type="button" class="btn btn-secondary" @click="openSettingsFromImport">Настройки</button>
          </div>

          <input ref="browserFileInput" class="hidden" type="file" multiple :accept="accepted" @change="onBrowserFilesChange" />
          <div class="import-file-picker">
            <div class="import-file-picker-icon" aria-hidden="true">＋</div>
            <strong>Выбери файлы для импорта</strong>
            <p>Можно выбрать несколько файлов за один раз. Имена-конфликты будут автоматически дополнены номером.</p>
            <button type="button" class="btn btn-secondary" :disabled="choosing || importing" @click="chooseFiles">{{ choosing ? 'Открываем выбор…' : 'Выбрать файлы' }}</button>
          </div>

          <section v-if="selectedFiles.length" class="import-queue">
            <div class="import-queue-heading">
              <div><strong>Очередь импорта</strong><span>{{ selectedFiles.length }} {{ selectedFiles.length === 1 ? 'файл' : selectedFiles.length < 5 ? 'файла' : 'файлов' }}</span></div>
              <button type="button" class="btn btn-quiet" :disabled="importing" @click="clearQueue">Очистить</button>
            </div>
            <div class="import-file-list">
              <div v-for="file in selectedFiles.slice(0, 7)" :key="file.id" class="import-file-row">
                <span class="import-file-type" aria-hidden="true">{{ kindIcons[kind] }}</span>
                <span class="import-file-name" :title="file.name">{{ file.name }}</span>
                <span class="import-file-size">{{ file.sizeBytes === null ? 'Локальный файл' : formatBytes(file.sizeBytes) }}</span>
                <button type="button" class="import-file-remove" :disabled="importing" :aria-label="`Убрать ${file.name}`" @click="removeFile(file.id)">×</button>
              </div>
              <p v-if="selectedFiles.length > 7" class="import-more-files">И ещё {{ selectedFiles.length - 7 }} файл(ов) в очереди.</p>
            </div>
          </section>

          <section v-if="importing || progressDetails || resultMessage" class="import-progress-panel" aria-live="polite">
            <div class="import-progress-heading"><span class="import-progress-label">{{ progressLabel }}</span><strong>{{ progress }}%</strong></div>
            <div class="import-progress-track" role="progressbar" :aria-valuenow="progress" aria-valuemin="0" aria-valuemax="100" aria-label="Прогресс импорта">
              <div class="import-progress-fill" :class="{ 'is-complete': progress >= 100 }" :style="{ width: `${progress}%` }"></div>
            </div>
            <div class="import-progress-footer">
              <span v-if="progressDetails && progressDetails.totalFiles">Файл {{ Math.min(progressDetails.fileIndex || progressDetails.completedFiles + 1, progressDetails.totalFiles) }} из {{ progressDetails.totalFiles }}</span>
              <span v-else-if="isDesktop">Копирование с проверкой размера</span>
              <span v-else>Подготовка временного превью</span>
              <span v-if="progressDetails && progressDetails.fileSize && (progressDetails.status === 'copying' || progressDetails.status === 'file-complete')">{{ formatBytes(progressDetails.bytesCopied) }} / {{ formatBytes(progressDetails.fileSize) }}</span>
            </div>
          </section>

          <section v-if="operationId" class="import-log-panel" aria-label="Журнал импорта">
            <div class="import-log-heading"><strong>Журнал операции</strong><span>{{ operationLogs.length }} событий</span></div>
            <div class="import-log-list" role="log" aria-live="polite">
              <p v-if="!operationLogs.length" class="import-log-empty">События импорта появятся здесь.</p>
              <article v-for="entry in operationLogs" :key="entry.id" class="import-log-entry" :class="'log-' + entry.level">
                <time>{{ new Date(entry.at).toLocaleTimeString('ru-RU', { hour: '2-digit', minute: '2-digit', second: '2-digit' }) }}</time>
                <span class="import-log-level">{{ entry.level === 'error' ? 'ОШИБКА' : entry.level === 'warning' ? 'ВНИМАНИЕ' : entry.level === 'success' ? 'ГОТОВО' : 'ИНФО' }}</span>
                <p>{{ entry.message }}</p>
              </article>
            </div>
          </section>

          <p v-if="resultMessage" class="notice notice-success import-feedback">{{ resultMessage }}</p>
          <p v-if="errorMessage" class="notice notice-error import-feedback">{{ errorMessage }}</p>
          <p v-if="warningMessage" class="notice notice-warning import-feedback" style="white-space: pre-line">{{ warningMessage }}</p>
        </div>

        <footer class="import-dialog-footer">
          <p v-if="isDesktop" class="import-footer-hint">Перемещение удалит источник только после успешного копирования и проверки.</p>
          <p v-else class="import-footer-hint">Браузерный режим не записывает файлы на диск медиатеки.</p>
          <div class="import-footer-actions">
            <button type="button" class="btn btn-secondary" :disabled="choosing" @click="closeWindow">{{ importing ? 'Свернуть' : resultMessage ? 'Закрыть' : 'Отмена' }}</button>
            <button type="button" class="btn btn-primary import-start-button" :disabled="!canStart || choosing" @click="startImport"><span aria-hidden="true">{{ importing ? '◌' : '⇧' }}</span>{{ importing ? 'Импорт…' : 'Начать импорт' }}</button>
          </div>
        </footer>
      </section>
    </div>
  </Teleport>
</template>
