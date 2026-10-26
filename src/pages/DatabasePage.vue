<script setup lang="ts">
import { computed, ref } from 'vue'
import PageIntro from '../components/common/PageIntro.vue'
import { formatBytes, formatDate } from '../services/formatters'
import { mediaRepository } from '../services/mediaRepository'
import { CATEGORY_LABELS, MEDIA_KIND_LABELS, type MediaRecord } from '../types/media'

const search = ref('')
const selectedKind = ref('all')
const records = ref<MediaRecord[]>([])

void mediaRepository.listAll().then((items) => { records.value = items })

const visibleRecords = computed(() => {
  const query = search.value.trim().toLocaleLowerCase('ru-RU')
  return records.value.filter((record) => {
    const matchesKind = selectedKind.value === 'all' || record.kind === selectedKind.value
    const haystack = [record.originalName, record.relativePath, record.extension, record.tags.join(' '), record.category ?? '']
      .join(' ')
      .toLocaleLowerCase('ru-RU')
    return matchesKind && (!query || haystack.includes(query))
  })
})

function clearFilters() {
  search.value = ''
  selectedKind.value = 'all'
}

const categories = CATEGORY_LABELS
const kinds = MEDIA_KIND_LABELS
</script>

<template>
  <PageIntro title="База данных файлов" description="Предварительный вид таблицы, которая в будущем будет читаться из локальной SQLite-базы. Сейчас здесь демонстрационные записи — изменения не сохраняются." />

  <div class="mb-4 flex flex-wrap items-center gap-2 rounded-xl border border-amber-200 bg-amber-50 px-4 py-3 text-sm text-amber-950">
    <span class="font-semibold">Демо-данные</span>
    <span>— это не настоящая база и не список файлов на вашем диске.</span>
  </div>

  <section class="rounded-xl border border-slate-200 bg-white p-4">
    <div class="grid gap-3 md:grid-cols-[minmax(220px,1fr)_190px_auto]">
      <label class="block">
        <span class="mb-1.5 block text-xs font-medium text-slate-600">Поиск</span>
        <input v-model="search" type="search" placeholder="Имя, путь, категория или тег..." class="w-full rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm outline-none placeholder:text-slate-400 focus:border-slate-500" />
      </label>
      <label class="block">
        <span class="mb-1.5 block text-xs font-medium text-slate-600">Тип файла</span>
        <select v-model="selectedKind" class="w-full rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm focus:border-slate-500">
          <option value="all">Все типы</option>
          <option value="video">Видео</option>
          <option value="photo">Фото</option>
          <option value="archive">Архивы</option>
        </select>
      </label>
      <div class="flex items-end">
        <button type="button" class="w-full rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium text-slate-700 hover:bg-slate-50 md:w-auto" @click="clearFilters">Сбросить фильтры</button>
      </div>
    </div>

    <div class="mt-4 flex flex-wrap items-center justify-between gap-2 border-t border-slate-100 pt-4 text-xs text-slate-500">
      <span>Показано записей: {{ visibleRecords.length }} из {{ records.length }}</span>
      <span>Поиск и фильтр работают в текущем демо-наборе</span>
    </div>

    <div class="mt-3 overflow-x-auto">
      <table class="w-full min-w-[980px] border-collapse text-left text-sm">
        <thead>
          <tr class="border-y border-slate-200 bg-slate-50 text-xs font-semibold uppercase tracking-wide text-slate-500">
            <th class="px-3 py-3">Имя файла</th>
            <th class="px-3 py-3">Тип</th>
            <th class="px-3 py-3">Категория</th>
            <th class="px-3 py-3">Размер</th>
            <th class="px-3 py-3">Дата файла</th>
            <th class="px-3 py-3">Теги</th>
            <th class="px-3 py-3">ID</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="record in visibleRecords" :key="record.id" class="border-b border-slate-100 align-top last:border-0 hover:bg-slate-50/70">
            <td class="max-w-[320px] px-3 py-3">
              <p class="break-words font-medium text-slate-900">{{ record.originalName }}</p>
              <p class="mt-1 break-all text-xs text-slate-500">{{ record.relativePath }}</p>
            </td>
            <td class="whitespace-nowrap px-3 py-3 text-slate-700">{{ kinds[record.kind] }} <span class="text-xs text-slate-400">{{ record.extension }}</span></td>
            <td class="whitespace-nowrap px-3 py-3 text-slate-700">{{ record.category ? categories[record.category] : '—' }}</td>
            <td class="whitespace-nowrap px-3 py-3 tabular-nums text-slate-700">{{ formatBytes(record.sizeBytes) }}</td>
            <td class="whitespace-nowrap px-3 py-3 text-slate-700">{{ formatDate(record.createdAt) }}</td>
            <td class="max-w-[240px] px-3 py-3">
              <div class="flex flex-wrap gap-1">
                <span v-for="tag in record.tags" :key="tag" class="rounded-md bg-slate-100 px-2 py-1 text-xs text-slate-600">{{ tag }}</span>
              </div>
            </td>
            <td class="whitespace-nowrap px-3 py-3 font-mono text-xs text-slate-500">{{ record.id }}</td>
          </tr>
          <tr v-if="visibleRecords.length === 0">
            <td colspan="7" class="px-3 py-10 text-center text-sm text-slate-500">По заданным условиям ничего не найдено.</td>
          </tr>
        </tbody>
      </table>
    </div>
  </section>

  <section class="mt-5 rounded-xl border border-slate-200 bg-white p-4">
    <h3 class="font-semibold">Какие сведения будут храниться</h3>
    <div class="mt-3 grid gap-3 text-sm sm:grid-cols-2 lg:grid-cols-3">
      <div class="rounded-lg bg-slate-50 p-3"><p class="font-medium">Общие сведения</p><p class="mt-1 text-slate-600">Исходное имя, расширение, размер, даты Windows, путь, ID.</p></div>
      <div class="rounded-lg bg-slate-50 p-3"><p class="font-medium">Классификация</p><p class="mt-1 text-slate-600">Тип файла, категория 3D / Аниме / IRL, теги.</p></div>
      <div class="rounded-lg bg-slate-50 p-3"><p class="font-medium">Технические данные</p><p class="mt-1 text-slate-600">По возможности: EXIF, разрешение, длительность, кодек, хеш.</p></div>
    </div>
  </section>
</template>
