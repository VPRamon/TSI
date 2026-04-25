/**
 * Advanced page — environment management.
 *
 * Hidden from the navigation bar but mounted at /advanced. Lists every
 * environment as a card and provides:
 *   - "Create environment" with an optional bulk-import drop zone.
 *   - Per-card "Add schedules", "Open compare", "Unassign" and "Delete env".
 *
 * Environments group schedules that share the same observatory, schedule
 * period and block set. Mismatched files are surfaced inline.
 */
import { useState } from 'react';
import {
  useEnvironments,
  useDeleteEnvironment,
  useRemoveScheduleFromEnvironment,
} from '@/hooks';
import { ErrorMessage, LoadingSpinner, PageContainer, PageHeader } from '@/components';
import {
  BulkUploadDialog,
  CreateEnvironmentDialog,
  EnvironmentCard,
  Modal,
  sortEnvironmentsByRecency,
} from '@/features/environments';
import type { EnvironmentInfo } from '@/api/types';

function ConfirmDeleteDialog({
  environment,
  onConfirm,
  onCancel,
  isDeleting,
}: {
  environment: EnvironmentInfo;
  onConfirm: () => void;
  onCancel: () => void;
  isDeleting: boolean;
}) {
  return (
    <Modal
      title={`Delete "${environment.name}"?`}
      onClose={onCancel}
      footer={
        <>
          <button
            type="button"
            onClick={onCancel}
            disabled={isDeleting}
            className="rounded-lg px-3 py-1.5 text-sm text-slate-300 hover:text-white disabled:opacity-50"
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={onConfirm}
            disabled={isDeleting}
            className="flex items-center gap-2 rounded-lg bg-red-600 px-4 py-1.5 text-sm font-semibold text-white hover:bg-red-500 disabled:opacity-40"
          >
            {isDeleting && <LoadingSpinner size="sm" />}
            Delete
          </button>
        </>
      }
    >
      <p className="text-sm text-slate-300">
        This removes the environment and its cached preschedule data. The
        member schedules themselves are preserved and become unassigned.
      </p>
    </Modal>
  );
}

function AdvancedPage() {
  const { data, isLoading, error } = useEnvironments();
  const deleteEnv = useDeleteEnvironment();
  const removeSchedule = useRemoveScheduleFromEnvironment();

  const [showCreate, setShowCreate] = useState(false);
  const [bulkTarget, setBulkTarget] = useState<EnvironmentInfo | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<EnvironmentInfo | null>(null);

  const isMutating = deleteEnv.isPending || removeSchedule.isPending;

  const environments = sortEnvironmentsByRecency(data?.environments ?? []);

  const handleDelete = async () => {
    if (!deleteTarget) return;
    try {
      await deleteEnv.mutateAsync(deleteTarget.environment_id);
      setDeleteTarget(null);
    } catch {
      // Error toasts are out of scope; the dialog stays open.
    }
  };

  const handleRemoveSchedule = (scheduleId: number) => {
    removeSchedule.mutate(scheduleId);
  };

  if (isLoading) {
    return (
      <div className="flex min-h-[60vh] items-center justify-center">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4">
        <ErrorMessage title="Failed to load environments" message={(error as Error).message} />
      </div>
    );
  }

  return (
    <PageContainer className="gap-6">
      <PageHeader
        title="Environments"
        description="Group schedules that share the same observatory, schedule period and blocks. The first uploaded schedule defines the structure; subsequent uploads must match."
        actions={
          <button
            type="button"
            onClick={() => setShowCreate(true)}
            className="rounded-lg bg-sky-600 px-4 py-2 text-sm font-semibold text-white shadow hover:bg-sky-500"
          >
            + Create environment
          </button>
        }
      />

      {environments.length === 0 ? (
        <div className="rounded-2xl border border-dashed border-slate-700 bg-slate-900/50 px-6 py-16 text-center text-slate-400">
          No environments yet. Create one to start grouping comparable schedules.
        </div>
      ) : (
        <div
          className="grid gap-4 lg:grid-cols-2 xl:grid-cols-3"
          data-testid="environment-grid"
        >
          {environments.map((env) => (
            <EnvironmentCard
              key={env.environment_id}
              environment={env}
              onAddSchedules={() => setBulkTarget(env)}
              onRemoveSchedule={handleRemoveSchedule}
              onDelete={() => setDeleteTarget(env)}
              isMutating={isMutating}
            />
          ))}
        </div>
      )}

      {showCreate && <CreateEnvironmentDialog onClose={() => setShowCreate(false)} />}
      {bulkTarget && (
        <BulkUploadDialog environment={bulkTarget} onClose={() => setBulkTarget(null)} />
      )}
      {deleteTarget && (
        <ConfirmDeleteDialog
          environment={deleteTarget}
          onConfirm={handleDelete}
          onCancel={() => setDeleteTarget(null)}
          isDeleting={deleteEnv.isPending}
        />
      )}
    </PageContainer>
  );
}

export default AdvancedPage;
