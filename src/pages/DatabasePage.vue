<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import PageIntro from '../components/common/PageIntro.vue'
import { formatBytes, formatDate } from '../services/formatters'
import { mediaRepository } from '../services/mediaRepository'
import type { MediaIndexResult, MediaIndexStatus } from '../services/filesystem'
import { loadLibraryRoot } from '../services/librarySettings'
import { openContainingFolder, openFolderPath } from '../services/fileLocations'
import { CATEGORY_LABELS, MEDIA_KIND_LABELS, type MediaRecord } from '../types/media'

const search = ref('')
const selectedKind = ref('all')
const records = ref<MediaRecord[]>([])
const rootPath = ref('')
const indexStatus = ref<MediaIndexStatus | null>(null)
const lastIndexResult = ref<MediaIndexResult | null>(null)
const loading = ref(false)
const errorMessage = ref('')
const successMessage = ref('')
const tagDrafts = reactive<Record<string, string>>({})
const savingTagsFor = ref<string | null>(null)
const isDesktop = computed(() => isTauri())

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

function syncTagDrafts() {
  for (const key of Object.keys(tagDrafts)) delete tagDrafts[key]
  for (const record of records.value) tagDrafts[record.relativePath] = record.tags.join(', ')
}

async function reloadData(rebuildIndex = false) {
  loading.value = true
  errorMessage.value = ''
  successMessage.value = ''
  lastIndexResult.value = null
  try {
    rootPath.value = await loadLibraryRoot()
    if (!isTauri()) {
      records.value = []
      indexStatus.value = null
      errorMessage.value = 'SQLite-индекс доступен в настольной версии Tauri. В браузерном режиме можно просматривать только интерфейс.'
      return
    }
    if (!rootPath.value) {
      records.value = []
      indexStatus.value = await mediaRepository.getIndexStatus()
      errorMessage.value = 'Сначала выбери корневую папку медиатеки в настройках.'
      return
    }
    if (rebuildIndex) {
      lastIndexResult.value = await mediaRepository.refreshIndex(rootPath.value)
      successMessage.value = `Индекс обновлён: ${lastIndexResult.value.scannedFiles} файлов, добавлено ${lastIndexResult.value.addedFiles}, изменено ${lastIndexResult.value.updatedFiles}, отсутствует ${lastIndexResult.value.missingFiles}.`
    }
    records.value = await mediaRepository.listAll(rootPath.value, true)
    syncTagDrafts()
    indexStatus.value = await mediaRepository.getIndexStatus()
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
    if (isTauri()) {
      try { indexStatus.value = await mediaRepository.getIndexStatus() } catch { /* Keep the original actionable error. */ }
    }
  } finally {
    loading.value = false
  }
}

async function saveTags(record: MediaRecord) {
  if (!rootPath.value) return
  savingTagsFor.value = record.relativePath
  errorMessage.value = ''
  successMessage.value = ''
  try {
    const tags = (tagDrafts[record.relativePath] ?? '')
      .split(',')
      .map((tag) => tag.trim())
      .filter(Boolean)
    await mediaRepository.setTags(rootPath.value, record.relativePath, [...new Set(tags)])
    records.value = await mediaRepository.listAll(rootPath.value, true)
    syncTagDrafts()
    successMessage.value = `Теги сохранены: ${record.originalName}`
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  } finally {
    savingTagsFor.value = null
  }
}


async function openDatabaseFolder(path: string) {
  try {
    await openContainingFolder(path)
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  }
}

async function openLibraryFolder(path: string) {
  try {
    await openFolderPath(path)
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  }
}

function clearFilters() {
  search.value = ''
  selectedKind.value = 'all'
}

const categories = CATEGORY_LABELS
const kinds = MEDIA_KIND_LABELS
onMounted(() => { void reloadData() })
</script>

