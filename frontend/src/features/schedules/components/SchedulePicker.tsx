/**
 * SchedulePicker - Dropdown to select a schedule for comparison.
 *
 * Used in:
 * - Layout top bar "Compare with..." action
 * - Any view that needs quick schedule selection
 */
import { useState, useRef, useEffect, useCallback } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useSchedules } from '@/hooks/useApi';
import type { ScheduleInfo } from '@/api/types';

interface SchedulePickerProps {
  /** Currently selected schedule ID (excluded from list in single-select mode) */
  excludeId?: number;
  /** Callback when a schedule is selected (single-select mode) */
  onSelect?: (schedule: ScheduleInfo) => void;
  /** If true, navigates to compare page on selection (single-select mode) */
  navigateToCompare?: boolean;
  /** Placeholder text */
  placeholder?: string;
  /** Additional CSS classes */
  className?: string;
  /** Enable multi-select mode with checkboxes */
  multiSelect?: boolean;
  /** Pre-selected schedule IDs (multi-select mode) */
  initialSelectedIds?: number[];
  /** Callback called with confirmed schedules (multi-select mode) */
  onConfirm?: (schedules: ScheduleInfo[]) => void;
}

export function SchedulePicker({
  excludeId,
  onSelect,
  navigateToCompare = false,
  placeholder = 'Select schedule...',
  className = '',
  multiSelect = false,
  initialSelectedIds = [],
  onConfirm,
}: SchedulePickerProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [search, setSearch] = useState('');
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set(initialSelectedIds));
  const dropdownRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const navigate = useNavigate();
  const { scheduleId } = useParams();

  const { data: schedulesData, isLoading } = useSchedules();

  // All schedules (excluding excludeId in single-select mode)
  const allSchedules =
    schedulesData?.schedules.filter((s) => {
      if (!multiSelect && excludeId && s.schedule_id === excludeId) return false;
      return true;
    }) ?? [];

  // Filter schedules based on search
  const filteredSchedules = allSchedules.filter((s) => {
    if (!search) return true;
    return (
      s.schedule_name.toLowerCase().includes(search.toLowerCase()) ||
      s.schedule_id.toString().includes(search)
    );
  });

  // Close dropdown when clicking outside
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // Close on escape
  useEffect(() => {
    function handleEscape(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        setIsOpen(false);
      }
    }
    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, []);

  const handleSelect = useCallback(
    (schedule: ScheduleInfo) => {
      setIsOpen(false);
      setSearch('');

      if (onSelect) {
        onSelect(schedule);
      }

      if (navigateToCompare && scheduleId) {
        navigate(`/schedules/${scheduleId}/compare/${schedule.schedule_id}`);
      }
    },
    [onSelect, navigateToCompare, scheduleId, navigate]
  );

  const handleToggleCheck = useCallback((scheduleId: number) => {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(scheduleId)) {
        next.delete(scheduleId);
      } else {
        next.add(scheduleId);
      }
      return next;
    });
  }, []);

  const handleConfirm = useCallback(() => {
    if (!onConfirm || !schedulesData) return;
    const confirmed = schedulesData.schedules.filter((s) => selectedIds.has(s.schedule_id));
    onConfirm(confirmed);
    setIsOpen(false);
    setSearch('');
  }, [onConfirm, schedulesData, selectedIds]);

  const handleInputFocus = () => {
    setIsOpen(true);
  };

  const selectedCount = selectedIds.size;

  if (multiSelect) {
    return (
      <div ref={dropdownRef} className={`relative ${className}`}>
        {/* Search input */}
        <div className="relative">
          <input
            ref={inputRef}
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            onFocus={handleInputFocus}
            placeholder={placeholder}
            className="w-full rounded-lg border border-slate-600 bg-slate-700 px-3 py-2 pr-8 text-sm text-white placeholder-slate-400 focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
            aria-label="Search schedules"
            aria-expanded={isOpen}
            aria-haspopup="listbox"
          />
          <button
            type="button"
            onClick={() => setIsOpen(!isOpen)}
            className="absolute right-2 top-1/2 -translate-y-1/2 text-slate-400 hover:text-white"
            aria-label={isOpen ? 'Close dropdown' : 'Open dropdown'}
          >
            <svg
              className={`h-4 w-4 transition-transform ${isOpen ? 'rotate-180' : ''}`}
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M19 9l-7 7-7-7"
              />
            </svg>
          </button>
        </div>

        {/* Dropdown */}
        {isOpen && (
          <div
            className="absolute z-50 mt-1 w-full overflow-hidden rounded-lg border border-slate-600 bg-slate-800 shadow-lg"
            role="listbox"
            aria-multiselectable="true"
          >
            <div className="max-h-52 overflow-auto py-1">
              {isLoading ? (
                <div className="px-3 py-2 text-sm text-slate-400">Loading...</div>
              ) : filteredSchedules.length === 0 ? (
                <div className="px-3 py-2 text-sm text-slate-400">
                  {search ? 'No matching schedules' : 'No schedules available'}
                </div>
              ) : (
                filteredSchedules.map((schedule) => (
                  <label
                    key={schedule.schedule_id}
                    className="flex cursor-pointer items-center gap-2 px-3 py-2 text-sm text-slate-200 hover:bg-slate-700"
                  >
                    <input
                      type="checkbox"
                      className="h-4 w-4 rounded border-slate-500 text-primary-500 accent-sky-500"
                      checked={selectedIds.has(schedule.schedule_id)}
                      onChange={() => handleToggleCheck(schedule.schedule_id)}
                    />
                    <span className="truncate">{schedule.schedule_name}</span>
                    <span className="ml-auto shrink-0 text-xs text-slate-500">
                      #{schedule.schedule_id}
                    </span>
                  </label>
                ))
              )}
            </div>
            {/* Confirm button */}
            <div className="border-t border-slate-700 p-2">
              <button
                type="button"
                disabled={selectedCount < 2}
                onClick={handleConfirm}
                className="w-full rounded-md bg-primary-600 px-3 py-1.5 text-sm font-medium text-white transition-colors hover:bg-primary-500 disabled:cursor-not-allowed disabled:opacity-40"
              >
                {selectedCount < 2
                  ? `Select at least 2 schedules`
                  : `Compare ${selectedCount} schedules`}
              </button>
            </div>
          </div>
        )}
      </div>
    );
  }

  return (
    <div ref={dropdownRef} className={`relative ${className}`}>
      {/* Search input */}
      <div className="relative">
        <input
          ref={inputRef}
          type="text"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          onFocus={handleInputFocus}
          placeholder={placeholder}
          className="w-full rounded-lg border border-slate-600 bg-slate-700 px-3 py-2 pr-8 text-sm text-white placeholder-slate-400 focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          aria-label="Search schedules"
          aria-expanded={isOpen}
          aria-haspopup="listbox"
        />
        <button
          type="button"
          onClick={() => setIsOpen(!isOpen)}
          className="absolute right-2 top-1/2 -translate-y-1/2 text-slate-400 hover:text-white"
          aria-label={isOpen ? 'Close dropdown' : 'Open dropdown'}
        >
          <svg
            className={`h-4 w-4 transition-transform ${isOpen ? 'rotate-180' : ''}`}
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
          </svg>
        </button>
      </div>

      {/* Dropdown */}
      {isOpen && (
        <div
          className="absolute z-50 mt-1 max-h-60 w-full overflow-auto rounded-lg border border-slate-600 bg-slate-800 py-1 shadow-lg"
          role="listbox"
        >
          {isLoading ? (
            <div className="px-3 py-2 text-sm text-slate-400">Loading...</div>
          ) : filteredSchedules.length === 0 ? (
            <div className="px-3 py-2 text-sm text-slate-400">
              {search ? 'No matching schedules' : 'No schedules available'}
            </div>
          ) : (
            filteredSchedules.map((schedule) => (
              <button
                key={schedule.schedule_id}
                type="button"
                onClick={() => handleSelect(schedule)}
                className="flex w-full items-center gap-2 px-3 py-2 text-left text-sm text-slate-200 hover:bg-slate-700 focus:bg-slate-700 focus:outline-none"
                role="option"
              >
                <span className="truncate">{schedule.schedule_name}</span>
              </button>
            ))
          )}
        </div>
      )}
    </div>
  );
}
