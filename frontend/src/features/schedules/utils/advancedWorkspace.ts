export interface AdvancedWorkspaceState {
  workspaceIds: number[];
  visibleIds: number[];
  baselineId: number | null;
}

function parseIdList(raw: string | null): number[] {
  if (!raw) return [];

  const parsed: number[] = [];
  const seen = new Set<number>();

  for (const part of raw.split(',')) {
    const value = Number.parseInt(part.trim(), 10);
    if (!Number.isFinite(value) || value <= 0 || seen.has(value)) {
      continue;
    }
    seen.add(value);
    parsed.push(value);
  }

  return parsed;
}

export function parseAdvancedWorkspaceSearch(
  searchParams: URLSearchParams
): AdvancedWorkspaceState {
  const workspaceIds = parseIdList(searchParams.get('workspace'));
  const visibleIds = parseIdList(searchParams.get('visible'));
  const baselineRaw = searchParams.get('baseline');
  const baselineId =
    baselineRaw && Number.isFinite(Number.parseInt(baselineRaw, 10))
      ? Number.parseInt(baselineRaw, 10)
      : null;

  return {
    workspaceIds,
    visibleIds,
    baselineId: baselineId && baselineId > 0 ? baselineId : null,
  };
}

export function normalizeAdvancedWorkspaceState(
  state: AdvancedWorkspaceState,
  validIds?: ReadonlySet<number>
): AdvancedWorkspaceState {
  const workspaceIds = state.workspaceIds.filter((id) => (validIds ? validIds.has(id) : true));
  let visibleIds = state.visibleIds.filter(
    (id) => workspaceIds.includes(id) && (validIds ? validIds.has(id) : true)
  );

  if (workspaceIds.length > 0 && visibleIds.length === 0) {
    visibleIds = [workspaceIds[0]];
  }

  let baselineId = state.baselineId;

  if (
    baselineId == null ||
    !workspaceIds.includes(baselineId) ||
    !visibleIds.includes(baselineId)
  ) {
    baselineId = visibleIds[0] ?? workspaceIds[0] ?? null;
  }

  if (baselineId != null && !visibleIds.includes(baselineId)) {
    visibleIds = [baselineId, ...visibleIds.filter((id) => id !== baselineId)];
  }

  return {
    workspaceIds,
    visibleIds,
    baselineId,
  };
}

export function serializeAdvancedWorkspaceState(state: AdvancedWorkspaceState): URLSearchParams {
  const searchParams = new URLSearchParams();

  if (state.workspaceIds.length > 0) {
    searchParams.set('workspace', state.workspaceIds.join(','));
  }

  if (state.visibleIds.length > 0) {
    searchParams.set('visible', state.visibleIds.join(','));
  }

  if (state.baselineId != null) {
    searchParams.set('baseline', String(state.baselineId));
  }

  return searchParams;
}

export function workspaceStatesEqual(
  left: AdvancedWorkspaceState,
  right: AdvancedWorkspaceState
): boolean {
  return (
    left.baselineId === right.baselineId &&
    left.workspaceIds.length === right.workspaceIds.length &&
    left.visibleIds.length === right.visibleIds.length &&
    left.workspaceIds.every((id, idx) => id === right.workspaceIds[idx]) &&
    left.visibleIds.every((id, idx) => id === right.visibleIds[idx])
  );
}

export function withScheduleAdded(
  state: AdvancedWorkspaceState,
  scheduleId: number,
  visibleAutoLimit = 6
): AdvancedWorkspaceState {
  const workspaceIds = state.workspaceIds.includes(scheduleId)
    ? state.workspaceIds
    : [...state.workspaceIds, scheduleId];
  const shouldAutoShow =
    state.visibleIds.includes(scheduleId) || state.visibleIds.length < visibleAutoLimit;
  const visibleIds = shouldAutoShow
    ? state.visibleIds.includes(scheduleId)
      ? state.visibleIds
      : [...state.visibleIds, scheduleId]
    : state.visibleIds;
  const normalized = normalizeAdvancedWorkspaceState({
    workspaceIds,
    visibleIds,
    baselineId: state.baselineId,
  });

  return normalized;
}

export function withScheduleRemoved(
  state: AdvancedWorkspaceState,
  scheduleId: number
): AdvancedWorkspaceState {
  const workspaceIds = state.workspaceIds.filter((id) => id !== scheduleId);
  const visibleIds = state.visibleIds.filter((id) => id !== scheduleId);
  const baselineId = state.baselineId === scheduleId ? null : state.baselineId;

  return normalizeAdvancedWorkspaceState({
    workspaceIds,
    visibleIds,
    baselineId,
  });
}

export function withVisibilityToggled(
  state: AdvancedWorkspaceState,
  scheduleId: number
): AdvancedWorkspaceState {
  if (!state.workspaceIds.includes(scheduleId)) {
    return state;
  }

  if (state.visibleIds.includes(scheduleId)) {
    if (state.visibleIds.length === 1) {
      return state;
    }

    return normalizeAdvancedWorkspaceState({
      workspaceIds: state.workspaceIds,
      visibleIds: state.visibleIds.filter((id) => id !== scheduleId),
      baselineId: state.baselineId === scheduleId ? null : state.baselineId,
    });
  }

  return normalizeAdvancedWorkspaceState({
    workspaceIds: state.workspaceIds,
    visibleIds: [...state.visibleIds, scheduleId],
    baselineId: state.baselineId,
  });
}

export function withBaselineSelected(
  state: AdvancedWorkspaceState,
  scheduleId: number
): AdvancedWorkspaceState {
  if (!state.workspaceIds.includes(scheduleId)) {
    return state;
  }

  return normalizeAdvancedWorkspaceState({
    workspaceIds: state.workspaceIds,
    visibleIds: state.visibleIds.includes(scheduleId)
      ? state.visibleIds
      : [...state.visibleIds, scheduleId],
    baselineId: scheduleId,
  });
}

export function orderedVisibleIds(state: AdvancedWorkspaceState): number[] {
  if (state.baselineId == null) {
    return state.visibleIds;
  }

  return [state.baselineId, ...state.visibleIds.filter((id) => id !== state.baselineId)];
}
