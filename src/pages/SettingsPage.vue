<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import PageIntro from '../components/common/PageIntro.vue'
import { selectLibraryParentFolder } from '../services/nativeDialogs'
import { createLibraryStructure, registerLibraryRoot } from '../services/filesystem'
import { getSavedLibraryRoot, joinPath, loadLibraryRoot, persistLibraryRoot, splitPath } from '../services/librarySettings'
import { applyAppearanceSettings, DEFAULT_APPEARANCE_SETTINGS, getSavedAppearanceSettings } from '../services/appearanceSettings'

const parentPath = ref('')
const folderName = ref('MediaLibrary')
const currentRoot = ref(getSavedLibraryRoot())
const busy = ref(false)
const errorMessage = ref('')
const statusMessage = ref('')
const appearance = ref(getSavedAppearanceSettings())
const isDesktop = computed(() => isTauri())
const previewRoot = computed(() => parentPath.value.trim() && folderName.value.trim() ? joinPath(parentPath.value.trim(), folderName.value.trim()) : currentRoot.value)
const categories = [
  { title: 'IRL', folder: 'IRL', folders: ['Фото', 'Видео', 'Архивы'] },
  { title: 'Аниме', folder: 'Аниме', folders: ['Фото', 'Видео', 'Архивы'] },
  { title: '3D', folder: '3D', folders: ['Фото', 'Видео', 'Архивы'] },
]

watch(appearance, (value) => applyAppearanceSettings(value), { deep: true, flush: 'sync' })

function resetAppearance() {
  appearance.value = { ...DEFAULT_APPEARANCE_SETTINGS }
}

onMounted(async () => {
  currentRoot.value = await loadLibraryRoot()
  if (currentRoot.value) {
    const split = splitPath(currentRoot.value)
    parentPath.value = split.parent
    folderName.value = split.name || 'MediaLibrary'
  }
})

async function chooseParent() {
  errorMessage.value = ''
  statusMessage.value = ''
  try {
    const selected = await selectLibraryParentFolder()
    if (selected) parentPath.value = selected
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  }
}

