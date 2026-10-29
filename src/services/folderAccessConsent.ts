import { reactive } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import { flushUserConfig, getUserConfig, updateUserConfig } from './userConfig'
import { notify } from './appState'

/** App-level consent state. Tauri's actual command access is declared in capabilities/default.json. */
export const folderAccessConsentState = reactive({ visible: false })

const pendingRequests: Array<(allowed: boolean) => void> = []
let initialized = false

/** Show the consent dialog on startup until the user has explicitly allowed Explorer access. */
export function initializeFolderAccessConsent(): void {
  if (initialized) return
  initialized = true
  if (isTauri() && !getUserConfig().permissions.openFolders) {
    folderAccessConsentState.visible = true
  }
}

/** Resolve immediately if already allowed; otherwise show the dialog and await the user's choice. */
export function requestFolderAccessConsent(): Promise<boolean> {
  if (!isTauri()) return Promise.resolve(false)
  if (getUserConfig().permissions.openFolders) return Promise.resolve(true)

  folderAccessConsentState.visible = true
  return new Promise((resolve) => {
    pendingRequests.push(resolve)
  })
}

export async function grantFolderAccessConsent(): Promise<void> {
  updateUserConfig({ permissions: { openFolders: true } })
  try {
    await flushUserConfig()
  } catch (error) {
    // The permission is allowed for this run; if disk persistence failed, ask again next launch.
    notify('warning', 'Согласие не удалось сохранить', error instanceof Error ? error.message : String(error))
  } finally {
    folderAccessConsentState.visible = false
    resolvePending(true)
  }
}

/** Defer the decision for this run. The prompt returns on the next launch or next folder-open action. */
export function deferFolderAccessConsent(): void {
  folderAccessConsentState.visible = false
  resolvePending(false)
}

function resolvePending(allowed: boolean): void {
  for (const resolve of pendingRequests.splice(0)) resolve(allowed)
}
