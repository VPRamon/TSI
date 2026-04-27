import type { ComponentType, ReactNode } from 'react';

/**
 * TSI frontend extension contract (v1).
 *
 * External integrators provide a module that exports `extensions: TsiExtensions`.
 * The path to that module is resolved at build time via the
 * `VITE_TSI_EXTENSIONS_PATH` Vite alias declared in `vite.config.ts`,
 * which points at the integrator's pack (defaults to
 * `webapp/phd-extensions` for the PhD/EST research distribution).
 *
 * Contract guarantees:
 *   - TSI never imports anything from the integrator pack except the
 *     single `extensions` value re-exported below.
 *   - The integrator pack must NOT import any private (`@/...`) TSI
 *     modules; it is a peer of TSI and may only depend on the public
 *     types in this file plus the `@/api/types` public surface.
 *   - Backwards-incompatible changes to this file bump the contract
 *     version (see `EXTENSION_CONTRACT_VERSION`).
 */
export const EXTENSION_CONTRACT_VERSION = 1 as const;

export interface TsiNavItem {
  path: string;
  label: string;
  scope: 'global' | 'schedule';
  icon?: ReactNode;
}

export interface TsiRoute {
  path: string;
  element: ReactNode;
}

/**
 * One sub-tab inside an algorithm-specific analysis surface.  Rendered
 * inside the `AlgorithmAnalysisPage` shell, which provides the active
 * schedule selection via the `AlgorithmContext` (see
 * `pages/AlgorithmAnalysis.tsx`).
 */
export interface TsiAlgorithmTab {
  /** URL slug, e.g. `'overview'`. Must be unique within the algorithm. */
  id: string;
  /** Human-readable label displayed in the tab bar. */
  label: string;
  /**
   * Component rendered inside the algorithm shell when the tab is active.
   *
   * May be a `React.lazy(...)` component — the shell wraps every tab in
   * a `<Suspense>` so heavy panels are code-split out of the main TSI
   * bundle.
   */
  component: ComponentType;
}

/**
 * An algorithm contributed by an extension pack.  Each entry adds one
 * algorithm-specific surface under `/algorithm/{id}`. The shell selects
 * the active algorithm by matching `id` against the
 * `schedule_metadata.algorithm` value of the currently selected
 * schedules.
 */
export interface TsiAlgorithm {
  /** Identifier matching `ScheduleMetadata.algorithm`, e.g. `'est'`. */
  id: string;
  /** Human-readable label, e.g. `'EST'`. */
  label: string;
  /** Tabs offered by this algorithm. The first tab is the default. */
  tabs: TsiAlgorithmTab[];
  icon?: ReactNode;
}

export interface TsiExtensions {
  routes: TsiRoute[];
  navItems: TsiNavItem[];
  /** Optional registry of algorithm-specific analysis surfaces. */
  algorithms?: TsiAlgorithm[];
}

// Re-export the integrator's `extensions` value through a single
// build-time alias. The alias is configured in `vite.config.ts` and
// can be overridden with `VITE_TSI_EXTENSIONS_PATH` when invoking
// `vite build` / `vite dev`.
export { extensions } from 'tsi-extensions-pack';
