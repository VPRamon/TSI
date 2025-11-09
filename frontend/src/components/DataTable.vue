<template>
  <div class="p-4 bg-white rounded-lg shadow">
    <h4 class="text-lg font-medium mb-3 text-gray-700">Computed Results</h4>
    
    <div v-if="rows.length === 0" class="text-gray-500 italic">
      No results yet
    </div>
    <table v-else class="w-full">
      <thead>
        <tr class="border-b border-gray-200">
          <th class="text-left py-2 px-3 text-sm font-semibold text-gray-600">Metric</th>
          <th class="text-left py-2 px-3 text-sm font-semibold text-gray-600">Value</th>
        </tr>
      </thead>
      <tbody>
        <tr 
          v-for="row in rows" 
          :key="row.key"
          class="border-b border-gray-100 hover:bg-gray-50"
        >
          <td class="py-2 px-3 text-sm text-gray-700">{{ row.key }}</td>
          <td class="py-2 px-3 text-sm font-mono text-gray-900">{{ row.value.toFixed(6) }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from 'vue'

interface TableRow {
  key: string
  value: number
}

export default defineComponent({
  props: {
    rows: {
      type: Array as PropType<TableRow[]>,
      default: () => []
    }
  }
})
</script>
