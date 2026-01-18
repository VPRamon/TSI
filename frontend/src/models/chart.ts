/**
 * Chart data view models for Plotly visualizations.
 */
import type { Data } from 'plotly.js';
import { STATUS_COLORS } from '@/constants/colors';

export interface ScatterPoint {
  x: number;
  y: number;
  text?: string;
}

export interface HistogramData {
  values: number[];
  name: string;
  color: string;
  opacity?: number;
}

/**
 * Create scatter plot data for sky map visualization.
 */
export function createSkyMapScatterData(
  scheduled: ScatterPoint[],
  unscheduled: ScatterPoint[]
): Data[] {
  return [
    {
      type: 'scattergl',
      mode: 'markers',
      name: 'Scheduled',
      x: scheduled.map((p) => p.x),
      y: scheduled.map((p) => p.y),
      marker: {
        size: 8,
        color: STATUS_COLORS.scheduled,
        opacity: 0.7,
      },
      text: scheduled.map((p) => p.text ?? ''),
      hoverinfo: 'text',
    },
    {
      type: 'scattergl',
      mode: 'markers',
      name: 'Unscheduled',
      x: unscheduled.map((p) => p.x),
      y: unscheduled.map((p) => p.y),
      marker: {
        size: 6,
        color: STATUS_COLORS.unscheduled,
        opacity: 0.5,
      },
      text: unscheduled.map((p) => p.text ?? ''),
      hoverinfo: 'text',
    },
  ];
}

/**
 * Create histogram data for distribution charts.
 */
export function createHistogramData(datasets: HistogramData[]): Data[] {
  return datasets.map((dataset) => ({
    type: 'histogram' as const,
    x: dataset.values,
    name: dataset.name,
    marker: { color: dataset.color },
    opacity: dataset.opacity ?? 0.7,
  }));
}

/**
 * Create scheduled/unscheduled histogram pair.
 */
export function createScheduledHistogramPair(
  scheduledValues: number[],
  unscheduledValues: number[]
): Data[] {
  return createHistogramData([
    { values: scheduledValues, name: 'Scheduled', color: STATUS_COLORS.scheduled },
    { values: unscheduledValues, name: 'Unscheduled', color: STATUS_COLORS.unscheduled },
  ]);
}

/**
 * Create timeline Gantt-style scatter data.
 */
export function createTimelineData(
  blocks: Array<{
    startDate: Date;
    endDate: Date;
    index: number;
    color: string;
    hoverText: string;
  }>
): Data[] {
  return blocks.map((block) => ({
    type: 'scatter' as const,
    mode: 'lines' as const,
    x: [block.startDate, block.endDate],
    y: [block.index, block.index],
    line: { width: 8, color: block.color },
    text: block.hoverText,
    hoverinfo: 'text' as const,
    showlegend: false,
  }));
}
