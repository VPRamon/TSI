import { describe, expect, it } from 'vitest';
import {
  normalizeAdvancedWorkspaceState,
  parseAdvancedWorkspaceSearch,
  withScheduleAdded,
  withScheduleRemoved,
  withVisibilityToggled,
} from './advancedWorkspace';

describe('advancedWorkspace utils', () => {
  it('parses positive deduplicated ids from search params', () => {
    const state = parseAdvancedWorkspaceSearch(
      new URLSearchParams('workspace=2,2,foo,-1,1&visible=1,1,3&baseline=3')
    );

    expect(state).toEqual({
      workspaceIds: [2, 1],
      visibleIds: [1, 3],
      baselineId: 3,
    });
  });

  it('normalizes invalid workspace, visible, and baseline ids', () => {
    const normalized = normalizeAdvancedWorkspaceState(
      {
        workspaceIds: [2, 1, 999],
        visibleIds: [999, 2],
        baselineId: 999,
      },
      new Set([1, 2])
    );

    expect(normalized).toEqual({
      workspaceIds: [2, 1],
      visibleIds: [2],
      baselineId: 2,
    });
  });

  it('auto-shows the first schedule when workspace exists but visible is empty', () => {
    const normalized = normalizeAdvancedWorkspaceState({
      workspaceIds: [5, 6],
      visibleIds: [],
      baselineId: null,
    });

    expect(normalized).toEqual({
      workspaceIds: [5, 6],
      visibleIds: [5],
      baselineId: 5,
    });
  });

  it('adds schedules to the visible subset only while below the auto-show limit', () => {
    const base = {
      workspaceIds: [1, 2, 3, 4, 5, 6],
      visibleIds: [1, 2, 3, 4, 5, 6],
      baselineId: 1,
    };

    const next = withScheduleAdded(base, 7);

    expect(next.workspaceIds).toEqual([1, 2, 3, 4, 5, 6, 7]);
    expect(next.visibleIds).toEqual([1, 2, 3, 4, 5, 6]);
  });

  it('reassigns the baseline when a visible schedule is removed or hidden', () => {
    const base = {
      workspaceIds: [1, 2],
      visibleIds: [1, 2],
      baselineId: 1,
    };

    expect(withVisibilityToggled(base, 1)).toEqual({
      workspaceIds: [1, 2],
      visibleIds: [2],
      baselineId: 2,
    });

    expect(withScheduleRemoved(base, 1)).toEqual({
      workspaceIds: [2],
      visibleIds: [2],
      baselineId: 2,
    });
  });
});
