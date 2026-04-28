/**
 * Shared helpers for exporting chart images from browser-rendered views.
 */

/**
 * Convert a chart title into a filesystem-friendly filename stem.
 */
export function sanitizeImageFilename(label: string): string {
  const normalized = label
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');

  return normalized || 'tsi-plot';
}

/**
 * Trigger a PNG download from a data URL.
 */
export function downloadPngDataUrl(dataUrl: string, filename: string): void {
  const link = document.createElement('a');
  link.href = dataUrl;
  link.download = filename.endsWith('.png') ? filename : `${filename}.png`;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
}

/**
 * Export an existing canvas element as a PNG image.
 */
export function downloadCanvasAsPng(canvas: HTMLCanvasElement, filename: string): void {
  downloadPngDataUrl(canvas.toDataURL('image/png'), filename);
}

/**
 * Trigger a download for an SVG markup string.
 */
export function downloadSvgString(svg: string, filename: string): void {
  const blob = new Blob([svg], { type: 'image/svg+xml;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename.endsWith('.svg') ? filename : `${filename}.svg`;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}
