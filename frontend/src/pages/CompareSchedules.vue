<template>
  <div class="min-h-screen bg-gray-50 p-6">
    <div class="max-w-7xl mx-auto">
      <!-- Header -->
      <div class="mb-6">
        <h1 class="text-3xl font-bold text-gray-900 mb-2">Compare Schedules</h1>
        <p class="text-gray-600">Upload a second dataset to compare against the primary schedule</p>
      </div>

      <!-- Upload Section -->
      <div v-if="!comparisonLoaded" class="bg-white rounded-lg shadow p-6 mb-6">
        <h2 class="text-xl font-semibold mb-4">Upload Comparison Dataset</h2>
        <div class="border-2 border-dashed border-gray-300 rounded-lg p-8 text-center">
          <input
            type="file"
            ref="fileInput"
            @change="handleFileUpload"
            accept=".csv"
            class="hidden"
          />
          <svg
            class="mx-auto h-12 w-12 text-gray-400"
            stroke="currentColor"
            fill="none"
            viewBox="0 0 48 48"
          >
            <path
              d="M28 8H12a4 4 0 00-4 4v20m32-12v8m0 0v8a4 4 0 01-4 4H12a4 4 0 01-4-4v-4m32-4l-3.172-3.172a4 4 0 00-5.656 0L28 28M8 32l9.172-9.172a4 4 0 015.656 0L28 28m0 0l4 4m4-24h8m-4-4v8m-12 4h.02"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
          <p class="mt-2 text-sm text-gray-600">
            <button
              @click="$refs.fileInput.click()"
              class="text-blue-600 hover:text-blue-700 font-medium"
            >
              Click to upload
            </button>
            or drag and drop
          </p>
          <p class="mt-1 text-xs text-gray-500">CSV files only</p>
          <button
            v-if="selectedFile"
            @click="uploadFile"
            :disabled="uploading"
            class="mt-4 px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed"
          >
            {{ uploading ? 'Uploading...' : `Upload ${selectedFile.name}` }}
          </button>
        </div>
        <p v-if="uploadError" class="mt-4 text-sm text-red-600">{{ uploadError }}</p>
      </div>

      <!-- Loading State -->
      <div v-if="loading" class="bg-white rounded-lg shadow p-12 text-center">
        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"></div>
        <p class="mt-4 text-gray-600">Loading comparison data...</p>
      </div>

      <!-- Comparison Results -->
      <div v-if="comparison && !loading" class="space-y-6">
        <!-- Dataset Info Cards -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <!-- Primary Dataset -->
          <div class="bg-white rounded-lg shadow p-6">
            <h3 class="text-lg font-semibold mb-4 text-blue-600">Primary Dataset</h3>
            <div class="space-y-2">
              <div class="flex justify-between">
                <span class="text-gray-600">Filename:</span>
                <span class="font-medium">{{ comparison.primary.filename }}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-gray-600">Total Blocks:</span>
                <span class="font-medium">{{ comparison.primary.total_blocks }}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-gray-600">Scheduled:</span>
                <span class="font-medium text-green-600">{{ comparison.primary.scheduled_blocks }}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-gray-600">Unscheduled:</span>
                <span class="font-medium text-red-600">{{ comparison.primary.unscheduled_blocks }}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-gray-600">Scheduling Rate:</span>
                <span class="font-medium">{{ comparison.primary.scheduling_rate.toFixed(2) }}%</span>
              </div>
              <div class="flex justify-between">
                <span class="text-gray-600">Utilization:</span>
                <span class="font-medium">{{ comparison.primary.utilization.toFixed(2) }}%</span>
              </div>
              <div class="flex justify-between">
                <span class="text-gray-600">Avg Priority:</span>
                <span class="font-medium">{{ comparison.primary.avg_priority.toFixed(2) }}</span>
              </div>
            </div>
          </div>

          <!-- Comparison Dataset -->
          <div class="bg-white rounded-lg shadow p-6">
            <div class="flex justify-between items-center mb-4">
              <h3 class="text-lg font-semibold text-purple-600">Comparison Dataset</h3>
              <button
                @click="clearComparison"
                class="text-sm text-red-600 hover:text-red-700"
              >
                Clear
              </button>
            </div>
            <div class="space-y-2">
              <div class="flex justify-between">
                <span class="text-gray-600">Filename:</span>
                <span class="font-medium">{{ comparison.comparison.filename }}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-gray-600">Total Blocks:</span>
                <span class="font-medium">{{ comparison.comparison.total_blocks }}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-gray-600">Scheduled:</span>
                <span class="font-medium text-green-600">{{ comparison.comparison.scheduled_blocks }}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-gray-600">Unscheduled:</span>
                <span class="font-medium text-red-600">{{ comparison.comparison.unscheduled_blocks }}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-gray-600">Scheduling Rate:</span>
                <span class="font-medium">{{ comparison.comparison.scheduling_rate.toFixed(2) }}%</span>
              </div>
              <div class="flex justify-between">
                <span class="text-gray-600">Utilization:</span>
                <span class="font-medium">{{ comparison.comparison.utilization.toFixed(2) }}%</span>
              </div>
              <div class="flex justify-between">
                <span class="text-gray-600">Avg Priority:</span>
                <span class="font-medium">{{ comparison.comparison.avg_priority.toFixed(2) }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Diff Summary Cards -->
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div class="bg-green-50 rounded-lg p-4 border border-green-200">
            <div class="text-2xl font-bold text-green-700">{{ comparison.diff.blocks_added }}</div>
            <div class="text-sm text-green-600">Blocks Added</div>
          </div>
          <div class="bg-red-50 rounded-lg p-4 border border-red-200">
            <div class="text-2xl font-bold text-red-700">{{ comparison.diff.blocks_removed }}</div>
            <div class="text-sm text-red-600">Blocks Removed</div>
          </div>
          <div class="bg-yellow-50 rounded-lg p-4 border border-yellow-200">
            <div class="text-2xl font-bold text-yellow-700">{{ comparison.diff.blocks_modified }}</div>
            <div class="text-sm text-yellow-600">Blocks Modified</div>
          </div>
          <div class="bg-gray-50 rounded-lg p-4 border border-gray-200">
            <div class="text-2xl font-bold text-gray-700">{{ comparison.diff.blocks_unchanged }}</div>
            <div class="text-sm text-gray-600">Blocks Unchanged</div>
          </div>
        </div>

        <!-- Metric Differences -->
        <div class="bg-white rounded-lg shadow p-6">
          <h3 class="text-lg font-semibold mb-4">Metric Differences (Comparison - Primary)</h3>
          <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div class="text-center">
              <div class="text-sm text-gray-600 mb-1">Scheduling Rate Change</div>
              <div :class="getDiffColor(comparison.diff.scheduling_rate_diff)" class="text-3xl font-bold">
                {{ comparison.diff.scheduling_rate_diff > 0 ? '+' : '' }}{{ comparison.diff.scheduling_rate_diff.toFixed(2) }}%
              </div>
            </div>
            <div class="text-center">
              <div class="text-sm text-gray-600 mb-1">Utilization Change</div>
              <div :class="getDiffColor(comparison.diff.utilization_diff)" class="text-3xl font-bold">
                {{ comparison.diff.utilization_diff > 0 ? '+' : '' }}{{ comparison.diff.utilization_diff.toFixed(2) }}%
              </div>
            </div>
            <div class="text-center">
              <div class="text-sm text-gray-600 mb-1">Average Priority Change</div>
              <div :class="getDiffColor(comparison.diff.avg_priority_diff)" class="text-3xl font-bold">
                {{ comparison.diff.avg_priority_diff > 0 ? '+' : '' }}{{ comparison.diff.avg_priority_diff.toFixed(2) }}
              </div>
            </div>
          </div>
          <div v-if="comparison.diff.newly_scheduled > 0 || comparison.diff.newly_unscheduled > 0" class="mt-6 pt-6 border-t">
            <div class="grid grid-cols-2 gap-4">
              <div class="text-center">
                <div class="text-sm text-gray-600 mb-1">Newly Scheduled</div>
                <div class="text-2xl font-bold text-green-600">{{ comparison.diff.newly_scheduled }}</div>
              </div>
              <div class="text-center">
                <div class="text-sm text-gray-600 mb-1">Newly Unscheduled</div>
                <div class="text-2xl font-bold text-red-600">{{ comparison.diff.newly_unscheduled }}</div>
              </div>
            </div>
          </div>
        </div>

        <!-- Changes Table -->
        <div class="bg-white rounded-lg shadow p-6">
          <div class="flex justify-between items-center mb-4">
            <h3 class="text-lg font-semibold">Block Changes</h3>
            <div class="flex gap-2">
              <button
                v-for="filter in changeFilters"
                :key="filter.value"
                @click="selectedFilter = filter.value"
                :class="[
                  'px-3 py-1 text-sm rounded',
                  selectedFilter === filter.value
                    ? 'bg-blue-600 text-white'
                    : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
                ]"
              >
                {{ filter.label }} ({{ getFilteredChanges(filter.value).length }})
              </button>
            </div>
          </div>
          <div class="overflow-x-auto">
            <table class="min-w-full divide-y divide-gray-200">
              <thead class="bg-gray-50">
                <tr>
                  <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">Block ID</th>
                  <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">Change Type</th>
                  <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">Primary Status</th>
                  <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">Comparison Status</th>
                  <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">Primary Priority</th>
                  <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">Comparison Priority</th>
                </tr>
              </thead>
              <tbody class="bg-white divide-y divide-gray-200">
                <tr v-for="change in paginatedChanges" :key="change.scheduling_block_id">
                  <td class="px-4 py-3 text-sm font-mono text-gray-900">{{ change.scheduling_block_id }}</td>
                  <td class="px-4 py-3 text-sm">
                    <span :class="getChangeTypeClass(change.change_type)">
                      {{ formatChangeType(change.change_type) }}
                    </span>
                  </td>
                  <td class="px-4 py-3 text-sm">
                    <span v-if="change.primary_scheduled !== null" :class="change.primary_scheduled ? 'text-green-600' : 'text-gray-500'">
                      {{ change.primary_scheduled ? 'Scheduled' : 'Unscheduled' }}
                    </span>
                    <span v-else class="text-gray-400">N/A</span>
                  </td>
                  <td class="px-4 py-3 text-sm">
                    <span v-if="change.comparison_scheduled !== null" :class="change.comparison_scheduled ? 'text-green-600' : 'text-gray-500'">
                      {{ change.comparison_scheduled ? 'Scheduled' : 'Unscheduled' }}
                    </span>
                    <span v-else class="text-gray-400">N/A</span>
                  </td>
                  <td class="px-4 py-3 text-sm">
                    <span v-if="change.primary_priority !== null">{{ change.primary_priority.toFixed(2) }}</span>
                    <span v-else class="text-gray-400">N/A</span>
                  </td>
                  <td class="px-4 py-3 text-sm">
                    <span v-if="change.comparison_priority !== null">{{ change.comparison_priority.toFixed(2) }}</span>
                    <span v-else class="text-gray-400">N/A</span>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
          <!-- Pagination -->
          <div v-if="filteredChanges.length > itemsPerPage" class="mt-4 flex justify-between items-center">
            <div class="text-sm text-gray-600">
              Showing {{ (currentPage - 1) * itemsPerPage + 1 }} to {{ Math.min(currentPage * itemsPerPage, filteredChanges.length) }}
              of {{ filteredChanges.length }} changes
            </div>
            <div class="flex gap-2">
              <button
                @click="currentPage--"
                :disabled="currentPage === 1"
                class="px-3 py-1 bg-gray-100 text-gray-700 rounded hover:bg-gray-200 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Previous
              </button>
              <button
                @click="currentPage++"
                :disabled="currentPage * itemsPerPage >= filteredChanges.length"
                class="px-3 py-1 bg-gray-100 text-gray-700 rounded hover:bg-gray-200 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Next
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Error State -->
      <div v-if="error" class="bg-red-50 border border-red-200 rounded-lg p-4">
        <p class="text-red-800">{{ error }}</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import axios from 'axios'

