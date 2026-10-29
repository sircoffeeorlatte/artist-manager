import { invoke, isTauri } from '@tauri-apps/api/core'
import type { ContentCategory, ImportResult, LibraryMediaItem, MediaKind } from '../types/media'

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

export async function listLibraryMedia(rootPath: string, kind: MediaKind): Promise<LibraryMediaItem[]> {
  requireDesktop('Чтение медиатеки')
  if (!rootPath) return []
  await registerLibraryRoot(rootPath)
  return invoke<LibraryMediaItem[]>('list_library_media', { rootPath, kind })
}
