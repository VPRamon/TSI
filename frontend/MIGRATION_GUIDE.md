# Migration Guide: Refactoring Existing Components

This guide walks through how to refactor existing TSI components to the new architecture.

## Overview

The new architecture uses:
- Feature-based modules
- Shared UI components
- Centralized services
- Composition API
- Design tokens

## Step-by-Step Migration

### Example: Migrating a Page Component

**Before** (`src/pages/SkyMap.vue`):
```vue
<script lang="ts">
import { defineComponent } from 'vue'
import axios from 'axios'

export default defineComponent({
  setup() {
    // Mixed concerns, inline API calls
    async function loadData() {
      const resp = await axios.get('http://localhost:8081/api/v1/datasets/current')
      return resp.data
    }
    return { loadData }
  }
})
</script>
```

**After** (`src/features/sky-map/SkyMapPage.vue`):
```vue
<script setup lang="ts">
import { onMounted } from 'vue'
import { TsiCard, TsiSpinner, TsiAlert } from '@/shared/components'
import { useAsync } from '@/shared/composables'
import { apiService } from '@/shared/services/api'

const { data, loading, error, execute } = useAsync(
  () => apiService.getCurrentDataset()
)

onMounted(() => execute())
</script>
```

### Checklist for Each Component

- [ ] Move to appropriate feature folder
- [ ] Convert to `<script setup>` syntax
- [ ] Extract API calls to `apiService`
- [ ] Extract complex logic to composables
- [ ] Replace inline styles with design tokens
- [ ] Use shared UI components
- [ ] Add TypeScript types
- [ ] Update imports to use path aliases

---

## Common Patterns

### 1. API Calls

**Before**:
```ts
const response = await axios.get(`http://localhost:8081/api/v1/datasets/current`)
```

**After**:
```ts
import { apiService } from '@/shared/services/api'
const data = await apiService.getCurrentDataset()
```

### 2. Loading States

**Before**:
```ts
const loading = ref(false)
const error = ref('')

async function fetch() {
  loading.value = true
  try {
    // ...
  } catch (e) {
    error.value = e.message
  } finally {
    loading.value = false
  }
}
```

**After**:
```ts
import { useAsync } from '@/shared/composables'

const { data, loading, error, execute } = useAsync(myAsyncFunction)
```

### 3. Buttons and UI Elements

**Before**:
```vue
<button class="px-4 py-2 bg-blue-600 text-white rounded" @click="onClick">
  Click Me
</button>
```

**After**:
```vue
<TsiButton variant="primary" @click="onClick">
  Click Me
</TsiButton>
```

### 4. Styling

**Before**:
```vue
<style scoped>
.card {
  background: white;
  padding: 24px;
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
}
</style>
```

**After**:
```vue
<TsiCard padding="lg">
  <!-- content -->
</TsiCard>
```

Or with design tokens:
```vue
<style scoped>
.custom-card {
  background: white;
  padding: var(--spacing-6);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-base);
}
</style>
```

---

## Feature Module Template

When creating a new feature, use this template:

```
features/
└── my-feature/
    ├── components/
    │   └── MyFeatureWidget.vue
    ├── composables/
    │   └── useMyFeature.ts
    ├── types/
    │   └── index.ts
    ├── styles/
    │   └── my-feature.css
    └── MyFeaturePage.vue
```

**MyFeaturePage.vue**:
```vue
<template>
  <div class="my-feature-page">
    <TsiCard>
      <h1>{{ title }}</h1>
      <TsiSpinner v-if="loading" />
      <TsiAlert v-else-if="error" variant="error" :message="error" />
      <MyFeatureWidget v-else :data="data" />
    </TsiCard>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { TsiCard, TsiSpinner, TsiAlert } from '@/shared/components'
import MyFeatureWidget from './components/MyFeatureWidget.vue'
import { useMyFeature } from './composables/useMyFeature'

const title = 'My Feature'
const { data, loading, error, loadData } = useMyFeature()

onMounted(() => loadData())
</script>

<style scoped src="./styles/my-feature.css"></style>
```

---

## TypeScript Integration

### Defining Types

Create `types/index.ts` in your feature:

```ts
export interface MyFeatureData {
  id: string
  name: string
  value: number
}

export interface MyFeatureFilters {
  minValue: number
  maxValue: number
}
```

### Using Types in Components

```vue
<script setup lang="ts">
import type { MyFeatureData } from '../types'

interface Props {
  data: MyFeatureData[]
  title?: string
}

const props = withDefaults(defineProps<Props>(), {
  title: 'Default Title'
})

const emit = defineEmits<{
  select: [item: MyFeatureData]
}>()
</script>
```

---

## Router Integration

Update `src/router.ts` to use lazy loading:

```ts
{
  path: '/my-feature',
  name: 'MyFeature',
  component: () => import('@/features/my-feature/MyFeaturePage.vue')
}
```

---

## Composable Pattern

Create reusable logic in `composables/`:

```ts
// features/my-feature/composables/useMyFeature.ts
import { ref } from 'vue'
import { apiService } from '@/shared/services/api'
import type { MyFeatureData } from '../types'

export function useMyFeature() {
  const data = ref<MyFeatureData[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function loadData() {
    loading.value = true
    error.value = null
    
    try {
      const response = await apiService.getMyFeatureData()
      data.value = response
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to load data'
    } finally {
      loading.value = false
    }
  }

  return {
    data,
    loading,
    error,
    loadData
  }
}
```

---

## Testing Considerations

When writing tests for migrated components:

1. **Test composables independently**:
```ts
import { useMyFeature } from './useMyFeature'

describe('useMyFeature', () => {
  it('loads data successfully', async () => {
    const { data, loadData } = useMyFeature()
    await loadData()
    expect(data.value).toBeDefined()
  })
})
```

2. **Test components with mocked services**:
```ts
vi.mock('@/shared/services/api')
```

---

## Gradual Migration Strategy

1. **Phase 1**: Set up shared infrastructure
   - ✅ Create folder structure
   - ✅ Build shared components
   - ✅ Set up API service
   - ✅ Create composables

2. **Phase 2**: Migrate one feature at a time
   - Start with simplest feature (e.g., upload)
   - Refactor components
   - Update router
   - Test thoroughly

3. **Phase 3**: Refactor remaining features
   - Sky map
   - Distributions
   - Insights
   - Timeline, Trends, Compare, Visibility

4. **Phase 4**: Cleanup
   - Remove old files
   - Update documentation
   - Run full test suite

---

## Common Issues & Solutions

### Issue: Import Errors

**Problem**: `Cannot find module '@/shared/components'`

**Solution**: Ensure path aliases are configured in `tsconfig.json` and `vite.config.ts`

### Issue: CSS Not Applied

**Problem**: Design tokens not working

**Solution**: Import global styles in `main.ts`:
```ts
import '@/shared/styles/main.css'
```

### Issue: Type Errors

**Problem**: TypeScript complaining about types

**Solution**: 
- Add proper type definitions
- Use `as` for type assertions sparingly
- Enable strict mode in tsconfig

---

## Resources

- [Vue 3 Composition API](https://vuejs.org/guide/extras/composition-api-faq.html)
- [TypeScript with Vue](https://vuejs.org/guide/typescript/overview.html)
- [Vue Router](https://router.vuejs.org/)
- [Pinia (State Management)](https://pinia.vuejs.org/)

---

## Need Help?

Refer to completed examples in the codebase:
- `src/features/upload/` - Complete example with upload logic
- `src/shared/components/` - UI component examples
- `src/shared/composables/` - Composable examples
