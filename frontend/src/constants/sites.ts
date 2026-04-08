/**
 * Well-known CTAO observatory site definitions.
 *
 * Coordinates are geodetic (WGS84): latitude in degrees north, longitude
 * in degrees east, elevation in metres above the ellipsoid.
 *
 * References:
 *   CTAO-N  – Roque de los Muchachos, La Palma, Canary Islands
 *   CTAO-S  – Paranal / Cerro Armazones, Atacama Desert, Chile
 */
import type { GeographicLocation } from '@/api/types';

export interface ObservatorySite {
  /** Short unique identifier used as a select option value. */
  id: string;
  /** Human-readable display name shown in the UI. */
  label: string;
  /** Geodetic coordinates sent to the backend as location_override. */
  location: GeographicLocation;
}

export const CTAO_SITES: ObservatorySite[] = [
  {
    id: 'CTAO-N',
    label: 'CTAO-N – Roque de los Muchachos (La Palma)',
    location: { latitude: 28.7624, longitude: -17.8892, elevation_m: 2396.0 },
  },
  {
    id: 'CTAO-S',
    label: 'CTAO-S – Paranal / Cerro Armazones (Chile)',
    location: { latitude: -24.6272, longitude: -70.4041, elevation_m: 2635.0 },
  },
];

/** Sentinel value meaning "use whatever location is in the file". */
export const SITE_FROM_FILE = 'from-file';
