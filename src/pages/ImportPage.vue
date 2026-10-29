<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { useRoute, useRouter } from 'vue-router'
import ImportWindow from '../components/import/ImportWindow.vue'
import { getAppState, type AppActivity, type AppLogEntry } from '../services/appState'
import type { MediaKind } from '../types/media'

const route = useRoute()
const router = useRouter()
const appState = getAppState()
type ImportWindowHandle = {
  acceptDroppedBrowserFiles: (files: File[]) => number
  acceptDroppedPaths: (paths: string[]) => number
}
const importWindowRef = ref<ImportWindowHandle | null>(null)
const isDraggingFiles = ref(false)
let dragDepth = 0
let pageIsMounted = false
let unlistenNativeDrop: UnlistenFn | null = null

function hasDraggedFiles(event: DragEvent): boolean {
  const types = Array.from(event.dataTransfer?.types ?? [])
  return types.includes('Files') || Boolean(event.dataTransfer?.files.length)
}
function onWindowDragEnter(event: DragEvent) {
  if (!hasDraggedFiles(event)) return
  event.preventDefault()
  dragDepth += 1
  isDraggingFiles.value = true
}
function onWindowDragOver(event: DragEvent) {
  if (!hasDraggedFiles(event)) return
  event.preventDefault()
  if (event.dataTransfer) event.dataTransfer.dropEffect = 'copy'
  isDraggingFiles.value = true
}
function onWindowDragLeave(event: DragEvent) {
  if (!isDraggingFiles.value) return
  if (!event.relatedTarget) {
    dragDepth = 0
    isDraggingFiles.value = false
    return
  }
  dragDepth = Math.max(0, dragDepth - 1)
  if (dragDepth === 0) isDraggingFiles.value = false
}
function onWindowDrop(event: DragEvent) {
  const filesInTransfer = hasDraggedFiles(event) || isDraggingFiles.value
  if (filesInTransfer) {
    event.preventDefault()
    event.stopPropagation()
  }
  dragDepth = 0
  isDraggingFiles.value = false
  // Tauri supplies native filesystem paths via onDragDropEvent; avoid queueing the same drop twice.
  if (isTauri()) return
  const files = Array.from(event.dataTransfer?.files ?? [])
  if (files.length) importWindowRef.value?.acceptDroppedBrowserFiles(files)
}

async function installNativeDropListener() {
  try {
    const unlisten = await getCurrentWebview().onDragDropEvent((event) => {
      const payload = event.payload
      if (payload.type === 'enter' || payload.type === 'over') {
        isDraggingFiles.value = true
      } else if (payload.type === 'leave') {
        isDraggingFiles.value = false
        dragDepth = 0
      } else if (payload.type === 'drop') {
        isDraggingFiles.value = false
        dragDepth = 0
        if (payload.paths.length) importWindowRef.value?.acceptDroppedPaths(payload.paths)
      }
    })
    if (pageIsMounted) unlistenNativeDrop = unlisten
    else unlisten()
  } catch {
    // Browser mode has DOM drag-and-drop; native Tauri path drops are added when the Webview API is available.
  }
}

onMounted(() => {
  pageIsMounted = true
  window.addEventListener('dragenter', onWindowDragEnter, true)
  window.addEventListener('dragover', onWindowDragOver, true)
  window.addEventListener('dragleave', onWindowDragLeave, true)
  window.addEventListener('drop', onWindowDrop, true)
  if (isTauri()) void installNativeDropListener()
})
onBeforeUnmount(() => {
  pageIsMounted = false
  window.removeEventListener('dragenter', onWindowDragEnter, true)
  window.removeEventListener('dragover', onWindowDragOver, true)
  window.removeEventListener('dragleave', onWindowDragLeave, true)
  window.removeEventListener('drop', onWindowDrop, true)
  if (unlistenNativeDrop) {
    unlistenNativeDrop()
    unlistenNativeDrop = null
  }
  dragDepth = 0
  isDraggingFiles.value = false
})

const initialKind = computed<MediaKind>(() => {
  const requested = route.query.kind
  return requested === 'video' || requested === 'archive' ? requested : 'photo'
})
const recentActivities = computed(() =>
  [...appState.activities]
    .sort((a, b) => b.updatedAt - a.updatedAt)
    .slice(0, 8),
)
const recentLogs = computed(() => [...appState.logs].slice(-30).reverse())

