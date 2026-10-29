import { invoke, isTauri } from '@tauri-apps/api/core'
import type { ContentCategory, ImportResult, LibraryMediaItem, MediaKind } from '../types/media'

export interface MediaIndexStatus {
  databasePath: string
  libraryRoot: string
  isScanning: boolean
  lastStartedAt: number | null
  lastCompletedAt: number | null
  lastFullScanAt: number | null
  lastSeenFileCount: number
  availableCount: number
  missingCount: number
  lastError: string | null
}

export interface MediaIndexResult {
  scannedFiles: number
  addedFiles: number
  updatedFiles: number
  missingFiles: number
  completedAt: number
}

export function requireDesktop(feature: string): void {
  if (!isTauri()) throw new Error(`${feature} доступно только в настольной версии. Запустите run-desktop.cmd.`)
}

export async function createLibraryStructure(rootPath: string): Promise<string> {
  requireDesktop('Создание папок медиатеки')
  return invoke<string>('create_library_structure', { rootPath })
}

export async function registerLibraryRoot(rootPath: string): Promise<void> {
  if (!isTauri() || !rootPath) return
  await invoke<void>('register_library_root', { rootPath })
}

export async function importMediaFiles(options: {
  sourcePaths: string[]
  rootPath: string
  category: ContentCategory
  kind: MediaKind
  mode: 'copy' | 'move'
  operationId: string
}): Promise<ImportResult> {
  requireDesktop('Импорт файлов')
  const result = await invoke<ImportResult>('import_media_files', options)
  if (typeof window !== 'undefined') {
    window.dispatchEvent(new CustomEvent('media-library:import-complete', {
      detail: {
        operationId: options.operationId,
        importedCount: result.imported.length,
        warningCount: result.warnings.length,
      },
    }))
  }
  return result
}

export async function listLibraryMedia(rootPath: string, kind: MediaKind, refreshIndex = false): Promise<LibraryMediaItem[]> {
  requireDesktop('Чтение медиатеки')
  if (!rootPath) return []
  await registerLibraryRoot(rootPath)
  if (refreshIndex) await refreshMediaIndex(rootPath)
  return invoke<LibraryMediaItem[]>('list_library_media', { rootPath, kind })
}

export async function refreshMediaIndex(rootPath: string): Promise<MediaIndexResult> {
  requireDesktop('Индексация медиатеки')
  if (!rootPath) throw new Error('Сначала выбери корневую папку медиатеки.')
  await registerLibraryRoot(rootPath)
  return invoke<MediaIndexResult>('refresh_media_index', { rootPath })
}

export async function getMediaIndexStatus(): Promise<MediaIndexStatus> {
  requireDesktop('Состояние SQLite-индекса')
  return invoke<MediaIndexStatus>('get_media_index_status')
}

export async function listIndexedMedia(rootPath: string, includeMissing = true): Promise<LibraryMediaItem[]> {
  requireDesktop('Чтение SQLite-индекса')
  if (!rootPath) return []
  await registerLibraryRoot(rootPath)
  return invoke<LibraryMediaItem[]>('list_indexed_media', { rootPath, kind: null, includeMissing })
}

export async function setMediaTags(rootPath: string, relativePath: string, tags: string[]): Promise<void> {
  requireDesktop('Сохранение тегов')
  await invoke<void>('set_media_tags', { rootPath, relativePath, tags })
}
