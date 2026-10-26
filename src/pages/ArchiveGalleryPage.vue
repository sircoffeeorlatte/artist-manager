<script setup lang="ts">
import PageIntro from '../components/common/PageIntro.vue'
import GalleryEmptyState from '../components/gallery/GalleryEmptyState.vue'
</script>

<template>
  <PageIntro title="Галерея архивов" description="Будущий каталог архивных файлов. MVP предполагает хранение и показ метаданных, тегов и исходного пути, а открытие архива — во внешней программе." />
  <div class="mb-4 flex flex-wrap gap-2">
    <span class="rounded-md border border-slate-200 bg-white px-3 py-1.5 text-sm text-slate-600">Примеры: ZIP, 7z, RAR, TAR, GZ</span>
    <span class="rounded-md border border-slate-200 bg-white px-3 py-1.5 text-sm text-slate-600">Открытие: внешняя программа — позже</span>
  </div>
  <GalleryEmptyState media-label="архивы" />
</template>
