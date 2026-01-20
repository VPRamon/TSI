/**
 * Main layout component with modern top navigation bar.
 * Professional app shell for analysis workspace.
 */
import { useState, useEffect } from 'react';
import { Outlet, NavLink, useParams, useLocation } from 'react-router-dom';
import { useAppStore } from '@/store';
import { useHealth } from '@/hooks';
import { useScheduleSync, SchedulePicker, AnalysisProvider } from '@/features/schedules';

// Navigation items for schedule-specific views
const scheduleNavItems = [
  { path: 'sky-map', label: 'Sky Map', icon: '🌌' },
  { path: 'distributions', label: 'Distributions', icon: '📊' },
  { path: 'visibility-map', label: 'Visibility Map', icon: '🗺️' },
  { path: 'timeline', label: 'Timeline', icon: '📅' },
  { path: 'insights', label: 'Insights', icon: '💡' },
  { path: 'trends', label: 'Trends', icon: '📈' },
  { path: 'validation', label: 'Validation', icon: '✅' },
];

function Layout() {
  const { scheduleId } = useParams();
  const { selectedSchedule } = useAppStore();
  const { data: health } = useHealth();
  const location = useLocation();
  const isLanding = location.pathname === '/';

  // Sync route scheduleId with Zustand store
  useScheduleSync();

  // Compare picker state
  const [showComparePicker, setShowComparePicker] = useState(false);

  // Mobile menu state
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  // Close mobile menu on route change
  useEffect(() => {
    setMobileMenuOpen(false);
  }, [location.pathname]);

  // Close mobile menu on escape key
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setMobileMenuOpen(false);
        setShowComparePicker(false);
      }
    };
    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, []);

  return (
    <div className="flex min-h-screen flex-col bg-slate-900">
      {/* Skip to main content link for keyboard navigation */}
      <a
        href="#main-content"
        className="sr-only focus:not-sr-only focus:absolute focus:left-4 focus:top-4 focus:z-50 focus:rounded-lg focus:bg-primary-600 focus:px-4 focus:py-2 focus:text-white focus:outline-none focus:ring-2 focus:ring-primary-400"
      >
        Skip to main content
      </a>

      {/* Modern Top Navigation Bar - only shown on non-landing pages */}
      {!isLanding && (
        <header className="sticky top-0 z-30 border-b border-slate-700 bg-slate-800/95 backdrop-blur supports-[backdrop-filter]:bg-slate-800/80">
          <div className="mx-auto w-full max-w-7xl px-4 sm:px-6 lg:px-8">
            {/* Main header row */}
            <div className="flex h-16 items-center justify-between">
              {/* Left section - Logo & Brand */}
              <div className="flex items-center gap-4">
                <NavLink
                  to="/"
                  className="flex items-center gap-2 rounded-lg px-2 py-1.5 transition-colors hover:bg-slate-700/50"
                >
                  <span className="text-2xl">🔭</span>
                  <span className="text-lg font-bold text-white">TSI</span>
                </NavLink>

                {/* Desktop Navigation Links */}
                {scheduleId && (
                  <nav className="hidden items-center gap-1 md:flex" role="navigation" aria-label="Main navigation">
                    {scheduleNavItems.map((item) => (
                      <NavLink
                        key={item.path}
                        to={`/schedules/${scheduleId}/${item.path}`}
                        className={({ isActive }) =>
                          `flex items-center gap-1.5 rounded-lg px-3 py-2 text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 ${
                            isActive
                              ? 'bg-primary-600 text-white'
                              : 'text-slate-300 hover:bg-slate-700/50 hover:text-white'
                          }`
                        }
                      >
                        <span className="text-base">{item.icon}</span>
                        <span>{item.label}</span>
                      </NavLink>
                    ))}
                  </nav>
                )}
              </div>

              {/* Right section - Schedule indicator, Compare, Status */}
              <div className="flex items-center gap-3">
                {/* Current schedule indicator */}
                {scheduleId && (
                  <div className="hidden items-center gap-3 lg:flex">
                    <div className="flex items-center gap-2">
                      <span className="text-sm text-slate-400">Schedule</span>
                      <span className="rounded-md bg-slate-700 px-2.5 py-1 text-sm font-medium text-white">
                        #{scheduleId}
                        {selectedSchedule?.schedule_name && (
                          <span className="ml-1.5 text-slate-400">• {selectedSchedule.schedule_name}</span>
                        )}
                      </span>
                    </div>

                    {/* Compare action */}
                    <div className="relative">
                      <button
                        onClick={() => setShowComparePicker(!showComparePicker)}
                        className="flex items-center gap-1.5 rounded-lg border border-slate-600 bg-slate-700/50 px-3 py-1.5 text-sm text-slate-300 transition-colors hover:bg-slate-700 hover:text-white"
                      >
                        <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                        </svg>
                        Compare
                        <svg className={`h-3 w-3 transition-transform ${showComparePicker ? 'rotate-180' : ''}`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                        </svg>
                      </button>

                      {/* Compare picker dropdown */}
                      {showComparePicker && (
                        <div className="absolute right-0 top-full z-50 mt-2 w-64">
                          <SchedulePicker
                            excludeId={parseInt(scheduleId, 10)}
                            navigateToCompare
                            placeholder="Compare with..."
                            onSelect={() => setShowComparePicker(false)}
                          />
                        </div>
                      )}
                    </div>
                  </div>
                )}

                {/* Connection status indicator */}
                <div className="flex items-center gap-2 rounded-lg bg-slate-700/30 px-3 py-1.5">
                  <span
                    className={`h-2 w-2 rounded-full ${
                      health?.database === 'connected' ? 'bg-emerald-500' : 'bg-red-500'
                    }`}
                    aria-hidden="true"
                  />
                  <span className="hidden text-xs text-slate-400 sm:inline">
                    {health?.database === 'connected' ? 'Connected' : 'Disconnected'}
                  </span>
                </div>

                {/* Mobile menu button */}
                <button
                  onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
                  className="rounded-lg p-2 text-slate-400 hover:bg-slate-700 hover:text-white md:hidden"
                  aria-label={mobileMenuOpen ? 'Close navigation menu' : 'Open navigation menu'}
                  aria-expanded={mobileMenuOpen}
                >
                  {mobileMenuOpen ? (
                    <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  ) : (
                    <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
                    </svg>
                  )}
                </button>
              </div>
            </div>

            {/* Mobile Navigation Menu */}
            {mobileMenuOpen && scheduleId && (
              <nav
                className="border-t border-slate-700 py-3 md:hidden"
                role="navigation"
                aria-label="Mobile navigation"
              >
                <div className="mb-3 flex items-center gap-2 px-1">
                  <span className="text-xs text-slate-400">Schedule</span>
                  <span className="rounded-md bg-slate-700 px-2 py-0.5 text-xs font-medium text-white">
                    #{scheduleId}
                    {selectedSchedule?.schedule_name && (
                      <span className="ml-1 text-slate-400">• {selectedSchedule.schedule_name}</span>
                    )}
                  </span>
                </div>
                <div className="grid grid-cols-2 gap-2">
                  {scheduleNavItems.map((item) => (
                    <NavLink
                      key={item.path}
                      to={`/schedules/${scheduleId}/${item.path}`}
                      className={({ isActive }) =>
                        `flex items-center gap-2 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors ${
                          isActive
                            ? 'bg-primary-600 text-white'
                            : 'text-slate-300 hover:bg-slate-700/50 hover:text-white'
                        }`
                      }
                    >
                      <span>{item.icon}</span>
                      <span>{item.label}</span>
                    </NavLink>
                  ))}
                </div>
              </nav>
            )}
          </div>
        </header>
      )}

      {/* Main content with centered container and side margins */}
      <main
        id="main-content"
        className="flex-1 overflow-auto"
        role="main"
        tabIndex={-1}
      >
        <div className={isLanding ? '' : 'mx-auto w-full max-w-7xl px-4 py-6 sm:px-6 lg:px-8'}>
          {/* Wrap schedule pages with AnalysisProvider for shared filter/selection state */}
          {scheduleId ? (
            <AnalysisProvider syncToUrl>
              <Outlet />
            </AnalysisProvider>
          ) : (
            <Outlet />
          )}
        </div>
      </main>
    </div>
  );
}

export default Layout;
