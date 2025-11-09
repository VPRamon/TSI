<template>
  <div class="p-6">
    <h1 class="text-3xl font-bold text-gray-900 mb-6">Insights & Analytics Dashboard</h1>

    <!-- Loading State -->
    <div v-if="loading" class="text-center py-12">
      <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      <p class="mt-4 text-gray-600">Loading analytics...</p>
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="bg-red-50 border border-red-200 rounded-lg p-4">
      <p class="text-red-800">{{ error }}</p>
    </div>

    <!-- Content -->
    <div v-else>
      <!-- Key Metrics Cards -->
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
        <div class="bg-white p-6 rounded-lg shadow">
          <h3 class="text-sm font-semibold text-gray-600 mb-2">Total Blocks</h3>
          <p class="text-3xl font-bold text-gray-900">{{ metrics?.total_blocks || 0 }}</p>
        </div>
        <div class="bg-white p-6 rounded-lg shadow">
          <h3 class="text-sm font-semibold text-gray-600 mb-2">Scheduling Rate</h3>
          <p class="text-3xl font-bold text-green-600">
            {{ ((metrics?.scheduling_rate || 0) * 100).toFixed(1) }}%
          </p>
          <p class="text-xs text-gray-500 mt-1">
            {{ metrics?.scheduled_blocks || 0 }} scheduled / {{ metrics?.unscheduled_blocks || 0 }} unscheduled
          </p>
        </div>
        <div class="bg-white p-6 rounded-lg shadow">
          <h3 class="text-sm font-semibold text-gray-600 mb-2">Utilization Rate</h3>
          <p class="text-3xl font-bold text-blue-600">
            {{ ((metrics?.utilization_rate || 0) * 100).toFixed(2) }}%
          </p>
          <p class="text-xs text-gray-500 mt-1">
            {{ metrics?.total_scheduled_hours?.toFixed(1) }}h scheduled / 
            {{ metrics?.total_visibility_hours?.toFixed(0) }}h available
          </p>
        </div>
        <div class="bg-white p-6 rounded-lg shadow">
          <h3 class="text-sm font-semibold text-gray-600 mb-2">Avg Priority</h3>
          <p class="text-3xl font-bold text-purple-600">
            {{ metrics?.priority_stats?.mean?.toFixed(2) || 0 }}
          </p>
          <p class="text-xs text-gray-500 mt-1">
            Median: {{ metrics?.priority_stats?.median?.toFixed(2) || 0 }}
          </p>
        </div>
      </div>

      <!-- Priority Bin Distribution -->
      <div class="bg-white p-6 rounded-lg shadow mb-6">
        <h3 class="text-lg font-semibold mb-4">Priority Bins</h3>
        <div class="grid grid-cols-3 gap-4">
          <div 
            v-for="(count, bin) in metrics?.priority_bin_counts" 
            :key="bin"
            class="text-center p-4 bg-gray-50 rounded"
          >
            <p class="text-sm text-gray-600">{{ bin }}</p>
            <p class="text-2xl font-bold text-gray-900">{{ count }}</p>
          </div>
        </div>
      </div>

      <!-- Correlation Heatmap -->
      <div class="bg-white p-6 rounded-lg shadow mb-6">
        <h3 class="text-lg font-semibold mb-4">Correlation Matrix</h3>
        <v-chart 
          :option="correlationOption" 
          :autoresize="true"
          style="height: 400px"
        />
        <div class="mt-4 space-y-2">
          <h4 class="font-semibold text-sm text-gray-700">Key Insights:</h4>
          <div 
            v-for="corr in correlationInsights" 
            :key="`${corr.col1}-${corr.col2}`"
            class="text-sm text-gray-600 pl-4"
          >
            • {{ corr.insight }}
          </div>
        </div>
      </div>

      <!-- Conflicts -->
      <div class="bg-white p-6 rounded-lg shadow mb-6">
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-lg font-semibold">Conflicts Detected</h3>
          <span class="px-3 py-1 bg-red-100 text-red-800 rounded-full text-sm font-semibold">
            {{ conflicts?.total_conflicts || 0 }} Total
          </span>
        </div>
        <div class="grid grid-cols-3 gap-4 mb-4">
          <div class="text-center p-3 bg-red-50 rounded border border-red-200">
            <p class="text-xs text-red-600 font-semibold">Impossible</p>
            <p class="text-2xl font-bold text-red-700">{{ conflicts?.impossible_observations || 0 }}</p>
          </div>
          <div class="text-center p-3 bg-yellow-50 rounded border border-yellow-200">
            <p class="text-xs text-yellow-600 font-semibold">Insufficient Visibility</p>
            <p class="text-2xl font-bold text-yellow-700">{{ conflicts?.insufficient_visibility || 0 }}</p>
          </div>
          <div class="text-center p-3 bg-orange-50 rounded border border-orange-200">
            <p class="text-xs text-orange-600 font-semibold">Anomalies</p>
            <p class="text-2xl font-bold text-orange-700">{{ conflicts?.scheduling_anomalies || 0 }}</p>
          </div>
        </div>
        
        <!-- Conflicts Table -->
        <div class="overflow-x-auto max-h-96 overflow-y-auto">
          <table class="w-full text-sm">
            <thead class="bg-gray-50 sticky top-0">
              <tr>
                <th class="text-left py-2 px-3 font-semibold text-gray-600">Block ID</th>
                <th class="text-left py-2 px-3 font-semibold text-gray-600">Type</th>
                <th class="text-left py-2 px-3 font-semibold text-gray-600">Description</th>
                <th class="text-left py-2 px-3 font-semibold text-gray-600">Severity</th>
              </tr>
            </thead>
            <tbody>
              <tr 
                v-for="conflict in conflicts?.conflicts.slice(0, 50)" 
                :key="conflict.scheduling_block_id"
                class="border-b border-gray-100 hover:bg-gray-50"
              >
                <td class="py-2 px-3 font-mono text-xs">{{ conflict.scheduling_block_id }}</td>
                <td class="py-2 px-3">
                  <span class="px-2 py-1 rounded text-xs font-semibold"
                    :class="{
                      'bg-red-100 text-red-800': conflict.conflict_type === 'impossible_observation',
                      'bg-yellow-100 text-yellow-800': conflict.conflict_type === 'insufficient_visibility',
                      'bg-orange-100 text-orange-800': conflict.conflict_type === 'scheduling_anomaly'
                    }"
                  >
                    {{ formatConflictType(conflict.conflict_type) }}
                  </span>
                </td>
                <td class="py-2 px-3 text-gray-700">{{ conflict.description }}</td>
                <td class="py-2 px-3">
                  <span class="px-2 py-1 rounded text-xs font-semibold"
                    :class="{
                      'bg-red-100 text-red-800': conflict.severity === 'high',
                      'bg-yellow-100 text-yellow-800': conflict.severity === 'medium',
                      'bg-blue-100 text-blue-800': conflict.severity === 'low'
                    }"
                  >
                    {{ conflict.severity.toUpperCase() }}
                  </span>
                </td>
              </tr>
            </tbody>
          </table>
          <p v-if="(conflicts?.conflicts.length || 0) > 50" class="text-xs text-gray-500 mt-2 text-center">
            Showing first 50 of {{ conflicts?.total_conflicts }} conflicts
          </p>
        </div>
      </div>

      <!-- Top Observations -->
      <div class="bg-white p-6 rounded-lg shadow mb-6">
        <div class="flex justify-between items-center mb-4">
          <h3 class="text-lg font-semibold">Top Observations by Priority</h3>
          <div class="flex gap-2">
            <select v-model="topSortBy" @change="loadTopObservations" class="px-3 py-1 border rounded text-sm">
              <option value="priority">Priority</option>
              <option value="requested_hours">Requested Hours</option>
              <option value="visibility_hours">Visibility Hours</option>
            </select>
            <input 
              type="number" 
              v-model.number="topN" 
              @change="loadTopObservations"
              min="5" 
              max="50"
              class="w-20 px-3 py-1 border rounded text-sm"
            />
          </div>
        </div>
        <div class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead class="bg-gray-50">
              <tr>
                <th class="text-left py-2 px-3 font-semibold text-gray-600">Rank</th>
                <th class="text-left py-2 px-3 font-semibold text-gray-600">Block ID</th>
                <th class="text-left py-2 px-3 font-semibold text-gray-600">Priority</th>
                <th class="text-left py-2 px-3 font-semibold text-gray-600">Priority Bin</th>
                <th class="text-left py-2 px-3 font-semibold text-gray-600">Requested (h)</th>
                <th class="text-left py-2 px-3 font-semibold text-gray-600">Visibility (h)</th>
                <th class="text-left py-2 px-3 font-semibold text-gray-600">Status</th>
              </tr>
            </thead>
            <tbody>
              <tr 
                v-for="(obs, idx) in topObservations" 
                :key="obs.scheduling_block_id"
                class="border-b border-gray-100 hover:bg-gray-50"
              >
                <td class="py-2 px-3 font-semibold text-gray-700">{{ idx + 1 }}</td>
                <td class="py-2 px-3 font-mono text-xs">{{ obs.scheduling_block_id }}</td>
                <td class="py-2 px-3 font-semibold">{{ obs.priority.toFixed(2) }}</td>
                <td class="py-2 px-3">
                  <span class="px-2 py-1 rounded text-xs font-semibold"
                    :class="{
                      'bg-red-100 text-red-800': obs.priority_bin === 'High (10+)',
                      'bg-blue-100 text-blue-800': obs.priority_bin === 'Low (<10)'
                    }"
                  >
                    {{ obs.priority_bin }}
                  </span>
                </td>
                <td class="py-2 px-3">{{ obs.requested_hours.toFixed(3) }}</td>
                <td class="py-2 px-3">{{ obs.total_visibility_hours.toFixed(2) }}</td>
                <td class="py-2 px-3">
                  <span class="px-2 py-1 rounded text-xs font-semibold"
                    :class="{
                      'bg-green-100 text-green-800': obs.scheduled_flag,
                      'bg-gray-100 text-gray-800': !obs.scheduled_flag
                    }"
                  >
                    {{ obs.scheduled_flag ? 'Scheduled' : 'Unscheduled' }}
                  </span>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Export Reports -->
      <div class="bg-white p-6 rounded-lg shadow">
        <h3 class="text-lg font-semibold mb-4">Export Reports</h3>
        <div class="flex gap-3">
          <button 
            @click="exportReport('json')"
            class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
          >
            Download Metrics (JSON)
          </button>
          <button 
            @click="exportReport('csv')"
            class="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700"
          >
            Download Conflicts (CSV)
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted, computed } from 'vue'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { HeatmapChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  GridComponent,
  VisualMapComponent
} from 'echarts/components'
import VChart from 'vue-echarts'
import axios from 'axios'

