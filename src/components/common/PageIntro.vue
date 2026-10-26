<script setup lang="ts">
withDefaults(defineProps<{
  title: string
  description: string
  eyebrow?: string
  icon?: string
}>(), {
  eyebrow: 'MEDIA LIBRARY / WORKSPACE',
  icon: '◈',
})
</script>

<template>
  <div class="page-intro">
    <div class="page-intro-copy">
      <div class="page-eyebrow">{{ eyebrow }}</div>
      <h1 class="page-title">{{ title }}</h1>
      <p class="page-description">{{ description }}</p>
    </div>
    <div class="page-intro-mark" aria-hidden="true">{{ icon }}</div>
  </div>
</template>
