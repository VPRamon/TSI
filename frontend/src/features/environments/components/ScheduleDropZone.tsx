/**
 * A drop-zone + file list used by both create-env and add-schedules dialogs.
 * Only handles file ingestion; the parent decides what to do with the items.
 */
import { useCallback, useRef, useState } from 'react';
import { LoadingSpinner } from '@/components';
import type { BulkImportItem } from '@/api/types';
import { validateScheduleJsonFile } from '../utils';

interface PreparedFile {
  id: string;
  file: File;
  item: BulkImportItem | null;
  error: string | null;
}

interface ScheduleDropZoneProps {
  /** Currently-prepared files (controlled by parent). */
  files: PreparedFile[];
  onChange: (files: PreparedFile[]) => void;
  disabled?: boolean;
}

export function ScheduleDropZone({ files, onChange, disabled }: ScheduleDropZoneProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [isDragOver, setIsDragOver] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);

  const addFiles = useCallback(
    async (incoming: FileList | File[]) => {
      setIsProcessing(true);
      const prepared: PreparedFile[] = [];
      for (const file of Array.from(incoming)) {
        try {
          const item = await validateScheduleJsonFile(file);
          prepared.push({
            id: `${file.name}-${Date.now()}-${Math.random()}`,
            file,
            item,
            error: null,
          });
        } catch (err) {
          prepared.push({
            id: `${file.name}-${Date.now()}-${Math.random()}`,
            file,
            item: null,
            error: err instanceof Error ? err.message : 'invalid file',
          });
        }
      }
      onChange([...files, ...prepared]);
      setIsProcessing(false);
    },
    [files, onChange]
  );

  const remove = (id: string) => onChange(files.filter((f) => f.id !== id));

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
          void addFiles(e.dataTransfer.files);
        }}
        onClick={() => !disabled && inputRef.current?.click()}
        className={`flex cursor-pointer flex-col items-center justify-center gap-2 rounded-xl border-2 border-dashed px-6 py-6 text-center transition-all ${
          isDragOver
            ? 'border-sky-400 bg-sky-500/10 text-sky-300'
            : 'border-slate-600 text-slate-400 hover:border-sky-500/50 hover:text-slate-300'
        } ${disabled ? 'cursor-not-allowed opacity-50' : ''}`}
        role="button"
        tabIndex={0}
        aria-label="Drop schedule JSON files here or click to browse"
      >
        <p className="text-sm font-medium">
          {isProcessing
            ? 'Reading files…'
            : isDragOver
              ? 'Drop JSON files here'
              : 'Drop schedule JSON files here or click to browse'}
        </p>
        <p className="text-xs opacity-70">Multiple files supported</p>
      </div>

      <input
        ref={inputRef}
        type="file"
        accept=".json,application/json"
        multiple
        className="sr-only"
        onChange={(e) => {
          if (e.target.files) void addFiles(e.target.files);
          e.target.value = '';
        }}
      />

      {files.length > 0 && (
        <ul className="mt-3 space-y-1.5" data-testid="prepared-file-list">
          {files.map((f) => (
            <li
              key={f.id}
              className={`flex items-center gap-2 rounded-lg border px-3 py-2 text-sm ${
                f.error
                  ? 'border-red-700/60 bg-red-950/30 text-red-200'
                  : 'border-slate-700 bg-slate-800/40 text-slate-200'
              }`}
            >
              <span className="flex-1 truncate">{f.file.name}</span>
              {f.error && <span className="text-xs">{f.error}</span>}
              {!disabled && (
                <button
                  type="button"
                  onClick={() => remove(f.id)}
                  className="text-slate-400 hover:text-red-400"
                  aria-label={`Remove ${f.file.name}`}
                >
                  ×
                </button>
              )}
            </li>
          ))}
        </ul>
      )}

      {isProcessing && (
        <div className="mt-2 flex items-center gap-2 text-xs text-slate-400">
          <LoadingSpinner size="sm" /> Reading…
        </div>
      )}
    </div>
  );
}

export type { PreparedFile };
