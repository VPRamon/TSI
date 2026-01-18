# TSI Frontend

A modern React + TypeScript frontend for the Telescope Scheduling Intelligence system.

## Tech Stack

- **Framework**: React 18 with TypeScript
- **Build Tool**: Vite 5
- **Routing**: React Router v6
- **State Management**:
  - **Server State**: TanStack Query (React Query) v5
  - **UI State**: Zustand (minimal usage)
- **Styling**: Tailwind CSS with custom dark theme
- **Data Visualization**: Plotly.js
- **HTTP Client**: Axios with typed wrapper
- **Testing**: Vitest + React Testing Library

## Getting Started

```bash
# Install dependencies
npm install

# Start development server (port 3000)
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

## Available Scripts

| Script           | Description                                    |
| ---------------- | ---------------------------------------------- |
| `npm run dev`    | Start development server with hot reload       |
| `npm run build`  | TypeScript check + production build            |
| `npm run preview`| Preview production build locally               |
| `npm run lint`   | Run ESLint                                     |
| `npm run format` | Format code with Prettier                      |
| `npm run format:check` | Check formatting without changes         |
| `npm run type-check` | Run TypeScript type checking               |
| `npm run test`   | Run tests in watch mode                        |
| `npm run test:run` | Run tests once                               |
| `npm run test:coverage` | Run tests with coverage report          |

## Project Structure

```
src/
├── api/                  # API client and types
│   ├── client.ts         # Axios-based API client (singleton)
│   ├── types.ts          # TypeScript interfaces for API DTOs
│   └── index.ts          # Barrel export
│
├── components/           # Reusable UI components
│   ├── charts/           # Chart components (PlotlyChart)
│   ├── Card.tsx          # Card container component
│   ├── MetricCard.tsx    # Metric display card
│   ├── LoadingSpinner.tsx
│   ├── ErrorMessage.tsx
│   ├── ErrorBoundary.tsx # React error boundary
│   ├── Layout.tsx        # App shell with sidebar
│   └── index.ts          # Barrel export
│
├── constants/            # Shared constants
│   ├── colors.ts         # Color definitions for charts
│   ├── dates.ts          # Date utilities (MJD conversion)
│   ├── plotly.ts         # Plotly layout/config factories
│   └── index.ts          # Barrel export
│
├── features/             # Feature modules (domain-specific)
│   └── schedules/        # Schedule-related features
│       └── components/   # UploadScheduleCard, ScheduleListCard, etc.
│
├── hooks/                # Custom React hooks
│   ├── useApi.ts         # React Query hooks for API calls
│   ├── usePlotlyTheme.ts # Plotly theming hook
│   └── index.ts          # Barrel export
│
├── pages/                # Route page components
│   ├── Landing.tsx       # Home page with schedule list
│   ├── SkyMap.tsx        # Sky map visualization
│   ├── Distributions.tsx # Statistical distributions
│   ├── Timeline.tsx      # Observation timeline
│   ├── Insights.tsx      # Analytics dashboard
│   ├── Trends.tsx        # Scheduling trends
│   ├── Validation.tsx    # Validation report
│   ├── Compare.tsx       # Schedule comparison
│   └── index.ts          # Barrel export
│
├── store/                # Global state (Zustand)
│   ├── appStore.ts       # UI state (sidebar, etc.)
│   └── index.ts          # Barrel export
│
├── test/                 # Test utilities
│   ├── setup.ts          # Vitest setup file
│   └── test-utils.tsx    # Custom render with providers
│
├── App.tsx               # Route definitions
├── main.tsx              # App entry point
└── index.css             # Global styles + Tailwind
```

## Architecture

### Component Guidelines

1. **Pages**: Minimal logic, compose features/components. Handle loading/error states at the top.

2. **Features**: Own their components, hooks, and types. Avoid cross-feature imports except via shared layers (`components/`, `hooks/`, `api/`).

3. **Components**: Dumb/presentational by default. Accept typed props. No hidden side effects.

4. **Hooks**: Encapsulate logic. Use React Query for data fetching. Follow `use*` naming.

### Data Fetching Pattern

All API calls go through the typed `api` client. React Query hooks are pre-built:

```tsx
import { useSkyMap, useSchedules } from '@/hooks';

function MyComponent() {
  const { data, isLoading, error } = useSkyMap(scheduleId);
  
  if (isLoading) return <LoadingSpinner />;
  if (error) return <ErrorMessage message={error.message} />;
  
  return <div>{/* Use data */}</div>;
}
```

### Adding a New Page

1. Create `src/pages/MyPage.tsx`
2. Export from `src/pages/index.ts`
3. Add route in `src/App.tsx`
4. Add navigation link in `src/components/Layout.tsx` (if needed)

### Adding a New API Endpoint

1. Add type definitions in `src/api/types.ts`
2. Add method to `ApiClient` class in `src/api/client.ts`
3. Create React Query hook in `src/hooks/useApi.ts`
4. Export from `src/hooks/index.ts`

### Creating a New Feature

```
src/features/myFeature/
├── components/
│   ├── MyComponent.tsx
│   └── index.ts
├── hooks/
│   └── useMyFeature.ts
├── types.ts (optional)
└── index.ts
```

## Common Patterns

### Plotly Charts

Use the shared theming utilities:

```tsx
import { usePlotlyTheme } from '@/hooks';
import { PlotlyChart } from '@/components';
import { STATUS_COLORS } from '@/constants';

function MyChart({ data }) {
  const { layout, config } = usePlotlyTheme({
    title: 'My Chart',
    xAxis: { title: 'X Label' },
    yAxis: { title: 'Y Label' },
  });

  const plotData = [{
    type: 'scatter',
    x: data.map(d => d.x),
    y: data.map(d => d.y),
    marker: { color: STATUS_COLORS.scheduled },
  }];

  return <PlotlyChart data={plotData} layout={layout} config={config} height="400px" />;
}
```

### Date Handling

For astronomical dates (MJD), use the shared utilities:

```tsx
import { mjdToDate, formatMjd } from '@/constants';

const date = mjdToDate(block.scheduled_start_mjd);
const formatted = formatMjd(block.scheduled_start_mjd);
```

### Error Handling

Components are wrapped in `<ErrorBoundary>` at the app level. For API errors, use the pattern:

```tsx
if (error) {
  return (
    <ErrorMessage
      title="Failed to load data"
      message={error.message}
      onRetry={() => refetch()}
    />
  );
}
```

## Testing

Write tests in `*.test.tsx` files alongside components:

```tsx
import { describe, it, expect } from 'vitest';
import { render, screen } from '@/test/test-utils';
import MyComponent from './MyComponent';

describe('MyComponent', () => {
  it('renders correctly', () => {
    render(<MyComponent />);
    expect(screen.getByText('Expected text')).toBeInTheDocument();
  });
});
```

The custom `render` function wraps components with all necessary providers (Router, QueryClient).

## Code Quality

- **TypeScript**: Strict mode enabled. No `any` unless justified.
- **ESLint**: Run `npm run lint` before committing.
- **Prettier**: Run `npm run format` to auto-format.
- **Tests**: Run `npm run test:run` to verify changes.

## API Proxy

In development, `/api/*` requests are proxied to `http://localhost:8080`. Configure in `vite.config.ts`.
