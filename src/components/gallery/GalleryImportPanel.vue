<script setup lang="ts">
import { CATEGORY_LABELS, CATEGORY_OPTIONS, type ContentCategory } from '../../types/media'

defineProps<{
  kind: 'photo' | 'video'
  isDesktop: boolean
  destinationCategory: ContentCategory
  categoryFilter: 'all' | ContentCategory
  importMode: 'copy' | 'move'
  busy: boolean
}>()

const emit = defineEmits<{
  'update:destinationCategory': [value: ContentCategory]
  'update:categoryFilter': [value: 'all' | ContentCategory]
  'update:importMode': [value: 'copy' | 'move']
  'add-files': []
}>()

function updateDestinationCategory(event: Event) {
  emit('update:destinationCategory', (event.target as HTMLSelectElement).value as ContentCategory)
}
function updateCategoryFilter(event: Event) {
  emit('update:categoryFilter', (event.target as HTMLSelectElement).value as 'all' | ContentCategory)
}
function updateImportMode(event: Event) {
  emit('update:importMode', (event.target as HTMLSelectElement).value as 'copy' | 'move')
}
</script>

<template>
  <section class="panel gallery-import-panel">
    <div class="panel-heading" style="margin-bottom: 13px">
      <div>
        <h2 class="panel-title">Добавить в коллекцию</h2>
        <p class="panel-subtitle">Импортируй файлы в выбранную категорию или перетащи их на область галереи.</p>
      </div>
      <span class="pill pill-accent">{{ isDesktop ? 'ИМПОРТ В ХРАНИЛИЩЕ' : 'РЕЖИМ ПРЕВЬЮ' }}</span>
    </div>

    <div class="gallery-import-grid">
      <label>
        <span class="field-label">Категория для загрузки</span>
        <select
          class="field-control"
          :value="destinationCategory"
          @change="updateDestinationCategory"
        >
          <option v-for="category in CATEGORY_OPTIONS" :key="category.value" :value="category.value">{{ category.label }}</option>
        </select>
      </label>
      <label>
        <span class="field-label">Фильтр по категории</span>
        <select
          class="field-control"
          :value="categoryFilter"
          @change="updateCategoryFilter"
        >
          <option value="all">Все категории</option>
          <option v-for="category in CATEGORY_OPTIONS" :key="category.value" :value="category.value">{{ category.label }}</option>
        </select>
      </label>
      <label v-if="isDesktop">
        <span class="field-label">Способ импорта</span>
        <select
          class="field-control"
          :value="importMode"
          @change="updateImportMode"
        >
          <option value="copy">Копировать — оставить оригинал</option>
          <option value="move">Переместить после проверки копии</option>
        </select>
      </label>
      <div class="gallery-add-action">
        <button type="button" class="btn btn-primary" :disabled="busy" @click="emit('add-files')">
          <span aria-hidden="true">＋</span>{{ busy ? 'Импорт…' : (kind === 'photo' ? 'Добавить фото' : 'Добавить видео') }}
        </button>
      </div>
    </div>

    <p class="panel-subtitle" style="margin-top: 13px">
      <template v-if="isDesktop">Файлы сохраняются в категорию «{{ CATEGORY_LABELS[destinationCategory] }}». При перемещении исходник удаляется только после успешного копирования и проверки размера.</template>
      <template v-else>В браузере файлы используются только для временного предпросмотра и не записываются на диск медиатеки.</template>
    </p>
  </section>
</template>
