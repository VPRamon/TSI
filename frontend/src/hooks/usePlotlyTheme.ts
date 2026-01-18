/**
 * Custom hook for Plotly chart theming.
 * Provides consistent layout and config across all charts.
 */
import { useMemo } from 'react';
import type { Layout, Config } from 'plotly.js';
import {
  BASE_LAYOUT,
  DEFAULT_AXIS_STYLE,
  HORIZONTAL_LEGEND,
  DEFAULT_CONFIG,
  SKYMAP_CONFIG,
} from '@/constants/plotly';
import { CHART_COLORS } from '@/constants/colors';

export interface PlotlyThemeOptions {
  /** Chart title */
  title?: string;
  /** X-axis configuration */
  xAxis?: {
    title?: string;
    type?: 'linear' | 'log' | 'date' | 'category';
    range?: [number, number] | [string, string];
  };
  /** Y-axis configuration */
  yAxis?: {
    title?: string;
    type?: 'linear' | 'log' | 'date' | 'category';
    range?: [number, number];
  };
  /** Show horizontal legend below chart */
  showLegend?: boolean;
  /** Bar mode for histograms */
  barMode?: 'overlay' | 'group' | 'stack';
  /** Custom shapes (e.g., background regions) */
  shapes?: Partial<Plotly.Shape>[];
  /** Chart height in pixels */
  height?: number;
  /** Config preset */
  configPreset?: 'default' | 'minimal' | 'skymap';
}

export interface PlotlyTheme {
  layout: Partial<Layout>;
  config: Partial<Config>;
}

/**
 * Hook that generates consistent Plotly layout and config based on theme options.
 *
 * @example
 * const { layout, config } = usePlotlyTheme({
 *   title: 'My Chart',
 *   xAxis: { title: 'X Label' },
 *   yAxis: { title: 'Y Label' },
 * });
 */
export function usePlotlyTheme(options: PlotlyThemeOptions = {}): PlotlyTheme {
  const {
    title,
    xAxis,
    yAxis,
    showLegend = true,
    barMode,
    shapes,
    configPreset = 'default',
  } = options;

  const layout = useMemo((): Partial<Layout> => {
    const result: Partial<Layout> = {
      ...BASE_LAYOUT,
    };

    // Add title if provided
    if (title) {
      result.title = {
        text: title,
        font: { color: CHART_COLORS.titleColor },
      };
    }

    // Configure X-axis
    if (xAxis) {
      result.xaxis = {
        title: xAxis.title ? { text: xAxis.title } : undefined,
        type: xAxis.type,
        range: xAxis.range,
        ...DEFAULT_AXIS_STYLE,
      };
    }

    // Configure Y-axis
    if (yAxis) {
      result.yaxis = {
        title: yAxis.title ? { text: yAxis.title } : undefined,
        type: yAxis.type,
        range: yAxis.range,
        ...DEFAULT_AXIS_STYLE,
      };
    }

    // Add legend
    if (showLegend) {
      result.legend = HORIZONTAL_LEGEND;
    }

    // Add bar mode for histograms
    if (barMode) {
      result.barmode = barMode;
    }

    // Add shapes
    if (shapes) {
      result.shapes = shapes;
    }

    return result;
  }, [title, xAxis, yAxis, showLegend, barMode, shapes]);

  const config = useMemo((): Partial<Config> => {
    switch (configPreset) {
      case 'minimal':
        return { responsive: true, displayModeBar: false };
      case 'skymap':
        return SKYMAP_CONFIG;
      default:
        return DEFAULT_CONFIG;
    }
  }, [configPreset]);

  return { layout, config };
}

export default usePlotlyTheme;
