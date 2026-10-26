<script setup lang="ts">
defineProps<{
  title: string
  description: string
}>()
</script>

<template>
  <div class="mb-6">
    <h2 class="text-2xl font-semibold tracking-tight text-slate-900">{{ title }}</h2>
    <p class="mt-2 max-w-3xl text-sm leading-6 text-slate-600">{{ description }}</p>
  </div>
</template>
