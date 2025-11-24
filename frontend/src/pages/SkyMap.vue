<template>
  <div class="p-6">
    <h1 class="text-3xl font-bold text-gray-900 mb-6">Sky Map - RA/Dec Distribution</h1>

    <!-- Filter Controls -->
    <div class="bg-white p-6 rounded-lg shadow mb-6">
      <h3 class="text-lg font-semibold mb-4">Filters</h3>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
        <!-- Priority Range -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">
            Priority Range: {{ filters.priorityMin }} - {{ filters.priorityMax }}
          </label>
          <div class="flex gap-2">
            <input 
              type="number" 
              v-model.number="filters.priorityMin" 
              class="w-full px-3 py-2 border rounded"
              placeholder="Min"
            />
            <input 
              type="number" 
              v-model.number="filters.priorityMax" 
              class="w-full px-3 py-2 border rounded"
              placeholder="Max"
            />
          </div>
        </div>

        <!-- Scheduled Status -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">Scheduled Status</label>
          <select v-model="filters.scheduledStatus" class="w-full px-3 py-2 border rounded">
            <option value="all">All</option>
            <option value="scheduled">Scheduled Only</option>
            <option value="unscheduled">Unscheduled Only</option>
          </select>
        </div>

        <!-- Color By -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">Color By</label>
          <select v-model="filters.colorBy" class="w-full px-3 py-2 border rounded">
            <option value="priority_bin">Priority Bin</option>
            <option value="scheduled">Scheduled Status</option>
            <option value="priority">Priority (continuous)</option>
          </select>
        </div>
      </div>

      <div class="mt-4 flex gap-3">
        <button 
          @click="applyFilters" 
          class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
          :disabled="loading"
        >
          Apply Filters
        </button>
        <button 
          @click="resetFilters" 
          class="px-4 py-2 bg-gray-200 text-gray-700 rounded hover:bg-gray-300"
        >
          Reset
        </button>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="loading" class="text-center py-12">
      <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      <p class="mt-4 text-gray-600">Loading data...</p>
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="bg-red-50 border border-red-200 rounded-lg p-4">
      <p class="text-red-800">{{ error }}</p>
    </div>

    <!-- Chart -->
    <div v-else class="bg-white p-6 rounded-lg shadow">
      <v-chart 
        ref="chart"
        :option="chartOption" 
        :autoresize="true"
        style="height: 600px"
      />
      <div class="mt-4 text-sm text-gray-600">
        Showing {{ filteredCount }} of {{ totalCount }} observations
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted, computed } from 'vue'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { ScatterChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  GridComponent,
  LegendComponent,
  VisualMapComponent
} from 'echarts/components'
import VChart from 'vue-echarts'
import axios from 'axios'

use([
  CanvasRenderer,
  ScatterChart,
  TitleComponent,
  TooltipComponent,
  GridComponent,
  LegendComponent,
  VisualMapComponent
])

const API_BASE = 'http://localhost:8081/api/v1'

interface SchedulingBlock {
  schedulingBlockId: string
  raInDeg: number
  decInDeg: number
  priority: number
  priorityBin: string
  scheduledFlag: boolean
  requestedHours: number
  totalVisibilityHours: number
  elevationRangeDeg?: number
  targetName?: string
  targetId?: number
}

