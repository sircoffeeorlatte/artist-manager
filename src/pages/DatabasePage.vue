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
  <PageIntro title="База файлов" eyebrow="WORKSPACE / INDEX" icon="⌘" description="Табличный вид коллекции с быстрым поиском по имени, пути, категории и тегам. Сейчас используется демонстрационный набор — изменения не сохраняются." />

  <div class="notice notice-warning" style="margin-bottom:14px">
    <strong style="color:#f2d69a">Демо-данные.</strong> Это макет будущего индекса, а не реальные файлы с твоего диска. Подключение полноценной SQLite-базы — отдельный этап.
  </div>

  <section class="panel data-toolbar">
    <div class="panel-heading" style="margin-bottom:17px">
      <div><h2 class="panel-title">Каталог файлов</h2><p class="panel-subtitle">Поиск и фильтрация текущего набора записей.</p></div>
      <span class="pill">{{ visibleRecords.length }} / {{ records.length }} ЗАПИСЕЙ</span>
    </div>
    <div class="grid gap-3 md:grid-cols-[minmax(220px,1fr)_190px_auto]">
      <label>
        <span class="field-label">Поиск по каталогу</span>
        <input v-model="search" type="search" placeholder="Имя, путь, категория, тег…" class="field-control" />
      </label>
      <label>
        <span class="field-label">Тип файла</span>
        <select v-model="selectedKind" class="field-control">
          <option value="all">Все типы</option>
          <option value="video">Видео</option>
          <option value="photo">Фото</option>
          <option value="archive">Архивы</option>
        </select>
      </label>
      <div style="display:flex;align-items:end">
        <button type="button" class="btn btn-secondary" style="width:100%" @click="clearFilters">Сбросить фильтры</button>
      </div>
    </div>
    <div style="display:flex;flex-wrap:wrap;align-items:center;justify-content:space-between;gap:12px;margin-top:16px;padding-top:13px;border-top:1px solid #252f3c">
      <span style="color:#7e8da0;font-size:10px">Показано <strong style="color:#dce5ef">{{ visibleRecords.length }}</strong> записей из {{ records.length }}</span>
      <span class="pill">ПОИСК ПО ТЕКУЩЕМУ НАБОРУ</span>
    </div>

    <div class="data-table-wrap">
      <table class="data-table">
        <thead>
          <tr><th>Имя файла</th><th>Тип</th><th>Категория</th><th>Размер</th><th>Дата файла</th><th>Теги</th><th>ID</th></tr>
        </thead>
        <tbody>
          <tr v-for="record in visibleRecords" :key="record.id">
            <td><div class="file-name">{{ record.originalName }}</div><div class="file-path">{{ record.relativePath }}</div></td>
            <td><span class="file-type"><span class="file-type-dot" :style="{ background: record.kind === 'video' ? '#88baf7' : record.kind === 'photo' ? '#b9e87d' : '#d9a8f7' }"></span>{{ kinds[record.kind] }}</span><div style="margin-top:5px;color:#65758a;font-size:9px">{{ record.extension }}</div></td>
            <td><span v-if="record.category" class="pill">{{ categories[record.category] }}</span><span v-else>—</span></td>
            <td style="white-space:nowrap;font-variant-numeric:tabular-nums">{{ formatBytes(record.sizeBytes) }}</td>
            <td style="white-space:nowrap">{{ formatDate(record.createdAt) }}</td>
            <td style="max-width:230px"><span v-for="tag in record.tags" :key="tag" class="tag-chip">{{ tag }}</span><span v-if="!record.tags.length" class="dim">—</span></td>
            <td style="white-space:nowrap;color:#68798d;font-family:ui-monospace,SFMono-Regular,Consolas,monospace;font-size:9px">{{ record.id }}</td>
          </tr>
          <tr v-if="visibleRecords.length === 0"><td colspan="7" style="padding:40px 12px;text-align:center;color:#8290a3">Ничего не найдено. Попробуй изменить запрос или сбросить фильтры.</td></tr>
        </tbody>
      </table>
    </div>
  </section>

  <div class="section-heading"><div><h3>Что будет храниться в базе</h3><p>Пример структуры метаданных для каждого файла</p></div></div>
  <section class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
    <div class="data-info-card"><strong>01 / Общие сведения</strong><p>Исходное имя, расширение, размер, даты Windows, путь к файлу и внутренний ID.</p></div>
    <div class="data-info-card"><strong>02 / Классификация</strong><p>Тип файла, категория IRL / Аниме / 3D, пользовательские теги и поле автора на будущее.</p></div>
    <div class="data-info-card"><strong>03 / Технические данные</strong><p>По возможности: EXIF, разрешение, длительность видео, кодек и контрольная сумма.</p></div>
  </section>
</template>
