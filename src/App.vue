<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import AppLayout from './layouts/AppLayout.vue'
import {
  deferFolderAccessConsent,
  folderAccessConsentState,
  grantFolderAccessConsent,
  initializeFolderAccessConsent,
} from './services/folderAccessConsent'

const showFolderConsent = computed(() => isTauri() && folderAccessConsentState.visible)
onMounted(() => initializeFolderAccessConsent())
</script>

<template>
  <AppLayout />

  <Teleport to="body">
    <div v-if="showFolderConsent" class="folder-access-consent-overlay">
      <section
        class="folder-access-consent-panel"
        role="dialog"
        aria-modal="true"
        aria-labelledby="folder-access-consent-title"
        aria-describedby="folder-access-consent-description"
      >
        <div class="folder-access-consent-icon" aria-hidden="true">↗</div>
        <p class="folder-access-consent-eyebrow">ДОСТУП К ПАПКАМ</p>
        <h1 id="folder-access-consent-title">Разрешить открывать папки в Проводнике?</h1>
        <p id="folder-access-consent-description" class="folder-access-consent-description">
          По нажатию на путь приложение сможет открыть Проводник Windows для папки медиатеки,
          файла базы данных, пользовательского конфига или папки с фото и видео.
          Это разрешение используется только для явного действия по твоему нажатию.
        </p>
        <div class="folder-access-consent-details">
          <span aria-hidden="true">✓</span>
          <span>Приложение не получает дополнительных прав на чтение или изменение файлов через этот запрос.</span>
        </div>
        <div class="folder-access-consent-actions">
          <button type="button" class="btn btn-secondary" @click="deferFolderAccessConsent">Не сейчас</button>
          <button type="button" class="btn btn-primary" @click="void grantFolderAccessConsent()">Разрешить</button>
        </div>
        <p class="folder-access-consent-footnote">
          Пока доступ не разрешён, кнопки открытия папок будут запрашивать согласие повторно.
        </p>
      </section>
    </div>
  </Teleport>
</template>

<style scoped>
.folder-access-consent-overlay {
  position: fixed;
  z-index: 100000;
  inset: 0;
  display: grid;
  place-items: center;
  padding: 22px;
  background: rgb(4 7 12 / 72%);
  backdrop-filter: blur(12px);
  -webkit-backdrop-filter: blur(12px);
}

.folder-access-consent-panel {
  width: min(500px, 100%);
  padding: 27px;
  color: var(--text, #e5eaf2);
  border: 1px solid var(--border, #273140);
  border-radius: 17px;
  background: color-mix(in srgb, var(--panel-raised, #171f2b) 96%, transparent);
  box-shadow: 0 26px 90px rgb(0 0 0 / 42%);
}

.folder-access-consent-icon {
  display: grid;
  width: 42px;
  height: 42px;
  margin-bottom: 19px;
  place-items: center;
  color: var(--accent-text, #18230a);
  border-radius: 11px;
  background: var(--accent, #a3e635);
  font-size: 22px;
  font-weight: 700;
}

.folder-access-consent-eyebrow {
  margin: 0 0 8px;
  color: var(--accent, #a3e635);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 1.25px;
}

.folder-access-consent-panel h1 {
  margin: 0;
  color: var(--text, #e5eaf2);
  font-size: 22px;
  font-weight: 700;
  line-height: 1.3;
}

.folder-access-consent-description {
  margin: 13px 0 0;
  color: var(--text-muted, #9aa8ba);
  font-size: 13px;
  line-height: 1.7;
}

.folder-access-consent-details {
  display: flex;
  align-items: flex-start;
  gap: 9px;
  margin-top: 16px;
  padding: 12px;
  color: var(--text-muted, #9aa8ba);
  border: 1px solid var(--border, #273140);
  border-radius: 9px;
  background: color-mix(in srgb, var(--panel, #131923) 72%, transparent);
  font-size: 11px;
  line-height: 1.6;
}

.folder-access-consent-details span:first-child {
  color: var(--accent, #a3e635);
  font-weight: 800;
}

.folder-access-consent-actions {
  display: flex;
  justify-content: flex-end;
  gap: 9px;
  margin-top: 23px;
}

.folder-access-consent-footnote {
  margin: 14px 0 0;
  color: var(--text-dim, #68788c);
  font-size: 10px;
  line-height: 1.55;
}

:global(html[data-theme="light"]) .folder-access-consent-panel {
  color: var(--text, #202733);
  border-color: rgb(28 39 54 / 16%);
  background: color-mix(in srgb, var(--panel-raised, #ffffff) 96%, transparent);
  box-shadow: 0 24px 80px rgb(20 30 45 / 25%);
}

:global(html[data-theme="light"]) .folder-access-consent-panel h1 { color: var(--text, #202733); }
:global(html[data-theme="light"]) .folder-access-consent-details { border-color: rgb(28 39 54 / 15%); }

@media (max-width: 520px) {
  .folder-access-consent-panel { padding: 20px; }
  .folder-access-consent-actions { flex-direction: column-reverse; }
  .folder-access-consent-actions .btn { width: 100%; }
}
</style>