<template>
  <PageIntro
    title="База файлов"
    eyebrow="WORKSPACE / SQLITE INDEX"
    icon="⌘"
    description="Реестр из SQLite: относительные пути, категории, даты, пользовательские теги и состояние доступности файлов. Обычное открытие страницы читает базу, а не сканирует все папки заново."
  />

  <section class="panel data-toolbar sqlite-index-panel">
    <div class="panel-heading" style="margin-bottom: 14px">
      <div>
        <h2 class="panel-title">Состояние SQLite-индекса</h2>
        <p class="panel-subtitle">{{ loading ? 'Выполняется запрос к базе…' : 'Индекс обновляется только при первом запуске, смене медиатеки, импорте или по команде.' }}</p>
      </div>
      <button type="button" class="btn btn-secondary" :disabled="loading || !rootPath" @click="reloadData(true)">
        <span aria-hidden="true">↻</span> Переиндексировать
      </button>
    </div>

    <div class="sqlite-index-stats">
      <div class="sqlite-index-stat"><span>Файлов в индексе</span><strong>{{ indexStatus?.availableCount ?? 0 }}</strong></div>
      <div class="sqlite-index-stat"><span>Отсутствуют на диске</span><strong>{{ indexStatus?.missingCount ?? 0 }}</strong></div>
      <div class="sqlite-index-stat"><span>Последняя индексация</span><strong>{{ formatDate(indexStatus?.lastCompletedAt ?? null) }}</strong></div>
    </div>
    <div v-if="indexStatus?.databasePath" class="sqlite-index-path">
      <span>Файл базы данных</span>
      <button type="button" class="path-open-button selectable-path" :disabled="!isDesktop" :title="`Открыть папку с базой данных: ${indexStatus.databasePath}`" @click="openDatabaseFolder(indexStatus.databasePath)">
        <code>{{ indexStatus.databasePath }}</code><span aria-hidden="true">↗</span>
      </button>
    </div>
    <div v-if="indexStatus?.lastError" class="notice notice-warning sqlite-index-message">Последняя ошибка индексации: {{ indexStatus.lastError }}</div>
    <div v-if="errorMessage" class="notice notice-error sqlite-index-message">{{ errorMessage }}</div>
    <div v-if="successMessage" class="notice notice-success sqlite-index-message">{{ successMessage }}</div>
  </section>

  <section class="panel data-toolbar sqlite-catalog-panel">
    <div class="panel-heading" style="margin-bottom: 17px">
      <div>
        <h2 class="panel-title">Каталог файлов</h2>
        <p class="panel-subtitle">Поиск по реальным записям SQLite, включая файлы, которые индексатор больше не находит.</p>
      </div>
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
    <div class="sqlite-catalog-summary">
      <span>Показано <strong>{{ visibleRecords.length }}</strong> записей из {{ records.length }}</span>
      <button v-if="rootPath" type="button" class="path-open-button selectable-path sqlite-root-path" :disabled="!isDesktop" title="Открыть папку медиатеки" @click="openLibraryFolder(rootPath)">
        <span>Медиатека: {{ rootPath }}</span><span aria-hidden="true">↗</span>
      </button>
    </div>

    <div class="data-table-wrap">
      <table class="data-table sqlite-data-table">
        <thead>
          <tr><th>Имя файла</th><th>Тип</th><th>Категория</th><th>Размер</th><th>Дата файла</th><th>Теги (через запятую)</th><th>Состояние</th><th>ID</th></tr>
        </thead>
        <tbody>
          <tr v-for="record in visibleRecords" :key="record.id" :class="{ 'sqlite-row-missing': record.status === 'missing' }">
            <td><div class="file-name">{{ record.originalName }}</div><div class="file-path selectable-path">{{ record.relativePath }}</div></td>
            <td><span class="file-type"><span class="file-type-dot" :style="{ background: record.kind === 'video' ? '#88baf7' : record.kind === 'photo' ? '#b9e87d' : '#d9a8f7' }"></span>{{ kinds[record.kind] }}</span><div style="margin-top:5px;color:var(--text-muted);font-size:10px">{{ record.extension }}</div></td>
            <td><span v-if="record.category" class="pill">{{ categories[record.category] }}</span><span v-else>—</span></td>
            <td style="white-space:nowrap;font-variant-numeric:tabular-nums">{{ formatBytes(record.sizeBytes) }}</td>
            <td style="white-space:nowrap">{{ formatDate(record.modifiedAt ?? record.createdAt) }}</td>
            <td class="sqlite-tags-cell">
              <input v-model="tagDrafts[record.relativePath]" class="field-control sqlite-tag-input" :aria-label="`Теги ${record.originalName}`" placeholder="tag1, tag2" />
              <button type="button" class="btn btn-secondary sqlite-tag-save" :disabled="savingTagsFor === record.relativePath" @click="saveTags(record)">{{ savingTagsFor === record.relativePath ? '…' : 'Сохранить' }}</button>
            </td>
            <td><span class="sqlite-record-status" :class="`is-${record.status ?? 'available'}`">{{ record.status === 'missing' ? 'Нет на диске' : record.status === 'needs_review' ? 'Проверить' : 'На месте' }}</span></td>
            <td class="sqlite-record-id">{{ record.id }}</td>
          </tr>
          <tr v-if="visibleRecords.length === 0"><td colspan="8" class="sqlite-empty-row">{{ loading ? 'Загружаем индекс…' : 'Записей не найдено. Проверь путь медиатеки или обнови индекс.' }}</td></tr>
        </tbody>
      </table>
    </div>
  </section>

  <div class="section-heading"><div><h3>Что хранится в SQLite</h3><p>Файлы, индексация и пользовательская классификация</p></div></div>
  <section class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
    <div class="data-info-card"><strong>01 / Метаданные файла</strong><p>Тип, исходное имя, расширение, размер, даты, относительный путь и статус доступности.</p></div>
    <div class="data-info-card"><strong>02 / Теги</strong><p>Теги хранятся отдельно и связываются с файлами через таблицу связей. Здесь их уже можно редактировать.</p></div>
    <div class="data-info-card"><strong>03 / Состояние индекса</strong><p>Путь медиатеки, время последней успешной индексации, количество найденных и отсутствующих файлов, последняя ошибка.</p></div>
  </section>
