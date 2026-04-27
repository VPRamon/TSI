/**
 * HelpPopover — small "?" affordance with an accessible popover that
 * explains what a chart represents and how to read it.
 *
 * Anchored to the trigger button via a portal-mounted absolutely
 * positioned panel. Closes on ESC and outside click. Tightly scoped
 * styling so it can sit inside a ChartPanel header without bloating
 * layout.
 */
import { ReactNode, useCallback, useEffect, useId, useLayoutEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';

export interface HelpContent {
  /** Optional heading shown at the top of the popover. */
  title?: string;
  /** Short paragraph describing what the chart shows. */
  summary?: ReactNode;
  /** Bullet points with how-to-read guidance. */
  bullets?: ReactNode[];
}

export interface HelpPopoverProps {
  content: HelpContent;
  /** Accessible label for the trigger (defaults to 'Chart help'). */
  ariaLabel?: string;
}

const TRIGGER_CLASS =
  'inline-flex h-7 w-7 items-center justify-center rounded-full border border-slate-600 bg-slate-800/70 text-xs font-semibold text-slate-300 transition-colors hover:bg-slate-700 hover:text-white focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 focus:ring-offset-slate-900';

export function HelpPopover({ content, ariaLabel = 'Chart help' }: HelpPopoverProps) {
  const [open, setOpen] = useState(false);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const panelRef = useRef<HTMLDivElement>(null);
  const [coords, setCoords] = useState<{ top: number; left: number } | null>(null);
  const panelId = useId();

  const close = useCallback(() => setOpen(false), []);

  useEffect(() => {
    if (!open) return undefined;

    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') close();
    };
    const onClick = (e: MouseEvent) => {
      const target = e.target as Node | null;
      if (!target) return;
      if (panelRef.current?.contains(target)) return;
      if (triggerRef.current?.contains(target)) return;
      close();
    };
    window.addEventListener('keydown', onKey);
    window.addEventListener('mousedown', onClick);
    return () => {
      window.removeEventListener('keydown', onKey);
      window.removeEventListener('mousedown', onClick);
    };
  }, [open, close]);

  useLayoutEffect(() => {
    if (!open || !triggerRef.current) return;
    const rect = triggerRef.current.getBoundingClientRect();
    const panelWidth = 320;
    const margin = 8;
    const viewportW = typeof window === 'undefined' ? 1024 : window.innerWidth;
    let left = rect.right - panelWidth;
    if (left + panelWidth + margin > viewportW) left = viewportW - panelWidth - margin;
    if (left < margin) left = margin;
    setCoords({ top: rect.bottom + margin, left });
  }, [open]);

  return (
    <>
      <button
        ref={triggerRef}
        type="button"
        className={TRIGGER_CLASS}
        aria-label={ariaLabel}
        aria-haspopup="dialog"
        aria-expanded={open}
        aria-controls={open ? panelId : undefined}
        onClick={() => setOpen((prev) => !prev)}
      >
        ?
      </button>
      {open && coords && typeof document !== 'undefined'
        ? createPortal(
            <div
              ref={panelRef}
              id={panelId}
              role="dialog"
              aria-label={content.title ?? ariaLabel}
              className="fixed z-50 w-80 rounded-lg border border-slate-700 bg-slate-900/95 p-4 text-sm text-slate-200 shadow-xl backdrop-blur"
              style={{ top: coords.top, left: coords.left }}
            >
              {content.title && (
                <h3 className="mb-2 text-sm font-semibold text-white">{content.title}</h3>
              )}
              {content.summary && (
                <p className="mb-2 text-xs leading-relaxed text-slate-300">{content.summary}</p>
              )}
              {content.bullets && content.bullets.length > 0 && (
                <ul className="list-disc space-y-1 pl-4 text-xs leading-relaxed text-slate-300">
                  {content.bullets.map((bullet, i) => (
                    <li key={i}>{bullet}</li>
                  ))}
                </ul>
              )}
            </div>,
            document.body,
          )
        : null}
    </>
  );
}

export default HelpPopover;
