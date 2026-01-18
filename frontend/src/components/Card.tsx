/**
 * Card component for dashboard panels.
 */
import { ReactNode } from 'react';

interface CardProps {
  title?: string;
  children: ReactNode;
  className?: string;
  headerAction?: ReactNode;
}

function Card({ title, children, className = '', headerAction }: CardProps) {
  return (
    <div className={`rounded-xl border border-slate-700 bg-slate-800 ${className}`}>
      {title && (
        <div className="flex items-center justify-between border-b border-slate-700 px-6 py-4">
          <h2 className="text-lg font-semibold text-white">{title}</h2>
          {headerAction}
        </div>
      )}
      <div className="p-6">{children}</div>
    </div>
  );
}

export default Card;
