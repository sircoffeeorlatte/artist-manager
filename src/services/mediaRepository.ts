import type { LibraryMediaItem, MediaKind, MediaRecord } from '../types/media'
import { getMediaIndexStatus, listIndexedMedia, listLibraryMedia, refreshMediaIndex, setMediaTags, type MediaIndexStatus, type MediaIndexResult } from './filesystem'

/** The repository reads the persisted SQLite index; it does not scan the media folders on each query. */
export interface MediaRepository {
  listAll(rootPath: string, includeMissing?: boolean): Promise<MediaRecord[]>
  listByKind(rootPath: string, kind: MediaKind, refreshIndex?: boolean): Promise<LibraryMediaItem[]>
  refreshIndex(rootPath: string): Promise<MediaIndexResult>
  getIndexStatus(): Promise<MediaIndexStatus>
  setTags(rootPath: string, relativePath: string, tags: string[]): Promise<void>
}

export const mediaRepository: MediaRepository = {
  async listAll(rootPath, includeMissing = true) {
    return listIndexedMedia(rootPath, includeMissing)
  },
  async listByKind(rootPath, kind, refreshIndex = false) {
    return listLibraryMedia(rootPath, kind, refreshIndex)
  },
  async refreshIndex(rootPath) {
    return refreshMediaIndex(rootPath)
  },
  async getIndexStatus() {
    return getMediaIndexStatus()
  },
  async setTags(rootPath, relativePath, tags) {
    return setMediaTags(rootPath, relativePath, tags)
  },
}
