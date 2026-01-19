# Frontend Architecture

This document describes the frontend architecture after the UI/UX redesign, focusing on layout primitives, folder structure, and functional parity.

## Overview

The frontend is a React + TypeScript application using:
- **React 18** with React Router v6 for routing
- **Tailwind CSS** for styling (single styling approach)
- **Plotly.js** via `react-plotly.js` for all charts
- **Zustand** for global state management
- **TanStack React Query** for data fetching and caching

## Folder Structure

```
src/
├── api/                    # API client and TypeScript types
│   ├── client.ts          # Axios-based API client
│   └── types.ts           # DTOs matching backend
├── components/
│   ├── layout/            # NEW: Layout primitives
│   │   ├── PageHeader.tsx
│   │   ├── PageContainer.tsx
│   │   ├── SplitPane.tsx
│   │   ├── ToolbarRow.tsx
│   │   ├── MetricsGrid.tsx
│   │   ├── ChartPanel.tsx
│   │   └── index.ts
│   ├── charts/            # Chart components
│   │   └── PlotlyChart.tsx
│   ├── landing/           # Landing page components
│   │   ├── HeroSection.tsx
│   │   ├── UploadCard.tsx
│   │   └── ScheduleList.tsx
│   ├── Card.tsx           # Generic card container
│   ├── MetricCard.tsx     # Single value display
│   ├── DataTable.tsx      # Accessible table component
│   ├── Layout.tsx         # App shell with sidebar
│   ├── LoadingSpinner.tsx
│   ├── ErrorMessage.tsx
│   ├── ErrorBoundary.tsx
│   └── index.ts           # Barrel exports
├── constants/
│   ├── colors.ts          # Color constants for charts
│   └── dates.ts           # Date utilities (MJD conversion)
├── hooks/
│   ├── index.ts           # Custom hooks (useHealth, useSkyMap, etc.)
│   └── usePlotlyTheme.ts  # Consistent chart theming
├── pages/                 # Route page components
│   ├── Landing.tsx        # PRESERVED: Landing/upload page
│   ├── Compare.tsx        # Schedule comparison
│   ├── Distributions.tsx  # REDESIGNED: Statistical histograms
│   ├── Insights.tsx       # REDESIGNED: Analytics tables
│   ├── SkyMap.tsx         # REDESIGNED: Celestial coordinates
│   ├── Timeline.tsx       # REDESIGNED: Gantt-style chart
│   ├── Trends.tsx         # REDESIGNED: Rate analysis
│   ├── Validation.tsx     # REDESIGNED: Validation report
│   └── VisibilityMap.tsx  # REDESIGNED: Visibility histogram
├── store/
│   └── index.ts           # Zustand store
├── App.tsx                # Route definitions
├── main.tsx               # Entry point
└── index.css              # Tailwind + custom scrollbar
```

## Layout Primitives

### PageHeader
Consistent page title and description with optional actions slot.

```tsx
<PageHeader
  title="Page Title"
  description="Optional description text"
  actions={<button>Action</button>}
/>
```

### PageContainer
Wrapper providing consistent vertical spacing between sections.

```tsx
<PageContainer>
  <PageHeader ... />
  <MetricsGrid>...</MetricsGrid>
  <ChartPanel>...</ChartPanel>
</PageContainer>
```

### SplitPane
Side-by-side layout: controls on left, main content on right (desktop).
Stacks vertically on mobile.

```tsx
<SplitPane
  controls={<FilterControls />}
  controlsWidth="sm" | "md" | "lg"
>
  <ChartPanel>...</ChartPanel>
</SplitPane>
```

### ToolbarRow
Horizontal row of compact controls that wraps on small screens.

```tsx
<ToolbarRow>
  <input ... />
  <select ... />
  <button>Apply</button>
</ToolbarRow>
```

### MetricsGrid
Responsive grid for MetricCard components.

```tsx
<MetricsGrid columns={3 | 4 | 5}>
  <MetricCard label="Total" value={100} icon="📊" />
  ...
</MetricsGrid>
```

### ChartPanel
Flat panel for charts with minimal elevation (border, subtle background).

```tsx
<ChartPanel title="Optional Title">
  <PlotlyChart ... />
</ChartPanel>
```

