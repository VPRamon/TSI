/**
 * Composable for handling file uploads (CSV and JSON)
 */

import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { apiService } from '@/shared/services/api'

export interface UploadProgress {
  percent: number
  message: string
}

export function useFileUpload() {
  const router = useRouter()
  const loading = ref(false)
  const progress = ref<UploadProgress>({ percent: 0, message: '' })
  const successMessage = ref('')
  const errorMessage = ref('')

  async function uploadFiles(files: File[]) {
    if (!files || files.length === 0) return

    loading.value = true
    progress.value = { percent: 10, message: 'Uploading files...' }
    errorMessage.value = ''
    successMessage.value = ''

    try {
      const lowerNames = files.map(f => f.name.toLowerCase())
      const hasCsv = lowerNames.some(name => name.endsWith('.csv'))
      const hasJson = lowerNames.some(name => name.endsWith('.json'))

      if (!hasCsv && !hasJson) {
        throw new Error('Unsupported file type. Please upload CSV or JSON files.')
      }

      if (hasCsv && hasJson) {
        throw new Error('Please upload either CSV or JSON files, not both at the same time.')
      }

      if (hasCsv) {
        if (files.length > 1) {
          throw new Error('CSV upload accepts a single file.')
        }

        progress.value = { percent: 30, message: 'Parsing CSV...' }
        const result = await apiService.uploadCSV(files[0])
        progress.value = { percent: 100, message: 'Complete!' }
        successMessage.value = `Loaded ${result.metadata.num_blocks} scheduling blocks`

      } else if (hasJson) {
        const scheduleFile = files.find(f => f.name.toLowerCase().includes('schedule'))
        const visibilityFile = files.find(
          f => f.name.toLowerCase().includes('possible_periods') || 
             f.name.toLowerCase().includes('visibility')
        )

        if (!scheduleFile) {
          throw new Error('Please include schedule.json when uploading JSON data.')
        }

        progress.value = { percent: 30, message: 'Parsing JSON and preprocessing...' }
        const result = await apiService.uploadJSON(scheduleFile, visibilityFile)
        progress.value = { percent: 100, message: 'Complete!' }
        successMessage.value = `Loaded and preprocessed ${result.metadata.num_blocks} scheduling blocks`
      }

      // Redirect after success
      setTimeout(() => {
        router.push('/sky-map')
      }, 2000)

    } catch (error) {
      progress.value = { percent: 0, message: '' }
      errorMessage.value = error instanceof Error ? error.message : 'Upload failed'
    } finally {
      loading.value = false
    }
  }

  async function loadSample() {
    loading.value = true
    errorMessage.value = ''
    successMessage.value = ''

    try {
      const result = await apiService.loadSampleDataset()
      successMessage.value = `Loaded ${result.metadata.num_blocks} scheduling blocks`

      setTimeout(() => {
        router.push('/sky-map')
      }, 1500)
    } catch (error) {
      errorMessage.value = error instanceof Error ? error.message : 'Failed to load sample'
    } finally {
      loading.value = false
    }
  }

  return {
    loading,
    progress,
    successMessage,
    errorMessage,
    uploadFiles,
    loadSample
  }
}
