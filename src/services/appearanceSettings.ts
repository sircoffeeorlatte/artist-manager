import { getUserConfig, updateUserConfig, type UserAppearanceConfig } from './userConfig'

export type AppearanceSettings = UserAppearanceConfig

export const DEFAULT_APPEARANCE_SETTINGS: AppearanceSettings = {
  accentHue: 210,
  scale: 100,
  backgroundColor: '#101114',
  wallpaperPath: '',
  theme: 'dark',
  visualMode: 'liquid-glass',
}

function clamp(value: unknown, fallback: number, min: number, max: number): number {
  if (typeof value !== 'number' || !Number.isFinite(value)) return fallback
  return Math.min(max, Math.max(min, value))
}

function normalizeAppearance(value: Partial<AppearanceSettings> | null | undefined): AppearanceSettings {
  const accentHue = Math.round(clamp(value?.accentHue, DEFAULT_APPEARANCE_SETTINGS.accentHue, 0, 360))
  const rawScale = clamp(value?.scale, DEFAULT_APPEARANCE_SETTINGS.scale, 80, 120)
  const scale = Math.min(120, Math.max(80, Math.round(rawScale / 5) * 5))
  const candidate = typeof value?.backgroundColor === 'string' ? value.backgroundColor.trim() : ''
  const backgroundColor = /^#[0-9a-fA-F]{6}$/.test(candidate) ? candidate.toLowerCase() : DEFAULT_APPEARANCE_SETTINGS.backgroundColor
  const wallpaperCandidate = typeof value?.wallpaperPath === 'string' ? value.wallpaperPath.trim().replace(/\\/g, '/') : ''
  const wallpaperPath = wallpaperCandidate.startsWith('wallpapers/') && !wallpaperCandidate.split('/').some((part) => part === '..' || part === '.') ? wallpaperCandidate : ''
  const theme = value?.theme === 'light' ? 'light' : 'dark'
  const visualMode = value?.visualMode === 'standard' ? 'standard' : 'liquid-glass'
  return { accentHue, scale, backgroundColor, wallpaperPath, theme, visualMode }
}

function hexToRgb(hex: string): [number, number, number] {
  const value = hex.replace(/^#/, '')
  return [0, 2, 4].map((offset) => Number.parseInt(value.slice(offset, offset + 2), 16)) as [number, number, number]
}

function rgbToHex([red, green, blue]: [number, number, number]): string {
  return `#${[red, green, blue].map((channel) => Math.round(channel).toString(16).padStart(2, '0')).join('')}`
}

function rgbToHsl(hex: string): [number, number, number] {
  const [red, green, blue] = hexToRgb(hex).map((channel) => channel / 255) as [number, number, number]
  const max = Math.max(red, green, blue)
  const min = Math.min(red, green, blue)
  const delta = max - min
  let hue = 0
  let saturation = 0
  const lightness = (max + min) / 2

  if (delta !== 0) {
    saturation = delta / (1 - Math.abs(2 * lightness - 1))
    if (max === red) hue = 60 * (((green - blue) / delta) % 6)
    else if (max === green) hue = 60 * ((blue - red) / delta + 2)
    else hue = 60 * ((red - green) / delta + 4)
  }
  return [(hue + 360) % 360, saturation, lightness]
}

function hslToHex(hue: number, saturation: number, lightness: number): string {
  const chroma = (1 - Math.abs(2 * lightness - 1)) * saturation
  const segment = hue / 60
  const x = chroma * (1 - Math.abs((segment % 2) - 1))
  let red = 0
  let green = 0
  let blue = 0
  if (segment < 1) [red, green, blue] = [chroma, x, 0]
  else if (segment < 2) [red, green, blue] = [x, chroma, 0]
  else if (segment < 3) [red, green, blue] = [0, chroma, x]
  else if (segment < 4) [red, green, blue] = [0, x, chroma]
  else if (segment < 5) [red, green, blue] = [x, 0, chroma]
  else [red, green, blue] = [chroma, 0, x]
  const offset = lightness - chroma / 2
  return rgbToHex([(red + offset) * 255, (green + offset) * 255, (blue + offset) * 255])
}

function relativeLuminance(hex: string): number {
  const channels = hexToRgb(hex).map((channel) => {
    const normalized = channel / 255
    return normalized <= 0.04045 ? normalized / 12.92 : ((normalized + 0.055) / 1.055) ** 2.4
  }) as [number, number, number]
  return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2]
}

