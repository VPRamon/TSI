/**
 * Tests for the accessible DataTable component.
 */
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { DataTable, type TableColumn } from './DataTable';

interface TestData {
  id: number;
  name: string;
  value: number;
}

const testData: TestData[] = [
  { id: 1, name: 'Item One', value: 100 },
  { id: 2, name: 'Item Two', value: 200 },
  { id: 3, name: 'Item Three', value: 300 },
];

const testColumns: TableColumn<TestData>[] = [
  { header: 'Name', accessor: 'name' },
  { header: 'Value', accessor: 'value', align: 'right' },
];

describe('DataTable', () => {
  it('renders table with caption', () => {
    render(
      <DataTable
        data={testData}
        columns={testColumns}
        keyAccessor={(row) => row.id}
        caption="Test table description"
      />
    );

    expect(screen.getByRole('table')).toBeInTheDocument();
    expect(screen.getByText('Test table description')).toBeInTheDocument();
  });

  it('renders column headers', () => {
    render(
      <DataTable
        data={testData}
        columns={testColumns}
        keyAccessor={(row) => row.id}
        caption="Test table"
      />
    );

    expect(screen.getByRole('columnheader', { name: 'Name' })).toBeInTheDocument();
    expect(screen.getByRole('columnheader', { name: 'Value' })).toBeInTheDocument();
  });

  it('renders data rows', () => {
    render(
      <DataTable
        data={testData}
        columns={testColumns}
        keyAccessor={(row) => row.id}
        caption="Test table"
      />
    );

    expect(screen.getByText('Item One')).toBeInTheDocument();
    expect(screen.getByText('Item Two')).toBeInTheDocument();
    expect(screen.getByText('Item Three')).toBeInTheDocument();
    expect(screen.getByText('100')).toBeInTheDocument();
    expect(screen.getByText('200')).toBeInTheDocument();
    expect(screen.getByText('300')).toBeInTheDocument();
  });

  it('shows empty message when no data', () => {
    render(
      <DataTable
        data={[]}
        columns={testColumns}
        keyAccessor={(row: TestData) => row.id}
        caption="Test table"
        emptyMessage="No items found"
      />
    );

    expect(screen.getByText('No items found')).toBeInTheDocument();
    expect(screen.queryByRole('table')).not.toBeInTheDocument();
  });

  it('limits rows when maxRows is set', () => {
    render(
      <DataTable
        data={testData}
        columns={testColumns}
        keyAccessor={(row) => row.id}
        caption="Test table"
        maxRows={2}
      />
    );

    expect(screen.getByText('Item One')).toBeInTheDocument();
    expect(screen.getByText('Item Two')).toBeInTheDocument();
    expect(screen.queryByText('Item Three')).not.toBeInTheDocument();
    expect(screen.getByText('... and 1 more items')).toBeInTheDocument();
  });

  it('hides caption when captionHidden is true', () => {
    render(
      <DataTable
        data={testData}
        columns={testColumns}
        keyAccessor={(row) => row.id}
        caption="Hidden caption"
        captionHidden
      />
    );

    const caption = screen.getByText('Hidden caption');
    expect(caption).toHaveClass('sr-only');
  });

  it('supports function accessor for custom rendering', () => {
    const columnsWithAccessor: TableColumn<TestData>[] = [
      { header: 'Name', accessor: 'name' },
      { header: 'Double Value', accessor: (row) => row.value * 2 },
    ];

    render(
      <DataTable
        data={testData}
        columns={columnsWithAccessor}
        keyAccessor={(row) => row.id}
        caption="Test table"
      />
    );

    expect(screen.getByText('200')).toBeInTheDocument(); // 100 * 2
    expect(screen.getByText('400')).toBeInTheDocument(); // 200 * 2
    expect(screen.getByText('600')).toBeInTheDocument(); // 300 * 2
  });

  it('applies alignment classes correctly', () => {
    const columnsWithAlign: TableColumn<TestData>[] = [
      { header: 'Left', accessor: 'name', align: 'left' },
      { header: 'Center', accessor: 'id', align: 'center' },
      { header: 'Right', accessor: 'value', align: 'right' },
    ];

    render(
      <DataTable
        data={testData.slice(0, 1)}
        columns={columnsWithAlign}
        keyAccessor={(row) => row.id}
        caption="Test table"
      />
    );

    const headers = screen.getAllByRole('columnheader');
    expect(headers[0]).toHaveClass('text-left');
    expect(headers[1]).toHaveClass('text-center');
    expect(headers[2]).toHaveClass('text-right');
  });
});
