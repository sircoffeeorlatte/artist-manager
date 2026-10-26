<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, RouterView, useRoute } from 'vue-router'
import { getRuntimeLabel } from '../services/platform'

const route = useRoute()
const pageTitle = computed(() => String(route.meta.title ?? 'Локальная медиатека'))
const runtimeLabel = getRuntimeLabel()

const navigation = [
  { label: 'Главная', to: '/', short: 'Домой' },
  { label: 'Галерея видео', to: '/gallery/videos', short: 'Видео' },
  { label: 'Галерея фото', to: '/gallery/photos', short: 'Фото' },
  { label: 'Галерея архивов', to: '/gallery/archives', short: 'Архивы' },
  { label: 'База данных файлов', to: '/database', short: 'База данных' },
  { label: 'Настройки', to: '/settings', short: 'Настройки' },
]
</script>

<template>
  <div class="min-h-screen bg-slate-50 text-slate-900">
    <div class="min-h-screen md:grid md:grid-cols-[230px_minmax(0,1fr)]">
      <aside class="border-b border-slate-200 bg-white p-4 md:min-h-screen md:border-b-0 md:border-r md:p-5">
        <RouterLink to="/" class="mb-5 flex items-center gap-3 rounded-lg p-2 no-underline hover:bg-slate-50">
          <span class="flex h-10 w-10 items-center justify-center rounded-lg bg-slate-900 text-sm font-bold text-white">ML</span>
          <span>
            <span class="block font-semibold text-slate-900">Media Library</span>
            <span class="block text-xs text-slate-500">MVP · Windows</span>
          </span>
        </RouterLink>

        <p class="mb-2 px-2 text-xs font-semibold uppercase tracking-wide text-slate-400">Разделы</p>
        <nav class="flex gap-1 overflow-x-auto md:flex-col" aria-label="Основная навигация">
          <RouterLink
            v-for="item in navigation"
            :key="item.to"
            :to="item.to"
            class="shrink-0 rounded-lg px-3 py-2.5 text-sm no-underline transition-colors"
            :class="route.path === item.to ? 'bg-slate-900 font-medium text-white' : 'text-slate-600 hover:bg-slate-100 hover:text-slate-900'"
          >
            {{ item.label }}
          </RouterLink>
        </nav>

        <div class="mt-6 rounded-lg border border-slate-200 bg-slate-50 p-3 text-xs leading-5 text-slate-600">
          <p class="font-semibold text-slate-800">Режим запуска</p>
          <p class="mt-1">{{ runtimeLabel }}</p>
          <p class="mt-2 text-slate-500">Настройка папки и импорт доступны в Desktop. SQLite-интеграция будет следующим этапом.</p>
        </div>
      </aside>

      <div class="min-w-0">
        <header class="border-b border-slate-200 bg-white px-5 py-4 md:px-8">
          <p class="text-xs text-slate-500">Локальная медиатека / MVP</p>
          <h1 class="mt-1 text-xl font-semibold tracking-tight">{{ pageTitle }}</h1>
        </header>
        <main class="mx-auto w-full max-w-[1440px] p-5 md:p-8">
          <RouterView />
        </main>
        <footer class="border-t border-slate-200 px-5 py-4 text-xs text-slate-500 md:px-8">
          MVP · Хранилище медиатеки находится отдельно от программы
        </footer>
      </div>
    </div>
  </div>
</template>