/**
 * Preserve the user's chosen background hue while gently shifting only its
 * lightness to keep the currently selected theme's foreground text readable.
 * The exact color remains in user-config.json; only the rendered canvas is tuned.
 */
function contrastSafeBackground(color: string, theme: 'light' | 'dark'): string {
  const targetLuminance = theme === 'light' ? 0.68 : 0.13
  const currentLuminance = relativeLuminance(color)
  const alreadySafe = theme === 'light' ? currentLuminance >= targetLuminance : currentLuminance <= targetLuminance
  if (alreadySafe) return color

  const [hue, saturation, currentLightness] = rgbToHsl(color)
  let low = theme === 'light' ? currentLightness : 0
  let high = theme === 'light' ? 1 : currentLightness
  for (let iteration = 0; iteration < 24; iteration += 1) {
    const middle = (low + high) / 2
    const candidate = hslToHex(hue, saturation, middle)
    const safe = theme === 'light'
      ? relativeLuminance(candidate) >= targetLuminance
      : relativeLuminance(candidate) <= targetLuminance
    if (theme === 'light') {
      if (safe) high = middle
      else low = middle
    } else {
      if (safe) low = middle
      else high = middle
    }
  }
  return hslToHex(hue, saturation, theme === 'light' ? high : low)
}

/** Select readable foreground text for the chosen accent color. */
function accentForeground(hue: number): string {
  const saturation = 0.77
  const lightness = 0.55
  const chroma = (1 - Math.abs(2 * lightness - 1)) * saturation
  const hueSegment = hue / 60
  const x = chroma * (1 - Math.abs((hueSegment % 2) - 1))
  let red = 0
  let green = 0
  let blue = 0

  if (hueSegment < 1) [red, green, blue] = [chroma, x, 0]
  else if (hueSegment < 2) [red, green, blue] = [x, chroma, 0]
  else if (hueSegment < 3) [red, green, blue] = [0, chroma, x]
  else if (hueSegment < 4) [red, green, blue] = [0, x, chroma]
  else if (hueSegment < 5) [red, green, blue] = [x, 0, chroma]
  else [red, green, blue] = [chroma, 0, x]

  const m = lightness - chroma / 2
  const linearize = (channel: number) => {
    const value = channel + m
    return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4
  }
  const luminance = 0.2126 * linearize(red) + 0.7152 * linearize(green) + 0.0722 * linearize(blue)
  const darkTextLuminance = 0.008
  const darkContrast = (luminance + 0.05) / (darkTextLuminance + 0.05)
  const lightContrast = 1.05 / (luminance + 0.05)
  return darkContrast >= lightContrast ? '#11160c' : '#f7f9fc'
}

export function getSavedAppearanceSettings(): AppearanceSettings {
  return { ...getUserConfig().appearance }
}

export function applyAppearanceSettings(settings: AppearanceSettings, persist = true): void {
  const normalized = normalizeAppearance(settings)
  const { accentHue, scale, backgroundColor, theme, visualMode } = normalized

  if (typeof document !== 'undefined') {
    const root = document.documentElement
    root.style.setProperty('--accent-hue', String(accentHue))
    root.style.setProperty('--accent', `hsl(${accentHue} 77% 55%)`)
    root.style.setProperty('--accent-strong', `hsl(${accentHue} 82% 72%)`)
    root.style.setProperty('--accent-soft-text', theme === 'light' ? `hsl(${accentHue} 67% 31%)` : `hsl(${accentHue} 75% 82%)`)
    root.style.setProperty('--accent-text', accentForeground(accentHue))
    root.style.setProperty('--ui-scale', String(scale / 100))

    // An untouched dark default should become a light canvas in light theme.
    // Custom colors remain saved exactly, but are gently theme-tinted at runtime
    // when their luminance would make the current theme's text unreadable.
    const requestedBackground = theme === 'light' && backgroundColor.toLowerCase() === DEFAULT_APPEARANCE_SETTINGS.backgroundColor
      ? '#f2f5f9'
      : backgroundColor
    root.style.setProperty('--user-background-color', requestedBackground)
    root.style.setProperty('--theme-base-bg', theme === 'light' ? '#f2f5f9' : '#101114')
    root.style.setProperty('--app-bg', contrastSafeBackground(requestedBackground, theme))
    root.dataset.theme = theme
    root.dataset.visualMode = visualMode
    root.style.colorScheme = theme
  }

  if (persist) updateUserConfig({ appearance: normalized })
}
