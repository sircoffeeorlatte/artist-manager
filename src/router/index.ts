import { createRouter, createWebHistory } from 'vue-router'
import HomePage from '../pages/HomePage.vue'
import VideoGalleryPage from '../pages/VideoGalleryPage.vue'
import PhotoGalleryPage from '../pages/PhotoGalleryPage.vue'
import ArchiveGalleryPage from '../pages/ArchiveGalleryPage.vue'
import DatabasePage from '../pages/DatabasePage.vue'
import SettingsPage from '../pages/SettingsPage.vue'
import ImportPage from '../pages/ImportPage.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', name: 'home', component: HomePage, meta: { title: 'Главная' } },
    { path: '/gallery/videos', name: 'videos', component: VideoGalleryPage, meta: { title: 'Галерея видео' } },
    { path: '/gallery/photos', name: 'photos', component: PhotoGalleryPage, meta: { title: 'Галерея фото' } },
    { path: '/gallery/archives', name: 'archives', component: ArchiveGalleryPage, meta: { title: 'Галерея архивов' } },
    { path: '/database', name: 'database', component: DatabasePage, meta: { title: 'База данных файлов' } },
    { path: '/settings', name: 'settings', component: SettingsPage, meta: { title: 'Настройки' } },
    { path: '/import', name: 'import', component: ImportPage, meta: { title: 'Импорт файлов' } },
    { path: '/:pathMatch(.*)*', redirect: '/' },
  ],
  scrollBehavior: () => ({ top: 0 }),
})

export default router
