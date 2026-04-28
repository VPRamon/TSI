/**
 * Plotly bundle registry — lazily loads either the lightweight basic
 * bundle (`plotly.js-basic-dist-min`) or the full bundle
 * (`plotly.js-dist-min`) on demand.
 *
 * Charts default to `'basic'`. Charts that need 3-D, mapbox/geo,
 * or other traces missing from the basic build must opt in with
 * `'full'`.
 *
 * Both modules expose the same `Plotly` namespace shape, so callers
 * can treat the resolved value uniformly.
 */
import type Plotly from 'plotly.js-dist-min';

export type PlotlyBundle = 'basic' | 'full';

export type PlotlyNamespace = typeof Plotly;

let basicPromise: Promise<PlotlyNamespace> | null = null;
let fullPromise: Promise<PlotlyNamespace> | null = null;
let preferred: PlotlyBundle = 'basic';

function resolveDefault<T>(mod: T | { default: T }): T {
  return (mod as { default?: T }).default ?? (mod as T);
}

/**
 * Asynchronously load a Plotly bundle. Subsequent calls for the same
 * bundle return the cached promise, so the browser fetches each
 * bundle at most once.
 */
export function loadPlotly(kind: PlotlyBundle = 'basic'): Promise<PlotlyNamespace> {
  if (kind === 'full') {
    preferred = 'full';
    if (!fullPromise) {
      fullPromise = import('plotly.js-dist-min').then((m) =>
        resolveDefault(m as unknown as PlotlyNamespace | { default: PlotlyNamespace })
      );
    }
    return fullPromise;
  }
  if (!basicPromise) {
    basicPromise = import('plotly.js-basic-dist-min').then((m) =>
      resolveDefault(m as unknown as PlotlyNamespace | { default: PlotlyNamespace })
    );
  }
  return basicPromise;
}

/**
 * Returns the most capable Plotly bundle that has been requested so
 * far. Helpers that operate on already-rendered chart DOM (e.g.
 * `Plotly.toImage`, `Plotly.Plots.resize`) call this so they don't
 * downgrade a chart that opted into the full bundle.
 */
export function getPreferredBundle(): PlotlyBundle {
  return preferred;
}

/**
 * Convenience: load whichever bundle is most appropriate for chart
 * chrome helpers (download / resize / fullscreen). Resolves to the
 * full bundle if any chart has opted in, otherwise the basic bundle.
 */
export function loadPreferredPlotly(): Promise<PlotlyNamespace> {
  return loadPlotly(getPreferredBundle());
}
