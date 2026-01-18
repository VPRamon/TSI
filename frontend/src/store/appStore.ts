/**
 * Global application state using Zustand.
 */
import { create } from 'zustand';
import type { ScheduleInfo } from '@/api/types';

interface AppState {
  // Currently selected schedule
  selectedSchedule: ScheduleInfo | null;
  setSelectedSchedule: (schedule: ScheduleInfo | null) => void;

  // Comparison schedule (for compare page)
  comparisonSchedule: ScheduleInfo | null;
  setComparisonSchedule: (schedule: ScheduleInfo | null) => void;

  // UI state
  sidebarOpen: boolean;
  toggleSidebar: () => void;
  setSidebarOpen: (open: boolean) => void;
}

export const useAppStore = create<AppState>((set) => ({
  // Selected schedule
  selectedSchedule: null,
  setSelectedSchedule: (schedule) => set({ selectedSchedule: schedule }),

  // Comparison schedule
  comparisonSchedule: null,
  setComparisonSchedule: (schedule) => set({ comparisonSchedule: schedule }),

  // UI state
  sidebarOpen: true,
  toggleSidebar: () => set((state) => ({ sidebarOpen: !state.sidebarOpen })),
  setSidebarOpen: (open) => set({ sidebarOpen: open }),
}));
