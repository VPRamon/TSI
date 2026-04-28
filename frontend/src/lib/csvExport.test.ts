import { describe, expect, it, vi, afterEach } from 'vitest';
import { downloadCsv, escapeCsvField, exportRowsAsCsv, toCsv } from './csvExport';

describe('csvExport', () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('passes simple values through unchanged', () => {
    expect(escapeCsvField('foo')).toBe('foo');
    expect(escapeCsvField(42)).toBe('42');
    expect(escapeCsvField(true)).toBe('true');
    expect(escapeCsvField(null)).toBe('');
    expect(escapeCsvField(undefined)).toBe('');
  });

  it('quotes fields containing commas, quotes, or newlines', () => {
    expect(escapeCsvField('a,b')).toBe('"a,b"');
    expect(escapeCsvField('say "hi"')).toBe('"say ""hi"""');
    expect(escapeCsvField('line1\nline2')).toBe('"line1\nline2"');
    expect(escapeCsvField('a\r\nb')).toBe('"a\r\nb"');
  });

  it('builds a CSV with header row and CRLF separators', () => {
    const rows = [
      { name: 'alpha', score: 0.5 },
      { name: 'beta, gamma', score: null as number | null },
    ];
    const csv = toCsv(rows, [
      { header: 'Name', accessor: (r) => r.name },
      { header: 'Score', accessor: (r) => r.score },
    ]);
    expect(csv).toBe('Name,Score\r\nalpha,0.5\r\n"beta, gamma",');
  });

  it('returns empty string when no columns are provided', () => {
    expect(toCsv([{ a: 1 }], [])).toBe('');
  });

  it('exportRowsAsCsv triggers a download via a temporary anchor', () => {
    const click = vi.fn();
    const appendChild = vi.spyOn(document.body, 'appendChild');
    const removeChild = vi.spyOn(document.body, 'removeChild');
    const originalCreateElement = document.createElement.bind(document);
    vi.spyOn(document, 'createElement').mockImplementation((tag: string) => {
      if (tag === 'a') {
        const a = originalCreateElement('a');
        a.click = click;
        return a;
      }
      return originalCreateElement(tag);
    });
    const createUrl = vi.fn().mockReturnValue('blob:fake');
    const revokeUrl = vi.fn();
    URL.createObjectURL = createUrl as unknown as typeof URL.createObjectURL;
    URL.revokeObjectURL = revokeUrl as unknown as typeof URL.revokeObjectURL;

    exportRowsAsCsv('Run inventory', [{ a: 1 }], [{ header: 'A', accessor: (r) => r.a }]);

    expect(createUrl).toHaveBeenCalledTimes(1);
    expect(click).toHaveBeenCalledTimes(1);
    expect(appendChild).toHaveBeenCalledTimes(1);
    expect(removeChild).toHaveBeenCalledTimes(1);
    expect(revokeUrl).toHaveBeenCalledWith('blob:fake');
  });

  it('downloadCsv writes the provided string verbatim', () => {
    let capturedBlob: Blob | undefined;
    URL.createObjectURL = ((b: Blob) => {
      capturedBlob = b;
      return 'blob:fake';
    }) as unknown as typeof URL.createObjectURL;
    URL.revokeObjectURL = (() => {}) as unknown as typeof URL.revokeObjectURL;
    const click = vi.fn();
    const originalCreateElement = document.createElement.bind(document);
    vi.spyOn(document, 'createElement').mockImplementation((tag: string) => {
      if (tag === 'a') {
        const a = originalCreateElement('a');
        a.click = click;
        return a;
      }
      return originalCreateElement(tag);
    });

    downloadCsv('Stats report', 'a,b\r\n1,2');

    expect(capturedBlob).toBeDefined();
    expect(capturedBlob?.type).toContain('text/csv');
  });
});
