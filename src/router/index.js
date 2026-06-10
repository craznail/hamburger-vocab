import { createRouter, createMemoryHistory } from 'vue-router'

const routes = [
  {
    path: '/',
    name: 'Home',
    component: () => import('../pages/HomePage.vue')
  },
  {
    path: '/study',
    name: 'Study',
    component: () => import('../pages/StudyPage.vue')
  },
  {
    path: '/dictation',
    name: 'Dictation',
    component: () => import('../pages/DictationPage.vue')
  },
  {
    path: '/deck/:id',
    name: 'DeckDetail',
    component: () => import('../pages/DeckDetailPage.vue')
  }
]

const router = createRouter({
  history: createMemoryHistory(),
  routes
})

export default router
