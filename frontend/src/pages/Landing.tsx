/**
 * Landing page - Schedule list and upload.
 * Redesigned for minimal, modern aesthetic with space theme.
 */
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import type { ScheduleInfo } from '@/api/types';
import { useSchedules } from '@/hooks';
import { LoadingSpinner, ErrorMessage } from '@/components';
import {
  LandingHeader,
  UploadScheduleCard,
  ScheduleListCard,
  downloadAllSchedulesAsZip,
  downloadScheduleJson,
} from '@/features/schedules';

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
      await downloadScheduleJson(schedule);
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
      await downloadAllSchedulesAsZip(schedules);
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
            onOpenAdvanced={() => navigate('/advanced')}
            downloadingScheduleIds={downloadingScheduleIds}
            isDownloadingAll={isDownloadingAll}
          />
        </div>
      </div>
    </div>
  );
}

export default Landing;
