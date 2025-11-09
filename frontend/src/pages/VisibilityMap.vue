<template>
  <div class="p-6">
    <h1 class="text-3xl font-bold text-gray-900 mb-6">Visibility Map</h1>

    <!-- Block Selector -->
    <div class="mb-6 bg-white p-4 rounded-lg shadow">
      <label class="block text-sm font-medium text-gray-700 mb-2">
        Select Scheduling Block
      </label>
      <div class="flex gap-4">
        <select
          v-model="selectedBlockId"
          @change="loadVisibilityData"
          class="flex-1 border border-gray-300 rounded-md px-3 py-2 focus:ring-blue-500 focus:border-blue-500"
        >
          <option value="">-- Choose a block --</option>
          <option v-for="block in allBlocks" :key="block.schedulingBlockId" :value="block.schedulingBlockId">
            {{ block.schedulingBlockId }} - Priority: {{ block.priority.toFixed(2) }}
          </option>
        </select>
        <button
          @click="loadVisibilityData"
          :disabled="!selectedBlockId || loading"
          class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed"
        >
          Load
        </button>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="loading" class="text-center py-12">
      <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      <p class="mt-4 text-gray-600">Loading visibility data...</p>
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="bg-red-50 border border-red-200 rounded-lg p-4">
      <p class="text-red-800">{{ error }}</p>
    </div>

    <!-- Visibility Data Display -->
    <div v-else-if="visibilityData" class="space-y-6">
      <!-- Block Info Cards -->
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div class="bg-white p-4 rounded-lg shadow">
          <h3 class="text-sm font-medium text-gray-500">Position</h3>
          <p class="mt-2 text-lg font-semibold">
            RA: {{ visibilityData.right_ascension_deg.toFixed(2) }}°<br />
            Dec: {{ visibilityData.declination_deg.toFixed(2) }}°
          </p>
        </div>

        <div class="bg-white p-4 rounded-lg shadow">
          <h3 class="text-sm font-medium text-gray-500">Time Requirements</h3>
          <p class="mt-2 text-lg font-semibold">
            Requested: {{ visibilityData.requested_hours.toFixed(2) }}h<br />
            Available: {{ visibilityData.total_visibility_hours.toFixed(2) }}h
          </p>
        </div>

        <div class="bg-white p-4 rounded-lg shadow">
          <h3 class="text-sm font-medium text-gray-500">Priority & Status</h3>
          <p class="mt-2 text-lg font-semibold">
            Priority: {{ visibilityData.priority.toFixed(2) }}<br />
            <span :class="visibilityData.scheduled_flag ? 'text-green-600' : 'text-amber-600'">
              {{ visibilityData.scheduled_flag ? 'Scheduled' : 'Unscheduled' }}
            </span>
          </p>
        </div>
      </div>

      <!-- Constraints -->
      <div class="bg-white p-4 rounded-lg shadow">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">Observation Constraints</h3>
        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <h4 class="text-sm font-medium text-gray-700">Azimuth Range</h4>
            <p class="mt-1 text-gray-900">
              {{ visibilityData.azimuth_min_deg?.toFixed(1) || 'N/A' }}° to 
              {{ visibilityData.azimuth_max_deg?.toFixed(1) || 'N/A' }}°
            </p>
          </div>
          <div>
            <h4 class="text-sm font-medium text-gray-700">Elevation Range</h4>
            <p class="mt-1 text-gray-900">
              {{ visibilityData.elevation_min_deg?.toFixed(1) || 'N/A' }}° to 
              {{ visibilityData.elevation_max_deg?.toFixed(1) || 'N/A' }}°
              <span v-if="visibilityData.elevation_range_deg" class="text-gray-600">
                ({{ visibilityData.elevation_range_deg.toFixed(1) }}° range)
              </span>
            </p>
          </div>
        </div>
      </div>

      <!-- Visibility Periods Chart -->
      <div class="bg-white p-4 rounded-lg shadow">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">
          Visibility Windows ({{ visibilityData.visibility_periods.length }} periods)
        </h3>
        <div ref="chartContainer" style="height: 400px"></div>
      </div>

      <!-- Visibility Periods Table -->
      <div class="bg-white p-4 rounded-lg shadow">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">Visibility Period Details</h3>
        <div class="overflow-x-auto">
          <table class="min-w-full divide-y divide-gray-200">
            <thead class="bg-gray-50">
              <tr>
                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Period
                </th>
                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Start (MJD)
                </th>
                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Stop (MJD)
                </th>
                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Duration (hours)
                </th>
              </tr>
            </thead>
            <tbody class="bg-white divide-y divide-gray-200">
              <tr v-for="(period, index) in visibilityData.visibility_periods" :key="index">
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  {{ index + 1 }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  {{ period.start.toFixed(4) }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  {{ period.stop.toFixed(4) }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  {{ ((period.stop - period.start) * 24).toFixed(2) }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div v-else class="text-center py-12 text-gray-500">
      <p>Select a scheduling block to view its visibility windows</p>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, computed, onMounted, watch, nextTick } from 'vue'
import * as echarts from 'echarts/core'
import { BarChart } from 'echarts/charts'
import {
  GridComponent,
  TooltipComponent,
  LegendComponent
} from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'

echarts.use([BarChart, GridComponent, TooltipComponent, LegendComponent, CanvasRenderer])

interface VisibilityPeriod {
  start: number
  stop: number
}

interface VisibilityMapData {
  scheduling_block_id: string
  right_ascension_deg: number
  declination_deg: number
  requested_hours: number
  total_visibility_hours: number
  priority: number
  scheduled_flag: boolean
  visibility_periods: VisibilityPeriod[]
  azimuth_min_deg?: number
  azimuth_max_deg?: number
  elevation_min_deg?: number
  elevation_max_deg?: number
  elevation_range_deg?: number
}

interface Block {
  schedulingBlockId: string
  priority: number
}

export default defineComponent({
  setup() {
    const selectedBlockId = ref('')
    const visibilityData = ref<VisibilityMapData | null>(null)
    const allBlocks = ref<Block[]>([])
    const loading = ref(false)
    const error = ref('')
    const chartContainer = ref<HTMLElement | null>(null)
    let chart: echarts.ECharts | null = null

    const loadAllBlocks = async () => {
      try {
        const response = await fetch('http://localhost:8081/api/v1/datasets/current')
        if (!response.ok) throw new Error('Failed to load dataset')
        const data = await response.json()
        
        // Map to simpler format for dropdown, sort by priority descending
        allBlocks.value = data.blocks
          .map((b: any) => ({
            schedulingBlockId: b.schedulingBlockId,
            priority: b.priority
          }))
          .sort((a: Block, b: Block) => b.priority - a.priority)
          .slice(0, 100) // Limit to first 100 for performance
      } catch (err) {
        console.error('Error loading blocks:', err)
        error.value = 'Failed to load scheduling blocks'
      }
    }

    const loadVisibilityData = async () => {
      if (!selectedBlockId.value) return

      loading.value = true
      error.value = ''
      visibilityData.value = null

      try {
        const response = await fetch(
          `http://localhost:8081/api/v1/visualizations/visibility-map?block_id=${selectedBlockId.value}`
        )
        if (!response.ok) {
          throw new Error('Failed to load visibility data')
        }
        visibilityData.value = await response.json()
        
        await nextTick()
        renderChart()
      } catch (err) {
        console.error('Error loading visibility data:', err)
        error.value = 'Failed to load visibility data for this block'
      } finally {
        loading.value = false
      }
    }

    const renderChart = () => {
      if (!chartContainer.value || !visibilityData.value) return

      if (!chart) {
        chart = echarts.init(chartContainer.value)
      }

      const periods = visibilityData.value.visibility_periods

      // Create bar chart showing visibility periods as horizontal bars
      const option = {
        tooltip: {
          trigger: 'axis',
          axisPointer: {
            type: 'shadow'
          },
          formatter: (params: any) => {
            const period = periods[params[0].dataIndex]
            const duration = ((period.stop - period.start) * 24).toFixed(2)
            return `
              <strong>Period ${params[0].dataIndex + 1}</strong><br/>
              Start: ${period.start.toFixed(4)} MJD<br/>
              Stop: ${period.stop.toFixed(4)} MJD<br/>
              Duration: ${duration} hours
            `
          }
        },
        grid: {
          left: '10%',
          right: '10%',
          bottom: '15%',
          top: '10%'
        },
        xaxis: {
          type: 'category',
          data: periods.map((_: any, i: number) => `Period ${i + 1}`),
          axisLabel: {
            rotate: 45,
            fontSize: 10
          }
        },
        yAxis: {
          type: 'value',
          name: 'Duration (hours)',
          nameLocation: 'middle',
          nameGap: 50
        },
        series: [
          {
            name: 'Visibility Duration',
            type: 'bar',
            data: periods.map((p: VisibilityPeriod) => ((p.stop - p.start) * 24).toFixed(2)),
            itemStyle: {
              color: '#3b82f6'
            },
            label: {
              show: true,
              position: 'top',
              formatter: '{c}h',
              fontSize: 10
            }
          }
        ]
      }

      chart.setOption(option)
    }

    onMounted(() => {
      loadAllBlocks()
      window.addEventListener('resize', () => chart?.resize())
    })

    watch(() => visibilityData.value, () => {
      if (visibilityData.value) {
        nextTick(() => renderChart())
      }
    })

    return {
      selectedBlockId,
      visibilityData,
      allBlocks,
      loading,
      error,
      chartContainer,
      loadVisibilityData
    }
  }
})
</script>
