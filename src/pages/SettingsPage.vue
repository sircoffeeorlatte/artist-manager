<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import PageIntro from '../components/common/PageIntro.vue'
import WallpaperLibrary from '../components/appearance/WallpaperLibrary.vue'
import { selectLibraryParentFolder } from '../services/nativeDialogs'
import { createLibraryStructure, registerLibraryRoot } from '../services/filesystem'
import { getSavedLibraryRoot, joinPath, loadLibraryRoot, persistLibraryRoot, splitPath } from '../services/librarySettings'
import { applyAppearanceSettings, DEFAULT_APPEARANCE_SETTINGS, getSavedAppearanceSettings } from '../services/appearanceSettings'
import { beginActivity, errorText, finishActivity } from '../services/appState'
import { getUserConfig, getUserConfigPath, updateUserConfig, type StartupPage } from '../services/userConfig'
import { openContainingFolder, openFolderOrParent } from '../services/fileLocations'

const parentPath = ref('')
const folderName = ref('MediaLibrary')
const currentRoot = ref(getSavedLibraryRoot())
const busy = ref(false)
const errorMessage = ref('')
const statusMessage = ref('')
const appearance = ref(getSavedAppearanceSettings())
const userConfig = getUserConfig()
const startupPage = ref<StartupPage>(userConfig.startupPage)
const configPath = ref('Определяем путь конфигурации…')
const startupPages: Array<{ value: StartupPage; label: string }> = [
  { value: '/', label: 'Главная' },
  { value: '/gallery/videos', label: 'Галерея видео' },
  { value: '/gallery/photos', label: 'Галерея фото' },
  { value: '/gallery/archives', label: 'Галерея архивов' },
  { value: '/import', label: 'Импорт файлов' },
  { value: '/database', label: 'База данных файлов' },
  { value: '/settings', label: 'Настройки' },
]
const isDesktop = computed(() => isTauri())
const configPathAvailable = computed(() => {
  const path = configPath.value.trim()
  return isDesktop.value && (/^[a-zA-Z]:[\\/]/.test(path) || path.startsWith('\\\\') || path.startsWith('/'))
})
const previewRoot = computed(() => parentPath.value.trim() && folderName.value.trim() ? joinPath(parentPath.value.trim(), folderName.value.trim()) : currentRoot.value)
const categories = [
  { title: 'IRL', folder: 'IRL', description: 'Реальные фото и видео', folders: ['Фото', 'Видео', 'Архивы'] },
  { title: 'Аниме', folder: 'Аниме', description: 'Анимация и связанные медиа', folders: ['Фото', 'Видео', 'Архивы'] },
  { title: '3D', folder: '3D', description: 'Рендеры и 3D-контент', folders: ['Фото', 'Видео', 'Архивы'] },
]
const folderTreeText = computed(() => {
  const lines = [`${folderName.value || 'MediaLibrary'}/`]
  for (const category of categories) {
    lines.push(`├── ${category.folder}/`)
    category.folders.forEach((folder, index) => {
      lines.push(`│   ${index === category.folders.length - 1 ? '└' : '├'}── ${folder}/`)
    })
  }
  lines.push('└── wallpapers/')
  return lines.join('\n')
})
const backgroundPresets = [
  { name: 'Графит', color: '#101114' },
  { name: 'Холодный', color: '#10151c' },
  { name: 'Сине-серый', color: '#111923' },
  { name: 'Баклажан', color: '#19121b' },
  { name: 'Тёмный зелёный', color: '#111914' },
  { name: 'Тёплый', color: '#191613' },
  { name: 'Светлый', color: '#f5f7fa' },
  { name: 'Белый', color: '#ffffff' },
  { name: 'Светло-серый', color: '#e9edf3' },
]

watch(appearance, (value) => applyAppearanceSettings(value), { deep: true, flush: 'sync' })
watch(startupPage, (value) => updateUserConfig({ startupPage: value }))

function setAppearanceTheme(theme: 'dark' | 'light') {
  const previousDefault = appearance.value.theme === 'light' ? '#f5f7fa' : '#101114'
  const nextDefault = theme === 'light' ? '#f5f7fa' : '#101114'
  // Follow the theme's recommended background until the user chooses a custom one.
  if (appearance.value.backgroundColor.toLowerCase() === previousDefault) {
    appearance.value.backgroundColor = nextDefault
  }
  appearance.value.theme = theme
}

function handleThemeChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value
  setAppearanceTheme(value === 'light' ? 'light' : 'dark')
}

function resetAppearance() {
  appearance.value = { ...DEFAULT_APPEARANCE_SETTINGS }
}

async function openStorageFolder(path: string) {
  try { await openFolderOrParent(path) }
  catch (error) { errorMessage.value = errorText(error) }
}

