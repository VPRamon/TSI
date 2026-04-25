/**
 * Dialog for creating a new environment.
 *
 * Two phases: name + initial files, then a result panel showing the bulk
 * import outcome. Empty bulk uploads are allowed — the env will simply be
 * created with no structure yet.
 */
import { useState } from 'react';
import { LoadingSpinner } from '@/components';
import { useCreateEnvironment, useBulkImportToEnvironment } from '@/hooks';
import type { BulkImportResponse } from '@/api/types';
import { Modal } from './Modal';
import { ScheduleDropZone, type PreparedFile } from './ScheduleDropZone';
import { BulkImportResult } from './BulkImportResult';

interface CreateEnvironmentDialogProps {
  onClose: () => void;
}

export function CreateEnvironmentDialog({ onClose }: CreateEnvironmentDialogProps) {
  const createEnv = useCreateEnvironment();
  const bulkImport = useBulkImportToEnvironment();

  const [name, setName] = useState('');
  const [files, setFiles] = useState<PreparedFile[]>([]);
  const [submitError, setSubmitError] = useState<string | null>(null);
  const [result, setResult] = useState<BulkImportResponse | null>(null);

  const isSubmitting = createEnv.isPending || bulkImport.isPending;
  const validFiles = files.filter((f) => f.item !== null);
  const canSubmit = name.trim().length > 0 && !isSubmitting && result === null;

  const handleSubmit = async () => {
    setSubmitError(null);
    try {
      const env = await createEnv.mutateAsync({ name: name.trim() });
      if (validFiles.length === 0) {
        setResult({ created: [], rejected: [] });
        return;
      }
      const resp = await bulkImport.mutateAsync({
        environmentId: env.environment_id,
        request: { items: validFiles.map((f) => f.item!) },
      });
      setResult(resp);
    } catch (err) {
      setSubmitError(err instanceof Error ? err.message : 'Failed to create environment');
    }
  };

  return (
    <Modal
      title="Create environment"
      onClose={onClose}
      footer={
        result ? (
          <button
            type="button"
            onClick={onClose}
            className="rounded-lg bg-sky-600 px-4 py-1.5 text-sm font-semibold text-white hover:bg-sky-500"
          >
            Done
          </button>
        ) : (
          <>
            <button
              type="button"
              onClick={onClose}
              disabled={isSubmitting}
              className="rounded-lg px-3 py-1.5 text-sm text-slate-300 hover:text-white disabled:opacity-50"
            >
              Cancel
            </button>
            <button
              type="button"
              onClick={handleSubmit}
              disabled={!canSubmit}
              className="flex items-center gap-2 rounded-lg bg-sky-600 px-4 py-1.5 text-sm font-semibold text-white hover:bg-sky-500 disabled:cursor-not-allowed disabled:opacity-40"
            >
              {isSubmitting && <LoadingSpinner size="sm" />}
              Create
            </button>
          </>
        )
      }
    >
      {result ? (
        <BulkImportResult result={result} />
      ) : (
        <div className="space-y-4">
          <div>
            <label htmlFor="env-name" className="mb-1 block text-xs font-medium text-slate-400">
              Environment name
            </label>
            <input
              id="env-name"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              disabled={isSubmitting}
              placeholder="e.g. CTAO South — March 2025"
              className="w-full rounded-lg border border-slate-700 bg-slate-950 px-3 py-2 text-sm text-white outline-none focus:border-sky-500 disabled:opacity-50"
              autoFocus
            />
          </div>

          <div>
            <p className="mb-1.5 text-xs font-medium text-slate-400">
              Initial schedules (optional — defines the env structure)
            </p>
            <ScheduleDropZone files={files} onChange={setFiles} disabled={isSubmitting} />
          </div>

          {submitError && (
            <p className="rounded-lg border border-red-700/60 bg-red-950/30 px-3 py-2 text-sm text-red-200">
              {submitError}
            </p>
          )}
        </div>
      )}
    </Modal>
  );
}
