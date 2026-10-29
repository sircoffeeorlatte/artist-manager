import { invoke, isTauri } from '@tauri-apps/api/core'
import { requestFolderAccessConsent } from './folderAccessConsent'

function requireDesktop(): void {
  if (!isTauri()) throw new Error('Открытие папок в Проводнике доступно только в настольной версии приложения.')
}

async function requireFolderConsent(): Promise<void> {
  const allowed = await requestFolderAccessConsent()
  if (!allowed) throw new Error('Открытие папки отменено: согласие на открытие папок в Проводнике не предоставлено.')
}

function containingFolder(filePath: string): string {
  const target = filePath.trim().replace(/[\\/]+$/, '')
  const separator = Math.max(target.lastIndexOf('\\'), target.lastIndexOf('/'))
  if (separator < 0) throw new Error('Не удалось определить родительскую папку файла.')

  let parent = target.slice(0, separator)
  // Preserve the root slash for paths such as C:\\file.sqlite3.
  if (/^[a-zA-Z]:$/.test(parent)) parent += target.includes('\\') ? '\\' : '/'
  if (!parent) parent = target.startsWith('\\\\') ? '\\\\' : '/'
  return parent
}

/** Open a folder directly in the operating system's file manager after app-level consent. */
export async function openFolderPath(path: string): Promise<void> {
  requireDesktop()
  const target = path.trim()
  if (!target) throw new Error('Путь к папке не указан.')
  await requireFolderConsent()
  await invoke<void>('open_folder_in_explorer', { path: target })
}

/** Open the folder which contains a file, not the file itself. */
export async function openContainingFolder(filePath: string): Promise<void> {
  requireDesktop()
  const parent = containingFolder(filePath)
  await requireFolderConsent()
  await invoke<void>('open_folder_in_explorer', { path: parent })
}

/** Try to open the target folder. If it does not exist yet, open its parent. */
export async function openFolderOrParent(path: string): Promise<void> {
  requireDesktop()
  const target = path.trim()
  if (!target) throw new Error('Путь к папке не указан.')
  await requireFolderConsent()

  try {
    await invoke<void>('open_folder_in_explorer', { path: target })
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    try {
      await invoke<void>('open_folder_in_explorer', { path: containingFolder(target) })
    } catch {
      throw new Error(message)
    }
  }
}
