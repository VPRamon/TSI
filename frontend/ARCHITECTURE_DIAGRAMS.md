# Frontend Architecture Diagrams

## 🏗️ Overall Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         TSI Frontend                             │
│                     (Vue 3 + TypeScript)                         │
└─────────────────────────────────────────────────────────────────┘
                                │
                ┌───────────────┴───────────────┐
                │                               │
        ┌───────▼────────┐            ┌────────▼────────┐
        │   Features      │            │     Shared      │
        │   (Business)    │            │  (Reusable)     │
        └───────┬────────┘            └────────┬────────┘
                │                               │
    ┌───────────┼───────────┐       ┌──────────┼──────────┐
    │           │           │       │          │          │
┌───▼──┐  ┌────▼───┐  ┌────▼───┐ ┌─▼────┐ ┌──▼────┐ ┌───▼────┐
│Upload│  │Sky Map │  │Insights│ │Comps │ │Compos.│ │Services│
└──────┘  └────────┘  └────────┘ └──────┘ └───────┘ └────────┘
```

## 📂 Feature Module Structure

```
Feature Module (e.g., upload/)
┌──────────────────────────────────────────┐
│  UploadPage.vue (Entry Point)            │
│  ┌────────────────────────────────────┐  │
│  │  Template (UI)                     │  │
│  └────────────────────────────────────┘  │
│  ┌────────────────────────────────────┐  │
│  │  <script setup>                    │  │
│  │   ├─ Import shared components      │  │
│  │   ├─ Use composables               │  │
│  │   └─ Local state & logic           │  │
│  └────────────────────────────────────┘  │
└──────────────────────────────────────────┘
           │              │              │
    ┌──────▼──┐    ┌─────▼────┐   ┌────▼────┐
    │Component│    │Composable│   │ Styles  │
    │ Widget  │    │useUpload │   │feature  │
    └─────────┘    └──────────┘   └─────────┘
```

## 🔄 Data Flow

```
User Interaction
      │
      ▼
┌──────────────┐
│  Component   │──┐
│  (Upload)    │  │
└──────────────┘  │
                  │
      ┌───────────┘
      │
      ▼
┌──────────────┐      ┌──────────────┐
│  Composable  │─────▶│  API Service │
│ useUpload()  │      │  (Centralized)│
└──────────────┘      └──────┬───────┘
      │                      │
      │                      ▼
      │               ┌─────────────┐
      │               │   Backend   │
      │               │  (Rust API) │
      │               └──────┬──────┘
      │                      │
      ◄──────────────────────┘
      │
      ▼
┌──────────────┐
│  Component   │
│  (Update UI) │
└──────────────┘
```

## 🎨 Styling Hierarchy

```
Global Styles
┌────────────────────────────────────┐
│ main.css                           │
│  ├─ tokens.css (Design Tokens)    │
│  │   ├─ Colors                     │
│  │   ├─ Spacing                    │
│  │   ├─ Typography                 │
│  │   └─ Shadows                    │
│  │                                 │
│  ├─ base.css (Resets & Base)      │
│  │   ├─ Box-sizing                 │
│  │   ├─ Body defaults              │
│  │   └─ Element defaults           │
│  │                                 │
│  └─ utilities.css (Utility Classes)│
│      ├─ Display (flex, grid)       │
│      ├─ Spacing (m-4, p-2)         │
│      └─ Typography (text-lg)       │
└────────────────────────────────────┘
                │
    ┌───────────┴────────────┐
    │                        │
Component Styles      Feature Styles
┌──────────────┐      ┌──────────────┐
│ TsiButton    │      │ landing-page │
│ <style>      │      │ <style>      │
│ scoped       │      │ scoped       │
└──────────────┘      └──────────────┘
```

## 🧩 Component Hierarchy

```
App.vue
  │
  ├─ Navigation.vue (shared)
  │
  └─ <router-view>
        │
        ├─ UploadPage.vue (feature)
        │    ├─ TsiCard (shared)
        │    ├─ TsiButton (shared)
        │    └─ FileUploadWidget (feature)
        │
        ├─ SkyMapPage.vue (feature)
        │    ├─ TsiCard (shared)
        │    ├─ TsiSpinner (shared)
        │    └─ SkyMapChart (feature)
        │
        └─ InsightsPage.vue (feature)
             ├─ TsiCard (shared)
             ├─ TsiAlert (shared)
             └─ MetricsWidget (feature)
