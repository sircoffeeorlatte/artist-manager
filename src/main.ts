import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import './styles.css'
import { applyAppearanceSettings, getSavedAppearanceSettings } from './services/appearanceSettings'

// Apply saved appearance before mounting Vue, so the app does not flash its default theme.
applyAppearanceSettings(getSavedAppearanceSettings(), false)

createApp(App).use(router).mount('#app')
