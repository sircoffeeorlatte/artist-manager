import { reactive } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { ImportProgress } from '../types/media'

export type ActivityKind = 'import' | 'video' | 'gallery' | 'archive' | 'system'
export type ActivityStatus = 'running' | 'success' | 'warning' | 'error' | 'interrupted'
export type LogLevel = 'info' | 'success' | 'warning' | 'error'

export interface AppActivity {
  id: string
  kind: ActivityKind
  title: string
  details: string
  status: ActivityStatus
  progress?: number
  startedAt: number
  updatedAt: number
}

export interface AppLogEntry {
  id: string
  at: number
  level: LogLevel
  message: string
  activityId?: string
}

export interface AppAlert {
  id: string
  level: LogLevel
  title: string
  message: string
  createdAt: number
}

interface PersistedState {
  activities: AppActivity[]
  logs: AppLogEntry[]
}

const STORAGE_KEY = 'media-library:app-state:v1'
const state = reactive({
  activities: [] as AppActivity[],
  logs: [] as AppLogEntry[],
  alerts: [] as AppAlert[],
  initialized: false,
})
const alertTimers = new Map<string, number>()
let initialization: Promise<void> | null = null
let nextId = 0
let lastPersistAt = 0
let persistTimer: number | undefined
let unlistenImportProgress: UnlistenFn | null = null
const lastImportFileByOperation = new Map<string, string>()

function createId(prefix: string): string {
  nextId += 1
  return `${prefix}-${Date.now().toString(36)}-${nextId.toString(36)}-${Math.random().toString(36).slice(2, 7)}`
}

function persist(force = false): void {
  const now = Date.now()
  if (!force && now - lastPersistAt < 500) {
    if (persistTimer === undefined) {
      persistTimer = window.setTimeout(() => {
        persistTimer = undefined
        persist(true)
      }, Math.max(25, 500 - (now - lastPersistAt)))
    }
    return
  }
  if (persistTimer !== undefined) {
    window.clearTimeout(persistTimer)
    persistTimer = undefined
  }
  try {
    const data: PersistedState = {
      activities: state.activities.slice(-40),
      logs: state.logs.slice(-250),
    }
    localStorage.setItem(STORAGE_KEY, JSON.stringify(data))
    lastPersistAt = now
  } catch {
    // Storage can be disabled or full; the in-memory tracker must keep working.
  }
}

function hydrate(): void {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return
    const parsed = JSON.parse(raw) as Partial<PersistedState>
    if (Array.isArray(parsed.activities)) state.activities = parsed.activities.slice(-40)
    if (Array.isArray(parsed.logs)) state.logs = parsed.logs.slice(-250)
  } catch {
    state.activities = []
    state.logs = []
  }
}

export function addLog(level: LogLevel, message: string, activityId?: string): AppLogEntry {
  const entry: AppLogEntry = { id: createId('log'), at: Date.now(), level, message, activityId }
  state.logs.push(entry)
  if (state.logs.length > 250) state.logs.splice(0, state.logs.length - 250)
  persist()
  return entry
}

export function notify(level: LogLevel, title: string, message: string): void {
  const alert: AppAlert = { id: createId('alert'), level, title, message, createdAt: Date.now() }
  state.alerts.push(alert)
  if (state.alerts.length > 4) dismissAlert(state.alerts[0].id)
  addLog(level, `${title}: ${message}`)
  const timer = window.setTimeout(() => dismissAlert(alert.id), level === 'error' ? 10000 : 6500)
  alertTimers.set(alert.id, timer)
}

export function dismissAlert(id: string): void {
  const timer = alertTimers.get(id)
  if (timer !== undefined) window.clearTimeout(timer)
  alertTimers.delete(id)
  const index = state.alerts.findIndex((alert) => alert.id === id)
  if (index >= 0) state.alerts.splice(index, 1)
}

export function beginActivity(input: {
  id?: string
  kind: ActivityKind
  title: string
  details?: string
  progress?: number
}): string {
  const id = input.id ?? createId(input.kind)
  const existing = state.activities.find((activity) => activity.id === id)
  const now = Date.now()
  if (existing) {
    Object.assign(existing, {
      kind: input.kind,
      title: input.title,
      details: input.details ?? existing.details,
      status: 'running' as const,
      progress: input.progress ?? existing.progress,
      updatedAt: now,
    })
  } else {
    state.activities.push({
      id,
      kind: input.kind,
      title: input.title,
      details: input.details ?? 'Выполняется',
      status: 'running',
      progress: input.progress,
      startedAt: now,
      updatedAt: now,
    })
    while (state.activities.length > 60) {
      const oldFinishedIndex = state.activities.findIndex((activity) => activity.status !== 'running')
      if (oldFinishedIndex < 0) break
      state.activities.splice(oldFinishedIndex, 1)
    }
  }
  addLog('info', `${input.title}${input.details ? ` — ${input.details}` : ''}`, id)
  persist(true)
  return id
}

export function updateActivity(
  id: string,
  patch: Partial<Pick<AppActivity, 'title' | 'details' | 'progress' | 'status'>>,
): void {
  const activity = state.activities.find((item) => item.id === id)
  if (!activity) return
  Object.assign(activity, patch, { updatedAt: Date.now() })
  persist()
}