```

## 🔀 Import Patterns

```
Feature Component
┌─────────────────────────────────────┐
│ UploadPage.vue                      │
│                                     │
│ import { TsiButton }                │
│   from '@/shared/components'        │
│        ▲                            │
│        │                            │
│ import { useAsync }                 │
│   from '@/shared/composables'       │
│        ▲                            │
│        │                            │
│ import { apiService }               │
│   from '@/shared/services/api'      │
│        ▲                            │
│        │                            │
│ import FileUploadWidget             │
│   from './components/Widget.vue'    │
│        ▲                            │
└────────┼───────────────────────────┘
         │
    Path Aliases
    (@/, @shared/)
```

## 📦 Module Dependencies

```
        Shared Layer
    ┌──────────────────┐
    │  Components      │
    │  Composables     │◄───────┐
    │  Services        │        │
    │  Types           │        │
    │  Utils           │        │
    └──────────────────┘        │
            ▲                   │
            │                   │
            │ Import            │
            │                   │
    ┌───────┴────────┐   ┌──────┴──────┐
    │  Upload        │   │  Sky Map    │
    │  Feature       │   │  Feature    │
    └────────────────┘   └─────────────┘
         ▲                      ▲
         │                      │
         └──────────┬───────────┘
                    │
            Router connects
            features to routes
```

## 🎯 Composable Pattern

```
Component
    │
    ├─ Calls composable
    │     useFileUpload()
    │          │
    ▼          ▼
┌────────────────────────┐
│  Composable            │
│                        │
│  const loading = ref() │
│  const error = ref()   │
│  const data = ref()    │
│                        │
│  async function load() │
│    └─▶ apiService.xyz()│
│                        │
│  return {              │
│    loading,            │
│    error,              │
│    data,               │
│    load                │
│  }                     │
└────────────────────────┘
         │
         │ Returns reactive refs
         │ and methods
         ▼
    Component uses
    returned values
```

## 🔐 Type Safety Flow

```
Backend API Response
        │
        ▼
┌─────────────────┐
│ API Service     │
│ (Typed Methods) │
└────────┬────────┘
         │ Returns typed data
         ▼
┌─────────────────┐
│ Composable      │
│ (Typed Refs)    │
└────────┬────────┘
         │ Typed reactive data
         ▼
┌─────────────────┐
│ Component       │
│ (Typed Props)   │
└─────────────────┘
```

## 🌐 Routing Architecture

```
router.ts
    │
    ├─ / ──────────────────▶ UploadPage.vue (feature/upload)
    │
    ├─ /sky-map ───────────▶ SkyMapPage.vue (feature/sky-map)
    │
    ├─ /distributions ─────▶ DistributionsPage.vue
    │
    ├─ /insights ──────────▶ InsightsPage.vue
    │
    └─ /timeline ──────────▶ TimelinePage.vue

    All routes use lazy loading:
    component: () => import('@/features/...')
```

## 📱 Responsive Design Flow

```
Design Tokens (CSS Variables)
        │
        ├─ Mobile: 320px-768px
        │   └─ Single column layouts
        │
        ├─ Tablet: 768px-1024px
        │   └─ 2-column layouts
        │
        └─ Desktop: 1024px+
            └─ Multi-column layouts

Utility Classes + Media Queries
        │
        ▼
Components adapt automatically
```

## 🧪 Testing Strategy (Future)

```
┌─────────────────────────────────────┐
│         Testing Pyramid             │
│                                     │
│        ┌───────────┐                │
│        │   E2E     │                │
│        └───────────┘                │
│      ┌───────────────┐              │
│      │  Integration  │              │
│      └───────────────┘              │
│   ┌──────────────────────┐          │
│   │   Component Tests    │          │
│   └──────────────────────┘          │
│ ┌────────────────────────────┐      │
│ │   Unit Tests (Composables) │      │
│ └────────────────────────────┘      │
└─────────────────────────────────────┘
```

---

## Legend

- `┌─┐ └─┘` - Modules/Components
- `│` - Relationships/Flow
- `▲ ▼` - Direction of dependency
- `◄ ►` - Bidirectional flow
