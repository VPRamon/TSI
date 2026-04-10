/**
 * Schedule Management page.
 * Allows editing schedule metadata (name, observatory location) and deleting schedules.
 */
import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useSchedules, useDeleteSchedule, useUpdateSchedule } from '@/hooks';
import { LoadingSpinner, ErrorMessage } from '@/components';
import type { ScheduleInfo, GeographicLocation } from '@/api/types';
import { OBSERVATORY_SITES, formatSiteLabel } from '@/constants';

// =============================================================================
// Icons
// =============================================================================

function SettingsIcon() {
  return (
    <svg className="h-8 w-8" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
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
  );
}

function TrashIcon() {
  return (
    <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
      />
    </svg>
  );
}

function PencilIcon() {
  return (
    <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
      />
    </svg>
  );
}

function ChevronLeftIcon() {
  return (
    <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
    </svg>
  );
}

// =============================================================================
// Subcomponents
// =============================================================================

interface DeleteConfirmDialogProps {
  schedule: ScheduleInfo;
  onConfirm: () => void;
  onCancel: () => void;
  isDeleting: boolean;
}

function DeleteConfirmDialog({ schedule, onConfirm, onCancel, isDeleting }: DeleteConfirmDialogProps) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="mx-4 w-full max-w-md rounded-2xl border border-slate-700 bg-slate-800 p-6 shadow-2xl">
        <h3 className="mb-2 text-lg font-semibold text-white">Delete Schedule</h3>
        <p className="mb-1 text-sm text-slate-300">
          Are you sure you want to delete this schedule?
        </p>
        <p className="mb-6 rounded-lg bg-slate-900/60 px-3 py-2 text-sm font-medium text-white">
          {schedule.schedule_name}
          <span className="ml-2 text-xs text-slate-500">ID: {schedule.schedule_id}</span>
        </p>
        <p className="mb-6 text-xs text-red-400">
          This action cannot be undone. All associated data (blocks, analytics, validation results) will be permanently removed.
        </p>
        <div className="flex justify-end gap-3">
          <button
            onClick={onCancel}
            disabled={isDeleting}
            className="rounded-lg border border-slate-600 px-4 py-2 text-sm font-medium text-slate-300 transition-colors hover:bg-slate-700 hover:text-white disabled:opacity-50"
          >
            Cancel
          </button>
          <button
            onClick={onConfirm}
            disabled={isDeleting}
            className="flex items-center gap-2 rounded-lg bg-red-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-red-700 disabled:opacity-50"
          >
            {isDeleting ? <LoadingSpinner size="sm" /> : <TrashIcon />}
            {isDeleting ? 'Deleting...' : 'Delete'}
          </button>
        </div>
      </div>
    </div>
  );
}

interface EditScheduleDialogProps {
  schedule: ScheduleInfo;
  onSave: (name: string, location?: GeographicLocation) => void;
  onCancel: () => void;
  isSaving: boolean;
}

