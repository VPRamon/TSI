import type { GeographicLocation } from '@/api/types';
import rawSites from '@root/observatories.json';

export interface ObservatorySite {
  /** Short unique identifier used as a select option value. */
  id: string;
  /** Human-readable display name shown in the UI. */
  label: string;
  /** Geodetic coordinates sent to the backend as location_override. */
  location: GeographicLocation;
}

export const OBSERVATORY_SITES: ObservatorySite[] = rawSites as ObservatorySite[];

/** Sentinel value meaning "use whatever location is in the file". */
export const SITE_FROM_FILE = 'from-file';