## App Shell (Layout.tsx)

The redesigned app shell provides:

### Top Bar (non-landing pages only)
- Mobile menu hamburger button
- App identity (logo + title)
- Current schedule indicator (schedule ID + name)
- Desktop sidebar collapse toggle

### Left Sidebar (desktop)
- Collapsible (56px collapsed, 224px expanded)
- Home link
- Schedule-specific navigation when viewing a schedule
- Health status indicator in footer

### Mobile Drawer
- Slide-out overlay drawer triggered by hamburger
- Closes on route change or Escape key
- Shows current schedule info at top

### Responsive Behavior
- **Desktop (lg+)**: Sidebar visible, top bar visible
- **Tablet/Mobile**: Top bar with hamburger, drawer for nav

## Page Layouts

### Control-Heavy Pages (VisibilityMap, Trends)
Uses `SplitPane` for side-by-side controls + visualization:
- Controls in narrow left panel (desktop) or stacked above (mobile)
- Main chart(s) in flexible right area
- Preserves all original filter inputs and functionality

### Read-Only Pages (SkyMap, Timeline, Distributions, Insights, Validation)
Uses vertical `PageContainer` with:
- `PageHeader` for title/description
- `MetricsGrid` for key metrics
- `ChartPanel` or flat sections for content
- Tables with consistent styling (hover states, dividers)

## Design System

### Colors
- **Background**: slate-900 (`#0f172a`)
- **Surface**: slate-800 (`#1e293b`), slate-800/30 for subtle panels
- **Border**: slate-700 (`#334155`)
- **Text**: white (primary), slate-300/400 (secondary)
- **Accent**: primary-500/600 (blue)
- **Status**: emerald (success), red (error), amber (warning)

### Elevation
- Flat by default (border only)
- Subtle background (`bg-slate-800/30` or `/50`) for interactive areas
- No heavy shadows - clean, professional appearance

### Typography
- Page titles: `text-2xl font-bold`
- Section headings: `text-lg font-semibold`
- Panel titles: `text-sm font-medium`
- Body text: `text-sm`
- Labels: `text-xs` with `uppercase tracking-wide` for table headers

### Spacing
- Page sections: `gap-6` (24px)
- Within cards: `gap-4` (16px)
- Tight groups: `gap-2` or `gap-3`

## Functional Parity Checklist

Each redesigned page was verified to preserve:

| Page | Routes | Controls | Data Display | Charts |
|------|--------|----------|--------------|--------|
| Validation | ✅ | N/A | ✅ Tables, metrics | N/A |
| Distributions | ✅ | N/A | ✅ Stats grids | ✅ Priority & visibility histograms |
| Insights | ✅ | N/A | ✅ Tables, correlations | N/A |
| SkyMap | ✅ | N/A | ✅ Metrics, legend | ✅ Scatter plot |
| Timeline | ✅ | N/A | ✅ Metrics, month tags | ✅ Gantt chart |
| Trends | ✅ | ✅ Bins, bandwidth | ✅ Metrics | ✅ Bar + line charts |
| VisibilityMap | ✅ | ✅ Binning, priority filters | ✅ Metrics | ✅ Bar histogram |

### Preserved Functionality
- All route paths unchanged
- All API calls and data hooks unchanged
- All filter state and inputs preserved
- All chart types and configurations preserved
- Loading and error states maintained
- Navigation and routing behavior unchanged

## Accessibility

- Skip-to-content link for keyboard navigation
- Focus ring styles on all interactive elements
- Semantic HTML (header, nav, main, section, table)
- ARIA labels on icon-only buttons
- Proper focus management for mobile drawer

## Build & Development

```bash
# Development
npm run dev

# Type checking
npm run build  # Runs tsc before vite build

# Linting
npm run lint

# Tests
npm run test
```

## Future Considerations

1. **Code Splitting**: The main bundle is large (~5MB). Consider:
   - Dynamic imports for page components
   - Manual chunks for Plotly.js

2. **Theme System**: Currently uses Tailwind classes directly. Could extract:
   - Design tokens to CSS variables
   - Component variant system

3. **Mobile Optimizations**: Charts could benefit from:
   - Reduced data points on mobile
   - Touch-friendly controls
   - Swipe gestures for navigation
