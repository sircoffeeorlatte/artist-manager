import { invoke, isTauri } from '@tauri-apps/api/core'

const LIBRARY_ROOT_KEY = 'media-library.root-path'

export function getSavedLibraryRoot(): string {
  try {
    return localStorage.getItem(LIBRARY_ROOT_KEY) ?? ''
  } catch {
    return ''
  }
}

export function saveLibraryRoot(path: string): void {
  try {
    localStorage.setItem(LIBRARY_ROOT_KEY, path)
  } catch {
    // The native settings file remains the source of truth in Desktop mode.
  }
}

export async function loadLibraryRoot(): Promise<string> {
  if (isTauri()) {
    try {
      const nativePath = await invoke<string>('get_saved_library_root')
      if (nativePath.trim()) {
        saveLibraryRoot(nativePath)
        return nativePath
      }
      return ''
    } catch {
      // A previous/older build may not have the native settings command.
    }
  }
  return getSavedLibraryRoot()
}

export async function persistLibraryRoot(path: string): Promise<void> {
  saveLibraryRoot(path)
  if (isTauri()) await invoke<void>('save_library_root', { rootPath: path })
}

export function joinPath(parent: string, child: string): string {
  const cleanParent = parent.replace(/[\\/]+$/, '')
  const cleanChild = child.replace(/^[\\/]+|[\\/]+$/g, '')
  const separator = parent.includes('\\') ? '\\' : '/'
  return `${cleanParent}${separator}${cleanChild}`
}

export function splitPath(path: string): { parent: string; name: string } {
  const match = path.match(/^(.*?)[\\/]+([^\\/]+)[\\/]*$/)
  if (!match) return { parent: '', name: path }
  return { parent: match[1], name: match[2] }
}
