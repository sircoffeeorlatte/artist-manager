import { isTauri } from '@tauri-apps/api/core'

export type RuntimeMode = 'desktop' | 'browser'

export function getRuntimeMode(): RuntimeMode {
  return isTauri() ? 'desktop' : 'browser'
}

export function getRuntimeLabel(): string {
  return getRuntimeMode() === 'desktop'
    ? 'Настольный режим Tauri'
    : 'Браузерный режим превью'
}
