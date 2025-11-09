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
    component: () => import('./pages/SkyMap.vue')
  },
  {
    path: '/distributions',
    name: 'Distributions',
    component: () => import('./pages/Distributions.vue')
  },
  {
    path: '/insights',
    name: 'Insights',
    component: () => import('./pages/Insights.vue')
  },
  {
    path: '/visibility-map',
    name: 'VisibilityMap',
    component: () => import('./pages/VisibilityMap.vue')
  },
  {
    path: '/timeline',
    name: 'Timeline',
    component: () => import('./pages/ScheduledTimeline.vue')
  },
  {
    path: '/trends',
    name: 'Trends',
    component: () => import('./pages/Trends.vue')
  },
  {
    path: '/compare',
    name: 'Compare',
    component: () => import('./pages/Placeholder.vue') // Placeholder for Phase 5
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
