<script setup lang="ts">
import { computed, ref } from 'vue'
import { dismissAlert, getAppState, type AppActivity, type AppAlert } from '../../services/appState'

const appState = getAppState()
const expanded = ref(false)
const activeActivities = computed(() =>
  appState.activities
    .filter((item) => item.status === 'running' && item.kind !== 'video' && item.kind !== 'gallery' && !(item.kind === 'system' && item.title === 'Загрузка настроек'))
    .sort((a, b) => b.updatedAt - a.updatedAt),
)
const currentStatus = computed(() => activeActivities.value[0]?.title ?? 'Нет активных задач')

function alertIcon(alert: AppAlert): string {
  return alert.level === 'error' ? '!' : alert.level === 'success' ? '✓' : alert.level === 'warning' ? '!' : 'i'
}
function progressText(activity: AppActivity): string {
  return typeof activity.progress === 'number' ? `${Math.round(activity.progress)}%` : '…'
}
</script>

<template>
  <div class="app-global-status" aria-label="Состояние приложения">
    <button
      type="button"
      class="app-status-pill"
      :class="{ 'has-running': activeActivities.length }"
      :aria-expanded="expanded"
      @click="expanded = !expanded"
    >
      <span class="app-status-light" :class="{ 'is-running': activeActivities.length }"></span>
      <span class="app-status-pill-copy">
        <strong>{{ currentStatus }}</strong>
        <small>{{ activeActivities.length ? `${activeActivities.length} активн. ${activeActivities.length === 1 ? 'задача' : 'задачи'}` : 'Всё спокойно' }}</small>
      </span>
      <span class="app-status-caret">{{ expanded ? '⌃' : '⌄' }}</span>
    </button>

    <section v-if="expanded" class="app-status-panel">
      <header class="app-status-panel-heading">
        <div><strong>Состояние приложения</strong><span>Активные фоновые операции</span></div>
        <button type="button" class="app-status-close" aria-label="Свернуть состояние приложения" @click="expanded = false">×</button>
      </header>
      <div class="app-status-list">
        <p v-if="!activeActivities.length" class="app-status-empty">Сейчас нет выполняющихся операций. Последние задачи и события доступны на странице импорта.</p>
        <article v-for="activity in activeActivities" :key="activity.id" class="app-status-item is-active">
          <span class="app-status-item-icon">◌</span>
          <div class="app-status-item-copy">
            <strong>{{ activity.title }}</strong>
            <p>{{ activity.details }}</p>
            <div
              v-if="typeof activity.progress === 'number'"
              class="app-status-progress-track"
              role="progressbar"
              :aria-valuenow="activity.progress"
              aria-valuemin="0"
              aria-valuemax="100"
            >
              <span :style="{ width: `${activity.progress}%` }"></span>
            </div>
          </div>
          <span v-if="typeof activity.progress === 'number'" class="app-status-percent">{{ progressText(activity) }}</span>
        </article>
      </div>
    </section>
  </div>

  <Teleport to="body">
    <div class="app-alert-stack" aria-live="polite" aria-relevant="additions">
      <TransitionGroup name="app-alert">
        <article v-for="alert in appState.alerts" :key="alert.id" class="app-alert" :class="`alert-${alert.level}`" :role="alert.level === 'error' ? 'alert' : 'status'">
          <span class="app-alert-icon">{{ alertIcon(alert) }}</span>
          <div class="app-alert-copy"><strong>{{ alert.title }}</strong><p>{{ alert.message }}</p></div>
          <button type="button" class="app-alert-dismiss" :aria-label="`Закрыть уведомление: ${alert.title}`" @click="dismissAlert(alert.id)">×</button>
        </article>
      </TransitionGroup>
    </div>
  </Teleport>
</template>
