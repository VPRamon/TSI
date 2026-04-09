import type { ImgHTMLAttributes } from 'react';

interface BrandMarkProps extends Omit<ImgHTMLAttributes<HTMLImageElement>, 'src' | 'alt'> {
  title?: string;
}

function BrandMark({ className = 'h-10 w-10', title, ...props }: BrandMarkProps) {
  return (
    <img
      src="/telescope.png"
      alt={title ?? ''}
      title={title}
      aria-hidden={title ? undefined : true}
      className={className}
      draggable={false}
      {...props}
    />
  );
}

export default BrandMark;
