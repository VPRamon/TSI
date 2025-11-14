# Frontend Architecture Documentation

## Table of Contents
1. [Overview](#overview)
2. [Directory Structure](#directory-structure)
3. [Architecture Principles](#architecture-principles)
4. [Module Organization](#module-organization)
5. [Naming Conventions](#naming-conventions)
6. [Best Practices](#best-practices)
7. [Development Workflow](#development-workflow)

---

## Overview

The TSI frontend is built with **Vue 3**, **TypeScript**, and **Vite**, following a **feature-based modular architecture**. This design promotes:

- **Scalability**: Features are self-contained and can grow independently
- **Maintainability**: Clear boundaries reduce cognitive load
- **Reusability**: Shared components and utilities are centralized
- **Type Safety**: TypeScript ensures compile-time correctness
- **Modern Patterns**: Composition API, scoped styles, and lazy loading

---

## Directory Structure

```
src/
├── features/              # Feature modules (domain-specific)
│   ├── upload/
│   │   ├── components/    # Feature-specific components
│   │   ├── composables/   # Feature-specific composition functions
│   │   ├── services/      # Feature-specific business logic
│   │   ├── types/         # Feature-specific TypeScript types
│   │   ├── assets/        # Feature-specific assets (images, icons)
│   │   ├── styles/        # Feature-specific CSS
│   │   └── UploadPage.vue # Feature entry point (page component)
│   │
│   ├── sky-map/
│   ├── distributions/
│   ├── insights/
│   ├── timeline/
│   ├── trends/
│   ├── compare/
│   └── visibility/
│
├── shared/                # Shared/reusable code
│   ├── components/        # UI primitives (TsiButton, TsiCard, etc.)
│   ├── composables/       # Reusable composition functions
│   ├── services/          # API client, external integrations
│   ├── stores/            # Pinia stores (global state)
│   ├── types/             # Shared TypeScript interfaces
│   ├── utils/             # Utility functions (formatters, helpers)
│   ├── assets/            # Global assets (logos, fonts)
│   └── styles/            # Global styles (tokens, base, utilities)
│
├── App.vue                # Root application component
├── main.ts                # Application entry point
└── router.ts              # Vue Router configuration
```

### Key Principles:

1. **Features are autonomous**: Each feature folder contains everything needed for that feature
2. **Shared code is centralized**: Reusable components, services, and utilities live in `shared/`
3. **Page components are entry points**: Each feature has a main page component (e.g., `UploadPage.vue`)

---

## Architecture Principles

### 1. Feature-Based Modules

Each feature represents a distinct user-facing capability (e.g., file upload, sky map visualization). Features are:

- **Self-contained**: All code for a feature lives in its folder
- **Loosely coupled**: Features communicate via shared services or stores
- **Independently testable**: Each feature can be tested in isolation

### 2. Composition API

All components use Vue 3's **Composition API** (`<script setup>`):

```vue
<script setup lang="ts">
import { ref, computed } from 'vue'

const count = ref(0)
const doubled = computed(() => count.value * 2)
</script>
```

**Benefits**:
- Better TypeScript support
- More explicit reactivity
- Easier code reuse with composables

### 3. Scoped Styles

Components use **scoped CSS** or external stylesheets:

```vue
<style scoped>
.my-component {
  color: red;
}
</style>
```

Or import feature-specific styles:

```vue
<style scoped src="../styles/my-feature.css"></style>
```

### 4. Type Safety

All code is written in **TypeScript**:
- Props are typed with interfaces
- API responses are typed
- Composables return typed values

---

## Module Organization

### Feature Module Structure

Each feature follows this structure:

```
features/
└── feature-name/
    ├── components/         # Feature-specific components
    │   ├── FeatureWidget.vue
    │   └── FeatureItem.vue
    ├── composables/        # Feature-specific logic hooks
    │   └── useFeatureData.ts
    ├── services/           # Feature-specific API calls
    │   └── featureApi.ts
    ├── types/              # Feature-specific types
    │   └── index.ts
    ├── assets/             # Feature-specific images/icons
    ├── styles/             # Feature-specific CSS
    │   └── feature-name.css
    └── FeaturePage.vue     # Main page component
```

### Shared Module Structure

```
shared/
├── components/          # Reusable UI components
│   ├── TsiButton.vue
│   ├── TsiCard.vue
│   ├── TsiInput.vue
│   └── index.ts         # Centralized exports
├── composables/         # Reusable composition functions
│   ├── useAsync.ts
│   ├── usePolling.ts
│   └── index.ts
├── services/            # API clients, external services
│   └── api.ts
├── types/               # Shared TypeScript types
│   └── index.ts
├── utils/               # Utility functions
│   ├── constants.ts
│   └── formatters.ts
└── styles/              # Global styles
    ├── tokens.css       # Design tokens (colors, spacing)
    ├── base.css         # Base styles and resets
    ├── utilities.css    # Utility classes
    └── main.css         # Main import file
```

---

## Naming Conventions

### Files and Folders

| Type | Convention | Example |
|------|-----------|---------|
| Feature folders | `kebab-case` | `sky-map/`, `file-upload/` |
| Page components | `PascalCase` + `Page.vue` | `UploadPage.vue`, `SkyMapPage.vue` |
| Shared components | `PascalCase` + prefix | `TsiButton.vue`, `TsiCard.vue` |
| Feature components | `PascalCase` | `FileUploadWidget.vue` |
| Composables | `camelCase` + `use` prefix | `useAsync.ts`, `useFileUpload.ts` |
| Services | `camelCase` + `Service` | `apiService`, `authService` |
| Types | `PascalCase` | `SchedulingBlock`, `ApiError` |
| Utilities | `camelCase` | `formatNumber`, `downloadBlob` |
| CSS files | `kebab-case.css` | `landing-page.css`, `tokens.css` |

### Code Conventions

**Components**:
```vue
<script setup lang="ts">
// Use PascalCase for component names in imports
import { TsiButton } from '@/shared/components'
</script>
```

**Composables**:
```ts
// Always prefix with "use"
export function useFileUpload() {
  // ...
}
```

**Types**:
```ts
// Use interfaces for objects, type aliases for unions
export interface SchedulingBlock {
  id: string
  priority: number
}

export type LoadingState = 'idle' | 'loading' | 'success' | 'error'
```

**Constants**:
```ts
// Use UPPER_SNAKE_CASE for true constants
export const API_BASE_URL = 'http://localhost:8081'

// Use camelCase for config objects
export const chartDefaults = {
  height: 600
}
```

---

## Best Practices

### Component Design

✅ **Do**:
- Keep components focused and single-responsibility
- Use composition API with `<script setup>`
- Type all props and emits
- Use scoped styles
- Extract complex logic to composables

❌ **Don't**:
- Mix business logic with presentation
- Use Options API
- Create god components with too many responsibilities
- Use global/unscoped styles for component-specific styles

**Example**:

```vue
<template>
  <TsiCard>
    <h2>{{ title }}</h2>
    <p>{{ formattedData }}</p>
  </TsiCard>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { TsiCard } from '@/shared/components'

interface Props {
  title: string
  data: number
}

const props = defineProps<Props>()

const formattedData = computed(() => props.data.toFixed(2))
</script>

<style scoped>
h2 {
  color: var(--color-gray-900);
  margin-bottom: var(--spacing-4);
}
</style>
```

### Composables

Composables encapsulate reusable stateful logic:

```ts
// useAsync.ts - Handles async operations with loading/error states
export function useAsync<T>(asyncFn: () => Promise<T>) {
  const data = ref<T | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function execute() {
    loading.value = true
    error.value = null
    
    try {
      data.value = await asyncFn()
    } catch (err) {
      error.value = err.message
    } finally {
      loading.value = false
    }
  }

  return { data, loading, error, execute }
}
```

### State Management

- **Local state**: Use `ref` and `reactive` within components
- **Shared state**: Use composables for simple cross-component state
- **Global state**: Use Pinia stores for complex application-wide state

### Styling Strategy

1. **Design Tokens**: Define colors, spacing, fonts in `tokens.css`
2. **Base Styles**: Global resets and element styles in `base.css`
3. **Utility Classes**: Common utilities in `utilities.css`
4. **Component Styles**: Scoped to components
5. **Feature Styles**: Feature-specific styles in feature folders

**CSS Variable Usage**:

```css
.my-component {
  color: var(--color-gray-900);
  padding: var(--spacing-4);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-base);
}
```

### API Integration

Use the centralized `apiService`:

```ts
import { apiService } from '@/shared/services/api'

// In composable or component
async function loadData() {
  const data = await apiService.getCurrentDataset()
  return data.blocks
}
```

### Error Handling

Always handle errors gracefully:

```ts
try {
  await riskyOperation()
} catch (error) {
  const message = error instanceof Error ? error.message : 'Unknown error'
  console.error('Operation failed:', message)
  // Show user-friendly error
}
```

---

## Development Workflow

### Initial Setup

```bash
cd frontend
npm install
```

### Development

```bash
npm run dev          # Start dev server (http://localhost:5173)
npm run typecheck    # Type check without emitting files
npm run lint         # Check code for linting errors
npm run format:check # Check code formatting
```

### Before Committing

```bash
npm run lint:fix     # Auto-fix linting issues
npm run format       # Auto-format code
npm run typecheck    # Ensure no type errors
```

### Production Build

```bash
npm run build        # Build for production (outputs to dist/)
npm run preview      # Preview production build locally
```

---

## Path Aliases

Use path aliases for cleaner imports:

```ts
// ✅ Good
import { TsiButton } from '@/shared/components'
import { apiService } from '@/shared/services/api'
import { useAsync } from '@/shared/composables'

// ❌ Avoid
import { TsiButton } from '../../../shared/components'
```

**Configured aliases**:
- `@/*` → `src/*`
- `@shared/*` → `src/shared/*`
- `@features/*` → `src/features/*`

---

## Testing Strategy (Future)

When adding tests:

1. **Unit Tests**: Test composables and utilities in isolation
2. **Component Tests**: Test components with Vue Test Utils
3. **Integration Tests**: Test feature workflows end-to-end
4. **E2E Tests**: Test critical user paths with Playwright/Cypress

---

## Key Architectural Decisions

### 1. Feature Modules Over Technical Layers

**Why**: Organizing by feature (upload, sky-map) rather than by type (components, services) makes it easier to locate and modify related code.

### 2. Composition API Only

**Why**: Better TypeScript support, more flexible code reuse, and aligns with Vue 3 best practices.

### 3. Centralized API Client

**Why**: Single source of truth for API configuration, easier to add auth, logging, or caching.

### 4. Design Tokens

**Why**: Ensures visual consistency and makes theme changes trivial.

### 5. Scoped Styles

**Why**: Prevents style leakage and makes components truly encapsulated.

---

## Migration Guide (Existing Components)

To migrate an existing component to the new structure:

1. **Identify the feature**: Determine which feature module it belongs to
2. **Move the file**: Place it in `features/<feature-name>/components/`
3. **Update imports**: Use path aliases (`@shared`, `@features`)
4. **Extract logic**: Move business logic to composables
5. **Use shared components**: Replace custom buttons/cards with `TsiButton`, `TsiCard`, etc.
6. **Apply scoping**: Ensure styles are scoped or use external stylesheet
7. **Type everything**: Add TypeScript types for props, emits, and data

---

## Questions & Support

For questions or suggestions about the architecture:
- Review this document
- Check existing features for examples
- Follow Vue 3 and TypeScript best practices
- Refer to the official Vue 3 docs: https://vuejs.org

---

**Last Updated**: 2024
