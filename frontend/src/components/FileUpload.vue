<template>
  <div 
    class="border-2 border-dashed rounded-lg p-8 text-center transition-colors"
    :class="isDragging ? 'border-blue-500 bg-blue-50' : 'border-gray-300 bg-white'"
    @dragover.prevent="isDragging = true"
    @dragleave.prevent="isDragging = false"
    @drop.prevent="onDrop"
  >
    <input 
      ref="fileInput"
      type="file" 
      :accept="accept"
      :multiple="multiple"
      style="display: none;" 
      @change="onFileSelect"
    />
    
    <div class="space-y-4">
      <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" 
          d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
      </svg>
      
      <div>
        <button 
          type="button"
          @click="triggerFileInput"
          class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
        >
          Choose File{{ multiple ? 's' : '' }}
        </button>
        <p class="text-sm text-gray-500 mt-2">or drag and drop here</p>
      </div>
      
      <p v-if="selectedFiles.length > 0" class="text-sm text-gray-700">
        Selected: {{ selectedFiles.map(f => f.name).join(', ') }}
      </p>
    </div>

    <button 
      v-if="selectedFiles.length > 0"
      type="button"
      @click="upload"
      :disabled="uploading"
      class="mt-4 px-6 py-2 bg-green-600 text-white rounded hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed"
    >
      {{ uploading ? 'Uploading...' : 'Upload' }}
    </button>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref, PropType } from 'vue'

export default defineComponent({
  props: {
    accept: {
      type: String,
      default: '*'
    },
    uploadType: {
      type: String as PropType<'csv' | 'json'>,
      required: true
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

    function onFileSelect(event: Event) {
      const input = event.target as HTMLInputElement
      if (input.files) {
        selectedFiles.value = Array.from(input.files)
      }
    }

    function onDrop(event: DragEvent) {
      isDragging.value = false
      if (event.dataTransfer?.files) {
        selectedFiles.value = Array.from(event.dataTransfer.files)
      }
    }

    function upload() {
      if (selectedFiles.value.length === 0) return
      
      uploading.value = true
      emit('upload', {
        type: props.uploadType,
        files: selectedFiles.value
      })
      
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
