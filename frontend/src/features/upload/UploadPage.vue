<template>
  <div class="landing-page">
    <!-- Hero Section -->
    <section class="hero">
      <div class="hero__content">
        <div class="hero__badge">🔭 Powered by Rust + Vue 3</div>
        <h1 class="hero__title">
          Telescope Scheduling
          <span class="hero__gradient">Intelligence</span>
        </h1>
        <p class="hero__subtitle">
          Analyze and visualize astronomical scheduling outputs with high-performance analytics,
          interactive sky maps, and comprehensive insights.
        </p>
        
        <!-- Quick Stats -->
        <div class="stats">
          <div class="stat-card">
            <div class="stat-card__icon">⚡</div>
            <div class="stat-card__value">Fast</div>
            <div class="stat-card__label">Rust Backend</div>
          </div>
          <div class="stat-card">
            <div class="stat-card__icon">📊</div>
            <div class="stat-card__value">8+</div>
            <div class="stat-card__label">Visualization Pages</div>
          </div>
          <div class="stat-card">
            <div class="stat-card__icon">🎯</div>
            <div class="stat-card__value">Real-time</div>
            <div class="stat-card__label">Analytics</div>
          </div>
        </div>
      </div>
    </section>

    <!-- Main Content -->
    <section class="content">
      <div class="container">
        <!-- Features Grid -->
        <div class="features">
          <TsiCard v-for="feature in features" :key="feature.title" hoverable padding="lg">
            <div class="feature">
              <div class="feature__icon">{{ feature.icon }}</div>
              <h3 class="feature__title">{{ feature.title }}</h3>
              <p class="feature__description">{{ feature.description }}</p>
            </div>
          </TsiCard>
        </div>

        <!-- Upload Section -->
        <div class="upload-section">
          <h2 class="section-title">Get Started</h2>
          <p class="section-subtitle">Upload your scheduling data or try our sample dataset</p>

          <div class="upload-grid">
            <!-- File Upload Card -->
            <TsiCard class="upload-card upload-card--primary" padding="lg">
              <div class="upload-card__icon">📤</div>
              <h3 class="upload-card__title">Upload Your Data</h3>
              <p class="upload-card__description">
                Supports preprocessed CSV files or raw JSON bundles with schedule and visibility data.
              </p>
              <ul class="upload-card__details">
                <li><span>CSV:</span> Upload a single preprocessed schedule file</li>
                <li><span>JSON:</span> Include <code>schedule.json</code> and optional <code>possible_periods.json</code></li>
              </ul>
              <FileUploadWidget 
                accept=".csv,.json" 
                :multiple="true"
                @upload="handleUpload"
              />
            </TsiCard>

            <!-- Sample Data Card -->
            <TsiCard class="upload-card upload-card--sample" padding="lg">
              <div class="upload-card__icon">✨</div>
              <h3 class="upload-card__title">Try Sample Data</h3>
              <p class="upload-card__description">
                Explore with 2,647 scheduling blocks
              </p>
              <TsiButton
                variant="primary"
                size="lg"
                :full-width="true"
                :loading="loading"
                @click="loadSample"
              >
                Load Sample Dataset
              </TsiButton>
            </TsiCard>
          </div>

          <!-- Progress Bar -->
          <transition name="fade">
            <TsiCard v-if="progress.percent > 0 && progress.percent < 100" padding="md">
              <div class="progress">
                <div class="progress__bar">
                  <div class="progress__fill" :style="{ width: progress.percent + '%' }"></div>
                </div>
                <p class="progress__text">{{ progress.message }}</p>
              </div>
            </TsiCard>
          </transition>

          <!-- Messages -->
          <transition name="fade">
            <TsiAlert
              v-if="successMessage"
              variant="success"
              :message="successMessage"
              dismissible
              @dismiss="successMessage = ''"
            />
          </transition>

          <transition name="fade">
            <TsiAlert
              v-if="errorMessage"
              variant="error"
              :message="errorMessage"
              dismissible
              @dismiss="errorMessage = ''"
            />
          </transition>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { TsiCard, TsiButton, TsiAlert } from '@/shared/components'
import FileUploadWidget from '../components/FileUploadWidget.vue'
import { useFileUpload } from '../composables/useFileUpload'

const features = [
  {
    icon: '🗺️',
    title: 'Sky Map',
    description: 'Interactive RA/Dec scatter plots with priority coloring and time filtering'
  },
  {
    icon: '📈',
    title: 'Trends & Insights',
    description: 'Time evolution analysis, correlations, and scheduling rate metrics'
  },
  {
    icon: '⏱️',
    title: 'Timeline View',
    description: 'Month-by-month scheduling with dark period overlays and CSV export'
  },
  {
    icon: '🔍',
    title: 'Compare Schedules',
    description: 'Side-by-side comparison of multiple scheduling runs'
  }
]

const {
  loading,
  progress,
  successMessage,
  errorMessage,
  uploadFiles,
  loadSample
} = useFileUpload()

function handleUpload(files: File[]) {
  uploadFiles(files)
}
</script>

<style scoped src="../styles/landing-page.css"></style>
