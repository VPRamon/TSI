<template>
  <div class="upload-widget">
    <input 
      ref="fileInput"
      type="file" 
      :accept="accept"
      :multiple="multiple"
      class="hidden-input"
      @change="onFileSelect"
    />

    <div 
      class="upload-dropzone"
      :class="{ 'is-dragging': isDragging }"
      @dragover.prevent="isDragging = true"
      @dragleave.prevent="isDragging = false"
      @drop.prevent="onDrop"
    >
      <svg class="dropzone-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
          d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
      </svg>
      <p class="dropzone-title">Drag &amp; drop files here</p>
      <p class="dropzone-subtitle">
        or click below to browse {{ multiple ? 'files' : 'a file' }}
      </p>
      <p v-if="selectedFiles.length > 0" class="selected-files">
        {{ selectedFiles.map(f => f.name).join(', ') }}
      </p>
    </div>

    <div class="upload-actions">
      <button 
        type="button"
        class="ghost-button"
        @click="triggerFileInput"
      >
        Browse {{ multiple ? 'Files' : 'File' }}
      </button>
      <button 
        type="button"
        class="primary-button"
        @click="upload"
        :disabled="selectedFiles.length === 0 || uploading"
      >
        <span v-if="uploading" class="button-spinner"></span>
        {{ uploading ? 'Uploading...' : 'Upload' }}
      </button>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue'

export default defineComponent({
  props: {
    accept: {
      type: String,
      default: '*'
    },
    multiple: {
      type: Boolean,
      default: false
    }
  },
  emits: ['upload'],
  setup(props, { emit }) {
    const isDragging = ref(false)
    const selectedFiles = ref<File[]>([])
    const uploading = ref(false)
    const fileInput = ref<HTMLInputElement | null>(null)

    function triggerFileInput() {
      fileInput.value?.click()
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

    return {
      isDragging,
      selectedFiles,
      uploading,
      fileInput,
      triggerFileInput,
      onFileSelect,
      onDrop,
      upload
    }
  }
})
</script>

<style scoped>
.upload-widget {
  display: flex;
  flex-direction: column;
  gap: 18px;
  min-height: 250px;
}

.hidden-input {
  display: none;
}

.upload-dropzone {
  border: 1.5px dashed #cfd8e3;
  border-radius: 12px;
  padding: 24px 16px;
  text-align: center;
  background: #f9fafb;
  transition: border-color 0.2s ease, background-color 0.2s ease;
}

.upload-dropzone.is-dragging {
  border-color: #667eea;
  background: rgba(102, 126, 234, 0.08);
}

.dropzone-icon {
  width: 48px;
  height: 48px;
  color: #94a3b8;
  margin-bottom: 12px;
}

.dropzone-title {
  font-weight: 600;
  color: #1f2937;
  margin-bottom: 4px;
}

.dropzone-subtitle {
  color: #6b7280;
  font-size: 0.9rem;
}

.selected-files {
  margin-top: 12px;
  font-size: 0.9rem;
  color: #374151;
  word-break: break-word;
}

.upload-actions {
  display: flex;
  gap: 12px;
  margin-top: auto;
}

.ghost-button {
  flex: 1;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  background: white;
  padding: 12px;
  font-weight: 600;
  color: #374151;
  transition: border-color 0.2s ease, transform 0.2s ease;
}

.ghost-button:hover {
  border-color: #667eea;
  transform: translateY(-1px);
}

.primary-button {
  flex: 1;
  padding: 12px;
  border-radius: 8px;
  color: white;
  font-weight: 600;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  transition: transform 0.2s ease, box-shadow 0.2s ease, opacity 0.2s ease;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.primary-button:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(102, 126, 234, 0.25);
}

.primary-button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.button-spinner {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 2px solid rgba(255, 255, 255, 0.35);
  border-top-color: white;
  animation: spin 0.7s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
