export interface AppearanceSettings {
  /** Hue in degrees, used to derive the application's accent palette. */
  accentHue: number
  /** Interface zoom percentage. Supported range: 80–120. */
  scale: number
}

export const DEFAULT_APPEARANCE_SETTINGS: AppearanceSettings = {
  accentHue: 82,
  scale: 100,
}

const STORAGE_KEY = 'media-library:appearance:v1'

function clamp(value: unknown, fallback: number, min: number, max: number): number {
  if (typeof value !== 'number' || !Number.isFinite(value)) return fallback
  return Math.min(max, Math.max(min, value))
}

function normalizeAppearance(value: Partial<AppearanceSettings> | null | undefined): AppearanceSettings {
  const accentHue = Math.round(clamp(value?.accentHue, DEFAULT_APPEARANCE_SETTINGS.accentHue, 0, 360))
  const rawScale = clamp(value?.scale, DEFAULT_APPEARANCE_SETTINGS.scale, 80, 120)
  const scale = Math.min(120, Math.max(80, Math.round(rawScale / 5) * 5))
  return { accentHue, scale }
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
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (!stored) return { ...DEFAULT_APPEARANCE_SETTINGS }
    const parsed: unknown = JSON.parse(stored)
    if (!parsed || typeof parsed !== 'object') return { ...DEFAULT_APPEARANCE_SETTINGS }
    return normalizeAppearance(parsed as Partial<AppearanceSettings>)
  } catch {
    return { ...DEFAULT_APPEARANCE_SETTINGS }
  }
}

export function applyAppearanceSettings(settings: AppearanceSettings, persist = true): void {
  const normalized = normalizeAppearance(settings)
  const { accentHue, scale } = normalized

  if (typeof document !== 'undefined') {
    const root = document.documentElement
    root.style.setProperty('--accent', `hsl(${accentHue} 77% 55%)`)
    root.style.setProperty('--accent-strong', `hsl(${accentHue} 82% 72%)`)
    root.style.setProperty('--accent-soft-text', `hsl(${accentHue} 75% 82%)`)
    root.style.setProperty('--accent-text', accentForeground(accentHue))
    root.style.setProperty('--ui-scale', String(scale / 100))
  }

  if (persist) {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(normalized))
    } catch {
      // Appearance still applies for this session if storage is unavailable.
    }
  }
}
