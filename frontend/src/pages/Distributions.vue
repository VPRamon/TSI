<template>
  <div class="p-6">
    <h1 class="text-3xl font-bold text-gray-900 mb-6">Distributions & Statistics</h1>

    <!-- Loading State -->
    <div v-if="loading" class="text-center py-12">
      <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      <p class="mt-4 text-gray-600">Loading data...</p>
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="bg-red-50 border border-red-200 rounded-lg p-4">
      <p class="text-red-800">{{ error }}</p>
    </div>

    <!-- Content -->
    <div v-else>
      <!-- Controls -->
      <div class="bg-white p-6 rounded-lg shadow mb-6">
        <div class="flex items-center gap-4">
          <label class="text-sm font-medium text-gray-700">Bins:</label>
          <input 
            type="number" 
            v-model.number="numBins" 
            min="5" 
            max="50"
            class="w-24 px-3 py-2 border rounded"
          />
          <button 
            @click="loadDistributions" 
            class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
          >
            Update Charts
          </button>
          <button 
            @click="exportToCSV" 
            class="ml-auto px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700"
          >
            Export Statistics (CSV)
          </button>
        </div>
      </div>

      <!-- Summary Statistics Cards -->
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
        <div 
          v-for="stat in summaryStats" 
          :key="stat.column"
          class="bg-white p-4 rounded-lg shadow"
        >
          <h3 class="text-sm font-semibold text-gray-600 mb-2">{{ stat.label }}</h3>
          <div class="space-y-1 text-sm">
            <div><span class="text-gray-600">Mean:</span> <span class="font-mono">{{ stat.mean?.toFixed(3) }}</span></div>
            <div><span class="text-gray-600">Median:</span> <span class="font-mono">{{ stat.median?.toFixed(3) }}</span></div>
            <div><span class="text-gray-600">Std:</span> <span class="font-mono">{{ stat.std?.toFixed(3) }}</span></div>
            <div><span class="text-gray-600">Min:</span> <span class="font-mono">{{ stat.min?.toFixed(3) }}</span></div>
            <div><span class="text-gray-600">Max:</span> <span class="font-mono">{{ stat.max?.toFixed(3) }}</span></div>
          </div>
        </div>
      </div>

      <!-- Histograms -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div 
          v-for="chart in charts" 
          :key="chart.column"
          class="bg-white p-6 rounded-lg shadow"
        >
          <v-chart 
            :option="chart.option" 
            :autoresize="true"
            style="height: 400px"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted } from 'vue'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { BarChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  GridComponent,
  LegendComponent
} from 'echarts/components'
import VChart from 'vue-echarts'
import axios from 'axios'

use([
  CanvasRenderer,
  BarChart,
  TitleComponent,
  TooltipComponent,
  GridComponent,
  LegendComponent
])

const API_BASE = 'http://localhost:8081/api/v1'

interface DistributionStats {
  column: string
  count: number
  mean: number
  median: number
  std: number
  min: number
  max: number
  q25: number
  q50: number
  q75: number
  p10: number
  p90: number
  p95: number
  p99: number
}

interface HistogramBin {
  bin_start: number
  bin_end: number
  count: number
  frequency: number
}

interface Histogram {
  column: string
  bins: HistogramBin[]
  total_count: number
  min: number
  max: number
}

export default defineComponent({
  components: { VChart },
  setup() {
    const loading = ref(false)
    const error = ref('')
    const numBins = ref(20)
    const summaryStats = ref<Array<{ column: string; label: string } & Partial<DistributionStats>>>([])
    const charts = ref<Array<{ column: string; option: any }>>([])

    const columns = [
      { key: 'priority', label: 'Priority' },
      { key: 'total_visibility_hours', label: 'Visibility Hours' },
      { key: 'requested_hours', label: 'Requested Duration' },
      { key: 'elevation_range_deg', label: 'Elevation Range (deg)' }
    ]

    async function loadDistributions() {
      loading.value = true
      error.value = ''
      
      try {
        // Load statistics for all columns
        const statsPromises = columns.map(async col => {
          const response = await axios.get(`${API_BASE}/analytics/distribution`, {
            params: {
              column: col.key,
              stats: true
            }
          })
          return { ...response.data, label: col.label }
        })

        const stats = await Promise.all(statsPromises)
        summaryStats.value = stats

        // Load histograms for all columns
        const histPromises = columns.map(async col => {
          const response = await axios.get(`${API_BASE}/analytics/distribution`, {
            params: {
              column: col.key,
              bins: numBins.value
            }
          })
          return { column: col.key, data: response.data as Histogram, label: col.label }
        })

        const histograms = await Promise.all(histPromises)
        
        charts.value = histograms.map(h => ({
          column: h.column,
          option: createHistogramOption(h.data, h.label)
        }))

      } catch (e: any) {
        error.value = e.response?.data?.error || e.message || 'Failed to load distributions'
      } finally {
        loading.value = false
      }
    }

    function createHistogramOption(histogram: Histogram, label: string) {
      const binLabels = histogram.bins.map(b => 
        `${b.bin_start.toFixed(2)} - ${b.bin_end.toFixed(2)}`
      )
      const counts = histogram.bins.map(b => b.count)

      return {
        title: {
          text: label,
          left: 'center'
        },
        tooltip: {
          trigger: 'axis',
          axisPointer: {
            type: 'shadow'
          },
          formatter: (params: any) => {
            const p = params[0]
            const bin = histogram.bins[p.dataIndex]
            return `
              <strong>Range:</strong> ${bin.bin_start.toFixed(2)} - ${bin.bin_end.toFixed(2)}<br/>
              <strong>Count:</strong> ${bin.count}<br/>
              <strong>Frequency:</strong> ${(bin.frequency * 100).toFixed(2)}%
            `
          }
        },
        grid: {
          left: '10%',
          right: '10%',
          bottom: '15%',
          top: '15%'
        },
        xAxis: {
          type: 'category',
          data: binLabels,
          axisLabel: {
            rotate: 45,
            fontSize: 10
          }
        },
        yAxis: {
          type: 'value',
          name: 'Count'
        },
        series: [{
          type: 'bar',
          data: counts,
          itemStyle: {
            color: '#3b82f6'
          }
        }]
      }
    }

    function exportToCSV() {
      if (summaryStats.value.length === 0) return

      // Create CSV content
      const headers = ['Column', 'Count', 'Mean', 'Median', 'Std', 'Min', 'Max', 'Q25', 'Q50', 'Q75', 'P10', 'P90', 'P95', 'P99']
      const rows = summaryStats.value.map(stat => [
        stat.label,
        stat.count,
        stat.mean?.toFixed(6),
        stat.median?.toFixed(6),
        stat.std?.toFixed(6),
        stat.min?.toFixed(6),
        stat.max?.toFixed(6),
        stat.q25?.toFixed(6),
        stat.q50?.toFixed(6),
        stat.q75?.toFixed(6),
        stat.p10?.toFixed(6),
        stat.p90?.toFixed(6),
        stat.p95?.toFixed(6),
        stat.p99?.toFixed(6)
      ])

      const csv = [
        headers.join(','),
        ...rows.map(row => row.join(','))
      ].join('\n')

      // Download
      const blob = new Blob([csv], { type: 'text/csv' })
      const url = window.URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = 'distribution_statistics.csv'
      document.body.appendChild(a)
      a.click()
      document.body.removeChild(a)
      window.URL.revokeObjectURL(url)
    }

    onMounted(() => {
      loadDistributions()
    })

    return {
      loading,
      error,
      numBins,
      summaryStats,
      charts,
      loadDistributions,
      exportToCSV
    }
  }
})
</script>
