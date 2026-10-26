<script setup lang="ts">
import { RouterLink } from 'vue-router'

defineProps<{
  mediaLabel: string
}>()
</script>

<template>
  <section class="rounded-xl border border-dashed border-slate-300 bg-white px-5 py-14 text-center">
    <div class="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-slate-100 text-xl text-slate-500" aria-hidden="true">＋</div>
    <h3 class="mt-4 text-base font-semibold text-slate-900">Пока нет файлов</h3>
    <p class="mx-auto mt-2 max-w-xl text-sm leading-6 text-slate-600">
      Здесь будут отображаться {{ mediaLabel }} после подключения импорта и локального хранилища. Сейчас это пустое состояние, а не ошибка.
    </p>
    <RouterLink
      to="/database"
      class="mt-5 inline-flex items-center rounded-lg border border-slate-300 bg-white px-4 py-2 text-sm font-medium text-slate-700 no-underline hover:bg-slate-50"
    >
      Посмотреть структуру базы данных
    </RouterLink>
  </section>
</template>