</template>

<style scoped>
.sqlite-index-panel, .sqlite-catalog-panel { padding: 18px; }
.sqlite-index-stats { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 10px; }
.sqlite-index-stat { min-width: 0; display: flex; flex-direction: column; gap: 6px; padding: 12px; border: 1px solid var(--border); border-radius: 9px; background: var(--surface-subtle); }
.sqlite-index-stat span, .sqlite-index-path > span { color: var(--text-muted); font-size: 11px; }
.sqlite-index-stat strong { color: var(--text-primary); font-size: 20px; font-variant-numeric: tabular-nums; }
.sqlite-index-path { display: flex; flex-direction: column; gap: 6px; margin-top: 11px; padding-top: 11px; border-top: 1px solid var(--border); }
.sqlite-index-path code { overflow-wrap: anywhere; color: var(--text-secondary); font-size: 11px; }
.sqlite-index-message { margin-top: 12px; white-space: pre-line; }
.sqlite-catalog-panel { margin-top: 14px; }
.sqlite-catalog-summary { display: flex; flex-wrap: wrap; justify-content: space-between; gap: 10px; margin-top: 16px; padding-top: 13px; border-top: 1px solid var(--border); color: var(--text-muted); font-size: 11px; }
.sqlite-catalog-summary strong { color: var(--text-primary); }
.sqlite-data-table { min-width: 1160px; }
.sqlite-row-missing { opacity: .72; }
.sqlite-tags-cell { min-width: 210px; }
.sqlite-tag-input { min-width: 180px; height: 31px; font-size: 11px; }
.sqlite-tag-save { margin-top: 5px; min-height: 27px; padding: 0 9px; font-size: 10px; }
.sqlite-record-status { display: inline-flex; align-items: center; padding: 4px 7px; border: 1px solid var(--border); border-radius: 6px; color: var(--text-secondary); font-size: 10px; white-space: nowrap; }
.sqlite-record-status.is-available { border-color: color-mix(in srgb, #65b985 35%, var(--border)); color: var(--text-success, #65b985); }
.sqlite-record-status.is-missing { border-color: color-mix(in srgb, #de7777 40%, var(--border)); color: var(--text-danger, #de7777); }
.sqlite-record-status.is-needs_review { border-color: color-mix(in srgb, #dda84d 40%, var(--border)); color: var(--text-warning, #dda84d); }
.sqlite-record-id { max-width: 230px; overflow-wrap: anywhere; color: var(--text-muted); font-family: ui-monospace, SFMono-Regular, Consolas, monospace; font-size: 10px; }
.sqlite-empty-row { padding: 40px 12px; text-align: center; color: var(--text-muted); }
@media (max-width: 740px) {
  .sqlite-index-stats { grid-template-columns: 1fr; }
  .sqlite-index-panel .panel-heading { align-items: stretch; flex-direction: column; }
}
</style>
