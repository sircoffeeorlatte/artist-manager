<script setup lang="ts">
import { RouterLink } from 'vue-router'

const sections = [
  { title: 'Галерея видео', to: '/gallery/videos', description: 'Видео и GIF в одной библиотеке. Импортируй файлы и выбирай удобную плотность сетки.', marker: '▶', hint: 'MP4 · WEBM · GIF', eyebrow: '01 / ВИДЕО' },
  { title: 'Галерея фото', to: '/gallery/photos', description: 'Собирай изображения в одном месте и просматривай их в настраиваемой сетке.', marker: '▧', hint: 'JPG · PNG · WEBP', eyebrow: '02 / ИЗОБРАЖЕНИЯ' },
  { title: 'Галерея архивов', to: '/gallery/archives', description: 'Отдельный каталог сжатых файлов и пакетов, без смешивания с медиа.', marker: '▤', hint: 'ZIP · 7Z · RAR', eyebrow: '03 / АРХИВЫ' },
  { title: 'База файлов', to: '/database', description: 'Табличный взгляд на коллекцию: пути, размеры, категории и теги.', marker: '⌘', hint: 'ПОИСК И ФИЛЬТРЫ', eyebrow: '04 / КАТАЛОГ' },
  { title: 'Настройки хранилища', to: '/settings', description: 'Выбери диск и создай понятную структуру папок IRL, Аниме и 3D.', marker: '⚙', hint: 'ПУТЬ И СТРУКТУРА', eyebrow: '05 / НАСТРОЙКИ' },
]
</script>

<template>
  <section class="home-hero">
    <div class="home-hero-copy">
      <div class="page-eyebrow">ТВОЯ КОЛЛЕКЦИЯ · ТОЛЬКО ЛОКАЛЬНО</div>
      <h2>Весь твой контент.<br /><span>В одном месте.</span></h2>
      <p>Личная медиатека для фото, видео и архивов. Без облака и обязательной регистрации — файлы лежат там, где ты сам выберешь.</p>
      <div class="home-hero-actions">
        <RouterLink to="/gallery/photos" class="btn btn-primary"><span aria-hidden="true">▧</span> Открыть галерею</RouterLink>
        <RouterLink to="/settings" class="btn btn-secondary">Настроить хранилище <span aria-hidden="true">↗</span></RouterLink>
      </div>
    </div>
    <div class="home-hero-art" aria-hidden="true">
      <div class="media-orbit">
        <div class="orbit-core">ML</div>
        <div class="orbit-tile tile-one">▧</div>
        <div class="orbit-tile tile-two">▶</div>
        <div class="orbit-tile tile-three">▤</div>
      </div>
    </div>
  </section>

  <section class="home-stats" aria-label="Состав медиатеки">
    <div class="home-stat"><div class="home-stat-icon">▣</div><div><strong>3 раздела</strong><span>Фото, видео и архивы</span></div></div>
    <div class="home-stat"><div class="home-stat-icon">◫</div><div><strong>3 категории</strong><span>IRL · Аниме · 3D</span></div></div>
    <div class="home-stat"><div class="home-stat-icon">⌂</div><div><strong>На твоём диске</strong><span>Расположение выбираешь ты</span></div></div>
  </section>

  <div class="section-heading">
    <div><h3>Рабочее пространство</h3><p>Выбери, чем займёмся сейчас</p></div>
    <span class="pill">5 РАЗДЕЛОВ</span>
  </div>

  <section class="section-card-grid" aria-label="Страницы приложения">
    <RouterLink v-for="section in sections" :key="section.to" :to="section.to" class="section-card">
      <div class="section-card-top">
        <div class="section-card-icon">{{ section.marker }}</div>
        <span class="section-card-arrow" aria-hidden="true">↗</span>
      </div>
      <div class="eyebrow" style="margin-top: 16px">{{ section.eyebrow }}</div>
      <h4>{{ section.title }}</h4>
      <p>{{ section.description }}</p>
      <span class="section-card-link">{{ section.hint }} <span aria-hidden="true">→</span></span>
    </RouterLink>
  </section>

  <div class="notice notice-warning" style="margin-top: 17px">
    <strong style="color: #f2d69a">Статус MVP.</strong> Сейчас это ранняя версия интерфейса: таблица базы использует демонстрационные записи, а полноценная индексация SQLite, миниатюры и редактирование тегов ещё впереди.
  </div>
</template>
