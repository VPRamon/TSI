import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { usePlotlyDownload } from './usePlotlyDownload';

// Mock the dependencies
vi.mock('@/lib/imageExport', () => ({
  sanitizeImageFilename: (label: string) => label.toLowerCase().replace(/\s+/g, '-'),
  downloadPngDataUrl: vi.fn(),
}));

describe('usePlotlyDownload', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Mock console methods to avoid noise in test output
    vi.spyOn(console, 'warn').mockImplementation(() => {});
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('renders a download button that is initially disabled', () => {
    function TestComponent() {
      const { downloadButton } = usePlotlyDownload('Test Chart');
      return <div>{downloadButton}</div>;
    }

    render(<TestComponent />);

    const button = screen.getByRole('button', { name: /Download PNG/i });
    expect(button).toBeDisabled();
  });

  it('shows error when graphDiv is not initialized', async () => {
    function TestComponent() {
      const { downloadButton } = usePlotlyDownload('Test Chart');
      return <div>{downloadButton}</div>;
    }

    render(<TestComponent />);

    const button = screen.getByRole('button', { name: /Download PNG/i });
    fireEvent.click(button);

    await waitFor(() => {
      expect(screen.getByText(/Chart element not ready/i)).toBeInTheDocument();
    });

    expect(console.warn).toHaveBeenCalledWith(
      expect.stringContaining('[PNG Export]')
    );
  });

  it('shows error when Plotly is not loaded', async () => {
    function TestComponent() {
      const { onInitialized, downloadButton } = usePlotlyDownload('Test Chart');
      const div = document.createElement('div');

      return (
        <div ref={(el) => {
          if (el) {
            onInitialized({}, el);
          }
        }}>
          {downloadButton}
        </div>
      );
    }

    // Mock window.Plotly to be undefined
    const originalPlotly = (window as any).Plotly;
    (window as any).Plotly = undefined;

    render(<TestComponent />);

    const button = screen.getByRole('button', { name: /Download PNG/i });
    fireEvent.click(button);

    await waitFor(() => {
      expect(screen.getByText(/Plotly.js not loaded/i)).toBeInTheDocument();
    });

    expect(console.error).toHaveBeenCalledWith(
      expect.stringContaining('[PNG Export]')
    );

    // Restore Plotly
    (window as any).Plotly = originalPlotly;
  });

  it('shows loading state while exporting', async () => {
    function TestComponent() {
      const { onInitialized, downloadButton } = usePlotlyDownload('Test Chart');
      const divRef = (el: HTMLElement | null) => {
        if (el) {
          onInitialized({}, el);
        }
      };

      return (
        <div ref={divRef}>
          {downloadButton}
        </div>
      );
    }

    // Mock Plotly with a slow toImage
    const mockPlotly = {
      toImage: vi.fn(
        () =>
          new Promise((resolve) =>
            setTimeout(() => resolve('data:image/png;base64,test'), 100)
          )
      ),
    };
    (window as any).Plotly = mockPlotly;

    render(<TestComponent />);

    const button = screen.getByRole('button', { name: /Download PNG/i });
    fireEvent.click(button);

    // Should show "Exporting..." while in progress
    expect(screen.getByRole('button', { name: /Exporting/i })).toBeInTheDocument();

    // Wait for it to finish
    await waitFor(() => {
      expect(screen.queryByRole('button', { name: /Exporting/i })).not.toBeInTheDocument();
    });
  });

  it('logs errors when Plotly.toImage fails', async () => {
    function TestComponent() {
      const { onInitialized, downloadButton } = usePlotlyDownload('Test Chart');
      const divRef = (el: HTMLElement | null) => {
        if (el) {
          onInitialized({}, el);
        }
      };

      return (
        <div ref={divRef}>
          {downloadButton}
        </div>
      );
    }

    // Mock Plotly with failing toImage
    const testError = new Error('CORS blocked image export');
    const mockPlotly = {
      toImage: vi.fn().mockRejectedValue(testError),
    };
    (window as any).Plotly = mockPlotly;

    render(<TestComponent />);

    const button = screen.getByRole('button', { name: /Download PNG/i });
    fireEvent.click(button);

    await waitFor(() => {
      expect(screen.getByText(/Export failed/i)).toBeInTheDocument();
    });

    expect(console.error).toHaveBeenCalledWith(
      expect.stringContaining('[PNG Export]'),
      testError
    );
  });
});
