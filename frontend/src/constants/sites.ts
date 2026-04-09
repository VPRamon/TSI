import type { GeographicLocation } from '@/api/types';
import rawSites from '@root/observatories.json';

export interface ObservatorySite {
  label: string;
  location: GeographicLocation;
}

export const OBSERVATORY_SITES: ObservatorySite[] = rawSites;

/** Sentinel value meaning "use whatever location is in the file". */
export const SITE_FROM_FILE = 'from-file';

export function formatSiteLabel(site: ObservatorySite): string {
  return site.label;
}
