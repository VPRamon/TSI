/**
 * A drop-zone + file list used by both create-env and add-schedules dialogs.
 *
 * Accepts paired schedule (`.json`) and trace (`.jsonl`) files. The naming
 * convention is `<stem>.json` ↔ `<stem>.<algo>_trace.jsonl` (e.g.
 * `e1-k1-b1.json` and `e1-k1-b1.est_trace.jsonl`); when a trace lacks the
 * `_trace` suffix the bare stem is used. Dropped traces without a matching
 * schedule are listed as orphans.
 */
import { useCallback, useRef, useState } from 'react';
import { LoadingSpinner } from '@/components';
import { prepareUploadFiles, type PreparedUploadEntry } from '../utils';

/**
 * One row in the drop-zone list. `item` is non-null when the schedule
 * parsed cleanly and is ready to submit; `error` carries the reason
 * otherwise.
 */
export interface PreparedFile {
  id: string;
  schedule: File;
  trace?: File;
  item: PreparedUploadEntry['item'];
  error?: string;
}

interface ScheduleDropZoneProps {
  /** Currently-prepared files (controlled by parent). */
  files: PreparedFile[];
  onChange: (files: PreparedFile[]) => void;
  /** Trace files dropped without a matching schedule (controlled). */
  orphanTraces?: File[];
  onOrphanTracesChange?: (orphans: File[]) => void;
  disabled?: boolean;
}

let nextId = 0;
const makeId = (name: string) => `${name}-${++nextId}`;

function toPreparedFiles(entries: PreparedUploadEntry[]): PreparedFile[] {
  return entries.map((e) => ({
    id: makeId(e.schedule.name),
    schedule: e.schedule,
    trace: e.trace,
    item: e.item,
    error: e.error,
  }));
}

