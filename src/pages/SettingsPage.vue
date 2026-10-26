<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import PageIntro from '../components/common/PageIntro.vue'
import { selectLibraryParentFolder } from '../services/nativeDialogs'
import { createLibraryStructure, registerLibraryRoot } from '../services/filesystem'
import { getSavedLibraryRoot, joinPath, loadLibraryRoot, persistLibraryRoot, splitPath } from '../services/librarySettings'

const parentPath = ref('')
const folderName = ref('MediaLibrary')
const currentRoot = ref(getSavedLibraryRoot())
const busy = ref(false)
const errorMessage = ref('')
const statusMessage = ref('')
const isDesktop = computed(() => isTauri())
const previewRoot = computed(() => parentPath.value.trim() && folderName.value.trim() ? joinPath(parentPath.value.trim(), folderName.value.trim()) : currentRoot.value)
const categories = [
  { title: 'IRL', folder: 'IRL', folders: ['Фото', 'Видео', 'Архивы'] },
  { title: 'Аниме', folder: 'Аниме', folders: ['Фото', 'Видео', 'Архивы'] },
  { title: '3D', folder: '3D', folders: ['Фото', 'Видео', 'Архивы'] },
]

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
  <PageIntro title="Настройки" description="Выбери место на диске для медиатеки. Приложение создаст папку-хранилище с категориями и подкаталогами; при обновлении или удалении приложения эта папка не удаляется." />

  <section class="rounded-xl border border-slate-200 bg-white p-5">
    <div class="flex flex-wrap items-start justify-between gap-3">
      <div>
        <h2 class="font-semibold text-slate-900">Расположение медиатеки</h2>
        <p class="mt-1 max-w-2xl text-sm leading-6 text-slate-500">Например, можно выбрать диск D: и создать там папку <code class="rounded bg-slate-100 px-1.5 py-0.5">MediaLibrary</code>. Это отдельное хранилище, не папка установки программы.</p>
      </div>
      <span class="rounded-full px-2.5 py-1 text-xs font-medium" :class="isDesktop ? 'bg-emerald-50 text-emerald-700' : 'bg-amber-50 text-amber-800'">{{ isDesktop ? 'Desktop Tauri' : 'Браузерный режим' }}</span>
    </div>

    <div v-if="!isDesktop" class="mt-4 rounded-lg border border-amber-200 bg-amber-50 p-3 text-sm leading-6 text-amber-900">В браузере можно проверить интерфейс, но браузер не может создавать произвольные папки на дисках. Запусти <code>run-desktop.cmd</code>, чтобы выбрать путь через системный диалог Windows.</div>

    <div class="mt-5 grid gap-4 md:grid-cols-[minmax(0,1fr)_220px]">
      <label class="block">
        <span class="mb-1.5 block text-xs font-medium text-slate-600">Папка, внутри которой создать хранилище</span>
        <div class="flex gap-2">
          <input v-model="parentPath" class="min-w-0 flex-1 rounded-lg border border-slate-300 bg-white px-3 py-2.5 text-sm" placeholder="Например: D:\\" readonly />
          <button type="button" class="shrink-0 rounded-lg border border-slate-300 px-3 py-2 text-sm font-medium text-slate-700 hover:bg-slate-50 disabled:opacity-50" :disabled="!isDesktop || busy" @click="chooseParent">Выбрать…</button>
        </div>
      </label>
      <label class="block">
        <span class="mb-1.5 block text-xs font-medium text-slate-600">Имя новой папки</span>
        <input v-model="folderName" class="w-full rounded-lg border border-slate-300 px-3 py-2.5 text-sm" placeholder="MediaLibrary" />
      </label>
    </div>

    <div class="mt-4 rounded-lg bg-slate-50 p-3">
      <p class="text-xs font-medium text-slate-500">Полный путь хранилища</p>
      <p class="mt-1 break-all font-mono text-sm text-slate-800">{{ previewRoot || 'Сначала выбери место на диске' }}</p>
    </div>

    <div class="mt-4 flex flex-wrap items-center gap-3">
      <button type="button" class="rounded-lg bg-slate-900 px-4 py-2.5 text-sm font-semibold text-white hover:bg-slate-700 disabled:cursor-wait disabled:opacity-60" :disabled="busy || !isDesktop" @click="createLibrary">{{ busy ? 'Создание папок…' : (currentRoot ? 'Создать / проверить структуру' : 'Создать папку медиатеки') }}</button>
      <button v-if="currentRoot" type="button" class="rounded-lg border border-slate-300 px-4 py-2.5 text-sm font-medium text-slate-700 hover:bg-slate-50" @click="parentPath = splitPath(currentRoot).parent; folderName = splitPath(currentRoot).name">Вернуться к текущему пути</button>
    </div>
    <div v-if="statusMessage" class="mt-4 whitespace-pre-line rounded-lg border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-900">{{ statusMessage }}</div>
    <div v-if="errorMessage" class="mt-4 whitespace-pre-line rounded-lg border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-900">{{ errorMessage }}</div>
  </section>

  <section class="mt-5 rounded-xl border border-slate-200 bg-white p-5">
    <h2 class="font-semibold text-slate-900">Структура каталогов</h2>
    <p class="mt-1 text-sm text-slate-500">Будет создана автоматически. Существующие каталоги и файлы не перезаписываются.</p>
    <div class="mt-4 rounded-lg bg-slate-950 p-4 font-mono text-xs leading-6 text-slate-100 sm:text-sm">
      <div>{{ folderName || 'MediaLibrary' }}/</div>
      <div v-for="category in categories" :key="category.folder" class="pl-4">
        <div>├─ {{ category.folder }}/</div>
        <div v-for="(folder, index) in category.folders" :key="folder" class="pl-6" :class="index === category.folders.length - 1 ? 'text-emerald-300' : 'text-slate-300'">{{ index === category.folders.length - 1 ? '└─' : '├─' }} {{ folder }}/</div>
      </div>
    </div>
    <p class="mt-4 text-sm leading-6 text-slate-600">Внутри каждой категории будут папки «Фото», «Видео» и «Архивы». Автор фотографии в будущем будет задаваться тегом типа <code>creator</code>; страницу авторов и интерфейс тегов пока не делаем.</p>
  </section>

  <section v-if="currentRoot" class="mt-5 rounded-xl border border-slate-200 bg-white p-5">
    <h2 class="font-semibold">Текущее хранилище</h2>
    <p class="mt-2 break-all font-mono text-sm text-slate-700">{{ currentRoot }}</p>
    <p class="mt-2 text-xs leading-5 text-slate-500">Программа не удаляет эту папку при обновлении или деинсталляции. Пока используется файловый сканер; полноценная индексация в SQLite будет добавлена отдельно.</p>
  </section>
</template>
