<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { RouterLink, RouterView, useRoute, useRouter } from 'vue-router'
import { getRuntimeLabel } from '../services/platform'
import AppActivityOverlay from '../components/status/AppActivityOverlay.vue'
import AppWallpaperBackground from '../components/appearance/AppWallpaperBackground.vue'
import { initializeAppState } from '../services/appState'

const route = useRoute()
const router = useRouter()
onMounted(() => { void initializeAppState() })
const pageTitle = computed(() => String(route.meta.title ?? 'Локальная медиатека'))
const runtimeLabel = getRuntimeLabel()
function openImport() {
  const kind = route.path.startsWith('/gallery/videos')
    ? 'video'
    : route.path.startsWith('/gallery/archives') ? 'archive' : 'photo'
  void router.push({ path: '/import', query: { kind } })
}

const navigation = [
  { label: 'Главная', to: '/', icon: '⌂', group: 'БИБЛИОТЕКА' },
  { label: 'Видео', to: '/gallery/videos', icon: '▶', group: '' },
  { label: 'Фото', to: '/gallery/photos', icon: '▧', group: '' },
  { label: 'Архивы', to: '/gallery/archives', icon: '▤', group: '' },
  { label: 'База файлов', to: '/database', icon: '⌘', group: 'ИНСТРУМЕНТЫ' },
  { label: 'Импорт файлов', to: '/import', icon: '⇧', group: '' },
  { label: 'Настройки', to: '/settings', icon: '⚙', group: '' },
]

function isActive(path: string) {
  if (path === '/') return route.path === '/'
  return route.path === path || route.path.startsWith(`${path}/`)
}
</script>

<template>
  <div class="app-shell">
    <AppWallpaperBackground />
    <aside class="app-sidebar">
      <RouterLink to="/" class="brand-lockup">
        <span class="brand-mark" aria-hidden="true"><span></span><span></span><span></span><span></span></span>
        <span class="brand-copy">
          <strong>Media Library</strong>
          <small>ЛИЧНОЕ ХРАНИЛИЩЕ</small>
        </span>
      </RouterLink>

      <nav class="main-navigation" aria-label="Основная навигация">
        <template v-for="item in navigation" :key="item.to">
          <p v-if="item.group" class="nav-group-label">{{ item.group }}</p>
          <RouterLink :to="item.to" class="nav-link" :class="{ 'is-active': isActive(item.to) }">
            <span class="nav-icon" aria-hidden="true">{{ item.icon }}</span>
            <span>{{ item.label }}</span>
            <span v-if="isActive(item.to)" class="nav-active-dot" aria-hidden="true"></span>
          </RouterLink>
        </template>
      </nav>

      <div class="sidebar-spacer"></div>
      <div class="storage-status">
        <div class="status-heading"><span class="status-light"></span><span>РЕЖИМ РАБОТЫ</span></div>
        <strong>{{ runtimeLabel }}</strong>
        <p>Файлы остаются на твоём компьютере. Хранилище не зависит от папки установки программы.</p>
      </div>
      <div class="sidebar-version"><span>MEDIA LIBRARY</span><span>PREVIEW · 0.2</span></div>
    </aside>

    <div class="app-main-column">
      <header class="app-topbar">
        <div class="topbar-location">
          <span class="topbar-kicker">МЕДИАТЕКА</span>
          <span class="topbar-separator">/</span>
          <span class="topbar-current">{{ pageTitle }}</span>
        </div>
        <div class="topbar-actions">
          <button type="button" class="btn btn-primary topbar-import-button" @click="openImport"><span aria-hidden="true">⇧</span> Импорт</button>
          <div class="topbar-status"><span class="status-light"></span>Локальное приложение</div>
          <AppActivityOverlay />
        </div>
      </header>
      <main class="app-content">
        <RouterView />
      </main>
      <footer class="app-footer">
        <span>Твои файлы. Твой компьютер. Твои правила.</span>
        <span>Media Library · MVP</span>
      </footer>
    </div>
  </div>
</template>