const API_BASE = 'http://localhost:8081'

// State
const selectedFile = ref<File | null>(null)
const uploading = ref(false)
const uploadError = ref('')
const comparisonLoaded = ref(false)
const loading = ref(false)
const error = ref('')
const comparison = ref<any>(null)
const fileInput = ref<HTMLInputElement | null>(null)

// Filter state
const selectedFilter = ref('all')
const currentPage = ref(1)
const itemsPerPage = 50

const changeFilters = [
  { label: 'All', value: 'all' },
  { label: 'Added', value: 'added' },
  { label: 'Removed', value: 'removed' },
  { label: 'Modified', value: 'modified' },
  { label: 'Unchanged', value: 'unchanged' },
]

// Methods
const handleFileUpload = (event: Event) => {
  const target = event.target as HTMLInputElement
  if (target.files && target.files.length > 0) {
    selectedFile.value = target.files[0]
    uploadError.value = ''
  }
}

const uploadFile = async () => {
  if (!selectedFile.value) return

  uploading.value = true
  uploadError.value = ''

  try {
    const formData = new FormData()
    formData.append('file', selectedFile.value)

    await axios.post(`${API_BASE}/api/v1/datasets/comparison/upload`, formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
    })

    comparisonLoaded.value = true
    selectedFile.value = null
    await fetchComparison()
  } catch (err: any) {
    uploadError.value = err.response?.data?.message || err.message || 'Failed to upload file'
  } finally {
    uploading.value = false
  }
}

