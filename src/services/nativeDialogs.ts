import { isTauri } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

export type PickableMediaKind = 'photo' | 'video'

const PHOTO_EXTENSIONS = ['jpg', 'jpeg', 'png', 'webp', 'bmp', 'tif', 'tiff', 'avif', 'heic']
const VIDEO_EXTENSIONS = ['mp4', 'mkv', 'avi', 'mov', 'webm', 'm4v', 'mpg', 'mpeg', 'wmv', 'flv', '3gp', 'ogv', 'gif']

/** Opens the native Windows file picker. GIFs are treated as animated media in the video gallery. */
export async function selectSourceFiles(kind: PickableMediaKind): Promise<string[]> {
  if (!isTauri()) {
    throw new Error('Реальный импорт файлов доступен в настольном приложении. В браузерном режиме можно проверить только временное превью.')
  }

  const extensions = kind === 'photo' ? PHOTO_EXTENSIONS : VIDEO_EXTENSIONS
  const selected = await open({
    multiple: true,
    directory: false,
    title: kind === 'photo' ? 'Выберите фотографии' : 'Выберите видео и GIF',
    filters: [
      { name: kind === 'photo' ? 'Фотографии' : 'Видео и GIF', extensions },
      { name: 'Все файлы', extensions: ['*'] },
    ],
  })

  if (!selected) return []
  return Array.isArray(selected) ? selected : [selected]
}

/** Select the parent folder in which the library root will be created. */
export async function selectLibraryParentFolder(): Promise<string | null> {
  if (!isTauri()) {
    throw new Error('Выбор реальной папки Windows доступен в настольном приложении Tauri.')
  }
  const selected = await open({
    directory: true,
    multiple: false,
    title: 'Выберите место для папки медиатеки',
  })
  return typeof selected === 'string' ? selected : null
}
