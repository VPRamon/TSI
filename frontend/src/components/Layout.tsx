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
  { 
    path: 'sky-map', 
    label: 'Sky Map', 
    icon: (
      <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
    )
  },
  { 
    path: 'distributions', 
    label: 'Distributions', 
    icon: (
      <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
      </svg>
    )
  },
  { 
    path: 'visibility-map', 
    label: 'Visibility', 
    icon: (
      <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7" />
      </svg>
    )
  },
  { 
    path: 'timeline', 
    label: 'Timeline', 
    icon: (
      <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
      </svg>
    )
  },
  { 
    path: 'insights', 
    label: 'Insights', 
    icon: (
      <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
      </svg>
    )
  },
  { 
    path: 'trends', 
    label: 'Trends', 
    icon: (
      <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
      </svg>
    )
  },
  { 
    path: 'validation', 
    label: 'Validation', 
    icon: (
      <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
    )
  },
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
          <div className="px-4 sm:px-6 lg:px-8">
            {/* Main header row */}
            <div className="flex h-16 items-center justify-between gap-6">
              {/* Left section - Logo & Brand */}
              <div className="flex items-center">
                <NavLink
                  to="/"
                  className="flex shrink-0 items-center gap-2 rounded-lg px-2 py-1.5 transition-colors hover:bg-slate-700/50"
                >
                  <span className="text-2xl">🔭</span>
                  <span className="text-lg font-bold text-white">TSI</span>
                </NavLink>
              </div>

              {/* Center section - Navigation Links */}
              {scheduleId && (
                <nav className="hidden flex-1 items-center justify-center gap-2 md:flex" role="navigation" aria-label="Main navigation">
                  {scheduleNavItems.map((item) => (
                    <NavLink
                      key={item.path}
                      to={`/schedules/${scheduleId}/${item.path}`}
                      className={({ isActive }) =>
                        `flex items-center gap-2 rounded-lg px-4 py-2 text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 ${
                          isActive
                            ? 'bg-primary-600 text-white shadow-sm'
                            : 'text-slate-300 hover:bg-slate-700/50 hover:text-white'
                        }`
                      }
                    >
                      {item.icon}
                      <span>{item.label}</span>
                    </NavLink>
                  ))}
                </nav>
              )}

              {/* Right section - Schedule indicator, Compare, Status */}
              <div className="flex items-center gap-4">
                {/* Current schedule indicator */}
                {scheduleId && (
                  <div className="hidden items-center gap-4 lg:flex">
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
                <div className="flex items-center" title={health?.database === 'connected' ? 'Connected' : 'Disconnected'}>
                  <span
                    className={`h-2.5 w-2.5 rounded-full ${
                      health?.database === 'connected' ? 'bg-emerald-500' : 'bg-red-500'
                    }`}
                    aria-label={health?.database === 'connected' ? 'Connected' : 'Disconnected'}
                  />
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
                      {item.icon}
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
