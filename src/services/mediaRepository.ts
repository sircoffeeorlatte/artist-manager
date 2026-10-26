import { demoMediaRecords } from '../data/demoMedia'
import type { MediaKind, MediaRecord } from '../types/media'

/**
 * Временный репозиторий для каркаса интерфейса.
 * Следующий этап: заменить на SQLite-реализацию, не меняя страницы.
 */
export interface MediaRepository {
  listAll(): Promise<MediaRecord[]>
  listByKind(kind: MediaKind): Promise<MediaRecord[]>
}

export const mediaRepository: MediaRepository = {
  async listAll() {
    return [...demoMediaRecords]
  },
  async listByKind(kind) {
    return demoMediaRecords.filter((record) => record.kind === kind)
  },
}
