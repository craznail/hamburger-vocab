import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'Home',
    component: () => import('../pages/HomePage.vue'),
  },
  {
    path: '/study',
    name: 'Study',
    component: () => import('../pages/StudyPage.vue'),
  },
  {
    path: '/library',
    name: 'Library',
    component: () => import('../pages/LibraryPage.vue'),
  },
  {
    path: '/stats',
    name: 'Stats',
    component: () => import('../pages/StatsPage.vue'),
  },
  {
    path: '/achievements',
    name: 'Achievements',
    component: () => import('../pages/AchievementPage.vue'),
  },
  {
    path: '/import',
    name: 'Import',
    component: () => import('../pages/ImportPage.vue'),
  },
  {
    path: '/settings',
    name: 'Settings',
    component: () => import('../pages/SettingsPage.vue'),
  },
  {
    path: '/sync-server',
    name: 'SyncServer',
    component: () => import('../pages/SyncServerPage.vue'),
  },
  {
    path: '/profile',
    name: 'Profile',
    component: () => import('../pages/ProfilePage.vue'),
  },
  {
    path: '/dictation',
    name: 'Dictation',
    component: () => import('../pages/DictationPage.vue'),
  },
  {
    path: '/errors',
    name: 'ErrorNotebook',
    component: () => import('../pages/ErrorNotebookPage.vue'),
  },
  {
    path: '/errors/add',
    name: 'ErrorAdd',
    component: () => import('../pages/ErrorAddPage.vue'),
  },
  {
    path: '/errors/review',
    name: 'ErrorReview',
    component: () => import('../pages/ErrorReviewPage.vue'),
  },
  {
    path: '/errors/:id',
    name: 'ErrorDetail',
    component: () => import('../pages/ErrorDetailPage.vue'),
  },
  {
    path: '/deck/:id',
    name: 'DeckDetail',
    component: () => import('../pages/DeckDetailPage.vue'),
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
