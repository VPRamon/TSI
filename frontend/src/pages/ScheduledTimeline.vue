<template>
  <div class="p-6">
    <h1 class="text-3xl font-bold text-gray-900 mb-6">Scheduled Timeline</h1>

    <!-- Filters -->
    <div class="mb-6 bg-white p-4 rounded-lg shadow">
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">
            Filter by Month
          </label>
          <select
            v-model="selectedMonth"
            @change="loadTimeline"
            class="w-full border border-gray-300 rounded-md px-3 py-2"
          >
            <option value="">All Months</option>
            <option v-for="month in 12" :key="month" :value="month">
              {{ getMonthName(month) }}
            </option>
          </select>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">
            Filter by Year
          </label>
          <select
            v-model="selectedYear"
            @change="loadTimeline"
            class="w-full border border-gray-300 rounded-md px-3 py-2"
          >
            <option value="">All Years</option>
            <option v-for="year in availableYears" :key="year" :value="year">
              {{ year }}
            </option>
          </select>
        </div>

        <div class="flex items-end">
          <button
            @click="exportCSV"
            :disabled="!observations.length"
            class="w-full px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:bg-gray-400 disabled:cursor-not-allowed"
          >
            Export CSV
          </button>
        </div>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="loading" class="text-center py-12">
      <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      <p class="mt-4 text-gray-600">Loading timeline data...</p>
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="bg-red-50 border border-red-200 rounded-lg p-4">
      <p class="text-red-800">{{ error }}</p>
    </div>

    <!-- Timeline Data -->
    <div v-else-if="observations.length > 0" class="space-y-6">
      <!-- Summary Card -->
      <div class="bg-white p-4 rounded-lg shadow">
        <h3 class="text-lg font-semibold text-gray-900 mb-2">
          Scheduled Observations Summary
        </h3>
        <p class="text-gray-700">
          Showing <span class="font-semibold">{{ observations.length }}</span> scheduled observations
          <span v-if="selectedMonth && selectedYear">
            for {{ getMonthName(parseInt(selectedMonth)) }} {{ selectedYear }}
          </span>
        </p>
        <p class="text-gray-700 mt-2">
          Total scheduled time: <span class="font-semibold">{{ totalScheduledHours.toFixed(2) }}</span> hours
        </p>
      </div>

      <!-- Timeline Chart -->
      <div class="bg-white p-4 rounded-lg shadow">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">Timeline Visualization</h3>
        <div ref="chartContainer" style="height: 500px"></div>
      </div>

      <!-- Observations Table -->
      <div class="bg-white p-4 rounded-lg shadow">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">Scheduled Observations (First 50)</h3>
        <div class="overflow-x-auto">
          <table class="min-w-full divide-y divide-gray-200">
            <thead class="bg-gray-50">
              <tr>
                <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                  Block ID
                </th>
                <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                  Scheduled Time
                </th>
                <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                  Duration (h)
                </th>
                <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                  Priority
                </th>
                <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                  RA / Dec
                </th>
              </tr>
            </thead>
            <tbody class="bg-white divide-y divide-gray-200">
              <tr v-for="obs in observations.slice(0, 50)" :key="obs.scheduling_block_id">
                <td class="px-4 py-3 whitespace-nowrap text-sm font-medium text-gray-900">
                  {{ obs.scheduling_block_id }}
                </td>
                <td class="px-4 py-3 whitespace-nowrap text-sm text-gray-900">
                  {{ obs.scheduled_time_iso }}
                </td>
                <td class="px-4 py-3 whitespace-nowrap text-sm text-gray-900">
                  {{ obs.scheduled_duration_hours.toFixed(2) }}
                </td>
                <td class="px-4 py-3 whitespace-nowrap text-sm text-gray-900">
                  <span :class="getPriorityColor(obs.priority)">
                    {{ obs.priority.toFixed(2) }}
                  </span>
                </td>
                <td class="px-4 py-3 whitespace-nowrap text-sm text-gray-900">
                  {{ obs.right_ascension_deg.toFixed(2) }}° / {{ obs.declination_deg.toFixed(2) }}°
                </td>
              </tr>
            </tbody>
          </table>
          <p v-if="observations.length > 50" class="mt-4 text-sm text-gray-600 text-center">
            Showing first 50 of {{ observations.length }} observations. Use filters or export CSV for full data.
          </p>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div v-else class="text-center py-12 text-gray-500">
      <p>No scheduled observations found for the selected filters</p>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, computed, onMounted, watch, nextTick } from 'vue'
import * as echarts from 'echarts/core'
import { ScatterChart } from 'echarts/charts'
import {
  GridComponent,
  TooltipComponent,
  LegendComponent,
  DataZoomComponent
} from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'

echarts.use([ScatterChart, GridComponent, TooltipComponent, LegendComponent, DataZoomComponent, CanvasRenderer])

interface TimelineObservation {
  scheduling_block_id: string
  scheduled_time_mjd: number
  scheduled_time_iso: string
  scheduled_duration_hours: number
  priority: number
  priority_bin: string
  right_ascension_deg: number
  declination_deg: number
}

