import { describe, expect, it, vi, afterEach } from 'vitest';
import { downloadCanvasAsPng, sanitizeImageFilename } from './imageExport';

describe('imageExport', () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('sanitizes chart labels into stable image filenames', () => {
    expect(sanitizeImageFilename('Altitude vs Time')).toBe('altitude-vs-time');
    expect(sanitizeImageFilename('  ')).toBe('tsi-plot');
  });

  it('downloads a canvas as a png file', () => {
    const click = vi.fn();
    const appendChild = vi.spyOn(document.body, 'appendChild');
    const removeChild = vi.spyOn(document.body, 'removeChild');
    const originalCreateElement = document.createElement.bind(document);
    const createElement = vi
      .spyOn(document, 'createElement')
      .mockImplementation((tagName: string) => {
        if (tagName === 'a') {
          const anchor = originalCreateElement('a');
          anchor.click = click;
          return anchor;
        }

        return originalCreateElement(tagName);
      });

    const canvas = {
      toDataURL: vi.fn(() => 'data:image/png;base64,abc123'),
    } as unknown as HTMLCanvasElement;

    downloadCanvasAsPng(canvas, 'sky-map');

    expect(canvas.toDataURL).toHaveBeenCalledWith('image/png');
    expect(createElement).toHaveBeenCalledWith('a');
    expect(appendChild).toHaveBeenCalled();
    expect(click).toHaveBeenCalledOnce();
    expect(removeChild).toHaveBeenCalled();
  });
});