function EditScheduleDialog({ schedule, onSave, onCancel, isSaving }: EditScheduleDialogProps) {
  const [name, setName] = useState(schedule.schedule_name);
  const [selectedSiteIdx, setSelectedSiteIdx] = useState<string>('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const trimmedName = name.trim();
    if (!trimmedName) return;

    const location =
      selectedSiteIdx !== '' ? OBSERVATORY_SITES[parseInt(selectedSiteIdx, 10)]?.location : undefined;

    // Only submit if something actually changed
    const nameChanged = trimmedName !== schedule.schedule_name;
    if (!nameChanged && !location) {
      onCancel();
      return;
    }

    onSave(nameChanged ? trimmedName : schedule.schedule_name, location);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <form
        onSubmit={handleSubmit}
        className="mx-4 w-full max-w-md rounded-2xl border border-slate-700 bg-slate-800 p-6 shadow-2xl"
      >
        <h3 className="mb-6 text-lg font-semibold text-white">Edit Schedule</h3>

        {/* Schedule Name */}
        <div className="mb-4">
          <label htmlFor="schedule-name" className="mb-1.5 block text-sm font-medium text-slate-300">
            Schedule Name
          </label>
          <input
            id="schedule-name"
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="w-full rounded-lg border border-slate-600 bg-slate-900/60 px-3 py-2 text-sm text-white placeholder-slate-500 transition-colors focus:border-indigo-500 focus:outline-none focus:ring-1 focus:ring-indigo-500"
            placeholder="Enter schedule name"
            required
          />
        </div>

        {/* Observatory Location */}
        <div className="mb-6">
          <label htmlFor="schedule-site" className="mb-1.5 block text-sm font-medium text-slate-300">
            Observatory Location
          </label>
          <select
            id="schedule-site"
            value={selectedSiteIdx}
            onChange={(e) => setSelectedSiteIdx(e.target.value)}
            className="w-full rounded-lg border border-slate-600 bg-slate-900/60 px-3 py-2 text-sm text-white transition-colors focus:border-indigo-500 focus:outline-none focus:ring-1 focus:ring-indigo-500"
          >
            <option value="">Keep current location</option>
            {OBSERVATORY_SITES.map((site, idx) => (
              <option key={idx} value={String(idx)}>
                {formatSiteLabel(site)}
              </option>
            ))}
          </select>
          <p className="mt-1 text-xs text-slate-500">
            Changing the location updates the stored observatory coordinates.
          </p>
        </div>

        <div className="flex justify-end gap-3">
          <button
            type="button"
            onClick={onCancel}
            disabled={isSaving}
            className="rounded-lg border border-slate-600 px-4 py-2 text-sm font-medium text-slate-300 transition-colors hover:bg-slate-700 hover:text-white disabled:opacity-50"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={isSaving || !name.trim()}
            className="flex items-center gap-2 rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-indigo-700 disabled:opacity-50"
          >
            {isSaving ? <LoadingSpinner size="sm" /> : <PencilIcon />}
            {isSaving ? 'Saving...' : 'Save Changes'}
          </button>
        </div>
      </form>
    </div>
  );
}

interface ScheduleRowProps {
  schedule: ScheduleInfo;
  onEdit: (schedule: ScheduleInfo) => void;
  onDelete: (schedule: ScheduleInfo) => void;
  onOpen: (scheduleId: number) => void;
}

function ScheduleRow({ schedule, onEdit, onDelete, onOpen }: ScheduleRowProps) {
  return (
    <div className="group flex items-center justify-between rounded-lg border border-slate-700/30 bg-slate-900/40 p-4 transition-all duration-200 hover:border-slate-600/50 hover:bg-slate-900/70">
      <button
        onClick={() => onOpen(schedule.schedule_id)}
        className="min-w-0 flex-1 text-left focus:outline-none"
      >
        <p className="truncate font-medium text-white transition-colors group-hover:text-indigo-300">
          {schedule.schedule_name}
        </p>
        <p className="mt-0.5 text-xs text-slate-500">ID: {schedule.schedule_id}</p>
      </button>

      <div className="ml-4 flex shrink-0 items-center gap-2">
        <button
          onClick={() => onEdit(schedule)}
          className="rounded-lg p-2 text-slate-400 transition-colors hover:bg-slate-700 hover:text-indigo-400 focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
          title="Edit schedule"
          aria-label={`Edit ${schedule.schedule_name}`}
        >
          <PencilIcon />
        </button>
        <button
          onClick={() => onDelete(schedule)}
          className="rounded-lg p-2 text-slate-400 transition-colors hover:bg-slate-700 hover:text-red-400 focus:outline-none focus:ring-2 focus:ring-red-500/50"
          title="Delete schedule"
          aria-label={`Delete ${schedule.schedule_name}`}
        >
          <TrashIcon />
        </button>
      </div>
    </div>
  );
}

// =============================================================================
// Main Component
// =============================================================================

