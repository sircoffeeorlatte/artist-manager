import { reactive } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { notify } from './appState'

export type StartupPage =
  | '/'
  | '/gallery/videos'
  | '/gallery/photos'
  | '/gallery/archives'
  | '/database'
  | '/settings'
  | '/import'

export type VideoGalleryMode = 'grid' | 'tiktok'
export type GridKind = 'photo' | 'video' | 'archive'

export type AppTheme = 'dark' | 'light'
export type VisualMode = 'liquid-glass' | 'standard'

export interface UserAppearanceConfig {
  accentHue: number
  scale: number
  backgroundColor: string
  wallpaperPath: string
  theme: AppTheme
  visualMode: VisualMode
}

export interface UserConfig {
  version: 1
  appearance: UserAppearanceConfig
  video: {
    volume: number
    muted: boolean
    tiktokMuted: boolean
  }
  gridColumns: {
    photo: number
    video: number
    archive: number
  }
  videoGalleryMode: VideoGalleryMode
  startupPage: StartupPage
  libraryRoot: string
}

export const DEFAULT_USER_CONFIG: UserConfig = {
  version: 1,
  appearance: { accentHue: 210, scale: 100, backgroundColor: '#101114', wallpaperPath: '', theme: 'dark', visualMode: 'liquid-glass' },
  video: { volume: 1, muted: false, tiktokMuted: true },
  gridColumns: { photo: 3, video: 3, archive: 3 },
  videoGalleryMode: 'grid',
  startupPage: '/',
  libraryRoot: '',
}

const CONFIG_KEY = 'media-library:user-config:v1'
const LEGACY_APPEARANCE_KEY = 'media-library:appearance:v1'
const STARTUP_PAGES = new Set<StartupPage>([
  '/', '/gallery/videos', '/gallery/photos', '/gallery/archives', '/database', '/settings', '/import',
])
const GRID_SIZES = [2, 3, 4, 5, 6, 7]

const config = reactive<UserConfig>(structuredClone(DEFAULT_USER_CONFIG))
let loadPromise: Promise<void> | null = null
let saveTimer: number | undefined
let saveChain: Promise<void> = Promise.resolve()
let loaded = false

function asRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === 'object' ? value as Record<string, unknown> : {}
}

function boundedNumber(value: unknown, fallback: number, min: number, max: number): number {
  if (typeof value !== 'number' || !Number.isFinite(value)) return fallback
  return Math.min(max, Math.max(min, value))
}

function normalizeConfig(value: unknown): UserConfig {
  const input = asRecord(value)
  const appearance = asRecord(input.appearance)
  const video = asRecord(input.video)
  const grid = asRecord(input.gridColumns)
  const scaleValue = Math.round(boundedNumber(appearance.scale, 100, 80, 120) / 5) * 5
  const normalizeGrid = (value: unknown, fallback: number) => {
    const n = Math.round(boundedNumber(value, fallback, 2, 7))
    return GRID_SIZES.includes(n) ? n : fallback
  }
  const startup = typeof input.startupPage === 'string' && STARTUP_PAGES.has(input.startupPage as StartupPage)
    ? input.startupPage as StartupPage
    : DEFAULT_USER_CONFIG.startupPage

  const backgroundCandidate = typeof appearance.backgroundColor === 'string' ? appearance.backgroundColor.trim() : ''
  const backgroundColor = /^#[0-9a-fA-F]{6}$/.test(backgroundCandidate) ? backgroundCandidate.toLowerCase() : DEFAULT_USER_CONFIG.appearance.backgroundColor
  const wallpaperCandidate = typeof appearance.wallpaperPath === 'string' ? appearance.wallpaperPath.trim().replace(/\\/g, '/') : ''
  const wallpaperPath = wallpaperCandidate.startsWith('wallpapers/') && !wallpaperCandidate.split('/').some((part) => part === '..' || part === '.') ? wallpaperCandidate : ''

  return {
    version: 1,
    appearance: {
      accentHue: Math.round(boundedNumber(appearance.accentHue, 210, 0, 360)),
      scale: Math.min(120, Math.max(80, scaleValue)),
      backgroundColor,
      wallpaperPath,
      theme: appearance.theme === 'light' ? 'light' : 'dark',
      visualMode: appearance.visualMode === 'standard' ? 'standard' : 'liquid-glass',
    },
    video: {
      volume: boundedNumber(video.volume, 1, 0, 1),
      muted: typeof video.muted === 'boolean' ? video.muted : false,
      tiktokMuted: typeof video.tiktokMuted === 'boolean' ? video.tiktokMuted : true,
    },
    gridColumns: {
      photo: normalizeGrid(grid.photo, 3),
      video: normalizeGrid(grid.video, 3),
      archive: normalizeGrid(grid.archive, 3),
    },
    videoGalleryMode: input.videoGalleryMode === 'tiktok' ? 'tiktok' : 'grid',
    startupPage: startup,
    libraryRoot: typeof input.libraryRoot === 'string' ? input.libraryRoot : '',
  }
}

