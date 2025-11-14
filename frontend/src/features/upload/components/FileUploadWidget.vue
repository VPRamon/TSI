<template>
  <div class="upload-widget">
    <input 
      ref="fileInputRef"
      type="file" 
      :accept="accept"
      :multiple="multiple"
      class="upload-widget__hidden-input"
      @change="onFileSelect"
    />

    <div 
      class="upload-widget__dropzone"
      :class="{ 'upload-widget__dropzone--dragging': isDragging }"
      @dragover.prevent="isDragging = true"
      @dragleave.prevent="isDragging = false"
      @drop.prevent="onDrop"
    >
      <svg class="upload-widget__icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
          d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
      </svg>
      <p class="upload-widget__title">Drag &amp; drop files here</p>
      <p class="upload-widget__subtitle">
        or click below to browse {{ multiple ? 'files' : 'a file' }}
      </p>
      <p v-if="selectedFiles.length > 0" class="upload-widget__selected">
        {{ selectedFiles.map(f => f.name).join(', ') }}
      </p>
    </div>

    <div class="upload-widget__actions">
      <TsiButton
        variant="ghost"
        :full-width="true"
        @click="triggerFileInput"
      >
        Browse {{ multiple ? 'Files' : 'File' }}
      </TsiButton>
      <TsiButton
        variant="primary"
        :full-width="true"
        :disabled="selectedFiles.length === 0 || uploading"
        :loading="uploading"
        loading-text="Uploading..."
        @click="upload"
      >
        Upload
      </TsiButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { TsiButton } from '@/shared/components'

interface Props {
  accept?: string
  multiple?: boolean
}

withDefaults(defineProps<Props>(), {
  accept: '*',
  multiple: false
})

const emit = defineEmits<{
  upload: [files: File[]]
}>()

const fileInputRef = ref<HTMLInputElement | null>(null)
const isDragging = ref(false)
const selectedFiles = ref<File[]>([])
const uploading = ref(false)

function triggerFileInput() {
  fileInputRef.value?.click()
}

function setFiles(files: FileList | null) {
  if (files) {
    selectedFiles.value = Array.from(files)
  }
}

function onFileSelect(event: Event) {
  const input = event.target as HTMLInputElement
  setFiles(input.files)
}

function onDrop(event: DragEvent) {
  isDragging.value = false
  if (event.dataTransfer?.files) {
    setFiles(event.dataTransfer.files)
  }
}

function upload() {
  if (selectedFiles.value.length === 0 || uploading.value) return
  
  uploading.value = true
  emit('upload', selectedFiles.value)
  
  // Reset after short delay
  setTimeout(() => {
    uploading.value = false
  }, 1000)
}
</script>

<style scoped>
.upload-widget {
  display: flex;
  flex-direction: column;
  gap: 18px;
  min-height: 250px;
}

.upload-widget__hidden-input {
  display: none;
}

.upload-widget__dropzone {
  border: 1.5px dashed #cfd8e3;
  border-radius: 12px;
  padding: 24px 16px;
  text-align: center;
  background: #f9fafb;
  transition: border-color 0.2s ease, background-color 0.2s ease;
}

.upload-widget__dropzone--dragging {
  border-color: #667eea;
  background: rgba(102, 126, 234, 0.08);
}

.upload-widget__icon {
  width: 48px;
  height: 48px;
  color: #94a3b8;
  margin: 0 auto 12px;
}

.upload-widget__title {
  font-weight: 600;
  color: #1f2937;
  margin-bottom: 4px;
}

.upload-widget__subtitle {
  color: #6b7280;
  font-size: 0.9rem;
}

.upload-widget__selected {
  margin-top: 12px;
  font-size: 0.9rem;
  color: #374151;
  word-break: break-word;
}

.upload-widget__actions {
  display: flex;
  gap: 12px;
  margin-top: auto;
}
</style>
