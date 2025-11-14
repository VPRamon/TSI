<template>
  <div class="tsi-input">
    <label v-if="label" :for="inputId" class="tsi-input__label">
      {{ label }}
      <span v-if="required" class="tsi-input__required">*</span>
    </label>
    
    <input
      :id="inputId"
      :type="type"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :readonly="readonly"
      :class="inputClasses"
      @input="handleInput"
      @blur="handleBlur"
      @focus="handleFocus"
    />
    
    <p v-if="error" class="tsi-input__error">{{ error }}</p>
    <p v-else-if="hint" class="tsi-input__hint">{{ hint }}</p>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'

interface Props {
  modelValue: string | number
  label?: string
  type?: 'text' | 'number' | 'email' | 'password' | 'tel' | 'url'
  placeholder?: string
  disabled?: boolean
  readonly?: boolean
  required?: boolean
  error?: string
  hint?: string
}

const props = withDefaults(defineProps<Props>(), {
  type: 'text'
})

const emit = defineEmits<{
  'update:modelValue': [value: string | number]
  blur: [event: FocusEvent]
  focus: [event: FocusEvent]
}>()

const inputId = ref(`tsi-input-${Math.random().toString(36).substr(2, 9)}`)
const isFocused = ref(false)

const inputClasses = computed(() => [
  'tsi-input__field',
  {
    'tsi-input__field--error': props.error,
    'tsi-input__field--disabled': props.disabled,
    'tsi-input__field--focused': isFocused.value
  }
])

function handleInput(event: Event) {
  const target = event.target as HTMLInputElement
  const value = props.type === 'number' ? parseFloat(target.value) : target.value
  emit('update:modelValue', value)
}

function handleBlur(event: FocusEvent) {
  isFocused.value = false
  emit('blur', event)
}

function handleFocus(event: FocusEvent) {
  isFocused.value = true
  emit('focus', event)
}
</script>

<style scoped>
.tsi-input {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.tsi-input__label {
  font-size: 0.875rem;
  font-weight: 500;
  color: #374151;
}

.tsi-input__required {
  color: #ef4444;
  margin-left: 2px;
}

.tsi-input__field {
  width: 100%;
  padding: 10px 12px;
  font-size: 1rem;
  font-family: inherit;
  color: #1f2937;
  background: white;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  transition: all 0.2s ease;
}

.tsi-input__field:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
}

.tsi-input__field--error {
  border-color: #ef4444;
}

.tsi-input__field--error:focus {
  border-color: #ef4444;
  box-shadow: 0 0 0 3px rgba(239, 68, 68, 0.1);
}

.tsi-input__field--disabled {
  background: #f3f4f6;
  color: #9ca3af;
  cursor: not-allowed;
}

.tsi-input__error {
  font-size: 0.875rem;
  color: #ef4444;
  margin: 0;
}

.tsi-input__hint {
  font-size: 0.875rem;
  color: #6b7280;
  margin: 0;
}
</style>
