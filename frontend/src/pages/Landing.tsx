/**
 * Landing page - Schedule list and upload.
 * Redesigned for minimal, modern aesthetic with space theme.
 */
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import JSZip from 'jszip';
import { api } from '@/api';
import type { ScheduleInfo } from '@/api/types';
import { useSchedules } from '@/hooks';
import { LoadingSpinner, ErrorMessage } from '@/components';
import { LandingHeader, UploadScheduleCard, ScheduleListCard } from '@/features/schedules';

function buildScheduleFilename(schedule: ScheduleInfo): string {
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

function triggerJsonDownload(content: string, filename: string): void {
  const blob = new Blob([content], { type: 'application/json' });
  triggerBlobDownload(blob, filename);
}

function buildZipFilename(): string {
  const date = new Date().toISOString().slice(0, 10);
  return `schedules_${date}.zip`;
}

function Landing() {
  const navigate = useNavigate();
  const { data, isLoading, error, refetch } = useSchedules();
  const [pageError, setPageError] = useState('');
  const [downloadingScheduleIds, setDownloadingScheduleIds] = useState<Set<number>>(new Set());
  const [isDownloadingAll, setIsDownloadingAll] = useState(false);

  const handleScheduleClick = (scheduleId: number) => {
    // Verify schedule exists in the current list before navigating
    const scheduleExists = data?.schedules?.some((s) => s.schedule_id === scheduleId);
    if (!scheduleExists) {
      setPageError('Selected schedule was not found. Please refresh the page.');
      refetch();
      return;
    }
    navigate(`/schedules/${scheduleId}/validation`);
  };

  const handleScheduleDownload = async (schedule: ScheduleInfo) => {
    // Verify schedule still exists before requesting full payload.
    const scheduleExists = data?.schedules?.some((s) => s.schedule_id === schedule.schedule_id);
    if (!scheduleExists) {
      setPageError('Selected schedule was not found. Please refresh the page.');
      refetch();
      return;
    }

    setPageError('');
    setDownloadingScheduleIds((prev) => {
      const next = new Set(prev);
      next.add(schedule.schedule_id);
      return next;
    });

    try {
      const schedulePayload = await api.getSchedule(schedule.schedule_id);
      const filename = buildScheduleFilename(schedule);
      const json = JSON.stringify(schedulePayload, null, 2);
      triggerJsonDownload(json, filename);
    } catch {
      setPageError('Failed to download schedule JSON. Please try again.');
    } finally {
      setDownloadingScheduleIds((prev) => {
        const next = new Set(prev);
        next.delete(schedule.schedule_id);
        return next;
      });
    }
  };

  const handleDownloadAll = async () => {
    const schedules = data?.schedules ?? [];
    if (schedules.length === 0) {
      return;
    }

    setPageError('');
    setIsDownloadingAll(true);

    const scheduleIds = schedules.map((schedule) => schedule.schedule_id);
    setDownloadingScheduleIds((prev) => {
      const next = new Set(prev);
      scheduleIds.forEach((id) => next.add(id));
      return next;
    });

    try {
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
    } catch {
      setPageError('Failed to download all schedules. Please try again.');
    } finally {
      setIsDownloadingAll(false);
      setDownloadingScheduleIds((prev) => {
        const next = new Set(prev);
        scheduleIds.forEach((id) => next.delete(id));
        return next;
      });
    }
  };

  if (isLoading) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex min-h-screen items-center justify-center p-4">
        <ErrorMessage
          title="Failed to load schedules"
          message={(error as Error).message}
          onRetry={() => refetch()}
        />
      </div>
    );
  }

  return (
    <div className="relative min-h-screen overflow-hidden">
      {/* Subtle space-themed background */}
      <div
        className="absolute inset-0 bg-gradient-to-b from-slate-950 via-slate-900 to-slate-950"
        aria-hidden="true"
      />
      <div
        className="absolute inset-0 opacity-30"
        style={{
          backgroundImage:
            'radial-gradient(circle at 2px 2px, rgb(148 163 184 / 0.15) 1px, transparent 0)',
          backgroundSize: '32px 32px',
        }}
        aria-hidden="true"
      />

      {/* Content */}
      <div className="relative z-10 mx-auto max-w-5xl px-4 py-16 sm:px-6 sm:py-24 lg:px-8">
        <LandingHeader />

        {/* Page-level error */}
        {pageError && (
          <div
            className="mx-auto mb-8 max-w-md rounded-lg border border-red-500/40 bg-red-500/10 px-4 py-3 text-center text-sm text-red-200"
            role="alert"
          >
            {pageError}
          </div>
        )}

        {/* Primary Action Cards */}
        <div className="mb-12 grid grid-cols-1 gap-6 lg:grid-cols-2 lg:gap-8">
          <UploadScheduleCard onError={(msg) => setPageError(msg)} />

          <ScheduleListCard
            schedules={data?.schedules ?? []}
            total={data?.total ?? 0}
            onScheduleClick={handleScheduleClick}
            onScheduleDownload={handleScheduleDownload}
            onDownloadAll={handleDownloadAll}
            onManageSchedules={() => navigate('/manage')}
            downloadingScheduleIds={downloadingScheduleIds}
            isDownloadingAll={isDownloadingAll}
          />
        </div>
      </div>
    </div>
  );
}

export default Landing;
