<script setup lang="ts">
import MediaGridCard from './MediaGridCard.vue'
import type { LibraryMediaItem } from '../../types/media'

withDefaults(defineProps<{
  items: LibraryMediaItem[]
  kind: 'photo' | 'video'
  columns: number
  sourceFor: (item: LibraryMediaItem) => string
  title: string
}>(), {})

const emit = defineEmits<{ open: [index: number] }>()
</script>

<template>
  <section class="gallery-grid" :style="{ '--gallery-columns': columns }" :aria-label="title">
    <MediaGridCard
      v-for="(item, index) in items"
      :key="item.id"
      :item="item"
      :index="index"
      :kind="kind"
      :source="sourceFor(item)"
      @open="emit('open', $event)"
    />
  </section>
</template>