async function createLibrary() {
  errorMessage.value = ''
  statusMessage.value = ''
  if (!isDesktop.value) {
    errorMessage.value = 'Создание папок на дисках доступно только в настольном приложении Tauri. В браузере эта кнопка специально не меняет файловую систему.'
    return
  }
  if (!parentPath.value.trim()) {
    errorMessage.value = 'Сначала выбери папку-родитель на диске.'
    return
  }
  if (!folderName.value.trim() || /[\\/]/.test(folderName.value) || ['.', '..'].includes(folderName.value.trim())) {
    errorMessage.value = 'Укажи имя одной папки без символов \\ и /.'
    return
  }
  busy.value = true
  try {
    const createdRoot = await createLibraryStructure(previewRoot.value)
    await registerLibraryRoot(createdRoot)
    await persistLibraryRoot(createdRoot)
    currentRoot.value = createdRoot
    const split = splitPath(createdRoot)
    parentPath.value = split.parent
    folderName.value = split.name
    statusMessage.value = `Медиатека готова: ${createdRoot}. Существующие файлы не удалялись.`
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <PageIntro title="Настройки" eyebrow="WORKSPACE / PREFERENCES" icon="⚙" description="Настрой место хранения медиатеки, акцентный цвет и масштаб интерфейса. Настройки внешнего вида сохраняются автоматически." />

  <div class="settings-layout">
    <section class="panel settings-panel">
      <div class="panel-heading">
        <div><h2 class="panel-title">Расположение медиатеки</h2><p class="panel-subtitle">Укажи папку-родитель и имя отдельного хранилища.</p></div>
        <span class="pill" :class="isDesktop ? 'pill-accent' : ''"><span v-if="isDesktop" class="status-light"></span>{{ isDesktop ? 'TAURI DESKTOP' : 'БРАУЗЕРНЫЙ РЕЖИМ' }}</span>
      </div>

      <div v-if="!isDesktop" class="notice notice-warning" style="margin-top:16px">В браузере можно осмотреть интерфейс, но создавать папки на дисках Windows нельзя. Для выбора пути запусти настольную версию через <code>run-desktop.cmd</code>.</div>

      <div style="margin-top:21px">
        <label>
          <span class="field-label">Где создать папку хранилища</span>
          <div style="display:flex;gap:8px">
            <input v-model="parentPath" class="field-control" placeholder="Например, D:\Media" readonly />
            <button type="button" class="btn btn-secondary" style="flex-shrink:0" :disabled="!isDesktop || busy" @click="chooseParent">Выбрать…</button>
          </div>
        </label>
        <label style="display:block;margin-top:15px">
          <span class="field-label">Имя корневой папки</span>
          <input v-model="folderName" class="field-control" placeholder="MediaLibrary" />
        </label>
      </div>

      <div class="settings-path-preview">
        <p>ПОЛНЫЙ ПУТЬ ХРАНИЛИЩА</p>
        <strong>{{ previewRoot || 'Сначала выбери папку на диске' }}</strong>
      </div>

      <div style="display:flex;flex-wrap:wrap;gap:9px;margin-top:17px">
        <button type="button" class="btn btn-primary" :disabled="busy || !isDesktop" @click="createLibrary"><span aria-hidden="true">{{ busy ? '…' : '＋' }}</span>{{ busy ? 'Создание папок…' : (currentRoot ? 'Создать / проверить структуру' : 'Создать медиатеку') }}</button>
        <button v-if="currentRoot" type="button" class="btn btn-secondary" @click="parentPath = splitPath(currentRoot).parent; folderName = splitPath(currentRoot).name">Вернуть текущий путь</button>
      </div>
      <div v-if="statusMessage" class="notice notice-success" style="margin-top:14px;white-space:pre-line">{{ statusMessage }}</div>
      <div v-if="errorMessage" class="notice notice-error" style="margin-top:14px;white-space:pre-line">{{ errorMessage }}</div>
    </section>

    <section class="panel settings-panel">
      <div class="panel-heading">
        <div><h2 class="panel-title">Структура папок</h2><p class="panel-subtitle">Папки создаются автоматически. Уже существующие файлы не удаляются.</p></div>
      </div>
      <div class="folder-tree" style="margin-top:17px">
        <div class="folder-root">{{ folderName || 'MediaLibrary' }}/</div>
        <div v-for="(category, categoryIndex) in categories" :key="category.folder">
          <div class="folder-category">{{ categoryIndex === categories.length - 1 ? '└─' : '├─' }} {{ category.folder }}/</div>
          <div v-for="(folder, index) in category.folders" :key="folder" class="folder-leaf" style="padding-left:18px">
            {{ index === category.folders.length - 1 ? '└─' : '├─' }} {{ folder }}/
          </div>
        </div>
      </div>
      <div class="category-preview-grid">
        <div class="category-preview"><strong>IRL</strong><span>Реальные фото и видео</span></div>
        <div class="category-preview"><strong>Аниме</strong><span>Анимация и связанные медиа</span></div>
        <div class="category-preview"><strong>3D</strong><span>Рендеры и 3D-контент</span></div>
      </div>
      <p class="panel-subtitle" style="margin-top:15px">В каждой категории будут подпапки «Фото», «Видео» и «Архивы». В будущем автора можно учитывать как тег типа <code>creator</code>; отдельную страницу авторов пока не добавляем.</p>
    </section>
  </div>

  <section class="panel settings-appearance-panel">
    <div class="panel-heading">
      <div>
        <h2 class="panel-title">Внешний вид</h2>
        <p class="panel-subtitle">Подстрой акцент и размер интерфейса под себя. Изменения применяются сразу.</p>
      </div>
      <span class="pill pill-accent">LIVE PREVIEW</span>
    </div>

    <div class="appearance-controls">
      <div class="appearance-control">
        <div class="appearance-control-heading">
          <label for="accent-hue" class="appearance-control-title">Акцентный цвет</label>
          <strong class="appearance-control-value">{{ appearance.accentHue }}°</strong>
        </div>
        <p class="appearance-control-hint">Меняет подсветку навигации, активные кнопки, рамки и другие элементы интерфейса.</p>
        <input id="accent-hue" v-model.number="appearance.accentHue" class="appearance-slider appearance-hue-slider" type="range" min="0" max="360" step="1" aria-label="Оттенок акцентного цвета" />
        <div class="appearance-slider-ends"><span>Красный</span><span>Жёлтый</span><span>Зелёный</span><span>Синий</span><span>Фиолетовый</span></div>
      </div>

      <div class="appearance-control">
        <div class="appearance-control-heading">
          <label for="interface-scale" class="appearance-control-title">Масштаб интерфейса</label>
          <strong class="appearance-control-value">{{ appearance.scale }}%</strong>
        </div>
        <p class="appearance-control-hint">Увеличь элементы, если текст мелкий, или уменьши масштаб, чтобы поместить больше содержимого.</p>
        <input id="interface-scale" v-model.number="appearance.scale" class="appearance-slider" type="range" min="80" max="120" step="5" aria-label="Масштаб интерфейса в процентах" />
        <div class="appearance-scale-markers"><span>80% · Компактно</span><span>100% · Обычно</span><span>120% · Крупно</span></div>
      </div>
    </div>

    <div class="appearance-preview">
      <span class="appearance-preview-label">Предпросмотр акцента</span>
      <div class="appearance-preview-samples">
        <span class="appearance-preview-mark" aria-hidden="true">✓</span>
        <button type="button" class="btn btn-primary">Акцентная кнопка</button>
        <span class="pill pill-accent">Выбрано</span>
      </div>
    </div>
    <div style="display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:10px;margin-top:13px">
      <p class="panel-subtitle" style="margin:0">Цвет и масштаб сохраняются автоматически на этом устройстве.</p>
      <button type="button" class="btn btn-secondary appearance-reset" style="margin-top:0" @click="resetAppearance">Сбросить внешний вид</button>
    </div>
  </section>

  <section v-if="currentRoot" class="panel" style="margin-top:15px;padding:17px">
    <div class="panel-heading">
      <div><h2 class="panel-title">Текущее хранилище</h2><p class="panel-subtitle">Сохранённый путь, который приложение использует для работы с файлами.</p></div>
      <span class="pill pill-accent">НАСТРОЕНО</span>
    </div>
    <p style="margin:12px 0 0;color:#c8d5e4;font-family:ui-monospace,SFMono-Regular,Consolas,monospace;font-size:11px;line-height:1.8;overflow-wrap:anywhere">{{ currentRoot }}</p>
  </section>
</template>
