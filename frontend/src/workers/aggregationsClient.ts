/**
 * Thin wrapper around the aggregations Web Worker.
 *
 * The worker is created lazily — and only once per session — so we avoid
 * spawning it on pages that never need heavy compute.  Calling code uses
 * `getAggregationsClient()` to obtain the comlink-wrapped proxy.
 *
 * In environments without a real `Worker` constructor (vitest/jsdom with no
 * worker shim), `getAggregationsClient()` returns `null` so callers can fall
 * back to running the same pure functions on the main thread.
 */
import { wrap, type Remote } from 'comlink';
import type { AggregationsApi } from './aggregations.worker';

let cached: Remote<AggregationsApi> | null | undefined;

export function getAggregationsClient(): Remote<AggregationsApi> | null {
  if (cached !== undefined) return cached;

  if (typeof Worker === 'undefined') {
    cached = null;
    return cached;
  }

  try {
    const worker = new Worker(new URL('./aggregations.worker.ts', import.meta.url), {
      type: 'module',
    });
    cached = wrap<AggregationsApi>(worker);
  } catch {
    // Browsers/test runners without module-worker support.
    cached = null;
  }

  return cached;
}

/** Test-only escape hatch — drop the cached proxy so the next call recreates it. */
export function __resetAggregationsClient(): void {
  cached = undefined;
}
