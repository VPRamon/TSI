<template>
  <div class="landing-page">
    <!-- Hero Section -->
    <section class="hero-section">
      <div class="hero-content">
        <div class="hero-badge">🔭 Powered by Rust + Vue 3</div>
        <h1 class="hero-title">
          Telescope Scheduling
          <span class="gradient-text">Intelligence</span>
        </h1>
        <p class="hero-subtitle">
          Analyze and visualize astronomical scheduling outputs with high-performance analytics,
          interactive sky maps, and comprehensive insights.
        </p>
        
        <!-- Quick Stats -->
        <div class="stats-grid">
          <div class="stat-card">
            <div class="stat-icon">⚡</div>
            <div class="stat-value">Fast</div>
            <div class="stat-label">Rust Backend</div>
          </div>
          <div class="stat-card">
            <div class="stat-icon">📊</div>
            <div class="stat-value">8+</div>
            <div class="stat-label">Visualization Pages</div>
          </div>
          <div class="stat-card">
            <div class="stat-icon">🎯</div>
            <div class="stat-value">Real-time</div>
            <div class="stat-label">Analytics</div>
          </div>
        </div>
      </div>
    </section>

    <!-- Main Content -->
    <section class="content-section">
      <div class="container">
        <!-- Features Grid -->
        <div class="features-grid">
          <div class="feature-card">
            <div class="feature-icon">🗺️</div>
            <h3 class="feature-title">Sky Map</h3>
            <p class="feature-description">
              Interactive RA/Dec scatter plots with priority coloring and time filtering
            </p>
          </div>
          <div class="feature-card">
            <div class="feature-icon">📈</div>
            <h3 class="feature-title">Trends & Insights</h3>
            <p class="feature-description">
              Time evolution analysis, correlations, and scheduling rate metrics
            </p>
          </div>
          <div class="feature-card">
            <div class="feature-icon">⏱️</div>
            <h3 class="feature-title">Timeline View</h3>
            <p class="feature-description">
              Month-by-month scheduling with dark period overlays and CSV export
            </p>
          </div>
          <div class="feature-card">
            <div class="feature-icon">🔍</div>
            <h3 class="feature-title">Compare Schedules</h3>
            <p class="feature-description">
              Side-by-side comparison of multiple scheduling runs
            </p>
          </div>
        </div>

        <!-- Upload Section -->
        <div class="upload-section">
          <h2 class="section-title">Get Started</h2>
          <p class="section-subtitle">Upload your scheduling data or try our sample dataset</p>

          <div class="upload-grid">
            <div class="upload-card primary-upload-card">
              <div class="upload-icon">📤</div>
              <h3 class="upload-title">Upload Your Data</h3>
              <p class="upload-description">
                Supports preprocessed CSV files or raw JSON bundles with schedule and visibility data.
              </p>
              <ul class="upload-details">
                <li><span>CSV:</span> Upload a single preprocessed schedule file</li>
                <li><span>JSON:</span> Include <code>schedule.json</code> and optional <code>possible_periods.json</code></li>
              </ul>
              <FileUpload 
                accept=".csv,.json" 
                :multiple="true"
                @upload="handleUpload"
              />
            </div>

            <!-- Sample Data -->
            <div class="upload-card sample-card">
              <div class="upload-icon">✨</div>
              <h3 class="upload-title">Try Sample Data</h3>
              <p class="upload-description">
                Explore with 2,647 scheduling blocks
              </p>
              <button 
                @click="loadSample"
                :disabled="loading"
                class="sample-button"
              >
                <span v-if="loading" class="loading-spinner"></span>
                {{ loading ? 'Loading...' : 'Load Sample Dataset' }}
              </button>
            </div>
          </div>

          <!-- Progress Bar -->
          <transition name="fade">
            <div v-if="uploadProgress > 0 && uploadProgress < 100" class="progress-container">
              <div class="progress-bar">
                <div 
                  class="progress-fill"
                  :style="{ width: uploadProgress + '%' }"
                ></div>
              </div>
              <p class="progress-text">{{ progressMessage }}</p>
            </div>
          </transition>

          <!-- Messages -->
          <transition name="fade">
            <div v-if="successMessage" class="message success-message">
              <span class="message-icon">✓</span>
              <p>{{ successMessage }}</p>
            </div>
          </transition>

          <transition name="fade">
            <div v-if="errorMessage" class="message error-message">
              <span class="message-icon">✕</span>
              <p>{{ errorMessage }}</p>
            </div>
          </transition>
        </div>
      </div>
    </section>
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

    async function handleUpload(files: File[]) {
      if (!files || files.length === 0) return

      loading.value = true
      uploadProgress.value = 10
      progressMessage.value = 'Uploading files...'
      errorMessage.value = ''
      successMessage.value = ''

      try {
        const formData = new FormData()
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

          formData.append('file', files[0])
          uploadProgress.value = 30
          progressMessage.value = 'Parsing CSV...'
          
          const resp = await axios.post(`${API_BASE}/datasets/upload/csv`, formData, {
            headers: { 'Content-Type': 'multipart/form-data' }
          })
          
          uploadProgress.value = 100
          successMessage.value = `Loaded ${resp.data.metadata.num_blocks} scheduling blocks`
          
        } else if (hasJson) {
          // For JSON, expect schedule.json and optional possible_periods.json
          const scheduleFile = files.find(f => f.name.toLowerCase().includes('schedule'))
          const visibilityFile = files.find(
            f => f.name.toLowerCase().includes('possible_periods') || f.name.toLowerCase().includes('visibility')
          )
          
          if (!scheduleFile) {
            throw new Error('Please include schedule.json when uploading JSON data.')
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

<style scoped>
.landing-page {
  min-height: 100vh;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.hero-section {
  padding: 80px 20px;
  text-align: center;
  color: white;
}

.hero-content {
  max-width: 1000px;
  margin: 0 auto;
}

.hero-badge {
  display: inline-block;
  padding: 8px 20px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 20px;
  font-size: 14px;
  font-weight: 500;
  margin-bottom: 20px;
  backdrop-filter: blur(10px);
}

.hero-title {
  font-size: 56px;
  font-weight: 800;
  margin-bottom: 20px;
  line-height: 1.2;
}

.gradient-text {
  background: linear-gradient(135deg, #ffd89b 0%, #19547b 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.hero-subtitle {
  font-size: 20px;
  line-height: 1.6;
  opacity: 0.95;
  margin-bottom: 40px;
  max-width: 700px;
  margin-left: auto;
  margin-right: auto;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 20px;
  max-width: 700px;
  margin: 0 auto;
}

.stat-card {
  background: rgba(255, 255, 255, 0.15);
  backdrop-filter: blur(10px);
  border-radius: 15px;
  padding: 20px;
  text-align: center;
}

.stat-icon {
  font-size: 32px;
  margin-bottom: 10px;
}

.stat-value {
  font-size: 24px;
  font-weight: 700;
  margin-bottom: 5px;
}

.stat-label {
  font-size: 14px;
  opacity: 0.9;
}

.content-section {
  background: #f9fafb;
  padding: 60px 20px;
  border-radius: 40px 40px 0 0;
  margin-top: -20px;
}

.container {
  max-width: 1200px;
  margin: 0 auto;
}

.features-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 20px;
  margin-bottom: 60px;
}

.feature-card {
  background: white;
  padding: 30px 20px;
  border-radius: 15px;
  text-align: center;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
  transition: all 0.3s ease;
}

.feature-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 8px 20px rgba(0, 0, 0, 0.1);
}

.feature-icon {
  font-size: 40px;
  margin-bottom: 15px;
  display: block;
}

.feature-title {
  font-size: 18px;
  font-weight: 600;
  color: #1f2937;
  margin-bottom: 10px;
}

.feature-description {
  font-size: 14px;
  color: #6b7280;
  line-height: 1.5;
}

.upload-section {
  max-width: 900px;
  margin: 0 auto;
}

.section-title {
  font-size: 36px;
  font-weight: 700;
  text-align: center;
  color: #1f2937;
  margin-bottom: 10px;
}

.section-subtitle {
  text-align: center;
  color: #6b7280;
  margin-bottom: 40px;
  font-size: 18px;
}

.upload-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 20px;
  margin-bottom: 30px;
}

.upload-card {
  background: white;
  padding: 30px;
  border-radius: 15px;
  border: 2px dashed #d1d5db;
  text-align: center;
  transition: all 0.3s ease;
}

.upload-card:hover {
  border-color: #667eea;
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.15);
}

.primary-upload-card {
  border-style: solid;
  border-color: rgba(102, 126, 234, 0.4);
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.06) 0%, rgba(118, 75, 162, 0.04) 100%);
}

