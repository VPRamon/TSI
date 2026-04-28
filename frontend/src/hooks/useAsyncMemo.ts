/**
 * `useAsyncMemo` — `useMemo` for values produced asynchronously.
 *
 * Behaviour:
 *   - On first render, returns `initial`.
 *   - Whenever any dependency changes, calls `compute()` (sync or async) and
 *     stores its result; while the new computation is pending, the previously
 *     resolved value is kept so the UI doesn't flash empty.
 *   - `pending` flips to `true` only after `pendingDelayMs` of waiting, to
 *     avoid flicker on near-instant computations.
 *   - Stale results from outdated computations are dropped.
 *
 * NOTE: kept tiny and dependency-free on purpose; no scheduler libraries.
 */
import { useEffect, useRef, useState } from 'react';

export interface UseAsyncMemoOptions {
  /** Delay before exposing `pending: true`. Defaults to 150 ms. */
  pendingDelayMs?: number;
}

export interface UseAsyncMemoResult<T> {
  value: T;
  pending: boolean;
}

export function useAsyncMemo<T>(
  compute: () => T | Promise<T>,
  deps: ReadonlyArray<unknown>,
  initial: T,
  options: UseAsyncMemoOptions = {}
): UseAsyncMemoResult<T> {
  const { pendingDelayMs = 150 } = options;
  const [state, setState] = useState<{ value: T; pending: boolean }>({
    value: initial,
    pending: false,
  });
  const generationRef = useRef(0);

  useEffect(() => {
    const myGen = ++generationRef.current;
    let pendingTimer: ReturnType<typeof setTimeout> | null = null;
    let settled = false;

    const result = compute();

    if (!(result instanceof Promise)) {
      setState((prev) => (prev.pending ? { value: result, pending: false } : { value: result, pending: false }));
      return () => {
        if (pendingTimer) clearTimeout(pendingTimer);
      };
    }

    pendingTimer = setTimeout(() => {
      if (settled) return;
      if (generationRef.current !== myGen) return;
      setState((prev) => ({ value: prev.value, pending: true }));
    }, pendingDelayMs);

    result
      .then((value) => {
        settled = true;
        if (generationRef.current !== myGen) return;
        setState({ value, pending: false });
      })
      .catch(() => {
        settled = true;
        if (generationRef.current !== myGen) return;
        setState((prev) => ({ value: prev.value, pending: false }));
      });

    return () => {
      settled = true;
      if (pendingTimer) clearTimeout(pendingTimer);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, deps);

  return state;
}