function localConfig(): UserConfig | null {
  try {
    const stored = localStorage.getItem(CONFIG_KEY)
    if (stored) return normalizeConfig(JSON.parse(stored) as unknown)
    const legacyAppearance = localStorage.getItem(LEGACY_APPEARANCE_KEY)
    const legacyRoot = localStorage.getItem('media-library.root-path') ?? ''
    if (legacyAppearance || legacyRoot) {
      return normalizeConfig({
        ...DEFAULT_USER_CONFIG,
        appearance: legacyAppearance ? JSON.parse(legacyAppearance) as unknown : DEFAULT_USER_CONFIG.appearance,
        libraryRoot: legacyRoot,
      })
    }
  } catch {
    // Ignore malformed old local settings and recover from defaults.
  }
  return null
}

function snapshot(): UserConfig {
  return normalizeConfig({
    version: config.version,
    appearance: { ...config.appearance },
    video: { ...config.video },
    gridColumns: { ...config.gridColumns },
    videoGalleryMode: config.videoGalleryMode,
    startupPage: config.startupPage,
    libraryRoot: config.libraryRoot,
  })
}

function writeLocalSnapshot(value: UserConfig): void {
  try { localStorage.setItem(CONFIG_KEY, JSON.stringify(value)) }
  catch { /* The native JSON configuration is the primary store in desktop mode. */ }
}

function scheduleSave(): void {
  if (!loaded) return
  writeLocalSnapshot(snapshot())
  if (saveTimer !== undefined) window.clearTimeout(saveTimer)
  saveTimer = window.setTimeout(() => {
    saveTimer = undefined
    const data = snapshot()
    if (isTauri()) {
      // Serialize writes to prevent a slower old snapshot overwriting newer preferences.
      saveChain = saveChain
        .then(() => invoke<void>('save_user_config', { config: data }))
        .catch((error: unknown) => {
          notify('error', 'Не удалось сохранить конфиг', error instanceof Error ? error.message : String(error))
        })
    }
  }, 350)
}

export function getUserConfig(): UserConfig {
  return config
}

export function updateUserConfig(patch: Partial<UserConfig>): void {
  if (patch.appearance) Object.assign(config.appearance, patch.appearance)
  if (patch.video) Object.assign(config.video, patch.video)
  if (patch.gridColumns) Object.assign(config.gridColumns, patch.gridColumns)
  if (patch.videoGalleryMode) config.videoGalleryMode = patch.videoGalleryMode
  if (patch.startupPage) config.startupPage = patch.startupPage
  if (patch.libraryRoot !== undefined) config.libraryRoot = patch.libraryRoot
  scheduleSave()
}

export function setGridColumns(kind: GridKind, columns: number): void {
  if (!GRID_SIZES.includes(columns)) return
  updateUserConfig({ gridColumns: { ...config.gridColumns, [kind]: columns } })
}

export async function flushUserConfig(): Promise<void> {
  if (saveTimer !== undefined) {
    window.clearTimeout(saveTimer)
    saveTimer = undefined
    const data = snapshot()
    writeLocalSnapshot(data)
    if (isTauri()) {
      saveChain = saveChain.then(() => invoke<void>('save_user_config', { config: data }))
    }
  }
  await saveChain
}

export async function loadUserConfig(): Promise<void> {
  if (loadPromise) return loadPromise
  loadPromise = (async () => {
    let loadedNative: UserConfig | null = null
    if (isTauri()) {
      try {
        loadedNative = await invoke<UserConfig | null>('load_user_config')
      } catch (error) {
        notify('warning', 'Не удалось загрузить конфиг', error instanceof Error ? error.message : String(error))
      }
    }

    if (loadedNative) {
      Object.assign(config, normalizeConfig(loadedNative))
      // Existing native config is authoritative; localStorage is only a recovery/preview cache.
    } else {
      const legacy = localConfig()
      if (legacy) Object.assign(config, legacy)
      if (isTauri() && !config.libraryRoot) {
        try { config.libraryRoot = await invoke<string>('get_saved_library_root') || '' } catch { /* no root saved yet */ }
      }
      // Create the first JSON configuration file (and migrate older local settings).
      loaded = true
      writeLocalSnapshot(snapshot())
      if (isTauri()) {
        try { await invoke<void>('save_user_config', { config: snapshot() }) }
        catch (error) { notify('error', 'Не удалось создать конфиг', error instanceof Error ? error.message : String(error)) }
      }
    }
    Object.assign(config, normalizeConfig(config))
    loaded = true
    writeLocalSnapshot(snapshot())
  })()
  return loadPromise
}

export async function getUserConfigPath(): Promise<string> {
  if (!isTauri()) return 'Браузерный режим: настройки сохраняются в localStorage.'
  return invoke<string>('get_user_config_path')
}
