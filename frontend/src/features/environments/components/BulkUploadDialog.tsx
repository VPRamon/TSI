/**
 * Dialog for adding more schedules to an existing environment.
 */
import { useState } from 'react';
import { LoadingSpinner } from '@/components';
import { useBulkImportToEnvironment } from '@/hooks';
import type { BulkImportResponse, EnvironmentInfo } from '@/api/types';
import { Modal } from './Modal';
import { ScheduleDropZone, type PreparedFile } from './ScheduleDropZone';
import { BulkImportResult } from './BulkImportResult';

interface BulkUploadDialogProps {
  environment: EnvironmentInfo;
  onClose: () => void;
}

export function BulkUploadDialog({ environment, onClose }: BulkUploadDialogProps) {
  const bulkImport = useBulkImportToEnvironment();
  const [files, setFiles] = useState<PreparedFile[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<BulkImportResponse | null>(null);

  const validFiles = files.filter((f) => f.item !== null);
  const canSubmit = validFiles.length > 0 && !bulkImport.isPending && result === null;

  const handleSubmit = async () => {
    setError(null);
    try {
      const resp = await bulkImport.mutateAsync({
        environmentId: environment.environment_id,
        request: { items: validFiles.map((f) => f.item!) },
      });
      setResult(resp);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to import schedules');
    }
  };

  return (
    <Modal
      title={`Add schedules to "${environment.name}"`}
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
              disabled={bulkImport.isPending}
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
              {bulkImport.isPending && <LoadingSpinner size="sm" />}
              Upload {validFiles.length || ''}
            </button>
          </>
        )
      }
    >
      {result ? (
        <BulkImportResult result={result} />
      ) : (
        <div className="space-y-3">
          <ScheduleDropZone files={files} onChange={setFiles} disabled={bulkImport.isPending} />
          {error && (
            <p className="rounded-lg border border-red-700/60 bg-red-950/30 px-3 py-2 text-sm text-red-200">
              {error}
            </p>
          )}
        </div>
      )}
    </Modal>
  );
}
