<template>
  <div :class="cardClasses">
    <div v-if="$slots.header || title" class="tsi-card__header">
      <slot name="header">
        <h3 class="tsi-card__title">{{ title }}</h3>
      </slot>
    </div>
    
    <div class="tsi-card__body">
      <slot />
    </div>
    
    <div v-if="$slots.footer" class="tsi-card__footer">
      <slot name="footer" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  title?: string
  padding?: 'none' | 'sm' | 'md' | 'lg'
  shadow?: boolean
  bordered?: boolean
  hoverable?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  padding: 'md',
  shadow: true,
  bordered: false,
  hoverable: false
})

const cardClasses = computed(() => [
  'tsi-card',
  `tsi-card--padding-${props.padding}`,
  {
    'tsi-card--shadow': props.shadow,
    'tsi-card--bordered': props.bordered,
    'tsi-card--hoverable': props.hoverable
  }
])
</script>

<style scoped>
.tsi-card {
  background: white;
  border-radius: 12px;
  transition: all 0.3s ease;
}

.tsi-card--shadow {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
}

.tsi-card--bordered {
  border: 1px solid #e5e7eb;
}

.tsi-card--hoverable:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 20px rgba(0, 0, 0, 0.1);
}

.tsi-card--padding-none .tsi-card__body {
  padding: 0;
}

.tsi-card--padding-sm .tsi-card__body {
  padding: 12px;
}

.tsi-card--padding-md .tsi-card__body {
  padding: 24px;
}

.tsi-card--padding-lg .tsi-card__body {
  padding: 32px;
}

.tsi-card__header {
  padding: 20px 24px;
  border-bottom: 1px solid #e5e7eb;
}

.tsi-card__title {
  font-size: 1.125rem;
  font-weight: 600;
  color: #1f2937;
  margin: 0;
}

.tsi-card__body {
  /* Padding controlled by card modifier classes */
}

.tsi-card__footer {
  padding: 16px 24px;
  border-top: 1px solid #e5e7eb;
  background: #f9fafb;
  border-radius: 0 0 12px 12px;
}
</style>
