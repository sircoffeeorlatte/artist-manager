<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import { openPath } from '@tauri-apps/plugin-opener'
import PageIntro from '../components/common/PageIntro.vue'
import { CATEGORY_LABELS, type LibraryMediaItem } from '../types/media'
import { getSavedLibraryRoot, loadLibraryRoot } from '../services/librarySettings'
import { listLibraryMedia } from '../services/filesystem'
import { formatBytes, formatDate } from '../services/formatters'
import { beginActivity, errorText, finishActivity } from '../services/appState'
import { getUserConfig, setGridColumns } from '../services/userConfig'

const items = ref<LibraryMediaItem[]>([])
const rootPath = ref(getSavedLibraryRoot())
const loading = ref(false)
const errorMessage = ref('')
const openError = ref('')
const isDesktop = computed(() => isTauri())
const archiveColumns = ref(getUserConfig().gridColumns.archive)
const gridSizes = [2, 3, 4, 5, 6, 7]
watch(archiveColumns, (value) => setGridColumns('archive', value))

const formats = [
  { extension: 'ZIP', description: 'Универсальные архивы', accent: '◫' },
  { extension: '7Z', description: 'Сжатие 7-Zip', accent: '▤' },
  { extension: 'RAR', description: 'Архивы WinRAR', accent: '▥' },
  { extension: 'TAR / GZ', description: 'Unix-архивы и сжатие', accent: '▧' },
]

async function reloadArchives() {
  const activityId = beginActivity({ kind: 'gallery', title: 'Обновление галереи архивов', details: 'Чтение списка архивов' })
  loading.value = true
  errorMessage.value = ''
  try {
    rootPath.value = await loadLibraryRoot()
    if (!isTauri() || !rootPath.value) {
      items.value = []
      finishActivity(activityId, 'success', !isTauri() ? 'Браузерный режим: доступен только просмотр интерфейса.' : 'Папка медиатеки не выбрана.')
      return
    }
    const result = await listLibraryMedia(rootPath.value, 'archive')
    items.value = result
    finishActivity(activityId, 'success', `Найдено архивов: ${result.length}.`)
  } catch (error) {
    const message = errorText(error)
    errorMessage.value = message
    finishActivity(activityId, 'error', message)
  } finally {
    loading.value = false
  }
}

async function openArchive(item: LibraryMediaItem) {
  if (!isTauri()) {
    openError.value = 'Открытие архивов во внешнем приложении доступно в настольной версии.'
    return
  }
  openError.value = ''
  const activityId = beginActivity({ kind: 'archive', title: 'Открытие архива', details: item.originalName })
  try {
    await openPath(item.absolutePath)
    finishActivity(activityId, 'success', `Файл передан внешнему приложению: ${item.originalName}`)
  } catch (error) {
    const message = errorText(error)
    openError.value = message
    finishActivity(activityId, 'error', `${item.originalName}: ${message}`)
  }
}
function onImportComplete() { void reloadArchives() }
onMounted(() => {
  void reloadArchives()
  window.addEventListener('media-library:import-complete', onImportComplete)
})
onBeforeUnmount(() => window.removeEventListener('media-library:import-complete', onImportComplete))
</script>

<template>
  <PageIntro
    title="Галерея архивов"
    eyebrow="LIBRARY / ARCHIVES"
    icon="▤"
    description="Отдельный каталог ZIP, 7z, RAR и других архивов. Импортируй файлы в категории IRL, Аниме или 3D, а затем открывай их во внешней программе."
  />

  <section class="panel archive-catalog-panel">
    <div class="panel-heading">
      <div>
        <h2 class="panel-title">Архивы в медиатеке</h2>
        <p class="panel-subtitle">{{ loading ? 'Обновляем список…' : `${items.length} файлов в хранилище` }}</p>
      </div>
      <div class="archive-heading-actions">
        <div class="gallery-density" role="group" aria-label="Размер сетки архивов">
          <button
            v-for="size in gridSizes"
            :key="size"
            type="button"
            class="gallery-density-button"
            :class="{ 'is-active': archiveColumns === size }"
            :aria-pressed="archiveColumns === size"
            :aria-label="`Сетка архивов ${size} на ${size}`"
            :title="`Сетка ${size} × ${size}`"
            @click="archiveColumns = size"
          >{{ size }}×{{ size }}</button>
        </div>
        <button type="button" class="btn btn-secondary" :disabled="loading" @click="reloadArchives"><span aria-hidden="true">↻</span> Обновить</button>
      </div>
    </div>

    <p v-if="errorMessage" class="notice notice-error archive-message">{{ errorMessage }}</p>
    <p v-if="openError" class="notice notice-error archive-message">{{ openError }}</p>

    <div v-if="items.length" class="archive-file-list" :style="{ '--archive-columns': archiveColumns }">
      <div v-for="item in items" :key="item.id" class="archive-file-row">
        <span class="archive-file-icon" aria-hidden="true">▤</span>
        <div class="archive-file-main">
          <strong :title="item.originalName">{{ item.originalName }}</strong>
          <span class="selectable-path">{{ item.relativePath }}</span>
        </div>
        <span class="pill archive-category-pill">{{ item.category ? CATEGORY_LABELS[item.category] : 'Без категории' }}</span>
        <span class="archive-file-meta">{{ formatBytes(item.sizeBytes) }}</span>
        <span class="archive-file-meta archive-file-date">{{ formatDate(item.modifiedAt) }}</span>
        <button type="button" class="btn btn-secondary archive-open-button" :disabled="!isDesktop" @click="openArchive(item)">Открыть</button>
      </div>
    </div>

    <div v-else-if="!loading" class="archive-empty-state">
      <span class="archive-empty-icon" aria-hidden="true">▤</span>
      <h3>{{ isDesktop ? 'Пока нет импортированных архивов' : 'Архивы доступны в настольном приложении' }}</h3>
      <p>{{ isDesktop ? (rootPath ? 'Нажми «Импорт» в верхней панели и выбери тип «Архивы».' : 'Сначала выбери корневую папку медиатеки в настройках, затем открой импорт.') : 'В браузерном режиме архивы можно только выбрать в интерфейсе. Для записи в медиатеку запусти настольную версию Tauri.' }}</p>
    </div>
  </section>

  <section class="panel archive-formats-panel">
    <div class="panel-heading">
      <div>
        <h2 class="panel-title">Форматы архивов</h2>
        <p class="panel-subtitle">Каталог хранит исходные файлы; распаковка внутри Media Library не выполняется.</p>
      </div>
      <span class="pill">ФОРМАТЫ</span>
    </div>
    <div class="archive-format-grid">
      <div v-for="format in formats" :key="format.extension" class="archive-format-card">
        <div class="archive-format-icon">{{ format.accent }}</div>
        <div><strong>{{ format.extension }}</strong><span>{{ format.description }}</span></div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.archive-catalog-panel, .archive-formats-panel { padding: 19px; }
