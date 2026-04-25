import { describe, it, expect } from 'vitest';
import {
  formatCreatedAt,
  formatStructureSummary,
  sortEnvironmentsByRecency,
  validateScheduleJsonFile,
} from './utils';
import type { EnvironmentInfo, EnvironmentStructure } from '@/api/types';

const sampleStructure: EnvironmentStructure = {
  period_start_mjd: 60000,
  period_end_mjd: 60030,
  lat_deg: -24.6,
  lon_deg: -70.4,
  elevation_m: 2400,
  blocks_hash: 'deadbeef',
};

describe('formatStructureSummary', () => {
  it('returns a friendly placeholder when structure is null', () => {
    expect(formatStructureSummary(null)).toMatch(/no structure/i);
  });

  it('renders location and period range', () => {
    const summary = formatStructureSummary(sampleStructure);
    expect(summary).toContain('24.60° S');
    expect(summary).toContain('70.40° W');
    expect(summary).toContain('–');
  });
});

describe('validateScheduleJsonFile', () => {
  it('parses a valid JSON file and strips the extension from the name', async () => {
    const file = new File(['{"foo":1}'], 'plan.json', { type: 'application/json' });
    const item = await validateScheduleJsonFile(file);
    expect(item.name).toBe('plan');
    expect(item.schedule_json).toEqual({ foo: 1 });
  });

  it('rejects non-JSON file extensions', async () => {
    const file = new File(['noop'], 'plan.txt');
    await expect(validateScheduleJsonFile(file)).rejects.toThrow(/\.json/);
  });

  it('rejects malformed JSON', async () => {
    const file = new File(['not-json'], 'broken.json');
    await expect(validateScheduleJsonFile(file)).rejects.toThrow(/broken\.json/);
  });
});

describe('sortEnvironmentsByRecency', () => {
  const make = (id: number, created_at: string): EnvironmentInfo => ({
    environment_id: id,
    name: `env-${id}`,
    structure: null,
    schedule_ids: [],
    created_at,
  });

  it('places the most recent environment first', () => {
    const result = sortEnvironmentsByRecency([
      make(1, '2025-01-01T00:00:00Z'),
      make(2, '2025-02-01T00:00:00Z'),
      make(3, '2024-12-01T00:00:00Z'),
    ]);
    expect(result.map((e) => e.environment_id)).toEqual([2, 1, 3]);
  });

  it('pushes invalid timestamps to the end without throwing', () => {
    const result = sortEnvironmentsByRecency([
      make(1, 'not-a-date'),
      make(2, '2025-01-01T00:00:00Z'),
    ]);
    expect(result[0].environment_id).toBe(2);
    expect(result[1].environment_id).toBe(1);
  });
});

describe('formatCreatedAt', () => {
  it('returns the raw value when the timestamp is unparseable', () => {
    expect(formatCreatedAt('garbage')).toBe('garbage');
  });

  it('returns a non-empty localized string for a valid timestamp', () => {
    const formatted = formatCreatedAt('2025-01-15T12:34:00Z');
    expect(formatted).not.toBe('2025-01-15T12:34:00Z');
    expect(formatted.length).toBeGreaterThan(0);
  });
});
