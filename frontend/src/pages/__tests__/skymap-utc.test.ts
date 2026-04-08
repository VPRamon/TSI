/**
 * Tests for SkyMap UTC conversion helpers.
 *
 * Verifies that datetime-local strings (no timezone suffix) are
 * treated as UTC rather than local time.
 */
import { describe, it, expect } from 'vitest';

// ─── Re-implemented conversion helpers (mirrors SkyMap.tsx) ─────────

const MJD_EPOCH = 2400000.5;
const UNIX_EPOCH_JD = 2440587.5;

function mjdToUtc(mjd: number): string {
  const jd = mjd + MJD_EPOCH;
  const unixMs = (jd - UNIX_EPOCH_JD) * 86400000;
  return new Date(unixMs).toISOString();
}

function utcToMjd(utcString: string): number {
  if (!utcString) return 0;
  const utc =
    /[Zz]$/.test(utcString) || /[+-]\d{2}:?\d{2}$/.test(utcString)
      ? utcString
      : utcString + 'Z';
  const unixMs = new Date(utc).getTime();
  const jd = unixMs / 86400000 + UNIX_EPOCH_JD;
  return jd - MJD_EPOCH;
}

function toDatetimeLocal(utcIso: string): string {
  if (!utcIso) return '';
  return utcIso.slice(0, 16);
}

// ─── Tests ──────────────────────────────────────────────────────────

describe('SkyMap UTC helpers', () => {
  it('round-trips MJD → UTC → datetime-local → MJD without timezone drift', () => {
    const originalMjd = 60000; // ~2023-02-25
    const utcIso = mjdToUtc(originalMjd);
    const dtLocal = toDatetimeLocal(utcIso);
    const recoveredMjd = utcToMjd(dtLocal);

    // Allow 1-minute precision loss from slicing seconds
    expect(Math.abs(recoveredMjd - originalMjd)).toBeLessThan(1 / 1440);
  });

  it('treats bare datetime-local strings as UTC, not local', () => {
    // "2024-01-15T12:00" has no timezone suffix
    const mjd = utcToMjd('2024-01-15T12:00');
    const utcIso = mjdToUtc(mjd);

    expect(utcIso).toContain('2024-01-15T12:00');
  });

  it('handles explicit Z suffix correctly', () => {
    const mjdBare = utcToMjd('2024-06-01T00:00');
    const mjdZ = utcToMjd('2024-06-01T00:00Z');
    expect(mjdBare).toBeCloseTo(mjdZ, 8);
  });

  it('returns 0 for empty string', () => {
    expect(utcToMjd('')).toBe(0);
  });
});