function ScheduleManagement() {
  const navigate = useNavigate();
  const { data, isLoading, error, refetch } = useSchedules();
  const deleteSchedule = useDeleteSchedule();
  const updateSchedule = useUpdateSchedule();

  const [editingSchedule, setEditingSchedule] = useState<ScheduleInfo | null>(null);
  const [deletingSchedule, setDeletingSchedule] = useState<ScheduleInfo | null>(null);
  const [feedback, setFeedback] = useState<{ type: 'success' | 'error'; message: string } | null>(null);

  const handleDelete = async () => {
    if (!deletingSchedule) return;
    try {
      await deleteSchedule.mutateAsync(deletingSchedule.schedule_id);
      setFeedback({
        type: 'success',
        message: `Schedule "${deletingSchedule.schedule_name}" deleted successfully.`,
      });
      setDeletingSchedule(null);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to delete schedule';
      setFeedback({ type: 'error', message });
    }
  };

  const handleUpdate = async (name: string, location?: GeographicLocation) => {
    if (!editingSchedule) return;
    try {
      const request: { name?: string; location?: GeographicLocation } = {};
      if (name !== editingSchedule.schedule_name) {
        request.name = name;
      }
      if (location) {
        request.location = location;
      }

      await updateSchedule.mutateAsync({
        scheduleId: editingSchedule.schedule_id,
        request,
      });
      setFeedback({
        type: 'success',
        message: `Schedule "${name}" updated successfully.`,
      });
      setEditingSchedule(null);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to update schedule';
      setFeedback({ type: 'error', message });
    }
  };

  const handleOpen = (scheduleId: number) => {
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

  const schedules = data?.schedules ?? [];

  return (
    <div className="relative min-h-screen overflow-hidden">
      {/* Background */}
      <div className="absolute inset-0 bg-gradient-to-b from-slate-950 via-slate-900 to-slate-950" aria-hidden="true" />
      <div
        className="absolute inset-0 opacity-30"
        style={{
          backgroundImage: 'radial-gradient(circle at 2px 2px, rgb(148 163 184 / 0.15) 1px, transparent 0)',
          backgroundSize: '32px 32px',
        }}
        aria-hidden="true"
      />

      {/* Content */}
      <div className="relative z-10 mx-auto max-w-3xl px-4 py-16 sm:px-6 sm:py-24 lg:px-8">
        {/* Back to Home */}
        <button
          onClick={() => navigate('/')}
          className="mb-8 flex items-center gap-1.5 text-sm text-slate-400 transition-colors hover:text-white"
        >
          <ChevronLeftIcon />
          Back to Home
        </button>

        {/* Header */}
        <div className="mb-10 flex items-start gap-4">
          <div className="rounded-xl bg-indigo-500/10 p-3 text-indigo-400">
            <SettingsIcon />
          </div>
          <div>
            <h1 className="text-3xl font-bold text-white">Manage Schedules</h1>
            <p className="mt-1 text-sm text-slate-400">
              Edit schedule metadata, change observatory location, or remove schedules from the database.
            </p>
          </div>
        </div>

        {/* Feedback toast */}
        {feedback && (
          <div
            className={`mb-6 rounded-lg border px-4 py-3 text-sm ${
              feedback.type === 'success'
                ? 'border-emerald-500/40 bg-emerald-500/10 text-emerald-200'
                : 'border-red-500/40 bg-red-500/10 text-red-200'
            }`}
            role="alert"
          >
            {feedback.message}
            <button
              onClick={() => setFeedback(null)}
              className="ml-3 text-xs underline opacity-70 hover:opacity-100"
            >
              dismiss
            </button>
          </div>
        )}

        {/* Schedule List */}
        <div className="rounded-2xl border border-slate-700/50 bg-slate-800/50 p-6 backdrop-blur-sm">
          {schedules.length === 0 ? (
            <div className="py-12 text-center">
              <p className="text-sm text-slate-400">No schedules in the database.</p>
              <p className="mt-1 text-xs text-slate-500">Upload one from the home page to get started.</p>
            </div>
          ) : (
            <>
              <div className="mb-4 flex items-center justify-between">
                <p className="text-sm text-slate-400">
                  {schedules.length} {schedules.length === 1 ? 'schedule' : 'schedules'}
                </p>
              </div>
              <div className="space-y-2">
                {schedules.map((schedule) => (
                  <ScheduleRow
                    key={schedule.schedule_id}
                    schedule={schedule}
                    onEdit={setEditingSchedule}
                    onDelete={setDeletingSchedule}
                    onOpen={handleOpen}
                  />
                ))}
              </div>
            </>
          )}
        </div>
      </div>

      {/* Dialogs */}
      {deletingSchedule && (
        <DeleteConfirmDialog
          schedule={deletingSchedule}
          onConfirm={handleDelete}
          onCancel={() => setDeletingSchedule(null)}
          isDeleting={deleteSchedule.isPending}
        />
      )}

      {editingSchedule && (
        <EditScheduleDialog
          schedule={editingSchedule}
          onSave={handleUpdate}
          onCancel={() => setEditingSchedule(null)}
          isSaving={updateSchedule.isPending}
        />
      )}
    </div>
  );
}

export default ScheduleManagement;