const fetchComparison = async () => {
  loading.value = true
  error.value = ''

  try {
    const response = await axios.get(`${API_BASE}/api/v1/analytics/compare`)
    comparison.value = response.data
  } catch (err: any) {
    error.value = err.response?.data || err.message || 'Failed to fetch comparison'
    comparisonLoaded.value = false
  } finally {
    loading.value = false
  }
}

const clearComparison = async () => {
  try {
    await axios.delete(`${API_BASE}/api/v1/datasets/comparison`)
    comparison.value = null
    comparisonLoaded.value = false
    selectedFilter.value = 'all'
    currentPage.value = 1
  } catch (err: any) {
    error.value = err.response?.data || err.message || 'Failed to clear comparison'
  }
}

const getDiffColor = (diff: number) => {
  if (diff > 0) return 'text-green-600'
  if (diff < 0) return 'text-red-600'
  return 'text-gray-600'
}

const getChangeTypeClass = (type: string) => {
  const classes: Record<string, string> = {
    added: 'px-2 py-1 bg-green-100 text-green-800 rounded text-xs font-medium',
    removed: 'px-2 py-1 bg-red-100 text-red-800 rounded text-xs font-medium',
    modified: 'px-2 py-1 bg-yellow-100 text-yellow-800 rounded text-xs font-medium',
    unchanged: 'px-2 py-1 bg-gray-100 text-gray-800 rounded text-xs font-medium',
  }
  return classes[type] || classes.unchanged
}

const formatChangeType = (type: string) => {
  return type.charAt(0).toUpperCase() + type.slice(1)
}

const getFilteredChanges = (filter: string) => {
  if (!comparison.value) return []
  if (filter === 'all') return comparison.value.changes
  return comparison.value.changes.filter((c: any) => c.change_type === filter)
}

const filteredChanges = computed(() => getFilteredChanges(selectedFilter.value))

const paginatedChanges = computed(() => {
  const start = (currentPage.value - 1) * itemsPerPage
  const end = start + itemsPerPage
  return filteredChanges.value.slice(start, end)
})

// Check if comparison dataset exists on mount
onMounted(async () => {
  try {
    await fetchComparison()
    comparisonLoaded.value = true
  } catch {
    // No comparison dataset loaded yet
    comparisonLoaded.value = false
  }
})
</script>
