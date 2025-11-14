/**
 * Application-wide constants
 */

export const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8081/api/v1'

export const PRIORITY_BIN_COLORS: Record<string, string> = {
  'Low (<10)': '#3b82f6',
  'High (10+)': '#ef4444',
  'Unknown': '#9ca3af'
}

export const STATUS_COLORS: Record<string, string> = {
  'Scheduled': '#10b981',
  'Unscheduled': '#f59e0b'
}

export const SEVERITY_COLORS: Record<string, string> = {
  'high': 'bg-red-100 text-red-800',
  'medium': 'bg-yellow-100 text-yellow-800',
  'low': 'bg-blue-100 text-blue-800'
}

export const CONFLICT_TYPE_COLORS: Record<string, string> = {
  'impossible_observation': 'bg-red-100 text-red-800',
  'insufficient_visibility': 'bg-yellow-100 text-yellow-800',
  'scheduling_anomaly': 'bg-orange-100 text-orange-800'
}

export const CHART_DEFAULTS = {
  HEIGHT: 600,
  MIN_SYMBOL_SIZE: 5,
  MAX_SYMBOL_SIZE: 20,
  SYMBOL_SIZE_MULTIPLIER: 3
}

export const POLL_INTERVAL = 2000 // ms
