# TSI Frontend - Refactored Architecture

> **Modern, scalable Vue 3 application with feature-based architecture**

## 🎯 Architecture Highlights

This refactored frontend implements enterprise-grade patterns:

✅ **Feature-Based Modules** - Self-contained features with clear boundaries  
✅ **Composition API** - Modern Vue 3 patterns throughout  
✅ **TypeScript** - Full type safety with strict mode  
✅ **Design System** - Reusable UI components with consistent styling  
✅ **Centralized Services** - Single API client with interceptors  
✅ **Smart Composables** - Reusable stateful logic  
✅ **Design Tokens** - CSS variables for maintainable styling  
✅ **Path Aliases** - Clean imports with `@/` prefixes  
✅ **Linting & Formatting** - ESLint + Prettier for code quality  

---

## 📁 Project Structure

```
src/
├── features/              # Feature modules (business logic)
│   ├── upload/           # File upload & landing page
│   ├── sky-map/          # Sky map visualization
│   ├── distributions/    # Data distributions & stats
│   ├── insights/         # Analytics dashboard
│   ├── timeline/         # Scheduled timeline view
│   ├── trends/           # Trend analysis
│   ├── compare/          # Schedule comparison
│   └── visibility/       # Visibility map
│
├── shared/               # Shared/reusable code
│   ├── components/       # UI primitives (TsiButton, TsiCard, etc.)
│   ├── composables/      # Reusable composition functions
│   ├── services/         # API client, external services
│   ├── stores/           # Pinia stores (global state)
│   ├── types/            # TypeScript interfaces
│   ├── utils/            # Utility functions
│   └── styles/           # Global styles (tokens, base, utilities)
│
├── App.vue               # Root component
├── main.ts               # Entry point
└── router.ts             # Route configuration
```

Each **feature module** contains:
```
feature-name/
├── components/         # Feature-specific components
├── composables/        # Feature-specific logic
├── services/           # Feature-specific API calls
├── types/              # Feature-specific TypeScript types
├── assets/             # Feature-specific images/icons
├── styles/             # Feature-specific CSS
└── FeaturePage.vue     # Main page component
```

---

## 🚀 Quick Start

### Install Dependencies
```bash
npm install
```

### Development
```bash
npm run dev          # Start dev server → http://localhost:5173
npm run typecheck    # Type check without building
npm run lint         # Check for linting errors
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
npm run build        # Build for production → dist/
npm run preview      # Preview production build
```

---

## 🧩 Key Components

### Shared UI Components

All UI primitives are prefixed with `Tsi` for easy identification:

- **TsiButton** - Consistent button with variants (primary, secondary, ghost, danger)
- **TsiCard** - Flexible card container with optional header/footer
- **TsiInput** - Form input with label, error, and hint support
- **TsiSpinner** - Loading indicator (sm, md, lg sizes)
- **TsiAlert** - Dismissible alerts (success, error, warning, info)

**Usage**:
```vue
<script setup lang="ts">
import { TsiButton, TsiCard } from '@/shared/components'
</script>

<template>
  <TsiCard title="My Card" padding="lg">
    <p>Card content goes here</p>
    <template #footer>
      <TsiButton variant="primary" @click="handleClick">
        Submit
      </TsiButton>
    </template>
  </TsiCard>
</template>
```

### Composables

Reusable composition functions for common patterns:

**useAsync** - Handle async operations with loading/error states:
```ts
import { useAsync } from '@/shared/composables'

const { data, loading, error, execute } = useAsync(myAsyncFunction)
```

**usePolling** - Poll a data source at intervals:
```ts
import { usePolling } from '@/shared/composables'

usePolling(checkForUpdates, { interval: 2000 })
```

### API Service

Centralized API client with typed methods:

```ts
import { apiService } from '@/shared/services/api'

// Dataset operations
const data = await apiService.getCurrentDataset()
const metadata = await apiService.getDatasetMetadata()
await apiService.uploadCSV(file)
await apiService.loadSampleDataset()

// Analytics operations
const metrics = await apiService.getMetrics()
const conflicts = await apiService.getConflicts()
const histogram = await apiService.getHistogram('priority', 20)
```

---

## 🎨 Styling System

### Design Tokens

CSS variables provide a single source of truth:

```css
.my-component {
  color: var(--color-gray-900);
  padding: var(--spacing-4);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-base);
  font-family: var(--font-family-sans);
}
```

**Available tokens**:
- Colors: `--color-primary-*`, `--color-gray-*`, `--color-success`, etc.
- Spacing: `--spacing-1` through `--spacing-16`
- Border radius: `--radius-sm`, `--radius-md`, `--radius-lg`
- Shadows: `--shadow-sm`, `--shadow-base`, `--shadow-lg`
- Typography: `--font-size-*`, `--font-weight-*`

### Utility Classes

Common utilities for rapid development:

```html
<div class="flex items-center justify-between gap-4">
  <p class="text-lg font-semibold text-gray-900">Title</p>
  <button class="px-4 py-2 bg-primary text-white rounded-lg">Click</button>
</div>
```

