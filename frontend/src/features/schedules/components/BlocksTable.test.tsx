/**
 * Tests for BlocksTable component, focusing on block_name display and filtering.
 */
import { describe, it, expect } from 'vitest';
import { render, screen } from '../../../test/test-utils';
import { AnalysisProvider } from '../context/AnalysisContext';
import { BlocksTable, type TableBlock } from './BlocksTable';

const makeBlock = (overrides: Partial<TableBlock> & { scheduling_block_id: number }): TableBlock => ({
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
    </AnalysisProvider>,
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
});
