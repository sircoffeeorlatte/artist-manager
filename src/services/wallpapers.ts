import { invoke, isTauri } from '@tauri-apps/api/core'
import { registerLibraryRoot } from './filesystem'

export interface WallpaperItem {
  name: string
  relativePath: string
  absolutePath: string
  extension: string
  kind: 'image' | 'video'
  sizeBytes: number
  modifiedAt: number | null
}

export function joinLibraryPath(rootPath: string, relativePath: string): string {
  const separator = rootPath.includes('\\') ? '\\' : '/'
  const root = rootPath.replace(/[\\/]+$/, '')
  const relative = relativePath.replace(/[\\/]+/g, separator).replace(/^[\\/]+/, '')
  return `${root}${separator}${relative}`
}

export async function listWallpapers(rootPath: string): Promise<WallpaperItem[]> {
  if (!isTauri()) throw new Error('Библиотека обоев доступна в настольной версии приложения.')
  if (!rootPath.trim()) throw new Error('Сначала укажи расположение медиатеки в настройках.')
  await registerLibraryRoot(rootPath)
  return invoke<WallpaperItem[]>('list_wallpapers', { rootPath })
}

export async function importWallpaper(sourcePath: string, rootPath: string): Promise<WallpaperItem> {
  if (!isTauri()) throw new Error('Добавление обоев доступно в настольной версии приложения.')
  if (!rootPath.trim()) throw new Error('Сначала создай медиатеку в настройках.')
  await registerLibraryRoot(rootPath)
  return invoke<WallpaperItem>('import_wallpaper', { sourcePath, rootPath })
}
