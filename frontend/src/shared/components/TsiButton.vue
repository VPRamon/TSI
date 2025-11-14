<template>
  <button
    :type="type"
    :class="buttonClasses"
    :disabled="disabled || loading"
    @click="handleClick"
  >
    <span v-if="loading" class="tsi-button__spinner"></span>
    <slot v-if="!loading" />
    <span v-if="loading">{{ loadingText }}</span>
  </button>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger'
  size?: 'sm' | 'md' | 'lg'
  type?: 'button' | 'submit' | 'reset'
  disabled?: boolean
  loading?: boolean
  loadingText?: string
  fullWidth?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'primary',
  size: 'md',
  type: 'button',
  disabled: false,
  loading: false,
  loadingText: 'Loading...',
  fullWidth: false
})

const emit = defineEmits<{
  click: [event: MouseEvent]
}>()

const buttonClasses = computed(() => [
  'tsi-button',
  `tsi-button--${props.variant}`,
  `tsi-button--${props.size}`,
  {
    'tsi-button--full-width': props.fullWidth,
    'tsi-button--disabled': props.disabled || props.loading
  }
])

function handleClick(event: MouseEvent) {
  if (!props.disabled && !props.loading) {
    emit('click', event)
  }
}
</script>

<style scoped>
.tsi-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-weight: 600;
  border-radius: 8px;
  border: none;
  cursor: pointer;
  transition: all 0.2s ease;
  font-family: inherit;
}

.tsi-button:focus {
  outline: 2px solid transparent;
  outline-offset: 2px;
}

.tsi-button--disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.tsi-button--full-width {
  width: 100%;
}

/* Sizes */
.tsi-button--sm {
  padding: 6px 12px;
  font-size: 0.875rem;
}

.tsi-button--md {
  padding: 10px 16px;
  font-size: 1rem;
}

.tsi-button--lg {
  padding: 12px 24px;
  font-size: 1.125rem;
}

/* Variants */
.tsi-button--primary {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.tsi-button--primary:hover:not(.tsi-button--disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(102, 126, 234, 0.3);
}

.tsi-button--secondary {
  background: #3b82f6;
  color: white;
}

.tsi-button--secondary:hover:not(.tsi-button--disabled) {
  background: #2563eb;
}

.tsi-button--ghost {
  background: white;
  color: #374151;
  border: 1px solid #d1d5db;
}

.tsi-button--ghost:hover:not(.tsi-button--disabled) {
  border-color: #667eea;
  transform: translateY(-1px);
}

.tsi-button--danger {
  background: #ef4444;
  color: white;
}

.tsi-button--danger:hover:not(.tsi-button--disabled) {
  background: #dc2626;
}

.tsi-button__spinner {
  display: inline-block;
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
