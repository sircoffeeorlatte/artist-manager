import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import './styles.css'
import { applyAppearanceSettings, getSavedAppearanceSettings } from './services/appearanceSettings'
import { initializeAppState } from './services/appState'
import { flushUserConfig, getUserConfig, loadUserConfig } from './services/userConfig'

async function bootstrap() {
  await loadUserConfig()
  applyAppearanceSettings(getSavedAppearanceSettings(), false)

  const app = createApp(App).use(router)
  await initializeAppState()
  await router.isReady()

  // Only override the default root route; honor explicit deep links when the OS reopens one.
  if (router.currentRoute.value.path === '/' && getUserConfig().startupPage !== '/') {
    await router.replace(getUserConfig().startupPage)
  }

  window.addEventListener('beforeunload', () => { void flushUserConfig() })
  app.mount('#app')
}

void bootstrap()
