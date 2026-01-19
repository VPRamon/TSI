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
  /** Currently selected schedule ID (excluded from list) */
  excludeId?: number;
  /** Callback when a schedule is selected */
  onSelect?: (schedule: ScheduleInfo) => void;
  /** If true, navigates to compare page on selection */
  navigateToCompare?: boolean;
  /** Placeholder text */
  placeholder?: string;
  /** Additional CSS classes */
  className?: string;
}

export function SchedulePicker({
  excludeId,
  onSelect,
  navigateToCompare = false,
  placeholder = 'Select schedule...',
  className = '',
}: SchedulePickerProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [search, setSearch] = useState('');
  const dropdownRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const navigate = useNavigate();
  const { scheduleId } = useParams();

  const { data: schedulesData, isLoading } = useSchedules();

  // Filter schedules based on search and exclude current
  const filteredSchedules =
    schedulesData?.schedules.filter((s) => {
      if (excludeId && s.schedule_id === excludeId) return false;
      if (!search) return true;
      return (
        s.schedule_name.toLowerCase().includes(search.toLowerCase()) ||
        s.schedule_id.toString().includes(search)
      );
    }) ?? [];

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

  const handleInputFocus = () => {
    setIsOpen(true);
  };

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
                <span className="font-mono text-xs text-slate-500">#{schedule.schedule_id}</span>
                <span className="truncate">{schedule.schedule_name}</span>
              </button>
            ))
          )}
        </div>
      )}
    </div>
  );
}
