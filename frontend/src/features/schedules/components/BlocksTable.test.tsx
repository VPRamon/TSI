/**
 * Tests for BlocksTable component, focusing on block_name display and filtering.
 */
import { describe, it, expect } from 'vitest';
import { render, screen, within } from '../../../test/test-utils';
import { fireEvent } from '@testing-library/react';
import { AnalysisProvider } from '../context/AnalysisContext';
import { BlocksTable, type TableBlock } from './BlocksTable';

const makeBlock = (
  overrides: Partial<TableBlock> & { scheduling_block_id: number }
): TableBlock => ({
  scheduling_block_id: overrides.scheduling_block_id,
  original_block_id: overrides.original_block_id ?? `BLOCK-${overrides.scheduling_block_id}`,
  priority: overrides.priority ?? 5,
  scheduled: overrides.scheduled ?? false,
  block_name: overrides.block_name,
  total_visibility_hours: overrides.total_visibility_hours,
  requested_hours: overrides.requested_hours,
});

function renderTable(blocks: TableBlock[], title = 'Blocks') {
  return render(
    <AnalysisProvider syncToUrl={false}>
      <BlocksTable blocks={blocks} title={title} />
    </AnalysisProvider>
  );
}

describe('BlocksTable', () => {
  it('renders original_block_id for each block', () => {
    const blocks = [makeBlock({ scheduling_block_id: 1, original_block_id: 'OB-001' })];
    renderTable(blocks);
    expect(screen.getByText('OB-001')).toBeInTheDocument();
  });

  it('shows block_name when provided', () => {
    const blocks = [
      makeBlock({ scheduling_block_id: 1, original_block_id: 'OB-001', block_name: 'Crab Nebula' }),
    ];
    renderTable(blocks);
    expect(screen.getByText('Crab Nebula')).toBeInTheDocument();
  });

  it('omits block_name when not provided', () => {
    const blocks = [makeBlock({ scheduling_block_id: 1, original_block_id: 'OB-001' })];
    renderTable(blocks);
    expect(screen.queryByText('Crab Nebula')).not.toBeInTheDocument();
  });

  it('renders scheduled and unscheduled status badges', () => {
    const blocks = [
      makeBlock({ scheduling_block_id: 1, scheduled: true }),
      makeBlock({ scheduling_block_id: 2, scheduled: false }),
    ];
    renderTable(blocks);
    expect(screen.getByText('Scheduled')).toBeInTheDocument();
    expect(screen.getByText('Unscheduled')).toBeInTheDocument();
  });

  it('shows empty state when no blocks', () => {
    renderTable([]);
    expect(screen.getByText('No blocks available')).toBeInTheDocument();
  });

  it('renders the provided table title', () => {
    renderTable([makeBlock({ scheduling_block_id: 1 })], 'My Blocks');
    expect(screen.getByText('My Blocks')).toBeInTheDocument();
  });

  it('paginates 250 rows into 3 pages of 100/100/50 with Prev/Next controls', () => {
    const blocks = Array.from({ length: 250 }, (_, i) =>
      makeBlock({ scheduling_block_id: i + 1, priority: i + 1 })
    );
    renderTable(blocks);

    const countDataRows = () => {
      const tbody = document.querySelector('tbody');
      return tbody ? within(tbody as HTMLElement).getAllByRole('row').length : 0;
    };

    expect(screen.getByText(/Showing 1–100 of 250/)).toBeInTheDocument();
    expect(screen.getByText(/Page 1 \/ 3/)).toBeInTheDocument();
    expect(countDataRows()).toBe(100);

    const prev = screen.getByRole('button', { name: /Prev/i });
    const next = screen.getByRole('button', { name: /Next/i });
    expect(prev).toBeDisabled();
    expect(next).not.toBeDisabled();

    fireEvent.click(next);
    expect(screen.getByText(/Showing 101–200 of 250/)).toBeInTheDocument();
    expect(screen.getByText(/Page 2 \/ 3/)).toBeInTheDocument();
    expect(countDataRows()).toBe(100);

    fireEvent.click(next);
    expect(screen.getByText(/Showing 201–250 of 250/)).toBeInTheDocument();
    expect(screen.getByText(/Page 3 \/ 3/)).toBeInTheDocument();
    expect(countDataRows()).toBe(50);
    expect(screen.getByRole('button', { name: /Next/i })).toBeDisabled();
  });
});
