/**
 * Vitest setup file for React Testing Library.
 */
import '@testing-library/jest-dom';
import { cleanup } from '@testing-library/react';
import { afterEach, vi } from 'vitest';

// Default Plotly mocks — keep both the basic and full bundles stubbed
// so dynamic imports in `plotlyRegistry` resolve in jsdom. Individual
// tests can override these with their own `vi.mock(...)` calls.
vi.mock('plotly.js-basic-dist-min', () => ({
  default: {
    newPlot: vi.fn(),
    react: vi.fn(),
    purge: vi.fn(),
    toImage: vi.fn(),
    Plots: { resize: vi.fn() },
  },
}));
vi.mock('plotly.js-dist-min', () => ({
  default: {
    newPlot: vi.fn(),
    react: vi.fn(),
    purge: vi.fn(),
    toImage: vi.fn(),
    Plots: { resize: vi.fn() },
  },
}));

// Cleanup after each test
afterEach(() => {
  cleanup();
});

// Mock ResizeObserver (used by Plotly)
global.ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}));

// Mock matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});
