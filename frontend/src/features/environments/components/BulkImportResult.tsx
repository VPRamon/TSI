/**
 * Render the per-file outcome of a bulk import. Pure presentational.
 */
import type { BulkImportResponse } from '@/api/types';

interface BulkImportResultProps {
  result: BulkImportResponse;
}

export function BulkImportResult({ result }: BulkImportResultProps) {
  const { created, rejected } = result;

  return (
    <div className="space-y-3" data-testid="bulk-import-result">
      <div className="rounded-lg border border-emerald-700/40 bg-emerald-950/20 px-4 py-3">
        <p className="text-sm font-semibold text-emerald-200">
          {created.length} schedule{created.length === 1 ? '' : 's'} accepted
        </p>
        {created.length > 0 && (
          <ul className="mt-2 space-y-0.5 text-xs text-emerald-200/80">
            {created.map((c) => (
              <li key={c.schedule_id}>
                {c.name} <span className="text-emerald-300/60">#{c.schedule_id}</span>
              </li>
            ))}
          </ul>
        )}
      </div>

      {rejected.length > 0 && (
        <div className="rounded-lg border border-red-700/40 bg-red-950/20 px-4 py-3">
          <p className="text-sm font-semibold text-red-200">
            {rejected.length} schedule{rejected.length === 1 ? '' : 's'} rejected
          </p>
          <ul className="mt-2 space-y-2 text-xs text-red-200/90" data-testid="rejected-list">
            {rejected.map((r, i) => (
              <li key={`${r.name}-${i}`} className="rounded border border-red-800/40 px-2 py-1.5">
                <p className="font-medium">{r.name}</p>
                <p className="opacity-90">{r.reason}</p>
                {r.mismatch_fields.length > 0 && (
                  <p className="mt-1 text-red-300/80">
                    Mismatch:{' '}
                    <span className="font-mono">{r.mismatch_fields.join(', ')}</span>
                  </p>
                )}
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