export default defineComponent({
  setup() {
    const observations = ref<TimelineObservation[]>([])
    const selectedMonth = ref('')
    const selectedYear = ref('')
    const loading = ref(false)
    const error = ref('')
    const chartContainer = ref<HTMLElement | null>(null)
    let chart: echarts.ECharts | null = null

    const availableYears = computed(() => {
      // Return years 2027-2029 based on sample data
      return [2027, 2028, 2029]
    })

    const totalScheduledHours = computed(() => {
      return observations.value.reduce((sum, obs) => sum + obs.scheduled_duration_hours, 0)
    })

    const getMonthName = (month: number): string => {
      const months = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec']
      return months[month - 1]
    }

    const getPriorityColor = (priority: number): string => {
      if (priority >= 20) return 'text-red-600 font-semibold'
      if (priority >= 15) return 'text-orange-600 font-semibold'
      if (priority >= 10) return 'text-yellow-600 font-semibold'
      return 'text-gray-900'
    }

    const loadTimeline = async () => {
      loading.value = true
      error.value = ''

      try {
        let url = 'http://localhost:8081/api/v1/visualizations/timeline'
        const params = new URLSearchParams()
        if (selectedMonth.value) params.append('month', selectedMonth.value)
        if (selectedYear.value) params.append('year', selectedYear.value)
        if (params.toString()) url += '?' + params.toString()

        const response = await fetch(url)
        if (!response.ok) throw new Error('Failed to load timeline data')
        
        const data = await response.json()
        observations.value = data.observations

        await nextTick()
        renderChart()
      } catch (err) {
        console.error('Error loading timeline:', err)
        error.value = 'Failed to load timeline data'
      } finally {
        loading.value = false
      }
    }

    const renderChart = () => {
      if (!chartContainer.value || observations.value.length === 0) return

      if (!chart) {
        chart = echarts.init(chartContainer.value)
      }

      // Convert observations to scatter plot data: [MJD, duration]
      const data = observations.value.map(obs => [
        obs.scheduled_time_mjd,
        obs.scheduled_duration_hours,
        obs.scheduling_block_id,
        obs.priority
      ])

      const option = {
        tooltip: {
          trigger: 'item',
          formatter: (params: any) => {
            const [mjd, duration, blockId, priority] = params.value
            return `
              <strong>Block ${blockId}</strong><br/>
              MJD: ${mjd.toFixed(2)}<br/>
              Duration: ${duration.toFixed(2)}h<br/>
              Priority: ${priority.toFixed(2)}
            `
          }
        },
        grid: {
          left: '12%',
          right: '10%',
          bottom: '20%',
          top: '10%'
        },
        xAxis: {
          type: 'value',
          name: 'Scheduled Time (MJD)',
          nameLocation: 'middle',
          nameGap: 30,
          scale: true
        },
        yAxis: {
          type: 'value',
          name: 'Duration (hours)',
          nameLocation: 'middle',
          nameGap: 50
        },
        dataZoom: [
          {
            type: 'slider',
            xAxisIndex: 0,
            start: 0,
            end: 100
          }
        ],
        series: [
          {
            name: 'Scheduled Observations',
            type: 'scatter',
            data: data,
            symbolSize: (value: number[]) => Math.max(5, Math.min(20, value[1] * 5)),
            itemStyle: {
              color: (params: any) => {
                const priority = params.value[3]
                if (priority >= 20) return '#dc2626'
                if (priority >= 15) return '#f97316'
                if (priority >= 10) return '#eab308'
                return '#3b82f6'
              },
              opacity: 0.7
            }
          }
        ]
      }

      chart.setOption(option)
    }

    const exportCSV = () => {
      if (observations.value.length === 0) return

      const headers = [
        'Block ID',
        'Scheduled Time (ISO)',
        'Scheduled Time (MJD)',
        'Duration (hours)',
        'Priority',
        'Priority Bin',
        'RA (deg)',
        'Dec (deg)'
      ]

      const rows = observations.value.map(obs => [
        obs.scheduling_block_id,
        obs.scheduled_time_iso,
        obs.scheduled_time_mjd.toFixed(4),
        obs.scheduled_duration_hours.toFixed(3),
        obs.priority.toFixed(3),
        obs.priority_bin,
        obs.right_ascension_deg.toFixed(6),
        obs.declination_deg.toFixed(6)
      ])

      const csvContent = [
        headers.join(','),
        ...rows.map(row => row.join(','))
      ].join('\n')

      const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' })
      const link = document.createElement('a')
      link.href = URL.createObjectURL(blob)
      link.download = `timeline_${selectedMonth.value || 'all'}_${selectedYear.value || 'all'}.csv`
      link.click()
      URL.revokeObjectURL(link.href)
    }

    onMounted(() => {
      loadTimeline()
      window.addEventListener('resize', () => chart?.resize())
    })

    watch([selectedMonth, selectedYear], () => {
      // Don't auto-reload, let user click button or use @change
    })

    return {
      observations,
      selectedMonth,
      selectedYear,
      availableYears,
      loading,
      error,
      chartContainer,
      totalScheduledHours,
      getMonthName,
      getPriorityColor,
      loadTimeline,
      exportCSV
    }
  }
})
</script>
