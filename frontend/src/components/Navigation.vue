<template>
  <nav class="bg-white shadow-sm border-b">
    <div class="max-w-7xl mx-auto px-4">
      <div class="flex justify-between items-center h-16">
        <div class="flex items-center space-x-8">
          <h1 class="text-xl font-bold text-gray-900">TSI</h1>
          <span class="text-sm text-gray-500">{{ datasetTitle }}</span>
        </div>
        
        <div class="flex space-x-4">
          <router-link 
            v-for="page in pages" 
            :key="page.path" 
            :to="page.path"
            class="px-3 py-2 rounded-md text-sm font-medium transition-colors"
            :class="isActive(page.path) 
              ? 'bg-blue-100 text-blue-700' 
              : 'text-gray-700 hover:bg-gray-100'"
          >
            {{ page.name }}
          </router-link>
        </div>
      </div>
    </div>
  </nav>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue'
import { useRoute } from 'vue-router'

export default defineComponent({
  props: {
    datasetTitle: {
      type: String,
      required: true
    }
  },
  setup() {
    const route = useRoute()
    
    const pages = [
      { name: 'Upload', path: '/' },
      { name: 'Sky Map', path: '/sky-map' },
      { name: 'Distributions', path: '/distributions' }
    ]

    function isActive(path: string) {
      return route.path === path
    }

    return { pages, isActive }
  }
})
</script>
