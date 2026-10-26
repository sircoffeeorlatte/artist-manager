<script setup lang="ts">
import { RouterLink } from 'vue-router'
import PageIntro from '../components/common/PageIntro.vue'

const sections = [
  { title: 'Галерея видео', to: '/gallery/videos', description: 'Импорт видео и GIF, сетка файлов, встроенный просмотрщик и управление звуком.', marker: 'VID' },
  { title: 'Галерея фото', to: '/gallery/photos', description: 'Импорт изображений, сетка 2–6 колонок и просмотр с масштабированием колёсиком.', marker: 'IMG' },
  { title: 'Галерея архивов', to: '/gallery/archives', description: 'Каталог ZIP, 7z, RAR и других архивов без встроенной распаковки на MVP-этапе.', marker: 'ARC' },
  { title: 'База данных файлов', to: '/database', description: 'Предварительный вид таблицы файлов. Подключение SQLite будет следующим этапом.', marker: 'DB' },
  { title: 'Настройки', to: '/settings', description: 'Выбрать место хранения и создать структуру папок IRL / Аниме / 3D.', marker: 'CFG' },
]
</script>

<template>
  <PageIntro
    title="Локальная медиатека — рабочий MVP-каркас"
    description="Выбери папку хранилища в настройках, затем добавляй фотографии и видео. В Desktop-режиме приложение копирует или перемещает файлы в свою структуру папок; в браузерном режиме можно временно проверить сетку и просмотрщик."
  />

  <section class="mb-6 grid gap-3 sm:grid-cols-3">
    <div class="rounded-xl border border-slate-200 bg-white p-4">
      <p class="text-sm text-slate-500">Целевая платформа</p>
      <p class="mt-2 text-lg font-semibold">Windows 10 / 11</p>
      <p class="mt-1 text-xs text-slate-500">64-bit · Tauri 2</p>
    </div>
    <div class="rounded-xl border border-slate-200 bg-white p-4">
      <p class="text-sm text-slate-500">Контент</p>
      <p class="mt-2 text-lg font-semibold">3 типа файлов</p>
      <p class="mt-1 text-xs text-slate-500">Видео · Фото · Архивы</p>
    </div>
    <div class="rounded-xl border border-slate-200 bg-white p-4">
      <p class="text-sm text-slate-500">Категории</p>
      <p class="mt-2 text-lg font-semibold">3 категории</p>
      <p class="mt-1 text-xs text-slate-500">3D · Аниме · IRL плюс теги</p>
    </div>
  </section>

  <div class="mb-3 flex items-center justify-between gap-3">
    <h3 class="text-base font-semibold">Разделы приложения</h3>
    <span class="text-xs text-slate-500">Выбери страницу</span>
  </div>
  <section class="grid gap-3 sm:grid-cols-2" aria-label="Страницы приложения">
    <RouterLink
      v-for="section in sections"
      :key="section.to"
      :to="section.to"
      class="group rounded-xl border border-slate-200 bg-white p-5 no-underline transition-colors hover:border-slate-400 hover:bg-slate-50"
    >
      <div class="flex items-start justify-between gap-4">
        <div class="flex h-10 w-12 items-center justify-center rounded-md bg-slate-100 text-xs font-bold tracking-wide text-slate-600 group-hover:bg-white">{{ section.marker }}</div>
        <span class="text-lg text-slate-400 transition-transform group-hover:translate-x-1" aria-hidden="true">→</span>
      </div>
      <h4 class="mt-4 font-semibold text-slate-900">{{ section.title }}</h4>
      <p class="mt-2 text-sm leading-6 text-slate-600">{{ section.description }}</p>
      <span class="mt-4 inline-block text-sm font-medium text-slate-700">Открыть раздел →</span>
    </RouterLink>
  </section>

  <section class="mt-6 rounded-xl border border-amber-200 bg-amber-50 p-4">
    <h3 class="text-sm font-semibold text-amber-950">Что пока не реализовано</h3>
    <p class="mt-1 text-sm leading-6 text-amber-900">
      Пока не подключены SQLite-индексация, извлечение EXIF и технических метаданных видео, редактирование тегов, авторы, генерация миниатюр и автоматические обновления.
    </p>
  </section>
</template>
