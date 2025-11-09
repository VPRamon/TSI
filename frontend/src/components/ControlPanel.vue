<template>
  <div class="p-4 bg-white rounded-lg shadow">
    <h3 class="text-lg font-semibold mb-3 text-gray-700">Control Panel</h3>
    
    <label class="block text-sm font-medium text-gray-600 mb-2">
      Values (comma-separated)
    </label>
    <input 
      v-model="text" 
      class="w-full p-2 border border-gray-300 rounded focus:ring-2 focus:ring-blue-500 focus:border-transparent" 
      placeholder="1,2,3,4"
    />
    
    <button 
      @click="run" 
      class="mt-3 w-full px-4 py-2 bg-blue-600 text-white font-medium rounded hover:bg-blue-700 transition"
    >
      Run Analysis
    </button>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue'

export default defineComponent({
  emits: ['run'],
  setup(_, { emit }) {
    const text = ref('1,2,3,4')
    
    function run() {
      const values = text.value
        .split(',')
        .map(s => parseFloat(s.trim()))
        .filter(n => !Number.isNaN(n))
      
      emit('run', values)
    }
    
    return { text, run }
  }
})
</script>
