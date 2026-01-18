/**
 * Tests for usePlotlyTheme hook.
 */
import { describe, it, expect } from 'vitest';
import { renderHook } from '@testing-library/react';
import { usePlotlyTheme } from './usePlotlyTheme';
import { CHART_COLORS } from '@/constants/colors';

describe('usePlotlyTheme', () => {
  it('returns default layout and config', () => {
    const { result } = renderHook(() => usePlotlyTheme());

    expect(result.current.layout).toBeDefined();
    expect(result.current.config).toBeDefined();
    expect(result.current.layout.paper_bgcolor).toBe(CHART_COLORS.background);
    expect(result.current.layout.plot_bgcolor).toBe(CHART_COLORS.plotBackground);
  });

  it('sets title when provided', () => {
    const { result } = renderHook(() =>
      usePlotlyTheme({ title: 'My Chart' })
    );

    expect(result.current.layout.title).toEqual({
      text: 'My Chart',
      font: { color: CHART_COLORS.titleColor },
    });
  });

  it('configures x-axis when provided', () => {
    const { result } = renderHook(() =>
      usePlotlyTheme({
        xAxis: { title: 'X Label', range: [0, 100] },
      })
    );

    expect(result.current.layout.xaxis?.title).toEqual({ text: 'X Label' });
    expect(result.current.layout.xaxis?.range).toEqual([0, 100]);
  });

  it('configures y-axis when provided', () => {
    const { result } = renderHook(() =>
      usePlotlyTheme({
        yAxis: { title: 'Y Label', range: [0, 50] },
      })
    );

    expect(result.current.layout.yaxis?.title).toEqual({ text: 'Y Label' });
    expect(result.current.layout.yaxis?.range).toEqual([0, 50]);
  });

  it('includes legend by default', () => {
    const { result } = renderHook(() => usePlotlyTheme());

    expect(result.current.layout.legend).toBeDefined();
    expect(result.current.layout.legend?.orientation).toBe('h');
  });

  it('excludes legend when showLegend is false', () => {
    const { result } = renderHook(() =>
      usePlotlyTheme({ showLegend: false })
    );

    expect(result.current.layout.legend).toBeUndefined();
  });

  it('sets bar mode for histograms', () => {
    const { result } = renderHook(() =>
      usePlotlyTheme({ barMode: 'overlay' })
    );

    expect(result.current.layout.barmode).toBe('overlay');
  });

  it('includes shapes when provided', () => {
    const shapes = [
      { type: 'rect' as const, x0: 0, x1: 1, y0: 0, y1: 1 },
    ];

    const { result } = renderHook(() =>
      usePlotlyTheme({ shapes })
    );

    expect(result.current.layout.shapes).toHaveLength(1);
  });

  it('returns default config by default', () => {
    const { result } = renderHook(() => usePlotlyTheme());

    expect(result.current.config.responsive).toBe(true);
    expect(result.current.config.displayModeBar).toBe(true);
  });

  it('returns minimal config when preset is minimal', () => {
    const { result } = renderHook(() =>
      usePlotlyTheme({ configPreset: 'minimal' })
    );

    expect(result.current.config.displayModeBar).toBe(false);
  });

  it('returns skymap config when preset is skymap', () => {
    const { result } = renderHook(() =>
      usePlotlyTheme({ configPreset: 'skymap' })
    );

    expect(result.current.config.modeBarButtonsToRemove).toContain('lasso2d');
    expect(result.current.config.modeBarButtonsToRemove).toContain('select2d');
  });

  it('memoizes layout and config', () => {
    const { result, rerender } = renderHook(() =>
      usePlotlyTheme({ title: 'Test' })
    );

    const initialLayout = result.current.layout;
    const initialConfig = result.current.config;

    rerender();

    expect(result.current.layout).toBe(initialLayout);
    expect(result.current.config).toBe(initialConfig);
  });
});
