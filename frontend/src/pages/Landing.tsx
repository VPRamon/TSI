/**
 * Landing page - Schedule list and upload.
 * Redesigned for minimal, modern aesthetic with space theme.
 */
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useSchedules } from '@/hooks';
import { LoadingSpinner, ErrorMessage } from '@/components';
import { LandingHeader, UploadScheduleCard, ScheduleListCard } from '@/features/schedules';

function Landing() {
  const navigate = useNavigate();
  const { data, isLoading, error, refetch } = useSchedules();
  const [pageError, setPageError] = useState('');

  const handleScheduleClick = (scheduleId: number) => {
    // Verify schedule exists in the current list before navigating
    const scheduleExists = data?.schedules?.some((s) => s.schedule_id === scheduleId);
    if (!scheduleExists) {
      setPageError(`Schedule ${scheduleId} not found. Please refresh the page.`);
      refetch();
      return;
    }
    navigate(`/schedules/${scheduleId}/validation`);
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
          />
        </div>
      </div>
    </div>
  );
}

export default Landing;
