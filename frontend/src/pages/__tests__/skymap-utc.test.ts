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

function isValidDate(date: Date): boolean {
  return Number.isFinite(date.getTime());
}

function mjdToUtc(mjd: number): string | null {
  if (!Number.isFinite(mjd)) return null;
  const jd = mjd + MJD_EPOCH;
  const unixMs = (jd - UNIX_EPOCH_JD) * 86400000;
  const date = new Date(unixMs);
  return isValidDate(date) ? date.toISOString() : null;
}

function utcToMjd(utcString: string): number | null {
  if (!utcString) return null;
  const utc =
    /[Zz]$/.test(utcString) || /[+-]\d{2}:?\d{2}$/.test(utcString)
      ? utcString
      : utcString + 'Z';
  const date = new Date(utc);
  if (!isValidDate(date)) return null;
  const unixMs = date.getTime();
  const jd = unixMs / 86400000 + UNIX_EPOCH_JD;
  return jd - MJD_EPOCH;
}

function toDatetimeLocal(utcIso: string | null): string {
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
    expect(recoveredMjd).not.toBeNull();
    expect(Math.abs(recoveredMjd! - originalMjd)).toBeLessThan(1 / 1440);
  });

  it('treats bare datetime-local strings as UTC, not local', () => {
    // "2024-01-15T12:00" has no timezone suffix
    const mjd = utcToMjd('2024-01-15T12:00');
    expect(mjd).not.toBeNull();
    const utcIso = mjdToUtc(mjd!);

    expect(utcIso).not.toBeNull();
    expect(utcIso!).toContain('2024-01-15T12:00');
  });

  it('handles explicit Z suffix correctly', () => {
    const mjdBare = utcToMjd('2024-06-01T00:00');
    const mjdZ = utcToMjd('2024-06-01T00:00Z');
    expect(mjdBare).not.toBeNull();
    expect(mjdZ).not.toBeNull();
    expect(mjdBare!).toBeCloseTo(mjdZ!, 8);
  });

  it('returns null for empty string', () => {
    expect(utcToMjd('')).toBeNull();
  });

  it('returns null for invalid UTC strings', () => {
    expect(utcToMjd('not-a-date')).toBeNull();
  });

  it('returns null for non-finite MJDs', () => {
    expect(mjdToUtc(Number.NaN)).toBeNull();
    expect(mjdToUtc(Number.POSITIVE_INFINITY)).toBeNull();
  });
});
