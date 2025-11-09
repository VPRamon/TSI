import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'Landing',
    component: () => import('./pages/LandingPage.vue')
  },
  {
    path: '/sky-map',
    name: 'SkyMap',
    component: () => import('./pages/SkyMapPlaceholder.vue')
  },
  {
    path: '/distributions',
    name: 'Distributions',
    component: () => import('./pages/DistributionsPlaceholder.vue')
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