.archive-heading-actions { display: flex; align-items: center; justify-content: flex-end; gap: 10px; flex-wrap: wrap; }
.archive-heading-actions .gallery-density { gap: 3px; }
.archive-heading-actions .gallery-density-button { min-width: 31px; padding: 0 5px; }
.archive-formats-panel { margin-top: 15px; }
.archive-message { margin-top: 13px; white-space: pre-line; }
.archive-file-list { display: grid; grid-template-columns: repeat(var(--archive-columns, 3), minmax(0, 1fr)); align-items: stretch; gap: 8px; margin-top: 17px; }
.archive-file-row { display: flex; min-width: 0; flex-direction: column; align-items: flex-start; gap: 9px; padding: 11px; border: 1px solid #263140; border-radius: 8px; background: #10151d; }
.archive-file-icon { display: grid; width: 36px; height: 36px; flex: 0 0 36px; place-items: center; color: var(--accent); border: 1px solid var(--accent-alpha-17); border-radius: 8px; background: var(--accent-alpha-5); font-size: 17px; }
.archive-file-main { display: flex; width: 100%; min-width: 0; flex: 1; flex-direction: column; gap: 4px; }
.archive-file-main strong { overflow: hidden; color: #e2e9f3; font-size: 11px; font-weight: 700; text-overflow: ellipsis; white-space: nowrap; }
.archive-file-main span { overflow: hidden; color: #77869a; font-size: 9px; text-overflow: ellipsis; white-space: nowrap; }
.archive-category-pill { flex: 0 0 auto; }
.archive-file-meta { flex: 0 0 auto; color: #9aa8ba; font-size: 10px; white-space: nowrap; }
.archive-open-button { min-height: 31px; width: 100%; }
.archive-empty-state { padding: 38px 15px 30px; text-align: center; }
.archive-empty-icon { display: inline-grid; width: 50px; height: 50px; place-items: center; color: var(--accent); border: 1px solid var(--accent-alpha-17); border-radius: 14px; background: var(--accent-alpha-5); font-size: 21px; }
.archive-empty-state h3 { margin: 15px 0 7px; color: #dfe7f1; font-size: 13px; }
.archive-empty-state p { max-width: 540px; margin: 0 auto; color: #8290a3; font-size: 11px; line-height: 1.8; }
.archive-format-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 10px; margin-top: 20px; }
.archive-format-card { display: flex; align-items: center; gap: 11px; padding: 13px; border: 1px solid #2b3645; border-radius: 8px; background: #111720; }
.archive-format-icon { display: grid; width: 35px; height: 35px; flex: 0 0 35px; place-items: center; color: var(--accent-strong); border: 1px solid color-mix(in srgb, var(--accent) 22%, #111720); border-radius: 8px; background: var(--accent-alpha-4); font-size: 15px; }
.archive-format-card strong { display: block; color: #e0e8f2; font-size: 11px; }
.archive-format-card span { display: block; margin-top: 5px; color: #7f8da0; font-size: 9px; line-height: 1.5; }
@media (max-width: 1000px) { .archive-file-list { grid-template-columns: repeat(2, minmax(0, 1fr)); } .archive-file-date { display: none; } .archive-format-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
@media (max-width: 720px) { .archive-file-list { grid-template-columns: 1fr; } .archive-file-row { flex-wrap: wrap; } .archive-file-main { flex-basis: 100%; } .archive-category-pill { margin-left: 0; } .archive-file-meta { margin-left: 0; } .archive-open-button { margin-left: 0; } }
@media (max-width: 480px) { .archive-format-grid { grid-template-columns: 1fr; } }
</style>
