<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { convertFileSrc, isTauri } from '@tauri-apps/api/core'
import { getUserConfig } from '../../services/userConfig'
import { registerLibraryRoot } from '../../services/filesystem'
import { joinLibraryPath } from '../../services/wallpapers'

const config = getUserConfig()
const source = ref('')
const failed = ref(false)
let requestId = 0

const isVideo = computed(() => /\.(mp4|mkv|avi|mov|webm|m4v|mpg|mpeg|wmv|flv|3gp|ogv)$/i.test(config.appearance.wallpaperPath))

watch(
  [() => config.libraryRoot, () => config.appearance.wallpaperPath],
  async ([rootPath, relativePath]) => {
    const currentRequest = ++requestId
    source.value = ''
    failed.value = false
    if (!isTauri() || !rootPath || !relativePath) return
    try {
      await registerLibraryRoot(rootPath)
      if (currentRequest !== requestId) return
      source.value = convertFileSrc(joinLibraryPath(rootPath, relativePath))
    } catch {
      // Missing drives or a stale wallpaper path fall back to the configured solid color.
      source.value = ''
    }
  },
  { immediate: true },
)

function mediaFailed(): void {
  failed.value = true
  source.value = ''
}
</script>

<template>
  <div v-if="source && !failed" class="app-wallpaper-layer" aria-hidden="true">
    <video v-if="isVideo" :key="source" class="app-wallpaper-media" :src="source" autoplay muted loop playsinline preload="metadata" @error="mediaFailed" />
    <img v-else :key="source" class="app-wallpaper-media" :src="source" alt="" @error="mediaFailed" />
  </div>
</template>
