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

function normalizeScheduleName(name: string): string {
  return name.trim().toLowerCase();
}

// =============================================================================
// Icons
// =============================================================================

function SettingsIcon() {
  return (
    <svg
      className="h-8 w-8"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
      aria-hidden="true"
    >
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
    <svg
      className="h-4 w-4"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
      aria-hidden="true"
    >
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
    <svg
      className="h-4 w-4"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
      aria-hidden="true"
    >
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
    <svg
      className="h-4 w-4"
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
      aria-hidden="true"
    >
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
    </svg>
  );
}

// =============================================================================
// Subcomponents
// =============================================================================

interface DeleteConfirmDialogProps {
  schedules: ScheduleInfo[];
  onConfirm: () => void;
  onCancel: () => void;
  isDeleting: boolean;
}

function DeleteConfirmDialog({
  schedules,
  onConfirm,
  onCancel,
  isDeleting,
}: DeleteConfirmDialogProps) {
  const isBulkDelete = schedules.length > 1;
  const previewSchedules = schedules.slice(0, 5);
  const remainingCount = schedules.length - previewSchedules.length;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="mx-4 w-full max-w-md rounded-2xl border border-slate-700 bg-slate-800 p-6 shadow-2xl">
        <h3 className="mb-2 text-lg font-semibold text-white">
          {isBulkDelete ? 'Delete Schedules' : 'Delete Schedule'}
        </h3>
        <p className="mb-3 text-sm text-slate-300">
          {isBulkDelete
            ? `Are you sure you want to delete these ${schedules.length} schedules?`
            : 'Are you sure you want to delete this schedule?'}
        </p>
        {isBulkDelete ? (
          <div className="mb-6 rounded-lg bg-slate-900/60 px-3 py-2">
            <ul className="space-y-1 text-sm font-medium text-white">
              {previewSchedules.map((schedule) => (
                <li key={schedule.schedule_id}>{schedule.schedule_name}</li>
              ))}
              {remainingCount > 0 && (
                <li className="text-slate-400">
                  and {remainingCount} more schedule{remainingCount === 1 ? '' : 's'}
                </li>
              )}
            </ul>
          </div>
        ) : (
          <p className="mb-6 rounded-lg bg-slate-900/60 px-3 py-2 text-sm font-medium text-white">
            {schedules[0]?.schedule_name}
          </p>
        )}
        <p className="mb-6 text-xs text-red-400">
          This action cannot be undone. All associated data (blocks, analytics, validation results)
          will be permanently removed.
        </p>
        <div className="flex justify-end gap-3">
          <button
            type="button"
            onClick={onCancel}
            disabled={isDeleting}
            className="rounded-lg border border-slate-600 px-4 py-2 text-sm font-medium text-slate-300 transition-colors hover:bg-slate-700 hover:text-white disabled:opacity-50"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={onConfirm}
            disabled={isDeleting}
            className="flex items-center gap-2 rounded-lg bg-red-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-red-700 disabled:opacity-50"
          >
            {isDeleting ? <LoadingSpinner size="sm" /> : <TrashIcon />}
            {isDeleting
              ? 'Deleting...'
              : `Delete ${schedules.length} schedule${schedules.length === 1 ? '' : 's'}`}
          </button>
        </div>
      </div>
    </div>
  );
}

interface EditScheduleDialogProps {
  schedule: ScheduleInfo;
  existingSchedules: ScheduleInfo[];
  onSave: (name: string, location?: GeographicLocation) => void;
  onCancel: () => void;
  isSaving: boolean;
}

function EditScheduleDialog({
  schedule,
  existingSchedules,
  onSave,
  onCancel,
  isSaving,
}: EditScheduleDialogProps) {
  const [name, setName] = useState(schedule.schedule_name);
  const [selectedSiteIdx, setSelectedSiteIdx] = useState<string>('');
  const normalizedName = normalizeScheduleName(name);
  const duplicateNameExists =
    normalizedName.length > 0 &&
    existingSchedules.some(
      (item) =>
        item.schedule_id !== schedule.schedule_id &&
        normalizeScheduleName(item.schedule_name) === normalizedName
    );

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const trimmedName = name.trim();
    if (!trimmedName) return;

    if (duplicateNameExists) {
      return;
    }

    const location =
      selectedSiteIdx !== ''
        ? OBSERVATORY_SITES[parseInt(selectedSiteIdx, 10)]?.location
        : undefined;

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

        <div className="mb-4">
          <label
            htmlFor="schedule-name"
            className="mb-1.5 block text-sm font-medium text-slate-300"
          >
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
          {duplicateNameExists && (
            <p className="mt-1 text-xs text-red-400" role="alert">
              A schedule with this name already exists.
            </p>
          )}
        </div>

        <div className="mb-6">
          <label
            htmlFor="schedule-site"
            className="mb-1.5 block text-sm font-medium text-slate-300"
          >
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
            disabled={isSaving || !name.trim() || duplicateNameExists}
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
  isSelected: boolean;
  onToggleSelect: (scheduleId: number) => void;
  onEdit: (schedule: ScheduleInfo) => void;
  onDelete: (schedule: ScheduleInfo) => void;
  onOpen: (scheduleId: number) => void;
}

