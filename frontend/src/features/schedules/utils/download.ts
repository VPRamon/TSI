import JSZip from 'jszip';
import { api } from '@/api';
import type { ScheduleInfo } from '@/api/types';

export function buildScheduleFilename(schedule: ScheduleInfo): string {
  const normalized = schedule.schedule_name
    .trim()
    .replace(/[^a-zA-Z0-9_-]+/g, '_')
    .replace(/^_+|_+$/g, '');
  const fallback = 'schedule';
  const stem = normalized.length > 0 ? normalized : fallback;
  return `${stem}.json`;
}

function buildUniqueFilename(filename: string, counters: Map<string, number>): string {
  const stem = filename.replace(/\.json$/i, '');
  const seen = counters.get(stem) ?? 0;
  counters.set(stem, seen + 1);

  if (seen === 0) {
    return `${stem}.json`;
  }

  return `${stem}_${seen + 1}.json`;
}

function triggerBlobDownload(blob: Blob, filename: string): void {
  const url = URL.createObjectURL(blob);

  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();

  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

export function triggerJsonDownload(content: string, filename: string): void {
  const blob = new Blob([content], { type: 'application/json' });
  triggerBlobDownload(blob, filename);
}

function buildZipFilename(): string {
  const date = new Date().toISOString().slice(0, 10);
  return `schedules_${date}.zip`;
}

export async function downloadScheduleJson(schedule: ScheduleInfo): Promise<void> {
  const schedulePayload = await api.getSchedule(schedule.schedule_id);
  const json = JSON.stringify(schedulePayload, null, 2);
  triggerJsonDownload(json, buildScheduleFilename(schedule));
}

export async function downloadAllSchedulesAsZip(schedules: ScheduleInfo[]): Promise<void> {
  const zip = new JSZip();
  const filenameCounters = new Map<string, number>();

  for (const schedule of schedules) {
    const schedulePayload = await api.getSchedule(schedule.schedule_id);
    const json = JSON.stringify(schedulePayload, null, 2);
    const filename = buildUniqueFilename(buildScheduleFilename(schedule), filenameCounters);
    zip.file(filename, json);
  }

  const zipBlob = await zip.generateAsync({ type: 'blob' });
  triggerBlobDownload(zipBlob, buildZipFilename());
}