use([
  CanvasRenderer,
  HeatmapChart,
  TitleComponent,
  TooltipComponent,
  GridComponent,
  VisualMapComponent
])

const API_BASE = 'http://localhost:8081/api/v1'

interface Metrics {
  total_blocks: number
  scheduled_blocks: number
  unscheduled_blocks: number
  scheduling_rate: number
  total_requested_hours: number
  total_scheduled_hours: number
  total_visibility_hours: number
  utilization_rate: number
  priority_stats: any
  priority_bin_counts: Record<string, number>
}

interface Conflict {
  scheduling_block_id: string
  conflict_type: string
  description: string
  severity: string
}

interface ConflictReport {
  total_conflicts: number
  impossible_observations: number
  insufficient_visibility: number
  scheduling_anomalies: number
  conflicts: Conflict[]
}

interface CorrelationData {
  columns: string[]
  matrix: number[][]
  correlations: Array<{
    col1: string
    col2: string
    correlation: number
    insight: string
  }>
}

interface TopObservation {
  scheduling_block_id: string
  priority: number
  priority_bin: string
  requested_hours: number
  total_visibility_hours: number
  scheduled_flag: boolean
}

export default defineComponent({
  components: { VChart },
  setup() {
    const loading = ref(false)
    const error = ref('')
    const metrics = ref<Metrics | null>(null)
    const conflicts = ref<ConflictReport | null>(null)
    const correlationData = ref<CorrelationData | null>(null)
    const topObservations = ref<TopObservation[]>([])
    const topSortBy = ref('priority')
    const topN = ref(10)

    async function loadData() {
      loading.value = true
      error.value = ''
      
      try {
        const [metricsResp, conflictsResp, corrResp] = await Promise.all([
          axios.get(`${API_BASE}/analytics/metrics`),
          axios.get(`${API_BASE}/analytics/conflicts`),
          axios.get(`${API_BASE}/analytics/correlations`, {
            params: {
              columns: 'priority,total_visibility_hours,requested_hours,elevation_range_deg'
            }
          })
        ])

        metrics.value = metricsResp.data
        conflicts.value = conflictsResp.data
        correlationData.value = corrResp.data

        await loadTopObservations()
      } catch (e: any) {
        error.value = e.response?.data?.error || e.message || 'Failed to load analytics'
      } finally {
        loading.value = false
      }
    }

    async function loadTopObservations() {
      try {
        const resp = await axios.get(`${API_BASE}/analytics/top`, {
          params: {
            by: topSortBy.value,
            order: 'descending',
            n: topN.value
          }
        })
        topObservations.value = resp.data
      } catch (e: any) {
        console.error('Failed to load top observations:', e)
      }
    }

    const correlationInsights = computed(() => {
      return correlationData.value?.correlations || []
    })

    const correlationOption = computed(() => {
      if (!correlationData.value) return {}

      const cols = correlationData.value.columns.map(c => c.replace(/_/g, ' '))
      const data: any[] = []
      
      correlationData.value.matrix.forEach((row, i) => {
        row.forEach((value, j) => {
          data.push([j, i, value.toFixed(3)])
        })
      })

      return {
        title: {
          text: 'Spearman Correlation',
          left: 'center'
        },
        tooltip: {
          position: 'top',
          formatter: (params: any) => {
            return `${cols[params.data[0]]} vs ${cols[params.data[1]]}<br/>Correlation: ${params.data[2]}`
          }
        },
        grid: {
          height: '70%',
          top: '15%',
          left: '20%'
        },
        xAxis: {
          type: 'category',
          data: cols,
          splitArea: {
            show: true
          },
          axisLabel: {
            rotate: 45,
            fontSize: 10
          }
        },
        yAxis: {
          type: 'category',
          data: cols,
          splitArea: {
            show: true
          },
          axisLabel: {
            fontSize: 10
          }
        },
        visualMap: {
          min: -1,
          max: 1,
          calculable: true,
          orient: 'horizontal',
          left: 'center',
          bottom: '5%',
          inRange: {
            color: ['#313695', '#4575b4', '#74add1', '#abd9e9', '#e0f3f8', '#ffffbf', '#fee090', '#fdae61', '#f46d43', '#d73027', '#a50026']
          }
        },
        series: [{
          name: 'Correlation',
          type: 'heatmap',
          data: data,
          label: {
            show: true,
            fontSize: 11
          },
          emphasis: {
            itemStyle: {
              shadowBlur: 10,
              shadowColor: 'rgba(0, 0, 0, 0.5)'
            }
          }
        }]
      }
    })

    function formatConflictType(type: string): string {
      return type.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase())
    }

    function exportReport(format: string) {
      if (format === 'json' && metrics.value) {
        const blob = new Blob([JSON.stringify(metrics.value, null, 2)], { type: 'application/json' })
        downloadBlob(blob, 'metrics.json')
      } else if (format === 'csv' && conflicts.value) {
        const headers = ['Block ID', 'Type', 'Description', 'Severity']
        const rows = conflicts.value.conflicts.map(c => [
          c.scheduling_block_id,
          formatConflictType(c.conflict_type),
          c.description,
          c.severity
        ])
        const csv = [headers.join(','), ...rows.map(row => row.join(','))].join('\n')
        const blob = new Blob([csv], { type: 'text/csv' })
        downloadBlob(blob, 'conflicts.csv')
      }
    }

    function downloadBlob(blob: Blob, filename: string) {
      const url = window.URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = filename
      document.body.appendChild(a)
      a.click()
      document.body.removeChild(a)
      window.URL.revokeObjectURL(url)
    }

    onMounted(() => {
      loadData()
    })

    return {
      loading,
      error,
      metrics,
      conflicts,
      correlationData,
      correlationInsights,
      correlationOption,
      topObservations,
      topSortBy,
      topN,
      formatConflictType,
      loadTopObservations,
      exportReport
    }
  }
})
</script>
