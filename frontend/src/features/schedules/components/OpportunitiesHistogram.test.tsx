import type { ReactNode } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';

vi.mock('@/components', () => ({
  ChartPanel: ({
    title,
    headerActions,
    children,
  }: {
    title?: string;
    headerActions?: ReactNode;
    children: ReactNode;
  }) => (
    <section>
      {title ? <h3>{title}</h3> : null}
      {headerActions}
      {children}
    </section>
  ),
}));

import OpportunitiesHistogram from './OpportunitiesHistogram';

describe('OpportunitiesHistogram', () => {
  it('renders a download action for exporting the histogram as an image', () => {
    render(
      <OpportunitiesHistogram
        histogramData={[
          {
            bin_start_unix: 1712966400,
            bin_end_unix: 1712970000,
            visible_count: 3,
          },
        ]}
      />
    );

    expect(screen.getByRole('button', { name: 'Download PNG' })).toBeInTheDocument();
  });

  it('renders bar columns with full-height containers so percentage bar heights can resolve', () => {
    render(
      <OpportunitiesHistogram
        histogramData={[
          {
            bin_start_unix: 1712966400,
            bin_end_unix: 1712970000,
            visible_count: 3,
          },
          {
            bin_start_unix: 1712970000,
            bin_end_unix: 1712973600,
            visible_count: 0,
          },
        ]}
      />
    );

    const columns = screen.getAllByTestId('visibility-histogram-column');
    const bars = screen.getAllByTestId('visibility-histogram-bar');

    expect(columns).toHaveLength(2);
    expect(columns[0]).toHaveClass('h-full');
    expect(bars[0]).toHaveStyle({ height: '100%', minHeight: '2px' });
    expect(bars[1]).toHaveStyle({ height: '0%' });
    expect(bars[1]).not.toHaveStyle({ minHeight: '2px' });
  });

  it('renders the y-axis with the largest value at the top and zero at the bottom', () => {
    render(
      <OpportunitiesHistogram
        histogramData={[
          {
            bin_start_unix: 1712966400,
            bin_end_unix: 1712970000,
            visible_count: 3,
          },
        ]}
      />
    );

    const yAxisLabels = screen.getAllByText(/^[0-9]+$/);

    expect(yAxisLabels[0]).toHaveTextContent('3');
    expect(yAxisLabels[yAxisLabels.length - 1]).toHaveTextContent('0');
    expect(yAxisLabels[0].parentElement).toHaveStyle({ top: '0%' });
    expect(yAxisLabels[yAxisLabels.length - 1].parentElement).toHaveStyle({ top: '100%' });
  });
});