---

## 📝 Naming Conventions

| Type | Convention | Example |
|------|-----------|---------|
| Feature folders | `kebab-case` | `sky-map/`, `file-upload/` |
| Page components | `PascalCase` + `Page.vue` | `UploadPage.vue` |
| Shared components | `Tsi` + `PascalCase` | `TsiButton.vue` |
| Composables | `use` + `camelCase` | `useAsync.ts` |
| Types | `PascalCase` | `SchedulingBlock` |
| Constants | `UPPER_SNAKE_CASE` | `API_BASE_URL` |

---

## 🛠️ Tech Stack

- **Vue 3** - Progressive JavaScript framework
- **TypeScript** - Type-safe development
- **Vite** - Fast build tool and dev server
- **Vue Router** - Official routing library
- **Axios** - HTTP client for API calls
- **ECharts** - Interactive charts (via vue-echarts)
- **Pinia** - State management (ready to use)
- **ESLint** - Code linting
- **Prettier** - Code formatting

---

## 📚 Documentation

- **[ARCHITECTURE.md](./ARCHITECTURE.md)** - Comprehensive architecture guide
- **[MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md)** - How to refactor existing components
- **[Vue 3 Docs](https://vuejs.org)** - Official Vue documentation
- **[TypeScript Docs](https://www.typescriptlang.org)** - TypeScript reference

---

## 🎓 Best Practices

### Component Design

✅ **Do**:
- Use Composition API (`<script setup>`)
- Type all props and emits
- Keep components focused and single-responsibility
- Use scoped styles or external stylesheets
- Extract complex logic to composables

❌ **Don't**:
- Mix business logic with presentation
- Use Options API
- Create overly complex components
- Use global/unscoped styles for component-specific styles

### Example Component

```vue
<template>
  <TsiCard>
    <TsiSpinner v-if="loading" message="Loading data..." />
    <TsiAlert v-else-if="error" variant="error" :message="error" />
    <div v-else>
      <h2>{{ title }}</h2>
      <p>{{ formattedData }}</p>
    </div>
  </TsiCard>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { TsiCard, TsiSpinner, TsiAlert } from '@/shared/components'
import { useAsync } from '@/shared/composables'
import { apiService } from '@/shared/services/api'

const title = 'My Feature'

const { data, loading, error, execute } = useAsync(
  () => apiService.getCurrentDataset()
)

const formattedData = computed(() => {
  return data.value?.blocks.length || 0
})

onMounted(() => execute())
</script>

<style scoped>
h2 {
  color: var(--color-gray-900);
  font-size: var(--font-size-2xl);
  margin-bottom: var(--spacing-4);
}
</style>
```

---

## 🔧 Configuration Files

- **tsconfig.json** - TypeScript configuration with path aliases
- **vite.config.ts** - Vite configuration with path resolution
- **eslintrc.json** - ESLint rules for Vue 3 + TypeScript
- **prettierrc.json** - Prettier code formatting rules

---

## 🤝 Contributing

When adding new features:

1. Create a feature module in `src/features/`
2. Follow the established folder structure
3. Use shared components and composables
4. Add TypeScript types
5. Write scoped/modular styles
6. Update router configuration
7. Run linting and type checks before committing

---

## 📊 Feature Status

| Feature | Status | Notes |
|---------|--------|-------|
| Upload | ✅ Refactored | Complete with composables |
| Sky Map | 🔄 Pending | Ready to migrate |
| Distributions | 🔄 Pending | Ready to migrate |
| Insights | 🔄 Pending | Ready to migrate |
| Timeline | 🔄 Pending | Ready to migrate |
| Trends | 🔄 Pending | Ready to migrate |
| Compare | 🔄 Pending | Ready to migrate |
| Visibility | 🔄 Pending | Ready to migrate |

---

## 🎯 Next Steps

1. **Migrate remaining features** following the pattern established in `features/upload/`
2. **Add unit tests** for composables and utilities
3. **Add component tests** using Vue Test Utils
4. **Implement Pinia stores** for complex global state
5. **Add E2E tests** for critical user flows
6. **Optimize bundle** with code splitting and lazy loading
7. **Add accessibility** features (ARIA labels, keyboard navigation)
8. **Document components** with JSDoc comments

---

## 🐛 Troubleshooting

### TypeScript Errors

If you see module resolution errors, ensure:
- Path aliases are configured in `tsconfig.json` and `vite.config.ts`
- You're using `@/` prefix for imports
- Node modules are installed (`npm install`)

### Styling Issues

If design tokens aren't working:
- Verify `@/shared/styles/main.css` is imported in `main.ts`
- Check that CSS variables are defined in `tokens.css`
- Ensure styles are scoped to avoid conflicts

### Build Errors

If the build fails:
- Run `npm run typecheck` to identify type errors
- Run `npm run lint` to find linting issues
- Clear `node_modules` and reinstall: `rm -rf node_modules && npm install`

---

## 📄 License

[Your License Here]

---

**Happy coding!** 🚀
