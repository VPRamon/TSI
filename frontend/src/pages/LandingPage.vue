<template>
  <div class="max-w-4xl mx-auto mt-12">
    <h1 class="text-4xl font-bold text-gray-900 mb-2">Telescope Scheduling Intelligence</h1>
    <p class="text-gray-600 mb-8">Upload schedule data to begin analysis</p>

    <!-- Upload Options -->
    <div class="grid grid-cols-2 gap-6 mb-8">
      <!-- CSV Upload -->
      <div class="bg-white p-6 rounded-lg shadow border">
        <h3 class="text-lg font-semibold mb-4">Upload Preprocessed CSV</h3>
        <FileUpload 
          accept=".csv" 
          uploadType="csv"
          @upload="handleUpload"
        />
      </div>

      <!-- JSON Upload -->
      <div class="bg-white p-6 rounded-lg shadow border">
        <h3 class="text-lg font-semibold mb-4">Upload Raw JSON</h3>
        <FileUpload 
          accept=".json" 
          uploadType="json"
          :multiple="true"
          @upload="handleUpload"
        />
        <p class="text-xs text-gray-500 mt-2">
          Upload schedule.json and optionally possible_periods.json
        </p>
      </div>
    </div>

    <!-- Sample Dataset -->
    <div class="bg-blue-50 p-6 rounded-lg border border-blue-200">
      <h3 class="text-lg font-semibold mb-2">Or Try Sample Data</h3>
      <p class="text-sm text-gray-600 mb-4">
        Load a sample dataset (2,647 scheduling blocks) to explore the app
      </p>
      <button 
        @click="loadSample"
        :disabled="loading"
        class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {{ loading ? 'Loading...' : 'Load Sample Dataset' }}
      </button>
    </div>

    <!-- Progress Bar -->
    <div v-if="uploadProgress > 0 && uploadProgress < 100" class="mt-6">
      <div class="bg-gray-200 rounded-full h-4 overflow-hidden">
        <div 
          class="bg-blue-600 h-full transition-all duration-300"
          :style="{ width: uploadProgress + '%' }"
        ></div>
      </div>
      <p class="text-sm text-gray-600 mt-2 text-center">{{ progressMessage }}</p>
    </div>

    <!-- Success Message -->
    <div v-if="successMessage" class="mt-6 p-4 bg-green-50 border border-green-200 rounded-lg">
      <p class="text-green-800">{{ successMessage }}</p>
    </div>

    <!-- Error Message -->
    <div v-if="errorMessage" class="mt-6 p-4 bg-red-50 border border-red-200 rounded-lg">
      <p class="text-red-800">{{ errorMessage }}</p>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue'
import { useRouter } from 'vue-router'
import FileUpload from '../components/FileUpload.vue'
import axios from 'axios'

const API_BASE = 'http://localhost:8081/api/v1'

export default defineComponent({
  components: { FileUpload },
  setup() {
    const router = useRouter()
    const loading = ref(false)
    const uploadProgress = ref(0)
    const progressMessage = ref('')
    const successMessage = ref('')
    const errorMessage = ref('')

    async function handleUpload(data: { type: string; files: File[] }) {
      loading.value = true
      uploadProgress.value = 10
      progressMessage.value = 'Uploading files...'
      errorMessage.value = ''
      successMessage.value = ''

      try {
        const formData = new FormData()
        
        if (data.type === 'csv') {
          formData.append('file', data.files[0])
          uploadProgress.value = 30
          progressMessage.value = 'Parsing CSV...'
          
          const resp = await axios.post(`${API_BASE}/datasets/upload/csv`, formData, {
            headers: { 'Content-Type': 'multipart/form-data' }
          })
          
          uploadProgress.value = 100
          successMessage.value = `Loaded ${resp.data.metadata.num_blocks} scheduling blocks`
          
        } else if (data.type === 'json') {
          // For JSON, expect schedule.json and optional possible_periods.json
          const scheduleFile = data.files.find(f => f.name.includes('schedule'))
          const visibilityFile = data.files.find(f => f.name.includes('possible_periods') || f.name.includes('visibility'))
          
          if (!scheduleFile) {
            throw new Error('Please upload a schedule.json file')
          }
          
          formData.append('schedule', scheduleFile)
          if (visibilityFile) {
            formData.append('visibility', visibilityFile)
          }
          
          uploadProgress.value = 30
          progressMessage.value = 'Parsing JSON and preprocessing...'
          
          const resp = await axios.post(`${API_BASE}/datasets/upload/json`, formData, {
            headers: { 'Content-Type': 'multipart/form-data' }
          })
          
          uploadProgress.value = 100
          successMessage.value = `Loaded and preprocessed ${resp.data.metadata.num_blocks} scheduling blocks`
        }

        // Redirect to Sky Map after 2 seconds
        setTimeout(() => {
          router.push('/sky-map')
        }, 2000)

      } catch (e: any) {
        uploadProgress.value = 0
        errorMessage.value = e.response?.data?.error || e.message || 'Upload failed'
      } finally {
        loading.value = false
      }
    }

    async function loadSample() {
      loading.value = true
      errorMessage.value = ''
      successMessage.value = ''

      try {
        const resp = await axios.post(`${API_BASE}/datasets/sample`)
        successMessage.value = `Loaded ${resp.data.metadata.num_blocks} scheduling blocks`
        
        setTimeout(() => {
          router.push('/sky-map')
        }, 1500)
      } catch (e: any) {
        errorMessage.value = e.response?.data?.error || e.message || 'Failed to load sample'
      } finally {
        loading.value = false
      }
    }

    return { 
      loading, 
      uploadProgress, 
      progressMessage,
      successMessage,
      errorMessage,
      handleUpload, 
      loadSample 
    }
  }
})
</script>
