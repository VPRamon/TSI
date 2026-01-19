/* eslint-disable react-refresh/only-export-components */
/**
 * AnalysisContext - Shared state for schedule analysis across views.
 *
 * Provides cross-view coordination for:
 * - Block selection (select blocks in histogram, see in table/sky map)
 * - Priority/time filters (consistent filtering across all views)
 * - Time window (brush on timeline, filter visibility histogram)
 * - Comparison target selection
 *
 * ARCHITECTURE:
 * - Context + reducer pattern for predictable state updates
 * - URL sync via useSearchParams for deep-linking/permalinks
 * - Separate from Zustand (app-level) vs this (analysis-session level)
 */
import {
  createContext,
  useContext,
  useReducer,
  useCallback,
  useEffect,
  type ReactNode,
  type Dispatch,
} from 'react';
import { useSearchParams } from 'react-router-dom';

// =============================================================================
// Types
// =============================================================================

export interface PriorityFilter {
  min?: number;
  max?: number;
}

export interface TimeWindow {
  startUnix?: number;
  endUnix?: number;
}

export interface AnalysisState {
  // Selected block IDs for cross-view highlighting
  selectedBlockIds: Set<number>;

  // Priority range filter
  priorityFilter: PriorityFilter;

  // Time window filter (from timeline brush)
  timeWindow: TimeWindow;

  // Scheduled/unscheduled filter
  scheduledFilter: 'all' | 'scheduled' | 'unscheduled';

  // Comparison schedule ID (for quick compare access)
  comparisonScheduleId: number | null;

  // Active block for details drawer
  activeBlockId: number | null;
}

type AnalysisAction =
  | { type: 'SELECT_BLOCKS'; blockIds: number[] }
  | { type: 'ADD_BLOCKS_TO_SELECTION'; blockIds: number[] }
  | { type: 'REMOVE_BLOCKS_FROM_SELECTION'; blockIds: number[] }
  | { type: 'CLEAR_SELECTION' }
  | { type: 'SET_PRIORITY_FILTER'; filter: PriorityFilter }
  | { type: 'SET_TIME_WINDOW'; window: TimeWindow }
  | { type: 'SET_SCHEDULED_FILTER'; filter: 'all' | 'scheduled' | 'unscheduled' }
  | { type: 'SET_COMPARISON_SCHEDULE'; scheduleId: number | null }
  | { type: 'SET_ACTIVE_BLOCK'; blockId: number | null }
  | { type: 'RESET_FILTERS' }
  | { type: 'HYDRATE_FROM_URL'; state: Partial<AnalysisState> };

// =============================================================================
// Initial State
// =============================================================================

const initialState: AnalysisState = {
  selectedBlockIds: new Set(),
  priorityFilter: {},
  timeWindow: {},
  scheduledFilter: 'all',
  comparisonScheduleId: null,
  activeBlockId: null,
};

// =============================================================================
// Reducer
// =============================================================================

function analysisReducer(state: AnalysisState, action: AnalysisAction): AnalysisState {
  switch (action.type) {
    case 'SELECT_BLOCKS':
      return { ...state, selectedBlockIds: new Set(action.blockIds) };

    case 'ADD_BLOCKS_TO_SELECTION': {
      const newSelection = new Set(state.selectedBlockIds);
      action.blockIds.forEach((id) => newSelection.add(id));
      return { ...state, selectedBlockIds: newSelection };
    }

    case 'REMOVE_BLOCKS_FROM_SELECTION': {
      const newSelection = new Set(state.selectedBlockIds);
      action.blockIds.forEach((id) => newSelection.delete(id));
      return { ...state, selectedBlockIds: newSelection };
    }

    case 'CLEAR_SELECTION':
      return { ...state, selectedBlockIds: new Set(), activeBlockId: null };

    case 'SET_PRIORITY_FILTER':
      return { ...state, priorityFilter: action.filter };

    case 'SET_TIME_WINDOW':
      return { ...state, timeWindow: action.window };

    case 'SET_SCHEDULED_FILTER':
      return { ...state, scheduledFilter: action.filter };

    case 'SET_COMPARISON_SCHEDULE':
      return { ...state, comparisonScheduleId: action.scheduleId };

    case 'SET_ACTIVE_BLOCK':
      return { ...state, activeBlockId: action.blockId };

    case 'RESET_FILTERS':
      return {
        ...state,
        priorityFilter: {},
        timeWindow: {},
        scheduledFilter: 'all',
        selectedBlockIds: new Set(),
        activeBlockId: null,
      };

    case 'HYDRATE_FROM_URL':
      return { ...state, ...action.state };

    default:
      return state;
  }
}

