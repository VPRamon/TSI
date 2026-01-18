/**
 * Tests for date utility functions.
 */
import { describe, it, expect } from 'vitest';
import { mjdToDate, dateToMjd, formatMjd, MJD_EPOCH } from './dates';

describe('date utilities', () => {
  describe('MJD_EPOCH', () => {
    it('is November 17, 1858', () => {
      const epochDate = new Date(MJD_EPOCH);
      expect(epochDate.getUTCFullYear()).toBe(1858);
      expect(epochDate.getUTCMonth()).toBe(10); // November (0-indexed)
      expect(epochDate.getUTCDate()).toBe(17);
    });
  });

  describe('mjdToDate', () => {
    it('converts MJD 0 to the epoch date', () => {
      const date = mjdToDate(0);
      expect(date.getTime()).toBe(MJD_EPOCH);
    });

    it('converts MJD 51544 to J2000 epoch (Jan 1, 2000)', () => {
      const date = mjdToDate(51544);
      expect(date.getUTCFullYear()).toBe(2000);
      expect(date.getUTCMonth()).toBe(0); // January
      expect(date.getUTCDate()).toBe(1);
    });

    it('handles fractional MJD values', () => {
      const date = mjdToDate(51544.5);
      expect(date.getUTCHours()).toBe(12);
    });
  });

  describe('dateToMjd', () => {
    it('converts epoch date to MJD 0', () => {
      const date = new Date(MJD_EPOCH);
      expect(dateToMjd(date)).toBe(0);
    });

    it('converts J2000 epoch to MJD 51544', () => {
      const j2000 = new Date(Date.UTC(2000, 0, 1));
      expect(dateToMjd(j2000)).toBe(51544);
    });

    it('is the inverse of mjdToDate', () => {
      const originalMjd = 60000.75;
      const date = mjdToDate(originalMjd);
      const convertedMjd = dateToMjd(date);
      expect(convertedMjd).toBeCloseTo(originalMjd, 10);
    });
  });

  describe('formatMjd', () => {
    it('formats MJD as readable date', () => {
      const formatted = formatMjd(51544);
      expect(formatted).toContain('2000');
      expect(formatted).toContain('Jan');
    });

    it('uses custom format options', () => {
      const formatted = formatMjd(51544, { year: 'numeric' });
      expect(formatted).toBe('2000');
    });
  });
});
