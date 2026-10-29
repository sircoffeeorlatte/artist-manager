import { beginActivity, finishActivity, updateActivity } from './appState'

export interface VideoLoadTracker {
  start: (reason?: string) => void
  ready: () => void
  fail: (reason?: string) => void
  stop: (reason?: string) => void
}

/** Tracks browser video loading/buffering in the same app-wide task list as native playback. */
export function createVideoLoadTracker(getTitle: () => string, context = 'Просмотр видео'): VideoLoadTracker {
  let activityId: string | null = null
  const title = () => getTitle().trim() || 'Видео'

  function start(reason = 'Загрузка данных'): void {
    if (activityId) {
      updateActivity(activityId, { details: `${title()} · ${context}: ${reason}` })
      return
    }
    activityId = beginActivity({
      kind: 'video',
      title: 'Загрузка видео',
      details: `${title()} · ${context}: ${reason}`,
    })
  }

  function ready(): void {
    if (!activityId) return
    finishActivity(activityId, 'success', `${title()} · ${context}: видео доступно для воспроизведения.`)
    activityId = null
  }

  function fail(reason = 'Файл удалён, диск отключён или кодек не поддерживается.'): void {
    if (!activityId) {
      start('Ошибка загрузки')
    }
    if (!activityId) return
    finishActivity(activityId, 'error', `${title()} · ${context}: ${reason}`)
    activityId = null
  }

  function stop(reason = 'Загрузка остановлена вместе с просмотром.'): void {
    if (!activityId) return
    finishActivity(activityId, 'success', `${title()} · ${context}: ${reason}`)
    activityId = null
  }

  return { start, ready, fail, stop }
}
