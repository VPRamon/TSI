/**
 * Tests for chart data view models.
 */
import { describe, it, expect } from 'vitest';
import type { ScatterData } from 'plotly.js';
import {
  createSkyMapScatterData,
  createHistogramData,
  createScheduledHistogramPair,
} from './chart';
import { STATUS_COLORS } from '@/constants/colors';

describe('createSkyMapScatterData', () => {
  it('creates scheduled and unscheduled scatter traces', () => {
    const scheduled = [
      { x: 180, y: 45, text: 'Block 1' },
      { x: 90, y: 30, text: 'Block 2' },
    ];
    const unscheduled = [{ x: 270, y: -15, text: 'Block 3' }];

    const result = createSkyMapScatterData(scheduled, unscheduled);

    expect(result).toHaveLength(2);
    expect(result[0].name).toBe('Scheduled');
    expect(result[1].name).toBe('Unscheduled');
  });

  it('uses correct status colors', () => {
    const result = createSkyMapScatterData([{ x: 0, y: 0 }], [{ x: 1, y: 1 }]);

    const scheduled = result[0] as ScatterData;
    const unscheduled = result[1] as ScatterData;
    expect(scheduled.marker?.color).toBe(STATUS_COLORS.scheduled);
    expect(unscheduled.marker?.color).toBe(STATUS_COLORS.unscheduled);
  });

  it('maps x, y, and text values correctly', () => {
    const scheduled = [{ x: 100, y: 50, text: 'hover text' }];

    const result = createSkyMapScatterData(scheduled, []);
    const trace = result[0] as ScatterData;

    expect(trace.x).toEqual([100]);
    expect(trace.y).toEqual([50]);
    expect(trace.text).toEqual(['hover text']);
  });

  it('handles empty inputs', () => {
    const result = createSkyMapScatterData([], []);
    const trace0 = result[0] as ScatterData;
    const trace1 = result[1] as ScatterData;

    expect(result).toHaveLength(2);
    expect(trace0.x).toEqual([]);
    expect(trace1.x).toEqual([]);
  });
});

describe('createHistogramData', () => {
  it('creates histogram traces from datasets', () => {
    const datasets = [
      { values: [1, 2, 3], name: 'Dataset A', color: '#ff0000' },
      { values: [4, 5, 6], name: 'Dataset B', color: '#00ff00', opacity: 0.5 },
    ];

    const result = createHistogramData(datasets);

    expect(result).toHaveLength(2);
    expect(result[0].type).toBe('histogram');
    // Use type assertion for histogram-specific properties
    const hist0 = result[0] as { x: number[]; name: string; marker: { color: string }; opacity: number };
    const hist1 = result[1] as { opacity: number };
    expect(hist0.x).toEqual([1, 2, 3]);
    expect(hist0.name).toBe('Dataset A');
    expect(hist0.marker.color).toBe('#ff0000');
    expect(hist0.opacity).toBe(0.7); // default opacity
    expect(hist1.opacity).toBe(0.5); // custom opacity
  });

  it('handles single dataset', () => {
    const datasets = [{ values: [10, 20], name: 'Single', color: '#0000ff' }];

    const result = createHistogramData(datasets);

    expect(result).toHaveLength(1);
  });

  it('handles empty values', () => {
    const datasets = [{ values: [], name: 'Empty', color: '#cccccc' }];

    const result = createHistogramData(datasets);
    const hist = result[0] as { x: number[] };

    expect(hist.x).toEqual([]);
  });
});

describe('createScheduledHistogramPair', () => {
  it('creates scheduled and unscheduled histogram traces', () => {
    const scheduledValues = [1, 2, 3];
    const unscheduledValues = [4, 5];

    const result = createScheduledHistogramPair(scheduledValues, unscheduledValues);
    const hist0 = result[0] as { x: number[]; name: string };
    const hist1 = result[1] as { x: number[]; name: string };

    expect(result).toHaveLength(2);
    expect(hist0.name).toBe('Scheduled');
    expect(hist1.name).toBe('Unscheduled');
    expect(hist0.x).toEqual([1, 2, 3]);
    expect(hist1.x).toEqual([4, 5]);
  });

  it('uses status colors', () => {
    const result = createScheduledHistogramPair([1], [2]);
    const hist0 = result[0] as { marker: { color: string } };
    const hist1 = result[1] as { marker: { color: string } };

    expect(hist0.marker.color).toBe(STATUS_COLORS.scheduled);
    expect(hist1.marker.color).toBe(STATUS_COLORS.unscheduled);
  });
});
