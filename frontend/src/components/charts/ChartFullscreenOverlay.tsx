/**
 * ChartFullscreenOverlay — re-mounts a Plotly graph div fullscreen.
 *
 * The overlay renders a placeholder element, then on mount it moves
 * the original graph div into the placeholder (so React doesn't have
 * to remount Plotly). On unmount the graph div is returned to its
 * original parent, preserving event listeners.
 *
 * Plotly.Plots.resize is invoked on mount, on window resize and on
 * unmount so the chart fits the new container both ways.
 */
import { useEffect, useRef } from 'react';
import Plotly from 'plotly.js-dist-min';

export interface ChartFullscreenOverlayProps {
  graphDiv: HTMLElement;
  title?: string;
  onClose: () => void;
}

export function ChartFullscreenOverlay({ graphDiv, title, onClose }: ChartFullscreenOverlayProps) {
  const slotRef = useRef<HTMLDivElement>(null);
  const originRef = useRef<{ parent: ParentNode | null; nextSibling: ChildNode | null } | null>(
    null,
  );

  useEffect(() => {
    if (!slotRef.current) return undefined;
    originRef.current = {
      parent: graphDiv.parentNode,
      nextSibling: graphDiv.nextSibling,
    };
    slotRef.current.appendChild(graphDiv);
    void Plotly.Plots.resize(graphDiv);

    const onResize = () => {
      void Plotly.Plots.resize(graphDiv);
    };
    window.addEventListener('resize', onResize);

    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', onKey);

    return () => {
      window.removeEventListener('resize', onResize);
      window.removeEventListener('keydown', onKey);
      const origin = originRef.current;
      if (origin?.parent) {
        origin.parent.insertBefore(graphDiv, origin.nextSibling);
      }
      void Plotly.Plots.resize(graphDiv);
    };
  }, [graphDiv, onClose]);

  return (
    <div
      className="fixed inset-0 z-50 flex flex-col bg-slate-950/95 p-4"
      role="dialog"
      aria-modal="true"
      aria-label={title ? `${title} — fullscreen` : 'Chart fullscreen'}
      onClick={onClose}
    >
      <header
        className="mb-3 flex items-center justify-between"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 className="text-sm font-semibold text-white">{title ?? 'Chart'}</h2>
        <button
          type="button"
          onClick={onClose}
          className="rounded-md border border-slate-600 bg-slate-800/70 px-3 py-1.5 text-xs font-medium text-slate-200 hover:bg-slate-700"
          aria-label="Exit fullscreen"
        >
          Exit fullscreen (Esc)
        </button>
      </header>
      <div
        ref={slotRef}
        className="flex-1 rounded-lg border border-slate-700 bg-slate-900 p-2"
        onClick={(e) => e.stopPropagation()}
      />
    </div>
  );
}

export default ChartFullscreenOverlay;
