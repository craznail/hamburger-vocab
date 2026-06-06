import { createRouter, createWebHashHistory } from 'vue-router'

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
    path: '/deck/:id',
    name: 'DeckDetail',
    component: () => import('../pages/DeckDetailPage.vue')
  }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})

export default router