export default defineComponent({
  components: { VChart },
  setup() {
    const loading = ref(false)
    const error = ref('')
    const allData = ref<SchedulingBlock[]>([])
    const filteredData = ref<SchedulingBlock[]>([])
    const totalCount = ref(0)
    const filteredCount = ref(0)

    const filters = ref({
      priorityMin: 0,
      priorityMax: 100,
      scheduledStatus: 'all' as 'all' | 'scheduled' | 'unscheduled',
      colorBy: 'priority_bin' as 'priority_bin' | 'scheduled' | 'priority'
    })

    const priorityBinColors: Record<string, string> = {
      'Low (<10)': '#3b82f6',     // blue
      'High (10+)': '#ef4444',    // red
      'Unknown': '#9ca3af'        // gray
    }

    const statusColors: Record<string, string> = {
      'Scheduled': '#10b981',     // green
      'Unscheduled': '#f59e0b'    // orange
    }

    async function loadData() {
      loading.value = true
      error.value = ''
      try {
        const response = await axios.get(`${API_BASE}/datasets/current`)
        allData.value = response.data.blocks
        totalCount.value = allData.value.length
        applyFilters()
      } catch (e: any) {
        error.value = e.response?.data?.error || e.message || 'Failed to load data'
      } finally {
        loading.value = false
      }
    }

    function applyFilters() {
      filteredData.value = allData.value.filter(block => {
        // Priority filter
        if (block.priority < filters.value.priorityMin || block.priority > filters.value.priorityMax) {
          return false
        }
        
        // Scheduled status filter
        if (filters.value.scheduledStatus === 'scheduled' && !block.scheduledFlag) {
          return false
        }
        if (filters.value.scheduledStatus === 'unscheduled' && block.scheduledFlag) {
          return false
        }
        
        return true
      })
      filteredCount.value = filteredData.value.length
    }

    function resetFilters() {
      filters.value = {
        priorityMin: 0,
        priorityMax: 100,
        scheduledStatus: 'all',
        colorBy: 'priority_bin'
      }
      applyFilters()
    }

    const chartOption = computed(() => {
      if (filteredData.value.length === 0) {
        return {}
      }

      // Prepare data based on color mode
      let seriesData: any[] = []
      let legendData: string[] = []
      let series: any[] = []

      if (filters.value.colorBy === 'priority_bin') {
        // Group by priority bin
        const groups = new Map<string, SchedulingBlock[]>()
        filteredData.value.forEach(block => {
          const bin = block.priorityBin || 'Unknown'
          if (!groups.has(bin)) {
            groups.set(bin, [])
          }
          groups.get(bin)!.push(block)
        })

        groups.forEach((blocks, bin) => {
          legendData.push(bin)
          series.push({
            name: bin,
            type: 'scatter',
            data: blocks.map(b => ({
              value: [b.raInDeg, b.decInDeg],
              symbolSize: Math.max(5, Math.min(20, b.requestedHours * 3)),
              itemStyle: {
                color: priorityBinColors[bin] || '#9ca3af'
              },
              block: b
            })),
            emphasis: {
              focus: 'series'
            }
          })
        })
      } else if (filters.value.colorBy === 'scheduled') {
        // Group by scheduled status
        const scheduled = filteredData.value.filter(b => b.scheduledFlag)
        const unscheduled = filteredData.value.filter(b => !b.scheduledFlag)

        if (scheduled.length > 0) {
          legendData.push('Scheduled')
          series.push({
            name: 'Scheduled',
            type: 'scatter',
            data: scheduled.map(b => ({
              value: [b.raInDeg, b.decInDeg],
              symbolSize: Math.max(5, Math.min(20, b.requestedHours * 3)),
              itemStyle: {
                color: statusColors['Scheduled']
              },
              block: b
            }))
          })
        }

        if (unscheduled.length > 0) {
          legendData.push('Unscheduled')
          series.push({
            name: 'Unscheduled',
            type: 'scatter',
            data: unscheduled.map(b => ({
              value: [b.raInDeg, b.decInDeg],
              symbolSize: Math.max(5, Math.min(20, b.requestedHours * 3)),
              itemStyle: {
                color: statusColors['Unscheduled']
              },
              block: b
            }))
          })
        }
      } else {
        // Continuous priority coloring
        series.push({
          name: 'Observations',
          type: 'scatter',
          data: filteredData.value.map(b => ({
            value: [b.raInDeg, b.decInDeg, b.priority],
            symbolSize: Math.max(5, Math.min(20, b.requestedHours * 3)),
            block: b
          }))
        })
      }

      return {
        title: {
          text: 'Sky Map - RA vs Declination',
          left: 'center'
        },
        tooltip: {
          trigger: 'item',
          formatter: (params: any) => {
            const block = params.data.block
            if (!block) return ''
            
            let tooltip = ''
            
            // Add target information if available
            if (block.targetName || block.targetId) {
              tooltip += '<div style="font-weight: bold; margin-bottom: 4px; color: #2563eb;">'
              if (block.targetName) {
                tooltip += `🎯 ${block.targetName}`
              }
              if (block.targetId) {
                tooltip += ` (ID: ${block.targetId})`
              }
              tooltip += '</div>'
            }
            
            tooltip += `
              <strong>Block ID:</strong> ${block.schedulingBlockId}<br/>
              <strong>RA:</strong> ${block.raInDeg.toFixed(2)}°<br/>
              <strong>Dec:</strong> ${block.decInDeg.toFixed(2)}°<br/>
              <strong>Priority:</strong> ${block.priority.toFixed(2)}<br/>
              <strong>Priority Bin:</strong> ${block.priorityBin}<br/>
              <strong>Status:</strong> ${block.scheduledFlag ? 'Scheduled' : 'Unscheduled'}<br/>
              <strong>Requested:</strong> ${block.requestedHours.toFixed(2)}h<br/>
              <strong>Visibility:</strong> ${block.totalVisibilityHours.toFixed(2)}h
            `
            
            return tooltip
          }
        },
        legend: {
          data: legendData,
          bottom: 10,
          show: filters.value.colorBy !== 'priority'
        },
        grid: {
          left: '10%',
          right: filters.value.colorBy === 'priority' ? '15%' : '10%',
          bottom: '15%',
          top: '10%'
        },
        xAxis: {
          name: 'Right Ascension (deg)',
          nameLocation: 'middle',
          nameGap: 30,
          min: 0,
          max: 360
        },
        yAxis: {
          name: 'Declination (deg)',
          nameLocation: 'middle',
          nameGap: 40,
          min: -90,
          max: 90
        },
        visualMap: filters.value.colorBy === 'priority' ? {
          min: Math.min(...filteredData.value.map(b => b.priority)),
          max: Math.max(...filteredData.value.map(b => b.priority)),
          dimension: 2,
          orient: 'vertical',
          right: 10,
          top: 'center',
          text: ['High', 'Low'],
          calculable: true,
          inRange: {
            color: ['#3b82f6', '#ef4444']
          }
        } : undefined,
        series
      }
    })

    onMounted(() => {
      loadData()
    })

    return {
      loading,
      error,
      filters,
      totalCount,
      filteredCount,
      chartOption,
      applyFilters,
      resetFilters
    }
  }
})
</script>
