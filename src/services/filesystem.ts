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
  kind: Extract<MediaKind, 'photo' | 'video'>
  mode: 'copy' | 'move'
}): Promise<ImportResult> {
  requireDesktop('Импорт файлов')
  return invoke<ImportResult>('import_media_files', options)
}

export async function listLibraryMedia(rootPath: string, kind: Extract<MediaKind, 'photo' | 'video'>): Promise<LibraryMediaItem[]> {
  requireDesktop('Чтение медиатеки')
  if (!rootPath) return []
  await registerLibraryRoot(rootPath)
  return invoke<LibraryMediaItem[]>('list_library_media', { rootPath, kind })
}
