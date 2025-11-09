<template>
  <div class="p-6">
    <h1 class="text-3xl font-bold text-gray-900 mb-6">Scheduling Trends Analysis</h1>

    <!-- Controls -->
    <div class="mb-6 bg-white p-4 rounded-lg shadow">
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">
            Select Metric
          </label>
          <select
            v-model="selectedMetric"
            @change="loadTrends"
            class="w-full border border-gray-300 rounded-md px-3 py-2"
          >
            <option value="scheduling_rate">Scheduling Rate (%)</option>
            <option value="utilization">Utilization Rate (%)</option>
            <option value="avg_priority">Average Priority</option>
          </select>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700 mb-2">
            Group By
          </label>
          <select
            v-model="selectedGroupBy"
            @change="loadTrends"
            class="w-full border border-gray-300 rounded-md px-3 py-2"
          >
            <option value="month">Month</option>
            <option value="week">Week</option>
            <option value="day">Day</option>
          </select>
        </div>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="loading" class="text-center py-12">
      <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
      <p class="mt-4 text-gray-600">Loading trends data...</p>
    </div>

    <!-- Error State -->
    <div v-else-if="error" class="bg-red-50 border border-red-200 rounded-lg p-4">
      <p class="text-red-800">{{ error }}</p>
    </div>

    <!-- Trends Data -->
    <div v-else-if="trendsData.length > 0" class="space-y-6">
      <!-- Summary Cards -->
      <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
        <div class="bg-white p-4 rounded-lg shadow">
          <h3 class="text-sm font-medium text-gray-500">Periods</h3>
          <p class="mt-2 text-2xl font-bold text-gray-900">{{ trendsData.length }}</p>
        </div>

        <div class="bg-white p-4 rounded-lg shadow">
          <h3 class="text-sm font-medium text-gray-500">Average Value</h3>
          <p class="mt-2 text-2xl font-bold text-gray-900">
            {{ averageValue.toFixed(2) }}{{ metricUnit }}
          </p>
        </div>

        <div class="bg-white p-4 rounded-lg shadow">
          <h3 class="text-sm font-medium text-gray-500">Minimum</h3>
          <p class="mt-2 text-2xl font-bold text-gray-900">
            {{ minValue.toFixed(2) }}{{ metricUnit }}
          </p>
        </div>

        <div class="bg-white p-4 rounded-lg shadow">
          <h3 class="text-sm font-medium text-gray-500">Maximum</h3>
          <p class="mt-2 text-2xl font-bold text-gray-900">
            {{ maxValue.toFixed(2) }}{{ metricUnit }}
          </p>
        </div>
      </div>

      <!-- Line Chart -->
      <div class="bg-white p-4 rounded-lg shadow">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">
          {{ metricTitle }} Over Time
        </h3>
        <div ref="chartContainer" style="height: 500px"></div>
      </div>

      <!-- Trends Table -->
      <div class="bg-white p-4 rounded-lg shadow">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">Detailed Trends Data</h3>
        <div class="overflow-x-auto">
          <table class="min-w-full divide-y divide-gray-200">
            <thead class="bg-gray-50">
              <tr>
                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                  Period
                </th>
                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                  {{ metricTitle }}
                </th>
                <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                  Observations
                </th>
              </tr>
            </thead>
            <tbody class="bg-white divide-y divide-gray-200">
              <tr v-for="(point, index) in trendsData" :key="index">
                <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                  {{ point.period }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  {{ point.value.toFixed(2) }}{{ metricUnit }}
                </td>
                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  {{ point.count }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div v-else class="text-center py-12 text-gray-500">
      <p>No trends data available</p>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, computed, onMounted, watch, nextTick } from 'vue'
import * as echarts from 'echarts/core'
import { LineChart } from 'echarts/charts'
import {
  GridComponent,
  TooltipComponent,
  LegendComponent,
  MarkLineComponent
} from 'echarts/components'
import { CanvasRenderer } from 'echarts/renderers'

echarts.use([LineChart, GridComponent, TooltipComponent, LegendComponent, MarkLineComponent, CanvasRenderer])

interface TrendPoint {
  period: string
  value: number
  count: number
}

export default defineComponent({
  setup() {
    const trendsData = ref<TrendPoint[]>([])
    const selectedMetric = ref('scheduling_rate')
    const selectedGroupBy = ref('month')
    const loading = ref(false)
    const error = ref('')
    const chartContainer = ref<HTMLElement | null>(null)
    let chart: echarts.ECharts | null = null

    const metricTitle = computed(() => {
      const titles: Record<string, string> = {
        'scheduling_rate': 'Scheduling Rate',
        'utilization': 'Utilization Rate',
        'avg_priority': 'Average Priority'
      }
      return titles[selectedMetric.value] || selectedMetric.value
    })

    const metricUnit = computed(() => {
      if (selectedMetric.value === 'scheduling_rate' || selectedMetric.value === 'utilization') {
        return '%'
      }
      return ''
    })

    const averageValue = computed(() => {
      if (trendsData.value.length === 0) return 0
      const sum = trendsData.value.reduce((acc, point) => acc + point.value, 0)
      return sum / trendsData.value.length
    })

    const minValue = computed(() => {
      if (trendsData.value.length === 0) return 0
      return Math.min(...trendsData.value.map(p => p.value))
    })

    const maxValue = computed(() => {
      if (trendsData.value.length === 0) return 0
      return Math.max(...trendsData.value.map(p => p.value))
    })

    const loadTrends = async () => {
      loading.value = true
      error.value = ''

      try {
        const url = `http://localhost:8081/api/v1/analytics/trends?metric=${selectedMetric.value}&group_by=${selectedGroupBy.value}`
        const response = await fetch(url)
        if (!response.ok) throw new Error('Failed to load trends data')
        
        const data = await response.json()
        trendsData.value = data.data

        await nextTick()
        renderChart()
      } catch (err) {
        console.error('Error loading trends:', err)
        error.value = 'Failed to load trends data'
      } finally {
        loading.value = false
      }
    }

    const renderChart = () => {
      if (!chartContainer.value || trendsData.value.length === 0) return

      if (!chart) {
        chart = echarts.init(chartContainer.value)
      }

      const periods = trendsData.value.map(p => p.period)
      const values = trendsData.value.map(p => p.value)

      const option = {
        tooltip: {
          trigger: 'axis',
          formatter: (params: any) => {
            const point = trendsData.value[params[0].dataIndex]
            return `
              <strong>${point.period}</strong><br/>
              ${metricTitle.value}: ${point.value.toFixed(2)}${metricUnit.value}<br/>
              Observations: ${point.count}
            `
          }
        },
        grid: {
          left: '12%',
          right: '10%',
          bottom: '15%',
          top: '10%'
        },
        xAxis: {
          type: 'category',
          data: periods,
          axisLabel: {
            rotate: 45,
            fontSize: 10
          }
        },
        yAxis: {
          type: 'value',
          name: `${metricTitle.value} ${metricUnit.value}`,
          nameLocation: 'middle',
          nameGap: 50
        },
        series: [
          {
            name: metricTitle.value,
            type: 'line',
            data: values,
            smooth: true,
            lineStyle: {
              width: 3,
              color: '#3b82f6'
            },
            itemStyle: {
              color: '#3b82f6'
            },
            areaStyle: {
              color: {
                type: 'linear',
                x: 0,
                y: 0,
                x2: 0,
                y2: 1,
                colorStops: [
                  { offset: 0, color: 'rgba(59, 130, 246, 0.3)' },
                  { offset: 1, color: 'rgba(59, 130, 246, 0.05)' }
                ]
              }
            },
            markLine: {
              silent: true,
              lineStyle: {
                color: '#ef4444',
                type: 'dashed'
              },
              data: [
                {
                  type: 'average',
                  name: 'Average',
                  label: {
                    formatter: 'Avg: {c}'
                  }
                }
              ]
            }
          }
        ]
      }

      chart.setOption(option)
    }

    onMounted(() => {
      loadTrends()
      window.addEventListener('resize', () => chart?.resize())
    })

    watch([selectedMetric, selectedGroupBy], () => {
      // Auto-reload on metric or grouping change
    })

    return {
      trendsData,
      selectedMetric,
      selectedGroupBy,
      loading,
      error,
      chartContainer,
      metricTitle,
      metricUnit,
      averageValue,
      minValue,
      maxValue,
      loadTrends
    }
  }
})
</script>
