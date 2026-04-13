/**
 * Hook to initialize siderust WASM modules.
 * Loads qtty, tempoch, and siderust-web once per app lifetime.
 */
import { useState, useEffect, useRef } from 'react';
import { loadSiderust } from '@/lib/siderust';

export type SiderustStatus = 'idle' | 'loading' | 'ready' | 'error';

let globalStatus: SiderustStatus = 'idle';
let globalPromise: Promise<void> | null = null;

async function loadWasm() {
  await loadSiderust();
}

/**
 * Returns the initialization status of the siderust WASM modules.
 * Triggers loading on first call; subsequent calls share the same promise.
 */
export function useSiderust(): { status: SiderustStatus; error: string | null } {
  const [status, setStatus] = useState<SiderustStatus>(globalStatus);
  const [error, setError] = useState<string | null>(null);
  const mounted = useRef(true);

  useEffect(() => {
    mounted.current = true;
    if (globalStatus === 'ready') {
      setStatus('ready');
      return;
    }
    if (!globalPromise) {
      globalStatus = 'loading';
      globalPromise = loadWasm()
        .then(() => {
          globalStatus = 'ready';
        })
        .catch((err) => {
          globalStatus = 'error';
          globalPromise = null;
          throw err;
        });
    }
    setStatus('loading');
    globalPromise
      .then(() => {
        if (mounted.current) setStatus('ready');
      })
      .catch((err) => {
        if (mounted.current) {
          setStatus('error');
          setError(err instanceof Error ? err.message : String(err));
        }
      });
    return () => {
      mounted.current = false;
    };
  }, []);

  return { status, error };
}