export function finishActivity(
  id: string,
  status: Exclude<ActivityStatus, 'running'> = 'success',
  details?: string,
): void {
  const activity = state.activities.find((item) => item.id === id)
  if (!activity) return
  activity.status = status
  if (details !== undefined) activity.details = details
  if (status === 'success') activity.progress = 100
  activity.updatedAt = Date.now()
  addLog(status === 'success' ? 'success' : status === 'warning' ? 'warning' : 'error', `${activity.title}: ${activity.details}`, id)
  if (status === 'error') notify('error', activity.title, activity.details)
  else if (status === 'warning') notify('warning', activity.title, activity.details)
  persist(true)
}

function trackImportProgress(payload: ImportProgress): void {
  const activity = state.activities.find((item) => item.id === payload.operationId)
  if (!activity || activity.kind !== 'import') return

  const detail = payload.status === 'copying'
    ? `Копирование: ${payload.currentFile}`
    : payload.status === 'started'
      ? `Подготовлена очередь: ${payload.totalFiles} файлов`
      : payload.status === 'file-complete'
        ? `Обработано ${payload.completedFiles} из ${payload.totalFiles}: ${payload.currentFile}`
        : payload.status === 'finished'
          ? 'Передача файлов завершена; проверяем результат'
          : activity.details

  updateActivity(payload.operationId, { progress: payload.overallPercent, details: detail })

  if (payload.status === 'started') {
    addLog('info', `Системный импорт начал обработку ${payload.totalFiles} файлов.`, payload.operationId)
  } else if (payload.status === 'copying' && payload.currentFile) {
    if (lastImportFileByOperation.get(payload.operationId) !== payload.currentFile) {
      lastImportFileByOperation.set(payload.operationId, payload.currentFile)
      addLog('info', `Начато копирование: ${payload.currentFile}`, payload.operationId)
    }
  } else if (payload.status === 'file-complete') {
    const succeeded = payload.fileSucceeded !== false
    addLog(succeeded ? 'success' : 'warning', `${succeeded ? 'Файл обработан' : 'Файл пропущен или завершён с ошибкой'}: ${payload.currentFile}`, payload.operationId)
  } else if (payload.status === 'finished') {
    addLog('info', 'Импорт завершил передачу файлов; ожидается итоговый результат операции.', payload.operationId)
    lastImportFileByOperation.delete(payload.operationId)
  }
}

export function getAppState() {
  return state
}

export function errorText(error: unknown): string {
  if (error instanceof Error) return error.message
  if (typeof error === 'string') return error
  try { return JSON.stringify(error) } catch { return 'Неизвестная ошибка' }
}

export function initializeAppState(): Promise<void> {
  if (initialization) return initialization
  initialization = (async () => {
    hydrate()
    const now = Date.now()
    const interrupted = state.activities.filter((activity) => activity.status === 'running')
    for (const activity of interrupted) {
      activity.status = 'interrupted'
      activity.updatedAt = now
      activity.details = 'Приложение завершилось до подтверждения результата. Проверь исходный файл и медиатеку.'
      state.logs.push({
        id: createId('log'), at: now, level: 'warning', activityId: activity.id,
        message: `Возможное прерывание: ${activity.title}. Состояние восстановлено при запуске.`,
      })
    }
    state.initialized = true
    persist(true)
    if (interrupted.length) {
      notify('warning', 'Восстановление состояния', `Найдено незавершённых операций: ${interrupted.length}. Проверь медиатеку и исходные файлы.`)
    }

    window.addEventListener('error', (event) => {
      notify('error', 'Ошибка приложения', event.message || 'Произошла неизвестная ошибка интерфейса.')
    })
    window.addEventListener('unhandledrejection', (event) => {
      notify('error', 'Ошибка фоновой операции', errorText(event.reason))
    })

    if (isTauri()) {
      try {
        if (!unlistenImportProgress) {
          unlistenImportProgress = await listen<ImportProgress>('import-progress', (event) => {
            trackImportProgress(event.payload)
          })
        }
      } catch (error) {
        notify('warning', 'Отслеживание импорта ограничено', errorText(error))
      }

      try {
        const recovery = await invoke<{ removed: string[]; failures: string[] }>('recover_interrupted_imports')
        if (recovery.removed.length) {
          const message = `Удалены временные остатки незавершённого импорта: ${recovery.removed.slice(0, 4).join(', ')}${recovery.removed.length > 4 ? ` и ещё ${recovery.removed.length - 4}` : ''}. Исходники восстановление не изменяло.`
          notify('warning', 'Восстановление импорта', message)
        } else {
          addLog('info', 'Проверка незавершённых файлов импорта выполнена: временных остатков нет.')
        }
        if (recovery.failures.length) {
          notify('warning', 'Не все временные файлы удалены', recovery.failures.slice(0, 3).join('\n'))
        }
      } catch (error) {
        notify('warning', 'Проверка восстановления не выполнена', errorText(error))
      }
    }
  })()
  return initialization
}