export function ScheduleDropZone({
  files,
  onChange,
  orphanTraces = [],
  onOrphanTracesChange,
  disabled,
}: ScheduleDropZoneProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const traceInputRef = useRef<HTMLInputElement>(null);
  const [isDragOver, setIsDragOver] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);
  // Which existing entry a `+ trace` click is targeting. Held in a ref so
  // the hidden file input's onChange can resolve it without a re-render.
  const pendingTraceTargetRef = useRef<string | null>(null);

  const ingest = useCallback(
    async (incoming: FileList | File[]) => {
      setIsProcessing(true);
      const result = await prepareUploadFiles(Array.from(incoming));
      // Pair freshly-dropped traces against any existing un-paired
      // schedules before publishing orphans, so a two-step drop
      // (schedules first, traces second) still pairs up.
      const newOrphanByStem = new Map<string, File>();
      for (const t of result.orphanTraces) {
        const stem = t.name.replace(/\.jsonl$/i, '').replace(/\.[A-Za-z0-9]+_trace$/i, '');
        newOrphanByStem.set(stem, t);
      }

      const patchedExisting = await Promise.all(
        files.map(async (f) => {
          if (f.trace || !f.item) return f;
          const stem = f.schedule.name.replace(/\.json$/i, '');
          const trace = newOrphanByStem.get(stem);
          if (!trace) return f;
          newOrphanByStem.delete(stem);
          const text = typeof trace.text === 'function' ? await trace.text() : '';
          return {
            ...f,
            trace,
            item: { ...f.item, algorithm_trace_jsonl: text },
          };
        })
      );

      onChange([...patchedExisting, ...toPreparedFiles(result.entries)]);
      onOrphanTracesChange?.([...orphanTraces, ...Array.from(newOrphanByStem.values())]);
      setIsProcessing(false);
    },
    [files, onChange, orphanTraces, onOrphanTracesChange]
  );

  const remove = (id: string) => {
    onChange(files.filter((f) => f.id !== id));
  };

  const removeOrphan = (idx: number) => {
    onOrphanTracesChange?.(orphanTraces.filter((_, i) => i !== idx));
  };

  const attachTrace = async (id: string, file: File) => {
    if (!file.name.toLowerCase().endsWith('.jsonl')) return;
    const text = typeof file.text === 'function' ? await file.text() : '';
    onChange(
      files.map((f) => {
        if (f.id !== id || !f.item) return f;
        return {
          ...f,
          trace: file,
          item: { ...f.item, algorithm_trace_jsonl: text },
        };
      })
    );
  };

  return (
    <div>
      <div
        onDragOver={(e) => {
          if (disabled) return;
          e.preventDefault();
          setIsDragOver(true);
        }}
        onDragLeave={() => setIsDragOver(false)}
        onDrop={(e) => {
          if (disabled) return;
          e.preventDefault();
          setIsDragOver(false);
          void ingest(e.dataTransfer.files);
        }}
        onClick={() => !disabled && inputRef.current?.click()}
        className={`flex cursor-pointer flex-col items-center justify-center gap-2 rounded-xl border-2 border-dashed px-6 py-6 text-center transition-all ${
          isDragOver
            ? 'border-sky-400 bg-sky-500/10 text-sky-300'
            : 'border-slate-600 text-slate-400 hover:border-sky-500/50 hover:text-slate-300'
        } ${disabled ? 'cursor-not-allowed opacity-50' : ''}`}
        role="button"
        tabIndex={0}
        aria-label="Drop schedule JSON files (and optional .jsonl traces) here or click to browse"
      >
        <p className="text-sm font-medium">
          {isProcessing
            ? 'Reading files…'
            : isDragOver
              ? 'Drop files here'
              : 'Drop schedule .json (and optional .jsonl trace) files here or click to browse'}
        </p>
        <p className="text-xs opacity-70">
          Multiple files supported · pairs by name (<code>plan.json</code> ↔{' '}
          <code>plan.&lt;algorithm&gt;_trace.jsonl</code>)
        </p>
      </div>

      <input
        ref={inputRef}
        type="file"
        accept=".json,application/json,.jsonl,application/jsonl,application/x-jsonlines"
        multiple
        className="sr-only"
        onChange={(e) => {
          if (e.target.files) void ingest(e.target.files);
          e.target.value = '';
        }}
      />

      <input
        ref={traceInputRef}
        type="file"
        accept=".jsonl,application/jsonl,application/x-jsonlines"
        className="sr-only"
        onChange={(e) => {
          const f = e.target.files?.[0];
          const target = pendingTraceTargetRef.current;
          pendingTraceTargetRef.current = null;
          e.target.value = '';
          if (f && target) void attachTrace(target, f);
        }}
      />

      {files.length > 0 && (
        <ul className="mt-3 space-y-1.5" data-testid="prepared-file-list">
          {files.map((f) => {
            const ready = f.item !== null && !f.error;
            return (
              <li
                key={f.id}
                className={`flex items-center gap-2 rounded-lg border px-3 py-2 text-sm ${
                  f.error
                    ? 'border-red-700/60 bg-red-950/30 text-red-200'
                    : 'border-slate-700 bg-slate-800/40 text-slate-200'
                }`}
              >
                <span className="flex-1 truncate">{f.schedule.name}</span>

                {ready && (
                  <span className="rounded bg-emerald-900/40 px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-emerald-200">
                    ready
                  </span>
                )}
                {f.error && (
                  <span className="rounded bg-red-900/60 px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-red-100">
                    error
                  </span>
                )}

                {f.trace ? (
                  <span
                    title={f.trace.name}
                    className="rounded bg-sky-900/40 px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-sky-200"
                    data-testid={`trace-paired-${f.id}`}
                  >
                    ✓ trace
                  </span>
                ) : ready && !disabled ? (
                  <button
                    type="button"
                    onClick={(e) => {
                      e.stopPropagation();
                      pendingTraceTargetRef.current = f.id;
                      traceInputRef.current?.click();
                    }}
                    className="rounded border border-slate-600 px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-slate-300 hover:border-sky-500/60 hover:text-sky-200"
                    aria-label={`Attach trace for ${f.schedule.name}`}
                  >
                    + trace
                  </button>
                ) : null}

                {f.error && <span className="text-xs">{f.error}</span>}

                {!disabled && (
                  <button
                    type="button"
                    onClick={() => remove(f.id)}
                    className="text-slate-400 hover:text-red-400"
                    aria-label={`Remove ${f.schedule.name}`}
                  >
                    ×
                  </button>
                )}
              </li>
            );
          })}
        </ul>
      )}

      {orphanTraces.length > 0 && (
        <div
          className="mt-3 rounded-lg border border-amber-700/40 bg-amber-950/20 px-3 py-2 text-xs text-amber-200/90"
          data-testid="orphan-traces"
        >
          <p className="font-semibold text-amber-200">Trace files with no matching schedule:</p>
          <ul className="mt-1 space-y-0.5">
            {orphanTraces.map((t, i) => (
              <li key={`${t.name}-${i}`} className="flex items-center gap-2">
                <span className="truncate">{t.name}</span>
                {!disabled && (
                  <button
                    type="button"
                    onClick={() => removeOrphan(i)}
                    className="text-amber-300/70 hover:text-amber-100"
                    aria-label={`Remove orphan trace ${t.name}`}
                  >
                    ×
                  </button>
                )}
              </li>
            ))}
          </ul>
        </div>
      )}

      {isProcessing && (
        <div className="mt-2 flex items-center gap-2 text-xs text-slate-400">
          <LoadingSpinner size="sm" /> Reading…
        </div>
      )}
    </div>
  );
}
