import { describe, it, expect, vi, beforeEach, afterEach, type Mock } from 'vitest';
import { render, screen, fireEvent, waitFor, act } from '@testing-library/react';
import { usePlotlyDownload } from './usePlotlyDownload';

type ToImageArgs = [graphDiv: HTMLElement, options?: unknown];
type ToImageMock = Mock<ToImageArgs, Promise<string>>;

const { mockedPlotly } = vi.hoisted(() => ({
  mockedPlotly: {
    toImage: vi.fn<ToImageArgs, Promise<string>>(),
  } as { toImage?: ToImageMock | undefined },
}));

vi.mock('plotly.js-dist-min', () => ({
  default: mockedPlotly,
}));

// Mock the dependencies
vi.mock('@/lib/imageExport', () => ({
  sanitizeImageFilename: (label: string) => label.toLowerCase().replace(/\s+/g, '-'),
  downloadPngDataUrl: vi.fn(),
}));

describe('usePlotlyDownload', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockedPlotly.toImage = vi.fn<ToImageArgs, Promise<string>>();
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

  it('enables the button after chart initialization', async () => {
    let capturedOnInit: ((figure: unknown, graphDiv: HTMLElement) => void) | undefined;

    function TestComponent() {
      const { onInitialized, downloadButton } = usePlotlyDownload('Test Chart');
      capturedOnInit = onInitialized;
      return <div>{downloadButton}</div>;
    }

    render(<TestComponent />);

    const button = screen.getByRole('button', { name: /Download PNG/i });
    expect(button).toBeDisabled();

    const div = document.createElement('div');
    act(() => {
      capturedOnInit?.({}, div);
    });

    await waitFor(() => {
      expect(button).not.toBeDisabled();
    });
  });

  it('shows error when Plotly is not loaded', async () => {
    function TestComponent() {
      const { onInitialized, downloadButton } = usePlotlyDownload('Test Chart');

      return (
        <div
          ref={(el) => {
            if (el) {
              onInitialized({}, el);
            }
          }}
        >
          {downloadButton}
        </div>
      );
    }

    const originalToImage = mockedPlotly.toImage;
    mockedPlotly.toImage = undefined;

    render(<TestComponent />);

    const button = screen.getByRole('button', { name: /Download PNG/i });
    fireEvent.click(button);

    await waitFor(() => {
      expect(screen.getByText(/Plotly.js not loaded/i)).toBeInTheDocument();
    });

    expect(console.error).toHaveBeenCalledWith(expect.stringContaining('[PNG Export]'));

    mockedPlotly.toImage = originalToImage;
  });

  it('shows loading state while exporting', async () => {
    function TestComponent() {
      const { onInitialized, downloadButton } = usePlotlyDownload('Test Chart');
      const divRef = (el: HTMLElement | null) => {
        if (el) {
          onInitialized({}, el);
        }
      };

      return <div ref={divRef}>{downloadButton}</div>;
    }

    // Mock Plotly with a slow toImage
    mockedPlotly.toImage = vi.fn<ToImageArgs, Promise<string>>().mockImplementation(
      () => new Promise((resolve) => setTimeout(() => resolve('data:image/png;base64,test'), 100))
    );

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

      return <div ref={divRef}>{downloadButton}</div>;
    }

    // Mock Plotly with failing toImage
    const testError = new Error('CORS blocked image export');
    mockedPlotly.toImage = vi.fn<ToImageArgs, Promise<string>>().mockRejectedValue(testError);

    render(<TestComponent />);

    const button = screen.getByRole('button', { name: /Download PNG/i });
    fireEvent.click(button);

    await waitFor(() => {
      expect(screen.getByText(/Export failed/i)).toBeInTheDocument();
    });

    expect(console.error).toHaveBeenCalledWith(expect.stringContaining('[PNG Export]'), testError);
  });
});
