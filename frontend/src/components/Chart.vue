<template>
  <div class="p-4 bg-white rounded-lg shadow">
    <h4 class="text-lg font-medium mb-3 text-gray-700">Input Series</h4>
    
    <div v-if="series.length === 0" class="text-gray-500 italic">
      No data to display
    </div>
    <div v-else class="space-y-2">
      <div class="text-sm text-gray-600">
        Count: {{ series.length }}
      </div>
      <div class="text-sm font-mono bg-gray-50 p-2 rounded">
        {{ series.join(', ') }}
      </div>
    </div>
    
    <!-- Placeholder for ECharts integration -->
    <div class="mt-4 h-48 border border-dashed border-gray-300 rounded flex items-center justify-center text-gray-400">
      Chart visualization (ECharts placeholder)
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue'

export default defineComponent({
  props: {
    series: {
      type: Array as PropType<number[]>,
      default: () => []
    }
  }
})
</script>
