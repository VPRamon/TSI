/**
 * Plotly chart configuration constants and factory functions.
 * Centralizes chart theming for consistent visualization.
 */
import type { Layout, Config } from 'plotly.js';
import { CHART_COLORS } from './colors';

/**
 * Base layout configuration for all Plotly charts.
 * Extends this for specific chart types.
 */
export const BASE_LAYOUT: Partial<Layout> = {
  paper_bgcolor: CHART_COLORS.background,
  plot_bgcolor: CHART_COLORS.plotBackground,
  font: { color: CHART_COLORS.textColor },
  margin: { t: 50, r: 20, b: 60, l: 60 },
  hovermode: 'closest',
};

/**
 * Default axis styling.
 */
export const DEFAULT_AXIS_STYLE = {
  gridcolor: CHART_COLORS.gridColor,
  zerolinecolor: CHART_COLORS.zeroLineColor,
};

/**
 * Horizontal legend configuration (for placement below chart).
 */
export const HORIZONTAL_LEGEND: Partial<Layout['legend']> = {
  orientation: 'h' as const,
  y: -0.15,
  font: { color: CHART_COLORS.textColor },
};

/**
 * Default Plotly config for interactivity.
 */
export const DEFAULT_CONFIG: Partial<Config> = {
  responsive: true,
  displayModeBar: true,
};

/**
 * Minimal config (no mode bar).
 */
export const MINIMAL_CONFIG: Partial<Config> = {
  responsive: true,
  displayModeBar: false,
};

/**
 * Sky map specific config (removes some tools).
 */
export const SKYMAP_CONFIG: Partial<Config> = {
  responsive: true,
  displayModeBar: true,
  modeBarButtonsToRemove: ['lasso2d', 'select2d'],
};

/**
 * Create a layout with title.
 * @param title - Chart title text
 * @param overrides - Additional layout properties
 */
export function createLayout(title: string, overrides: Partial<Layout> = {}): Partial<Layout> {
  return {
    ...BASE_LAYOUT,
    title: {
      text: title,
      font: { color: CHART_COLORS.titleColor },
    },
    ...overrides,
  };
}

/**
 * Create a layout for histogram/bar charts.
 */
export function createHistogramLayout(
  title: string,
  xAxisTitle: string,
  yAxisTitle: string = 'Count'
): Partial<Layout> {
  return createLayout(title, {
    barmode: 'overlay',
    xaxis: {
      title: { text: xAxisTitle },
      ...DEFAULT_AXIS_STYLE,
    },
    yaxis: {
      title: { text: yAxisTitle },
      ...DEFAULT_AXIS_STYLE,
    },
    legend: HORIZONTAL_LEGEND,
  });
}

/**
 * Create a layout for scatter plots.
 */
export function createScatterLayout(
  title: string,
  xAxisTitle: string,
  yAxisTitle: string
): Partial<Layout> {
  return createLayout(title, {
    xaxis: {
      title: { text: xAxisTitle },
      ...DEFAULT_AXIS_STYLE,
    },
    yaxis: {
      title: { text: yAxisTitle },
      ...DEFAULT_AXIS_STYLE,
    },
    legend: HORIZONTAL_LEGEND,
  });
}

/**
 * Create a layout for time series charts.
 */
export function createTimeSeriesLayout(title: string, yAxisTitle: string): Partial<Layout> {
  return createLayout(title, {
    xaxis: {
      title: { text: 'Date' },
      type: 'date',
      ...DEFAULT_AXIS_STYLE,
    },
    yaxis: {
      title: { text: yAxisTitle },
      ...DEFAULT_AXIS_STYLE,
    },
    legend: HORIZONTAL_LEGEND,
  });
}
