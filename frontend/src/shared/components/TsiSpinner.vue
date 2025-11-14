<template>
  <div class="tsi-spinner" :class="`tsi-spinner--${size}`" role="status" aria-label="Loading">
    <div class="tsi-spinner__circle"></div>
    <p v-if="message" class="tsi-spinner__message">{{ message }}</p>
  </div>
</template>

<script setup lang="ts">
interface Props {
  size?: 'sm' | 'md' | 'lg'
  message?: string
}

withDefaults(defineProps<Props>(), {
  size: 'md'
})
</script>

<style scoped>
.tsi-spinner {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
}

.tsi-spinner__circle {
  border-radius: 50%;
  border-style: solid;
  border-color: rgba(59, 130, 246, 0.2);
  border-top-color: #3b82f6;
  animation: spin 0.8s linear infinite;
}

.tsi-spinner--sm .tsi-spinner__circle {
  width: 20px;
  height: 20px;
  border-width: 2px;
}

.tsi-spinner--md .tsi-spinner__circle {
  width: 40px;
  height: 40px;
  border-width: 3px;
}

.tsi-spinner--lg .tsi-spinner__circle {
  width: 60px;
  height: 60px;
  border-width: 4px;
}

.tsi-spinner__message {
  font-size: 0.875rem;
  color: #6b7280;
  margin: 0;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