async function openConfigFolder() {
  if (!configPathAvailable.value) return
  try { await openContainingFolder(configPath.value) }
  catch (error) { errorMessage.value = errorText(error) }
}

onMounted(async () => {
  const activityId = beginActivity({ kind: 'system', title: 'Загрузка настроек', details: 'Чтение параметров медиатеки' })
  try {
    currentRoot.value = await loadLibraryRoot()
    try { configPath.value = await getUserConfigPath() }
    catch (error) { configPath.value = errorText(error) }
    startupPage.value = userConfig.startupPage
    if (currentRoot.value) {
      const split = splitPath(currentRoot.value)
      parentPath.value = split.parent
      folderName.value = split.name || 'MediaLibrary'
    }
    finishActivity(activityId, 'success', currentRoot.value ? 'Путь медиатеки загружен.' : 'Путь медиатеки пока не задан.')
  } catch (error) {
    const message = errorText(error)
    errorMessage.value = message
    finishActivity(activityId, 'error', message)
  }
})

async function chooseParent() {
  errorMessage.value = ''
  statusMessage.value = ''
  const activityId = beginActivity({ kind: 'system', title: 'Выбор папки хранения', details: 'Ожидание выбора в системном диалоге' })
  try {
    const selected = await selectLibraryParentFolder()
    if (selected) {
      parentPath.value = selected
      finishActivity(activityId, 'success', `Выбрана папка: ${selected}`)
    } else {
      finishActivity(activityId, 'success', 'Выбор папки отменён.')
    }
  } catch (error) {
    const message = errorText(error)
    errorMessage.value = message
    finishActivity(activityId, 'error', message)
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
  const activityId = beginActivity({ kind: 'system', title: 'Создание структуры медиатеки', details: `Папка: ${previewRoot.value}` })
  try {
    const createdRoot = await createLibraryStructure(previewRoot.value)
    await registerLibraryRoot(createdRoot)
    await persistLibraryRoot(createdRoot)
    currentRoot.value = createdRoot
    const split = splitPath(createdRoot)
    parentPath.value = split.parent
    folderName.value = split.name
    statusMessage.value = `Медиатека готова: ${createdRoot}. Существующие файлы не удалялись.`
    finishActivity(activityId, 'success', `Структура создана или проверена: ${createdRoot}`)
  } catch (error) {
    const message = errorText(error)
    errorMessage.value = message
    finishActivity(activityId, 'error', message)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <PageIntro title="Настройки" eyebrow="WORKSPACE / PREFERENCES" icon="⚙" description="Настрой место хранения медиатеки, цвета и масштаб интерфейса. Настройки внешнего вида сохраняются автоматически." />

  <section class="panel settings-current-storage-panel">
    <div class="panel-heading">
      <div><h2 class="panel-title">Текущее хранилище</h2><p class="panel-subtitle">Сохранённая папка, с которой приложение работает прямо сейчас.</p></div>
      <div class="settings-current-storage-actions">
        <span class="pill" :class="currentRoot ? 'pill-accent' : ''">{{ currentRoot ? 'НАСТРОЕНО' : 'НЕ ВЫБРАНО' }}</span>
        <button type="button" class="btn btn-secondary" :disabled="!isDesktop || !currentRoot" @click="openStorageFolder(currentRoot)">Открыть папку ↗</button>
      </div>
    </div>
    <button v-if="currentRoot" type="button" class="path-open-button settings-current-storage-path selectable-path" :disabled="!isDesktop" title="Открыть текущую папку хранилища" @click="openStorageFolder(currentRoot)">
      <code>{{ currentRoot }}</code><span aria-hidden="true">↗</span>
    </button>
    <p v-else class="panel-subtitle settings-current-storage-empty">Сначала выбери или создай папку медиатеки ниже.</p>
  </section>

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
        <button type="button" class="path-open-button settings-storage-path-action selectable-path" :disabled="!isDesktop || !previewRoot" title="Открыть папку хранилища или её родительскую папку" @click="openStorageFolder(previewRoot)">
          <strong>{{ previewRoot || 'Сначала выбери папку на диске' }}</strong><span aria-hidden="true">↗</span>
        </button>
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
      <pre class="folder-tree-plain" aria-label="Структура папок медиатеки">{{ folderTreeText }}</pre>
      <p class="panel-subtitle folder-tree-footnote">Категории содержат фото, видео и архивы. Обои хранятся отдельно в корневой папке wallpapers и не смешиваются с галереями.</p>
    </section>
  </div>

  <section class="panel settings-appearance-panel">
    <div class="panel-heading">
      <div>
        <h2 class="panel-title">Внешний вид</h2>
        <p class="panel-subtitle">Подстрой тему, прозрачность поверхностей, акцент, фон и масштаб. Изменения применяются сразу.</p>
      </div>
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

      <div class="appearance-control">
        <div class="appearance-control-heading">
          <label for="app-theme" class="appearance-control-title">Тема приложения</label>
          <strong class="appearance-control-value">{{ appearance.theme === 'light' ? 'Светлая' : 'Тёмная' }}</strong>
        </div>
        <p class="appearance-control-hint">Выбери светлую или тёмную палитру. Настройка применяется сразу.</p>
        <select id="app-theme" class="field-control appearance-select" :value="appearance.theme" @change="handleThemeChange">
          <option value="dark">Тёмная тема</option>
          <option value="light">Светлая тема</option>
        </select>
      </div>

      <div class="appearance-control">
        <div class="appearance-control-heading">
          <label for="visual-mode" class="appearance-control-title">Стиль поверхностей</label>
          <strong class="appearance-control-value">{{ appearance.visualMode === 'liquid-glass' ? 'Liquid Glass' : 'Standard' }}</strong>
        </div>
        <p class="appearance-control-hint">Liquid Glass — прозрачные поверхности с размытием фона. Standard — обычные непрозрачные панели.</p>
        <select id="visual-mode" v-model="appearance.visualMode" class="field-control appearance-select">
          <option value="liquid-glass">Liquid Glass</option>
          <option value="standard">Standard</option>
        </select>
      </div>

      <div class="appearance-control appearance-background-control">
        <div class="appearance-control-heading">
          <label for="app-background-color" class="appearance-control-title">Цвет фона приложения</label>
          <strong class="appearance-control-value">{{ appearance.backgroundColor.toUpperCase() }}</strong>
        </div>
        <p class="appearance-control-hint">Меняет оттенок фона приложения. Яркость выбранного цвета автоматически подстраивается под текущую тему, чтобы текст оставался читаемым; исходный HEX сохраняется в конфигурации.</p>
        <div class="appearance-color-picker-row">
          <input id="app-background-color" v-model="appearance.backgroundColor" class="appearance-color-input" type="color" aria-label="Цвет фона приложения" />
          <div class="appearance-color-presets" aria-label="Готовые цвета фона">
            <button
              v-for="preset in backgroundPresets"
              :key="preset.color"
              type="button"
              class="appearance-color-preset"
              :class="{ 'is-active': appearance.backgroundColor.toLowerCase() === preset.color }"
              :style="{ backgroundColor: preset.color }"
              :aria-label="preset.name"
              :title="`${preset.name}: ${preset.color}`"
              @click="appearance.backgroundColor = preset.color"
            />
          </div>
        </div>
      </div>
    </div>

    <WallpaperLibrary v-model="appearance.wallpaperPath" :root-path="currentRoot" />

    <div style="display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:10px;margin-top:13px">
      <p class="panel-subtitle" style="margin:0">Тема, стиль поверхностей, акцент, фон и масштаб сохраняются автоматически в конфигурации пользователя.</p>
      <button type="button" class="btn btn-secondary appearance-reset" style="margin-top:0" @click="resetAppearance">Сбросить внешний вид</button>
    </div>
  </section>

  <section class="panel settings-run-panel">
    <div class="panel-heading">
      <div>
        <h2 class="panel-title">Запуск приложения и конфигурация</h2>
        <p class="panel-subtitle">Выбери страницу, которую нужно открывать при обычном запуске, и посмотри, где хранится конфиг пользователя.</p>
      </div>
      <span class="pill pill-accent">USER CONFIG</span>
    </div>

    <div class="settings-run-grid">
      <label>
        <span class="field-label">Страница при запуске</span>
        <select v-model="startupPage" class="field-control">
          <option v-for="page in startupPages" :key="page.value" :value="page.value">{{ page.label }}</option>
        </select>
        <span class="appearance-control-hint">Настройка применяется при следующем запуске, если приложение открывается с главной страницы. Прямая ссылка на другую страницу имеет приоритет.</span>
      </label>
      <div class="settings-config-location">
        <span class="field-label">Файл настроек пользователя</span>
        <button type="button" class="path-open-button settings-config-path selectable-path" :disabled="!configPathAvailable" title="Открыть папку с user-config.json" @click="openConfigFolder">
          <code>{{ configPath }}</code><span aria-hidden="true">↗</span>
        </button>
        <p class="appearance-control-hint">В JSON сохраняются тема, стиль Liquid Glass/Standard, акцент, фон, масштаб, громкость, плотность сеток, режим видео, стартовая страница и пути медиатеки.</p>
      </div>
    </div>
  </section>


</template>
