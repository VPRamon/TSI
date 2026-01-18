/**
 * Tests for MetricCard component.
 */
import { describe, it, expect } from 'vitest';
import { render, screen } from '../test/test-utils';
import MetricCard from './MetricCard';

describe('MetricCard', () => {
  it('renders label and value', () => {
    render(<MetricCard label="Total Count" value={42} />);
    expect(screen.getByText('Total Count')).toBeInTheDocument();
    expect(screen.getByText('42')).toBeInTheDocument();
  });

  it('renders string value', () => {
    render(<MetricCard label="Rate" value="98.5%" />);
    expect(screen.getByText('98.5%')).toBeInTheDocument();
  });

  it('renders icon when provided', () => {
    render(<MetricCard label="Test" value={100} icon="🎯" />);
    expect(screen.getByText('🎯')).toBeInTheDocument();
  });

  it('renders trend indicator when provided', () => {
    render(<MetricCard label="Test" value={100} trend="up" trendValue="+5%" />);
    expect(screen.getByText(/\+5%/)).toBeInTheDocument();
  });

  it('applies custom className', () => {
    const { container } = render(<MetricCard label="Test" value={100} className="custom-class" />);
    expect(container.firstChild).toHaveClass('custom-class');
  });
});