// =============================================================================
// Context
// =============================================================================

interface AnalysisContextValue {
  state: AnalysisState;
  dispatch: Dispatch<AnalysisAction>;

  // Convenience actions
  selectBlocks: (blockIds: number[]) => void;
  addToSelection: (blockIds: number[]) => void;
  removeFromSelection: (blockIds: number[]) => void;
  clearSelection: () => void;
  setPriorityFilter: (filter: PriorityFilter) => void;
  setTimeWindow: (window: TimeWindow) => void;
  setScheduledFilter: (filter: 'all' | 'scheduled' | 'unscheduled') => void;
  setComparisonSchedule: (scheduleId: number | null) => void;
  setActiveBlock: (blockId: number | null) => void;
  resetFilters: () => void;

  // Computed helpers
  hasActiveFilters: boolean;
  selectionCount: number;
}

const AnalysisContext = createContext<AnalysisContextValue | null>(null);

// =============================================================================
// Provider
// =============================================================================

interface AnalysisProviderProps {
  children: ReactNode;
  /** Whether to sync state to URL params */
  syncToUrl?: boolean;
}

export function AnalysisProvider({ children, syncToUrl = true }: AnalysisProviderProps) {
  const [state, dispatch] = useReducer(analysisReducer, initialState);
  const [searchParams, setSearchParams] = useSearchParams();

  // Hydrate from URL on mount only - intentionally ignore dependency changes
  useEffect(() => {
    if (!syncToUrl) return;

    const priorityMin = searchParams.get('pmin');
    const priorityMax = searchParams.get('pmax');
    const scheduled = searchParams.get('scheduled');
    const compareId = searchParams.get('compare');
    const blockIds = searchParams.get('blocks');

    const urlState: Partial<AnalysisState> = {};

    if (priorityMin || priorityMax) {
      urlState.priorityFilter = {
        min: priorityMin ? parseFloat(priorityMin) : undefined,
        max: priorityMax ? parseFloat(priorityMax) : undefined,
      };
    }

    if (scheduled && ['all', 'scheduled', 'unscheduled'].includes(scheduled)) {
      urlState.scheduledFilter = scheduled as 'all' | 'scheduled' | 'unscheduled';
    }

    if (compareId) {
      urlState.comparisonScheduleId = parseInt(compareId, 10);
    }

    if (blockIds) {
      urlState.selectedBlockIds = new Set(blockIds.split(',').map((id) => parseInt(id, 10)));
    }

    if (Object.keys(urlState).length > 0) {
      dispatch({ type: 'HYDRATE_FROM_URL', state: urlState });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // Only on mount - read initial URL state once

  // Sync state changes to URL
  useEffect(() => {
    if (!syncToUrl) return;

    const newParams = new URLSearchParams(searchParams);

    // Priority filter
    if (state.priorityFilter.min !== undefined) {
      newParams.set('pmin', state.priorityFilter.min.toString());
    } else {
      newParams.delete('pmin');
    }
    if (state.priorityFilter.max !== undefined) {
      newParams.set('pmax', state.priorityFilter.max.toString());
    } else {
      newParams.delete('pmax');
    }

    // Scheduled filter
    if (state.scheduledFilter !== 'all') {
      newParams.set('scheduled', state.scheduledFilter);
    } else {
      newParams.delete('scheduled');
    }

    // Comparison schedule
    if (state.comparisonScheduleId !== null) {
      newParams.set('compare', state.comparisonScheduleId.toString());
    } else {
      newParams.delete('compare');
    }

    // Selected blocks (limit to avoid huge URLs)
    if (state.selectedBlockIds.size > 0 && state.selectedBlockIds.size <= 50) {
      newParams.set('blocks', Array.from(state.selectedBlockIds).join(','));
    } else {
      newParams.delete('blocks');
    }

    // Only update if params actually changed
    const currentParamsStr = searchParams.toString();
    const newParamsStr = newParams.toString();
    if (currentParamsStr !== newParamsStr) {
      setSearchParams(newParams, { replace: true });
    }
  }, [state, syncToUrl, searchParams, setSearchParams]);

  // Convenience action creators
  const selectBlocks = useCallback((blockIds: number[]) => {
    dispatch({ type: 'SELECT_BLOCKS', blockIds });
  }, []);

  const addToSelection = useCallback((blockIds: number[]) => {
    dispatch({ type: 'ADD_BLOCKS_TO_SELECTION', blockIds });
  }, []);

  const removeFromSelection = useCallback((blockIds: number[]) => {
    dispatch({ type: 'REMOVE_BLOCKS_FROM_SELECTION', blockIds });
  }, []);

  const clearSelection = useCallback(() => {
    dispatch({ type: 'CLEAR_SELECTION' });
  }, []);

  const setPriorityFilter = useCallback((filter: PriorityFilter) => {
    dispatch({ type: 'SET_PRIORITY_FILTER', filter });
  }, []);

  const setTimeWindow = useCallback((window: TimeWindow) => {
    dispatch({ type: 'SET_TIME_WINDOW', window });
  }, []);

  const setScheduledFilter = useCallback((filter: 'all' | 'scheduled' | 'unscheduled') => {
    dispatch({ type: 'SET_SCHEDULED_FILTER', filter });
  }, []);

  const setComparisonSchedule = useCallback((scheduleId: number | null) => {
    dispatch({ type: 'SET_COMPARISON_SCHEDULE', scheduleId });
  }, []);

  const setActiveBlock = useCallback((blockId: number | null) => {
    dispatch({ type: 'SET_ACTIVE_BLOCK', blockId });
  }, []);

  const resetFilters = useCallback(() => {
    dispatch({ type: 'RESET_FILTERS' });
  }, []);

  // Computed values
  const hasActiveFilters =
    state.priorityFilter.min !== undefined ||
    state.priorityFilter.max !== undefined ||
    state.scheduledFilter !== 'all' ||
    state.timeWindow.startUnix !== undefined ||
    state.selectedBlockIds.size > 0;

  const selectionCount = state.selectedBlockIds.size;

  const value: AnalysisContextValue = {
    state,
    dispatch,
    selectBlocks,
    addToSelection,
    removeFromSelection,
    clearSelection,
    setPriorityFilter,
    setTimeWindow,
    setScheduledFilter,
    setComparisonSchedule,
    setActiveBlock,
    resetFilters,
    hasActiveFilters,
    selectionCount,
  };

  return <AnalysisContext.Provider value={value}>{children}</AnalysisContext.Provider>;
}

// =============================================================================
// Hook
// =============================================================================

export function useAnalysis(): AnalysisContextValue {
  const context = useContext(AnalysisContext);
  if (!context) {
    throw new Error('useAnalysis must be used within an AnalysisProvider');
  }
  return context;
}

/**
 * Utility hook to get only selection-related state (for components that
 * don't need full analysis context).
 */
export function useBlockSelection() {
  const { state, selectBlocks, addToSelection, removeFromSelection, clearSelection, setActiveBlock } =
    useAnalysis();

  return {
    selectedBlockIds: state.selectedBlockIds,
    activeBlockId: state.activeBlockId,
    selectionCount: state.selectedBlockIds.size,
    selectBlocks,
    addToSelection,
    removeFromSelection,
    clearSelection,
    setActiveBlock,
    isSelected: (blockId: number) => state.selectedBlockIds.has(blockId),
  };
}
