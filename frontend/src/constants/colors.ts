/**
 * Chart color constants for consistent visualization theming.
 */

// Status colors for scheduled/unscheduled data
export const STATUS_COLORS = {
  scheduled: '#22c55e', // green-500
  unscheduled: '#ef4444', // red-500
  impossible: '#f59e0b', // amber-500
  warning: '#eab308', // yellow-500
} as const;

// Chart background and grid colors (slate theme)
export const CHART_COLORS = {
  background: 'transparent',
  plotBackground: '#1e293b', // slate-800
  gridColor: '#334155', // slate-700
  zeroLineColor: '#475569', // slate-600
  textColor: '#94a3b8', // slate-400
  titleColor: '#ffffff',
} as const;

// Priority bin default colors
export const PRIORITY_COLORS = {
  critical: '#ef4444', // red-500
  high: '#f97316', // orange-500
  medium: '#eab308', // yellow-500
  low: '#22c55e', // green-500
  veryLow: '#3b82f6', // blue-500
} as const;

// Criticality badge colors (Tailwind classes)
export const CRITICALITY_CLASSES = {
  critical: 'bg-red-500/20 text-red-400',
  high: 'bg-orange-500/20 text-orange-400',
  medium: 'bg-yellow-500/20 text-yellow-400',
  low: 'bg-blue-500/20 text-blue-400',
} as const;

// Change type colors for comparison
export const CHANGE_TYPE_COLORS = {
  scheduled: 'bg-green-500/20 text-green-400',
  unscheduled: 'bg-red-500/20 text-red-400',
  unchanged: 'bg-slate-500/20 text-slate-400',
} as const;

export type StatusColorKey = keyof typeof STATUS_COLORS;
export type CriticalityKey = keyof typeof CRITICALITY_CLASSES;
export type ChangeTypeKey = keyof typeof CHANGE_TYPE_COLORS;
