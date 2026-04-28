/**
 * EmptyState — consistent "nothing here yet" placeholder used across panels
 * that have no data, no matching filter results, or whose upstream queries
 * returned an empty payload.
 */
import { memo, type ReactNode } from 'react';

export interface EmptyStateProps {
  /** Headline; one short sentence. */
  title: string;
  /** Body copy explaining why and how to recover. */
  hint?: ReactNode;
  /** Optional decorative emoji/icon — kept simple to avoid a Lucide dependency. */
  icon?: ReactNode;
  /** Optional CTA rendered below the hint. */
  action?: ReactNode;
  /** Tighter variant for inline use in chart panels. */
  dense?: boolean;
  className?: string;
}

const EmptyState = memo(function EmptyState({
  title,
  hint,
  icon,
  action,
  dense = false,
  className = '',
}: EmptyStateProps) {
  return (
    <div
      role="status"
      className={[
        'flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed border-slate-600 text-center',
        dense ? 'py-6 text-xs text-slate-400' : 'py-12 text-sm text-slate-400',
        className,
      ].join(' ')}
    >
      {icon ? (
        <span aria-hidden className="text-2xl text-slate-500">
          {icon}
        </span>
      ) : null}
      <span className="font-medium text-slate-300">{title}</span>
      {hint ? <div className="max-w-md text-slate-400">{hint}</div> : null}
      {action ? <div className="mt-2">{action}</div> : null}
    </div>
  );
});

export default EmptyState;
