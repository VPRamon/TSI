/**
 * PageContainer component - Consistent page layout wrapper.
 */
import { ReactNode } from 'react';

interface PageContainerProps {
  children: ReactNode;
  className?: string;
}

function PageContainer({ children, className = '' }: PageContainerProps) {
  return (
    <div className={`flex flex-col gap-6 ${className}`}>
      {children}
    </div>
  );
}

export default PageContainer;
