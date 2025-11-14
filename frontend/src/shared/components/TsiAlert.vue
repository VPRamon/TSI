<template>
  <div :class="alertClasses" role="alert">
    <div class="tsi-alert__icon">
      <span v-if="variant === 'success'">✓</span>
      <span v-else-if="variant === 'error'">✕</span>
      <span v-else-if="variant === 'warning'">⚠</span>
      <span v-else-if="variant === 'info'">ℹ</span>
    </div>
    
    <div class="tsi-alert__content">
      <h4 v-if="title" class="tsi-alert__title">{{ title }}</h4>
      <p class="tsi-alert__message">
        <slot>{{ message }}</slot>
      </p>
    </div>
    
    <button
      v-if="dismissible"
      class="tsi-alert__close"
      @click="$emit('dismiss')"
      aria-label="Close alert"
    >
      ×
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  variant?: 'success' | 'error' | 'warning' | 'info'
  title?: string
  message?: string
  dismissible?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'info',
  dismissible: false
})

defineEmits<{
  dismiss: []
}>()

const alertClasses = computed(() => [
  'tsi-alert',
  `tsi-alert--${props.variant}`
])
</script>

<style scoped>
.tsi-alert {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 16px;
  border-radius: 8px;
  border: 1px solid transparent;
}

.tsi-alert__icon {
  font-size: 20px;
  font-weight: bold;
  flex-shrink: 0;
}

.tsi-alert__content {
  flex: 1;
}

.tsi-alert__title {
  font-size: 1rem;
  font-weight: 600;
  margin: 0 0 4px 0;
}

.tsi-alert__message {
  font-size: 0.875rem;
  margin: 0;
  line-height: 1.5;
}

.tsi-alert__close {
  background: none;
  border: none;
  font-size: 24px;
  line-height: 1;
  cursor: pointer;
  padding: 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: background 0.2s ease;
}

.tsi-alert__close:hover {
  background: rgba(0, 0, 0, 0.05);
}

/* Variants */
.tsi-alert--success {
  background: #d1fae5;
  border-color: #6ee7b7;
  color: #047857;
}

.tsi-alert--error {
  background: #fee2e2;
  border-color: #fca5a5;
  color: #dc2626;
}

.tsi-alert--warning {
  background: #fef3c7;
  border-color: #fcd34d;
  color: #b45309;
}

.tsi-alert--info {
  background: #dbeafe;
  border-color: #93c5fd;
  color: #1e40af;
}
</style>
