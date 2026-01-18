/**
 * Date utilities and constants.
 */

/**
 * Modified Julian Date (MJD) epoch: November 17, 1858 00:00:00 UTC
 * Used for astronomical date calculations.
 */
export const MJD_EPOCH = Date.UTC(1858, 10, 17); // Month is 0-indexed

/**
 * Milliseconds per day constant.
 */
export const MS_PER_DAY = 86400000;

/**
 * Convert Modified Julian Date to JavaScript Date.
 * @param mjd - Modified Julian Date value
 * @returns JavaScript Date object
 */
export function mjdToDate(mjd: number): Date {
  return new Date(MJD_EPOCH + mjd * MS_PER_DAY);
}

/**
 * Convert JavaScript Date to Modified Julian Date.
 * @param date - JavaScript Date object
 * @returns Modified Julian Date value
 */
export function dateToMjd(date: Date): number {
  return (date.getTime() - MJD_EPOCH) / MS_PER_DAY;
}

/**
 * Format MJD as a human-readable date string.
 * @param mjd - Modified Julian Date value
 * @param options - Intl.DateTimeFormat options
 * @returns Formatted date string
 */
export function formatMjd(
  mjd: number,
  options: Intl.DateTimeFormatOptions = {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  }
): string {
  return mjdToDate(mjd).toLocaleDateString('en-US', options);
}
