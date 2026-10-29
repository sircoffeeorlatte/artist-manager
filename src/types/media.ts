export type MediaKind = 'video' | 'photo' | 'archive'
export type ContentCategory = '3d' | 'anime' | 'irl'
/** Creator/author labels will be represented as tags in a later iteration. */
export type TagType = 'general' | 'creator'

export interface MediaTag {
  id: string | number
  name: string
  type: TagType
}

export interface MediaRecord {
  id: string
  kind: MediaKind
  originalName: string
  relativePath: string
  /** Absolute path is used only by the desktop viewer; paths should not be persisted in portable exports. */
  absolutePath?: string
  extension: string
  sizeBytes: number
  createdAt: string | number | null
  modifiedAt: string | number | null
  importedAt: string | number | null
  category: ContentCategory | null
  tags: string[]
  /** Future author page will use tags with type="creator". No author UI is implemented yet. */
  creatorTagId?: string | number | null
  /** True for temporary browser-only uploads and demo rows. */
  isDemo: boolean
  previewOnly?: boolean
}

export interface LibraryMediaItem extends MediaRecord {
  absolutePath: string
  previewOnly?: boolean
}

export interface ImportResult {
  imported: LibraryMediaItem[]
  warnings: string[]
}

export interface ImportProgress {
  operationId: string
  overallPercent: number
  fileIndex: number
  totalFiles: number
  currentFile: string
  bytesCopied: number
  fileSize: number
  completedFiles: number
  fileSucceeded?: boolean
  status: 'started' | 'copying' | 'file-complete' | 'finished'
}

export const MEDIA_KIND_LABELS: Record<MediaKind, string> = {
  video: 'Видео',
  photo: 'Фото',
  archive: 'Архив',
}

export const CATEGORY_LABELS: Record<ContentCategory, string> = {
  '3d': '3D',
  anime: 'Аниме',
  irl: 'IRL',
}

export const CATEGORY_OPTIONS: { value: ContentCategory; label: string; folderName: string }[] = [
  { value: 'irl', label: 'IRL', folderName: 'IRL' },
  { value: 'anime', label: 'Аниме', folderName: 'Аниме' },
  { value: '3d', label: '3D', folderName: '3D' },
]
