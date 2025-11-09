<template>
  <div class="min-h-screen bg-gray-50 p-4">
    <div class="max-w-6xl mx-auto">
      <h1 class="text-3xl font-bold mb-6 text-gray-800">TSI Analytics (Seed)</h1>
      
      <div class="grid grid-cols-4 gap-4">
        <!-- Control Panel -->
        <aside class="col-span-1">
          <ControlPanel @run="onRun" />
        </aside>

        <!-- Main content -->
        <main class="col-span-3 space-y-4">
          <Chart :series="series" />
          <DataTable :rows="tableRows" />
        </main>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue'
import ControlPanel from './components/ControlPanel.vue'
import Chart from './components/Chart.vue'
import DataTable from './components/DataTable.vue'
import axios from 'axios'

export default defineComponent({
  components: { ControlPanel, Chart, DataTable },
  setup() {
    const series = ref<number[]>([])
    const tableRows = ref<Array<{ key: string; value: number }>>([])

    async function onRun(values: number[]) {
      series.value = values
      try {
        const resp = await axios.post('/api/v1/compute', { values })
        const data = resp.data
        tableRows.value = [
          { key: 'mean', value: data.mean },
          { key: 'std', value: data.std }
        ]
      } catch (e) {
        console.error('Compute failed:', e)
      }
    }

    return { series, tableRows, onRun }
  }
})
</script>
