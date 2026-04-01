import type { SVGProps } from 'react';

export type IconName =
  | 'alert-octagon'
  | 'alert-triangle'
  | 'arrow-left'
  | 'arrow-right'
  | 'ban'
  | 'blocks'
  | 'brackets'
  | 'calendar'
  | 'calendar-days'
  | 'chart-bar'
  | 'chart-line'
  | 'check'
  | 'check-circle'
  | 'clock'
  | 'inbox'
  | 'link'
  | 'list'
  | 'moon'
  | 'search'
  | 'star'
  | 'target'
  | 'telescope'
  | 'x-circle';

interface IconProps extends SVGProps<SVGSVGElement> {
  name: IconName;
  title?: string;
}

function Icon({ name, className = 'h-6 w-6', title, ...props }: IconProps) {
  const sharedProps = {
    className,
    fill: 'none',
    stroke: 'currentColor',
    viewBox: '0 0 24 24',
    strokeWidth: 1.8,
    strokeLinecap: 'round' as const,
    strokeLinejoin: 'round' as const,
    'aria-hidden': title ? undefined : true,
    ...props,
  };

  const renderPaths = () => {
    switch (name) {
      case 'alert-octagon':
        return (
          <>
            <path d="M9.09 3h5.82a2 2 0 011.41.59l4.09 4.09a2 2 0 01.59 1.41v5.82a2 2 0 01-.59 1.41l-4.09 4.09a2 2 0 01-1.41.59H9.09a2 2 0 01-1.41-.59L3.59 16.32A2 2 0 013 14.91V9.09a2 2 0 01.59-1.41l4.09-4.09A2 2 0 019.09 3z" />
            <path d="M12 8v5" />
            <path d="M12 16h.01" />
          </>
        );
      case 'alert-triangle':
        return (
          <>
            <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z" />
            <path d="M12 9v4" />
            <path d="M12 17h.01" />
          </>
        );
      case 'arrow-left':
        return (
          <>
            <path d="M19 12H5" />
            <path d="M12 19l-7-7 7-7" />
          </>
        );
      case 'arrow-right':
        return (
          <>
            <path d="M5 12h14" />
            <path d="M12 5l7 7-7 7" />
          </>
        );
      case 'ban':
        return (
          <>
            <circle cx="12" cy="12" r="9" />
            <path d="M7.5 16.5l9-9" />
          </>
        );
      case 'blocks':
        return (
          <>
            <path d="M12 3l8 4.5-8 4.5-8-4.5L12 3z" />
            <path d="M4 7.5V16.5L12 21l8-4.5V7.5" />
            <path d="M12 12v9" />
          </>
        );
      case 'brackets':
        return (
          <>
            <path d="M8 4H6a2 2 0 00-2 2v3" />
            <path d="M4 15v3a2 2 0 002 2h2" />
            <path d="M16 4h2a2 2 0 012 2v3" />
            <path d="M20 15v3a2 2 0 01-2 2h-2" />
            <path d="M10 9l2 3-2 3" />
            <path d="M14 9l-2 3 2 3" />
          </>
        );
      case 'calendar':
        return (
          <>
            <path d="M8 2v4" />
            <path d="M16 2v4" />
            <rect x="3" y="4" width="18" height="18" rx="2" />
            <path d="M3 10h18" />
          </>
        );
      case 'calendar-days':
        return (
          <>
            <path d="M8 2v4" />
            <path d="M16 2v4" />
            <rect x="3" y="4" width="18" height="18" rx="2" />
            <path d="M3 10h18" />
            <path d="M8 14h.01" />
            <path d="M12 14h.01" />
            <path d="M16 14h.01" />
            <path d="M8 18h.01" />
            <path d="M12 18h.01" />
            <path d="M16 18h.01" />
          </>
        );
      case 'chart-bar':
        return (
          <>
            <path d="M4 20h16" />
            <path d="M7 20V10" />
            <path d="M12 20V4" />
            <path d="M17 20v-7" />
          </>
        );
      case 'chart-line':
        return (
          <>
            <path d="M4 19h16" />
            <path d="M5 15l4-4 4 3 6-7" />
            <path d="M18 7h1v1" />
          </>
        );
      case 'check':
        return <path d="M5 13l4 4L19 7" />;
      case 'check-circle':
        return (
          <>
            <circle cx="12" cy="12" r="9" />
            <path d="M8.5 12.5l2.5 2.5 4.5-5" />
          </>
        );
      case 'clock':
        return (
          <>
            <circle cx="12" cy="12" r="9" />
            <path d="M12 7v5l3 3" />
          </>
        );
      case 'inbox':
        return (
          <>
            <path d="M4 13.5l2.4-6.1A2 2 0 018.26 6h7.48a2 2 0 011.86 1.4L20 13.5V18a2 2 0 01-2 2H6a2 2 0 01-2-2v-4.5z" />
            <path d="M4 14h4l2 3h4l2-3h4" />
          </>
        );
      case 'link':
        return (
          <>
            <path d="M10 13a5 5 0 007.07 0l1.41-1.41A5 5 0 0011.41 4.5L10.7 5.2" />
            <path d="M14 11a5 5 0 00-7.07 0l-1.41 1.41A5 5 0 0012.59 19.5l.71-.7" />
          </>
        );
      case 'list':
        return (
          <>
            <path d="M9 6h11" />
            <path d="M9 12h11" />
            <path d="M9 18h11" />
            <path d="M4 6h.01" />
            <path d="M4 12h.01" />
            <path d="M4 18h.01" />
          </>
        );
      case 'moon':
        return <path d="M21 12.8A9 9 0 1111.2 3 7 7 0 0021 12.8z" />;
      case 'search':
        return (
          <>
            <circle cx="11" cy="11" r="7" />
            <path d="M20 20l-3.5-3.5" />
          </>
        );
      case 'star':
        return (
          <path d="M12 3l2.7 5.47 6.03.88-4.36 4.24 1.03 5.99L12 16.95 6.6 19.58l1.03-5.99L3.27 9.35l6.03-.88L12 3z" />
        );
      case 'target':
        return (
          <>
            <circle cx="12" cy="12" r="8" />
            <circle cx="12" cy="12" r="4" />
            <path d="M12 2v2" />
            <path d="M12 20v2" />
            <path d="M2 12h2" />
            <path d="M20 12h2" />
          </>
        );
      case 'telescope':
        return (
          <>
            <path d="M6 3l6 6" />
            <path d="M21 12l-9 9-3-3 9-9 3 3z" />
            <path d="M3 21l3-3" />
            <circle cx="4" cy="4" r="2" />
          </>
        );
      case 'x-circle':
        return (
          <>
            <circle cx="12" cy="12" r="9" />
            <path d="M9 9l6 6" />
            <path d="M15 9l-6 6" />
          </>
        );
      default:
        return null;
    }
  };

  return (
    <svg {...sharedProps}>
      {title ? <title>{title}</title> : null}
      {renderPaths()}
    </svg>
  );
}

export default Icon;