function activityStatus(activity: AppActivity): string {
  if (activity.status === 'running') return 'Выполняется'
  if (activity.status === 'success') return 'Завершено'
  if (activity.status === 'warning') return 'Предупреждения'
  if (activity.status === 'interrupted') return 'Возможно прервано'
  return 'Ошибка'
}
function activityIcon(activity: AppActivity): string {
  if (activity.status === 'success') return '✓'
  if (activity.status === 'warning' || activity.status === 'interrupted') return '!'
  if (activity.status === 'error') return '×'
  return '◌'
}
function logLevelLabel(entry: AppLogEntry): string {
  return entry.level === 'error' ? 'ОШИБКА'
    : entry.level === 'warning' ? 'ВНИМАНИЕ'
      : entry.level === 'success' ? 'ГОТОВО' : 'ИНФО'
}
function leaveImport() {
  const previous = window.history.state?.back
  if (typeof previous === 'string' && previous && !previous.includes('/import')) void router.back()
  else void router.push('/')
}
</script>

<template>
  <div class="import-page" :class="{ 'is-file-drag-active': isDraggingFiles }">
    <ImportWindow ref="importWindowRef" :open="true" :initial-kind="initialKind" variant="page" @close="leaveImport" />

    <div class="import-page-history">
      <section class="panel import-page-history-panel">
        <div class="panel-heading">
          <div>
            <h2 class="panel-title">ПОСЛЕДНИЕ ЗАДАЧИ</h2>
            <p class="panel-subtitle">Что приложение выполняло недавно и чем закончились операции.</p>
          </div>
          <span class="pill">{{ recentActivities.length }} записей</span>
        </div>
        <div v-if="recentActivities.length" class="import-history-task-list">
          <article
            v-for="activity in recentActivities"
            :key="activity.id"
            class="app-status-item"
            :class="[`status-${activity.status}`, { 'is-active': activity.status === 'running' }]"
          >
            <span class="app-status-item-icon">{{ activityIcon(activity) }}</span>
            <div class="app-status-item-copy">
              <strong>{{ activity.title }}</strong>
              <p>{{ activity.details }}</p>
              <small>{{ activityStatus(activity) }} · {{ new Date(activity.updatedAt).toLocaleString('ru-RU') }}</small>
              <div
                v-if="activity.status === 'running' && typeof activity.progress === 'number'"
                class="app-status-progress-track"
                role="progressbar"
                :aria-valuenow="activity.progress"
                aria-valuemin="0"
                aria-valuemax="100"
              >
                <span :style="{ width: `${activity.progress}%` }"></span>
              </div>
            </div>
            <span v-if="typeof activity.progress === 'number'" class="app-status-percent">{{ Math.round(activity.progress) }}%</span>
          </article>
        </div>
        <p v-else class="import-history-empty">Задач пока нет. Первые операции появятся здесь.</p>
      </section>

      <section class="panel import-page-history-panel">
        <div class="panel-heading">
          <div>
            <h2 class="panel-title">ПОСЛЕДНИЕ СОБЫТИЯ</h2>
            <p class="panel-subtitle">Ошибки, предупреждения и заметки журнала приложения.</p>
          </div>
          <span class="pill">{{ recentLogs.length }} событий</span>
        </div>
        <div v-if="recentLogs.length" class="import-page-event-list">
          <article v-for="entry in recentLogs" :key="entry.id" class="app-status-log-row" :class="`log-${entry.level}`">
            <time>{{ new Date(entry.at).toLocaleTimeString('ru-RU', { hour: '2-digit', minute: '2-digit', second: '2-digit' }) }}</time>
            <span class="import-page-event-level">{{ logLevelLabel(entry) }}</span>
            <p>{{ entry.message }}</p>
          </article>
        </div>
        <p v-else class="import-history-empty">Событий пока нет.</p>
      </section>
    </div>

    <div v-if="isDraggingFiles" class="import-drop-overlay" role="status" aria-live="polite">
      <div class="import-drop-overlay-card">
        <span class="import-drop-overlay-icon" aria-hidden="true">↓</span>
        <strong>Отпусти файлы, чтобы добавить их в очередь</strong>
        <p>Поддерживаемые расширения зависят от выбранного типа: фото, видео или архивы.</p>
      </div>
    </div>
  </div>
</template>
