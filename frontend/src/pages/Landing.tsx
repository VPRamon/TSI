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

        {/* Manage Schedules Link */}
        {(data?.total ?? 0) > 0 && (
          <div className="text-center">
            <button
              onClick={() => navigate('/manage')}
              className="inline-flex items-center gap-2 rounded-lg border border-slate-700/50 bg-slate-800/50 px-4 py-2.5 text-sm text-slate-400 transition-all hover:border-slate-600 hover:bg-slate-800/80 hover:text-white"
            >
              <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={1.5}
                  d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                />
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={1.5}
                  d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                />
              </svg>
              Manage Schedules
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

export default Landing;
