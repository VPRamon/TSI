/**
 * useRemountDetector - Development utility to detect component mount/unmount.
 * 
 * Add this hook to components suspected of unnecessary remounting.
 * Remove after debugging is complete.
 * 
 * Usage:
 *   useRemountDetector('ComponentName');
 */
import { useEffect, useRef } from 'react';

// Set to false to disable logging in production
const ENABLE_DETECTOR = false;

// Track mount counts for each component
const mountCounts: Record<string, number> = {};

export function useRemountDetector(componentName: string): void {
  const instanceId = useRef<number>(0);

  useEffect(() => {
    if (!ENABLE_DETECTOR) return;

    // Increment global mount count for this component type
    mountCounts[componentName] = (mountCounts[componentName] || 0) + 1;
    instanceId.current = mountCounts[componentName];

    console.log(
      `%c[MOUNT] ${componentName} (instance #${instanceId.current})`,
      'color: #22c55e; font-weight: bold;'
    );

    return () => {
      console.log(
        `%c[UNMOUNT] ${componentName} (instance #${instanceId.current})`,
        'color: #ef4444; font-weight: bold;'
      );
    };
  }, []); // Empty deps = only runs on mount/unmount
}

export function useRenderCounter(componentName: string): void {
  const renderCount = useRef(0);
  renderCount.current++;

  if (ENABLE_DETECTOR) {
    console.log(
      `%c[RENDER] ${componentName} (#${renderCount.current})`,
      'color: #3b82f6;'
    );
  }
}
