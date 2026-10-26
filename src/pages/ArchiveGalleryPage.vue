<script setup lang="ts">
import PageIntro from '../components/common/PageIntro.vue'

const formats = [
  { extension: 'ZIP', description: 'Универсальные архивы', accent: '◫' },
  { extension: '7Z', description: 'Сжатие 7-Zip', accent: '▤' },
  { extension: 'RAR', description: 'Архивы WinRAR', accent: '▥' },
  { extension: 'TAR / GZ', description: 'Unix-архивы и сжатие', accent: '▧' },
]
</script>

<template>
  <PageIntro
    title="Галерея архивов"
    eyebrow="LIBRARY / ARCHIVES"
    icon="▤"
    description="Отдельный каталог для ZIP, 7z, RAR и других архивов. Позже здесь появятся поиск, теги и открытие исходного файла во внешнем приложении."
  />

  <section class="panel" style="padding:19px">
    <div class="panel-heading">
      <div>
        <h2 class="panel-title">Форматы архивов</h2>
        <p class="panel-subtitle">Поддерживаемые форматы на уровне будущего каталога. Встроенная распаковка сейчас не выполняется.</p>
      </div>
      <span class="pill">КАТАЛОГ</span>
    </div>
    <div class="archive-format-grid">
      <div v-for="format in formats" :key="format.extension" class="archive-format-card">
        <div class="archive-format-icon">{{ format.accent }}</div>
        <div><strong>{{ format.extension }}</strong><span>{{ format.description }}</span></div>
      </div>
    </div>
  </section>

  <section class="gallery-empty" style="margin-top:15px">
    <div class="gallery-empty-icon">▤</div>
    <h2>Каталог архивов пока пуст</h2>
    <p>На следующем этапе сюда можно добавить сканирование папок «Архивы», карточки с размером и путём, теги и команду «Открыть во внешней программе».</p>
    <span class="pill" style="margin-top:16px">СЛЕДУЮЩИЙ ЭТАП</span>
  </section>
</template>

<style scoped>
.archive-format-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 10px; margin-top: 20px; }
.archive-format-card { display: flex; align-items: center; gap: 11px; padding: 13px; border: 1px solid #2b3645; border-radius: 8px; background: #111720; }
.archive-format-icon { display: grid; width: 35px; height: 35px; flex: 0 0 35px; place-items: center; color: #b9e87d; border: 1px solid #3a4a2e; border-radius: 8px; background: #a3e6350b; font-size: 15px; }
.archive-format-card strong { display: block; color: #e0e8f2; font-size: 11px; }
.archive-format-card span { display: block; margin-top: 5px; color: #7f8da0; font-size: 9px; line-height: 1.5; }
@media (max-width: 900px) { .archive-format-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); } }
@media (max-width: 480px) { .archive-format-grid { grid-template-columns: 1fr; } }
</style>