function ScheduleRow({
  schedule,
  isSelected,
  onToggleSelect,
  onEdit,
  onDelete,
  onOpen,
}: ScheduleRowProps) {
  return (
    <div className="group flex items-center justify-between rounded-lg border border-slate-700/30 bg-slate-900/40 p-4 transition-all duration-200 hover:border-slate-600/50 hover:bg-slate-900/70">
      <div className="flex min-w-0 flex-1 items-center gap-3">
        <input
          type="checkbox"
          checked={isSelected}
          onChange={() => onToggleSelect(schedule.schedule_id)}
          onClick={(event) => event.stopPropagation()}
          className="h-4 w-4 rounded border-slate-500 bg-slate-700 text-red-600 focus:ring-red-500"
          aria-label={`Select ${schedule.schedule_name}`}
        />
        <button
          type="button"
          onClick={() => onOpen(schedule.schedule_id)}
          className="min-w-0 flex-1 text-left focus:outline-none"
        >
          <p className="truncate font-medium text-white transition-colors group-hover:text-indigo-300">
            {schedule.schedule_name}
          </p>
        </button>
      </div>

      <div className="ml-4 flex shrink-0 items-center gap-2">
        <button
          type="button"
          onClick={() => onEdit(schedule)}
          className="rounded-lg p-2 text-slate-400 transition-colors hover:bg-slate-700 hover:text-indigo-400 focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
          title="Edit schedule"
          aria-label={`Edit ${schedule.schedule_name}`}
        >
          <PencilIcon />
        </button>
        <button
          type="button"
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

  const schedules = data?.schedules ?? [];
  const [editingSchedule, setEditingSchedule] = useState<ScheduleInfo | null>(null);
  const [deleteTargetIds, setDeleteTargetIds] = useState<number[] | null>(null);
  const [selectedScheduleIds, setSelectedScheduleIds] = useState<Set<number>>(new Set());
  const [feedback, setFeedback] = useState<{ type: 'success' | 'error'; message: string } | null>(
    null
  );

  const selectedSchedules = schedules.filter((schedule) =>
    selectedScheduleIds.has(schedule.schedule_id)
  );
  const allSchedulesSelected = schedules.length > 0 && selectedScheduleIds.size === schedules.length;
  const deleteTargets =
    deleteTargetIds === null
      ? []
      : schedules.filter((schedule) => deleteTargetIds.includes(schedule.schedule_id));

  const toggleScheduleSelection = (scheduleId: number) => {
    setSelectedScheduleIds((prev) => {
      const next = new Set(prev);
      if (next.has(scheduleId)) {
        next.delete(scheduleId);
      } else {
        next.add(scheduleId);
      }
      return next;
    });
  };

  const handleSelectAll = () => {
    setSelectedScheduleIds(allSchedulesSelected ? new Set() : new Set(schedules.map((s) => s.schedule_id)));
  };

  const openDeleteDialogForIds = (scheduleIds: number[]) => {
    if (scheduleIds.length === 0) return;
    setDeleteTargetIds(scheduleIds);
  };

  const handleDelete = async () => {
    if (!deleteTargetIds || deleteTargetIds.length === 0) return;

    const deletedIds = new Set<number>();
    let failedScheduleName: string | null = null;

    try {
      for (const schedule of schedules) {
        if (!deleteTargetIds.includes(schedule.schedule_id)) {
          continue;
        }

        await deleteSchedule.mutateAsync(schedule.schedule_id);
        deletedIds.add(schedule.schedule_id);
      }

      setSelectedScheduleIds((prev) => {
        const next = new Set(prev);
        for (const deletedId of deletedIds) {
          next.delete(deletedId);
        }
        return next;
      });
      setFeedback({
        type: 'success',
        message: `Deleted ${deletedIds.size} schedule${deletedIds.size === 1 ? '' : 's'} successfully.`,
      });
      setDeleteTargetIds(null);
    } catch (err) {
      failedScheduleName =
        schedules.find((schedule) => deleteTargetIds.includes(schedule.schedule_id) && !deletedIds.has(schedule.schedule_id))
          ?.schedule_name ?? null;

      setSelectedScheduleIds((prev) => {
        const next = new Set(prev);
        for (const deletedId of deletedIds) {
          next.delete(deletedId);
        }
        return next;
      });
      setDeleteTargetIds(null);

      const baseMessage =
        err instanceof Error ? err.message : 'Failed to delete the selected schedules';
      const failedPrefix = failedScheduleName
        ? `Deletion stopped at "${failedScheduleName}". `
        : 'Deletion stopped before the batch could complete. ';
      setFeedback({
        type: 'error',
        message: `${failedPrefix}${baseMessage}`,
      });
    }
  };

  const handleUpdate = async (name: string, location?: GeographicLocation) => {
    if (!editingSchedule) return;

    const trimmedName = name.trim();
    const duplicateNameExists = schedules.some(
      (schedule) =>
        schedule.schedule_id !== editingSchedule.schedule_id &&
        normalizeScheduleName(schedule.schedule_name) === normalizeScheduleName(trimmedName)
    );

    if (duplicateNameExists) {
      setFeedback({
        type: 'error',
        message: 'A schedule with this name already exists. Please choose a different name.',
      });
      return;
    }

    try {
      const request: { name?: string; location?: GeographicLocation } = {};
      if (trimmedName !== editingSchedule.schedule_name.trim()) {
        request.name = trimmedName;
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
        message: `Schedule "${trimmedName}" updated successfully.`,
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

  return (
    <div className="relative min-h-screen overflow-hidden">
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

      <div className="relative z-10 mx-auto max-w-3xl px-4 py-16 sm:px-6 sm:py-24 lg:px-8">
        <button
          type="button"
          onClick={() => navigate('/')}
          className="mb-8 flex items-center gap-1.5 text-sm text-slate-400 transition-colors hover:text-white"
        >
          <ChevronLeftIcon />
          Back to Home
        </button>

        <div className="mb-10 flex items-start gap-4">
          <div className="rounded-xl bg-indigo-500/10 p-3 text-indigo-400">
            <SettingsIcon />
          </div>
          <div>
            <h1 className="text-3xl font-bold text-white">Manage Schedules</h1>
            <p className="mt-1 text-sm text-slate-400">
              Edit schedule metadata, change observatory location, or remove schedules from the
              database.
            </p>
          </div>
        </div>

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
              type="button"
              onClick={() => setFeedback(null)}
              className="ml-3 text-xs underline opacity-70 hover:opacity-100"
            >
              dismiss
            </button>
          </div>
        )}

        <div className="rounded-2xl border border-slate-700/50 bg-slate-800/50 p-6 backdrop-blur-sm">
          {schedules.length === 0 ? (
            <div className="py-12 text-center">
              <p className="text-sm text-slate-400">No schedules in the database.</p>
              <p className="mt-1 text-xs text-slate-500">
                Upload one from the home page to get started.
              </p>
            </div>
          ) : (
            <>
              <div className="mb-4 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                <div className="flex flex-wrap items-center gap-3">
                  <p className="text-sm text-slate-400">
                    {schedules.length} {schedules.length === 1 ? 'schedule' : 'schedules'}
                  </p>
                  <label className="flex items-center gap-2 text-sm text-slate-300">
                    <input
                      type="checkbox"
                      checked={allSchedulesSelected}
                      onChange={handleSelectAll}
                      className="h-4 w-4 rounded border-slate-500 bg-slate-700 text-red-600 focus:ring-red-500"
                      aria-label="Select all schedules"
                    />
                    Select all
                  </label>
                  {selectedSchedules.length > 0 && (
                    <span className="rounded-full bg-red-500/10 px-2.5 py-1 text-xs font-medium text-red-300">
                      {selectedSchedules.length} selected
                    </span>
                  )}
                </div>
                <div className="flex items-center gap-2">
                  {selectedSchedules.length > 0 && (
                    <button
                      type="button"
                      onClick={() => setSelectedScheduleIds(new Set())}
                      className="rounded-lg border border-slate-600 px-3 py-2 text-xs font-medium text-slate-300 transition-colors hover:bg-slate-700 hover:text-white"
                    >
                      Clear selection
                    </button>
                  )}
                  <button
                    type="button"
                    onClick={() => openDeleteDialogForIds(selectedSchedules.map((s) => s.schedule_id))}
                    disabled={selectedSchedules.length === 0}
                    className="inline-flex items-center gap-2 rounded-lg bg-red-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-red-700 disabled:cursor-not-allowed disabled:opacity-50"
                  >
                    <TrashIcon />
                    Delete selected
                  </button>
                </div>
              </div>
              <div className="space-y-2">
                {schedules.map((schedule) => (
                  <ScheduleRow
                    key={schedule.schedule_id}
                    schedule={schedule}
                    isSelected={selectedScheduleIds.has(schedule.schedule_id)}
                    onToggleSelect={toggleScheduleSelection}
                    onEdit={setEditingSchedule}
                    onDelete={(item) => openDeleteDialogForIds([item.schedule_id])}
                    onOpen={handleOpen}
                  />
                ))}
              </div>
            </>
          )}
        </div>
      </div>

      {deleteTargetIds && deleteTargets.length > 0 && (
        <DeleteConfirmDialog
          schedules={deleteTargets}
          onConfirm={handleDelete}
          onCancel={() => setDeleteTargetIds(null)}
          isDeleting={deleteSchedule.isPending}
        />
      )}

      {editingSchedule && (
        <EditScheduleDialog
          schedule={editingSchedule}
          existingSchedules={schedules}
          onSave={handleUpdate}
          onCancel={() => setEditingSchedule(null)}
          isSaving={updateSchedule.isPending}
        />
      )}
    </div>
  );
}

export default ScheduleManagement;