.sample-card {
  border-style: solid;
  border-color: #667eea;
  background: linear-gradient(135deg, rgba(102, 126, 234, 0.05) 0%, rgba(118, 75, 162, 0.05) 100%);
}

.upload-icon {
  font-size: 48px;
  margin-bottom: 15px;
}

.upload-title {
  font-size: 20px;
  font-weight: 600;
  color: #1f2937;
  margin-bottom: 10px;
}

.upload-description {
  font-size: 14px;
  color: #6b7280;
  margin-bottom: 20px;
  line-height: 1.5;
}

.upload-details {
  list-style: none;
  padding: 0;
  margin: 0 auto 20px;
  text-align: left;
  max-width: 360px;
}

.upload-details li {
  font-size: 14px;
  color: #4b5563;
  margin-bottom: 8px;
  display: flex;
  gap: 6px;
  align-items: baseline;
}

.upload-details span {
  font-weight: 600;
  color: #1f2937;
}

.upload-details code {
  background: rgba(15, 23, 42, 0.05);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: 'Fira Code', Consolas, monospace;
  font-size: 13px;
}

.sample-button {
  padding: 12px 24px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  font-size: 16px;
  min-width: 180px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.sample-button:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(102, 126, 234, 0.4);
}

.sample-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.loading-spinner {
  display: inline-block;
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.progress-container {
  margin: 30px 0;
  padding: 20px;
  background: white;
  border-radius: 10px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
}

.progress-bar {
  height: 8px;
  background: #e5e7eb;
  border-radius: 10px;
  overflow: hidden;
  margin-bottom: 10px;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #667eea 0%, #764ba2 100%);
  transition: width 0.3s ease;
  border-radius: 10px;
}

.progress-text {
  text-align: center;
  color: #6b7280;
  font-size: 14px;
}

.message {
  padding: 16px 20px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  gap: 12px;
  margin: 20px 0;
}

.success-message {
  background: #d1fae5;
  border: 1px solid #6ee7b7;
  color: #047857;
}

.error-message {
  background: #fee2e2;
  border: 1px solid #fca5a5;
  color: #dc2626;
}

.message-icon {
  font-size: 20px;
  font-weight: bold;
}

.message p {
  flex: 1;
  margin: 0;
  font-weight: 500;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

@media (max-width: 1024px) {
  .features-grid {
    grid-template-columns: repeat(2, 1fr);
  }
  
  .upload-grid {
    grid-template-columns: 1fr;
  }
  
  .stats-grid {
    grid-template-columns: repeat(3, 1fr);
  }
}

@media (max-width: 768px) {
  .hero-title {
    font-size: 36px;
  }
  
  .hero-subtitle {
    font-size: 16px;
  }
  
  .features-grid {
    grid-template-columns: 1fr;
  }
  
  .stats-grid {
    grid-template-columns: 1fr;
  }
}
</style>
