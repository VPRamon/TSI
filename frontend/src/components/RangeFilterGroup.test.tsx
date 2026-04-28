/**
 * Tests for RangeFilterGroup — focuses on the 120 ms debounce: rapid
 * drags must coalesce into a single trailing onChange call carrying the
 * user's final position.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { fireEvent, render, screen } from '../test/test-utils';
import { RangeFilterGroup, type RangeFilterSpec } from './RangeFilterGroup';

const specs: RangeFilterSpec[] = [
  { key: 'e', label: 'E', values: [1, 2, 3, 4, 5] },
];

describe('RangeFilterGroup', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it('debounces rapid slider drags into a single trailing onChange', () => {
    const onChange = vi.fn();
    render(
      <RangeFilterGroup
        specs={specs}
        values={{ e: { min: 1, max: 5 } }}
        onChange={onChange}
      />,
    );

    const minInput = screen.getByLabelText('E minimum') as HTMLInputElement;

    // Rapid-fire drag: many synchronous events.
    for (const v of [2, 3, 4, 3, 4]) {
      fireEvent.change(minInput, { target: { value: String(v) } });
    }

    // Nothing fires before the debounce window elapses.
    expect(onChange).not.toHaveBeenCalled();

    vi.advanceTimersByTime(120);

    // Exactly one trailing call carrying the last drag position.
    expect(onChange).toHaveBeenCalledTimes(1);
    expect(onChange).toHaveBeenLastCalledWith({ e: { min: 4, max: 5 } });
  });
});
