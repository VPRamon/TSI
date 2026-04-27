/**
 * Dialog for adding more schedules to an existing environment.
 */
import { useState } from 'react';
import { LoadingSpinner } from '@/components';
import { useBulkImportToEnvironment } from '@/hooks';
import type { BulkImportResponse, EnvironmentInfo } from '@/api/types';
import { errorMessage } from '@/api/errors';
import { Modal } from './Modal';
import { ScheduleDropZone, type PreparedFile } from './ScheduleDropZone';
import { BulkImportResult } from './BulkImportResult';
import {
  BULK_IMPORT_PARALLEL_CHUNKS,
  chunkBulkImportItems,
  runWithConcurrency,
} from '../utils';

interface BulkUploadDialogProps {
  environment: EnvironmentInfo;
  onClose: () => void;
}

interface UploadProgress {
  index: number;
  total: number;
  currentName: string;
  inFlightChunks: number;
}

export function BulkUploadDialog({ environment, onClose }: BulkUploadDialogProps) {
  const bulkImport = useBulkImportToEnvironment();
  const [files, setFiles] = useState<PreparedFile[]>([]);
  const [orphanTraces, setOrphanTraces] = useState<File[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<BulkImportResponse | null>(null);
  const [progress, setProgress] = useState<UploadProgress | null>(null);

  const validFiles = files.filter((f) => f.item !== null);
  const inFlight = bulkImport.isPending || progress !== null;
  const canSubmit = validFiles.length > 0 && !inFlight && result === null;

  const handleSubmit = async () => {
    setError(null);
    const items = validFiles.map((f) => f.item!);

    // Backend parallelises items inside a single request, so we group
    // items into chunks (one HTTP request per chunk). We then run a
    // bounded number of chunks in flight at the same time. Each chunk
    // is awaited independently so a single chunk failure (network, 5xx,
    // timeout) cannot abort the rest: every item in a failed chunk is
    // surfaced as a `rejected` entry with reason `chunk_failed: <msg>`.
    const chunks = chunkBulkImportItems(items);
    const created: BulkImportResponse['created'] = [];
    const rejected: BulkImportResponse['rejected'] = [];
    const chunkErrors: string[] = [];
    let processed = 0;
    let inFlight = 0;
    const parallel = Math.min(BULK_IMPORT_PARALLEL_CHUNKS, chunks.length);

    const refreshProgress = (currentName: string) => {
      setProgress({
        index: processed,
        total: items.length,
        currentName,
        inFlightChunks: inFlight,
      });
    };

    try {
      refreshProgress(
        chunks.length === 1
          ? chunks[0][0].name
          : `${chunks.length} chunks (×${parallel} in flight)`,
      );
      await runWithConcurrency(chunks, parallel, async (chunk, c) => {
        inFlight += 1;
        refreshProgress(
          chunks.length === 1
            ? chunk[0].name
            : `chunk ${c + 1}/${chunks.length} (${chunk.length} items)`,
        );
        try {
          const resp = await bulkImport.mutateAsync({
            environmentId: environment.environment_id,
            request: { items: chunk },
          });
          created.push(...resp.created);
          rejected.push(...resp.rejected);
        } catch (err) {
          const msg = errorMessage(err);
          chunkErrors.push(`chunk ${c + 1}/${chunks.length}: ${msg}`);
          for (const item of chunk) {
            rejected.push({
              name: item.name,
              reason: `chunk_failed: ${msg}`,
              mismatch_fields: [],
            });
          }
        } finally {
          inFlight -= 1;
          processed += chunk.length;
          refreshProgress(
            chunks.length === 1
              ? chunk[0].name
              : `${processed}/${items.length} items`,
          );
        }
      });
      setResult({ created, rejected });
      if (chunkErrors.length > 0 && created.length === 0) {
        setError(chunkErrors.join('\n'));
      } else if (chunkErrors.length > 0) {
        setError(
          `${chunkErrors.length} chunk(s) failed; see rejected list below.`,
        );
      }
    } finally {
      setProgress(null);
    }
  };

  // Block close while an upload is in flight; otherwise the request
  // resolves into an unmounted component and the user can't see the
  // outcome.
  const handleClose = () => {
    if (inFlight) return;
    onClose();
  };

  return (
    <Modal
      title={`Add schedules to "${environment.name}"`}
      onClose={handleClose}
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
              onClick={handleClose}
              disabled={inFlight}
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
              {inFlight && <LoadingSpinner size="sm" />}
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
          {progress && (
            <UploadProgressBar progress={progress} />
          )}
          <ScheduleDropZone
            files={files}
            onChange={setFiles}
            orphanTraces={orphanTraces}
            onOrphanTracesChange={setOrphanTraces}
            disabled={inFlight}
          />
          {error && (
            <p
              className="rounded-lg border border-red-700/60 bg-red-950/30 px-3 py-2 text-sm text-red-200"
              role="alert"
            >
              {error}
            </p>
          )}
        </div>
      )}
    </Modal>
  );
}

function UploadProgressBar({ progress }: { progress: UploadProgress }) {
  const done = Math.min(progress.index, progress.total);
  const pct = progress.total > 0 ? Math.round((done / progress.total) * 100) : 0;
  return (
    <div
      className="rounded-lg border border-sky-700/40 bg-sky-950/20 px-3 py-2 text-xs text-sky-100"
      data-testid="upload-progress"
    >
      <div className="flex items-center justify-between gap-2">
        <span>
          Uploaded {done} / {progress.total} —{' '}
          <span className="font-mono">{progress.currentName}</span>
          {progress.inFlightChunks > 1 ? (
            <span className="ml-2 opacity-70">
              ({progress.inFlightChunks} chunks in flight)
            </span>
          ) : null}
        </span>
        <span className="opacity-70">{pct}%</span>
      </div>
      <div className="mt-1.5 h-1.5 overflow-hidden rounded bg-sky-900/40">
        <div
          className="h-full bg-sky-400 transition-all"
          style={{ width: `${pct}%` }}
          aria-valuenow={pct}
          aria-valuemin={0}
          aria-valuemax={100}
          role="progressbar"
        />
      </div>
    </div>
  );
}
