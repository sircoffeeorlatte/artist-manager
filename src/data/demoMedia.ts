import type { MediaRecord } from '../types/media'

/** Только данные-примеры для проверки интерфейса. В SQLite они не записываются. */
export const demoMediaRecords: MediaRecord[] = [
  {
    id: 'demo-0001', kind: 'video', originalName: 'sample_animation.mp4',
    relativePath: 'Videos/sample_animation.mp4', extension: '.mp4', sizeBytes: 184_320_000,
    createdAt: '2025-08-19T14:20:00', modifiedAt: '2025-08-20T10:15:00', importedAt: '2026-10-20T11:00:00',
    category: 'anime', tags: ['пример', 'анимация'], isDemo: true,
  },
  {
    id: 'demo-0002', kind: 'video', originalName: 'street_clip_01.mkv',
    relativePath: 'Videos/street_clip_01.mkv', extension: '.mkv', sizeBytes: 762_400_000,
    createdAt: '2024-11-03T22:15:00', modifiedAt: '2024-11-03T22:15:00', importedAt: '2026-10-20T11:01:00',
    category: 'irl', tags: ['пример', 'улица'], isDemo: true,
  },
  {
    id: 'demo-0003', kind: 'photo', originalName: 'character_render.png',
    relativePath: 'Photos/character_render.png', extension: '.png', sizeBytes: 8_420_112,
    createdAt: '2025-02-09T09:30:00', modifiedAt: '2025-02-09T09:30:00', importedAt: '2026-10-20T11:02:00',
    category: '3d', tags: ['пример', 'персонаж', 'рендер'], isDemo: true,
  },
  {
    id: 'demo-0004', kind: 'photo', originalName: 'reference_042.webp',
    relativePath: 'Photos/reference_042.webp', extension: '.webp', sizeBytes: 1_280_000,
    createdAt: '2025-06-01T18:42:00', modifiedAt: '2025-06-01T18:42:00', importedAt: '2026-10-20T11:03:00',
    category: 'anime', tags: ['пример', 'референс'], isDemo: true,
  },
  {
    id: 'demo-0005', kind: 'photo', originalName: 'camera_2024_0512.jpg',
    relativePath: 'Photos/camera_2024_0512.jpg', extension: '.jpg', sizeBytes: 5_740_000,
    createdAt: '2024-05-12T16:04:00', modifiedAt: '2024-05-12T16:04:00', importedAt: '2026-10-20T11:04:00',
    category: 'irl', tags: ['пример', 'камера'], isDemo: true,
  },
  {
    id: 'demo-0006', kind: 'archive', originalName: 'asset_pack.7z',
    relativePath: 'Archives/asset_pack.7z', extension: '.7z', sizeBytes: 2_147_483_648,
    createdAt: '2025-10-30T08:00:00', modifiedAt: '2025-10-30T08:00:00', importedAt: '2026-10-20T11:05:00',
    category: '3d', tags: ['пример', 'ассеты'], isDemo: true,
  },
  {
    id: 'demo-0007', kind: 'archive', originalName: 'saved_images.zip',
    relativePath: 'Archives/saved_images.zip', extension: '.zip', sizeBytes: 425_600_000,
    createdAt: '2025-12-11T12:12:00', modifiedAt: '2025-12-11T12:12:00', importedAt: '2026-10-20T11:06:00',
    category: 'anime', tags: ['пример', 'изображения'], isDemo: true,
  },
]
