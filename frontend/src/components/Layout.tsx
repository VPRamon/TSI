/**
 * Main layout component with navigation sidebar and top bar.
 * Professional app shell for analysis workspace.
 */
import { useState, useEffect } from 'react';
import { Outlet, NavLink, useParams, useLocation } from 'react-router-dom';
import { useAppStore } from '@/store';
import { useHealth } from '@/hooks';

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
  const { sidebarOpen, toggleSidebar, selectedSchedule } = useAppStore();
  const { data: health } = useHealth();
  const location = useLocation();
  const isLanding = location.pathname === '/';
  
  // Mobile drawer state
  const [mobileDrawerOpen, setMobileDrawerOpen] = useState(false);

  // Close mobile drawer on route change
  useEffect(() => {
    setMobileDrawerOpen(false);
  }, [location.pathname]);

  // Close mobile drawer on escape key
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        setMobileDrawerOpen(false);
      }
    };
    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, []);

  const NavContent = ({ isMobile = false }: { isMobile?: boolean }) => (
    <>
      {/* Navigation */}
      <nav className="flex-1 space-y-1 overflow-y-auto p-3">
        {/* Home link */}
        <NavLink
          to="/"
          className={({ isActive }) =>
            `flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 focus:ring-offset-slate-800 ${
              isActive
                ? 'bg-primary-600 text-white'
                : 'text-slate-300 hover:bg-slate-700/50 hover:text-white'
            }`
          }
        >
          <span className="text-lg">🏠</span>
          {(sidebarOpen || isMobile) && <span>Home</span>}
        </NavLink>

        {/* Schedule-specific navigation */}
        {scheduleId && (
          <>
            <div className="pb-1 pt-4">
              {(sidebarOpen || isMobile) && (
                <span className="px-3 text-xs font-semibold uppercase tracking-wider text-slate-500">
                  Analysis
                </span>
              )}
            </div>
            {scheduleNavItems.map((item) => (
              <NavLink
                key={item.path}
                to={`/schedules/${scheduleId}/${item.path}`}
                className={({ isActive }) =>
                  `flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-2 focus:ring-offset-slate-800 ${
                    isActive
                      ? 'bg-primary-600 text-white'
                      : 'text-slate-300 hover:bg-slate-700/50 hover:text-white'
                  }`
                }
              >
                <span className="text-lg">{item.icon}</span>
                {(sidebarOpen || isMobile) && <span>{item.label}</span>}
              </NavLink>
            ))}
          </>
        )}
      </nav>

      {/* Status footer */}
      <div className="border-t border-slate-700 p-3">
        <div className="flex items-center gap-2 rounded-lg px-3 py-2">
          <span
            className={`h-2 w-2 rounded-full ${
              health?.database === 'connected' ? 'bg-emerald-500' : 'bg-red-500'
            }`}
            aria-hidden="true"
          />
          {(sidebarOpen || isMobile) && (
            <span className="text-xs text-slate-400">
              {health?.database === 'connected' ? 'Connected' : 'Disconnected'}
            </span>
          )}
        </div>
      </div>
    </>
  );

  return (
    <div className="flex min-h-screen flex-col bg-slate-900">
      {/* Skip to main content link for keyboard navigation */}
      <a
        href="#main-content"
        className="sr-only focus:not-sr-only focus:absolute focus:left-4 focus:top-4 focus:z-50 focus:rounded-lg focus:bg-primary-600 focus:px-4 focus:py-2 focus:text-white focus:outline-none focus:ring-2 focus:ring-primary-400"
      >
        Skip to main content
      </a>

      {/* Top bar - only shown on non-landing pages */}
      {!isLanding && (
        <header className="sticky top-0 z-30 flex h-14 shrink-0 items-center justify-between border-b border-slate-700 bg-slate-800/95 px-4 backdrop-blur supports-[backdrop-filter]:bg-slate-800/80">
          {/* Left section */}
          <div className="flex items-center gap-3">
            {/* Mobile menu button */}
            <button
              onClick={() => setMobileDrawerOpen(true)}
              className="rounded-lg p-2 text-slate-400 hover:bg-slate-700 hover:text-white lg:hidden"
              aria-label="Open navigation menu"
            >
              <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
              </svg>
            </button>
            
            {/* App title */}
            <div className="flex items-center gap-2">
              <span className="text-xl">🔭</span>
              <span className="font-semibold text-white">TSI</span>
            </div>
          </div>

          {/* Center - Current schedule indicator */}
          {scheduleId && (
            <div className="hidden items-center gap-2 sm:flex">
              <span className="text-sm text-slate-400">Schedule</span>
              <span className="rounded-md bg-slate-700 px-2.5 py-1 text-sm font-medium text-white">
                #{scheduleId}
                {selectedSchedule?.schedule_name && (
                  <span className="ml-1.5 text-slate-400">• {selectedSchedule.schedule_name}</span>
                )}
              </span>
            </div>
          )}

          {/* Right section - Desktop sidebar toggle */}
          <div className="hidden lg:block">
            <button
              onClick={toggleSidebar}
              className="rounded-lg p-2 text-slate-400 hover:bg-slate-700 hover:text-white"
              aria-label={sidebarOpen ? 'Collapse sidebar' : 'Expand sidebar'}
              aria-expanded={sidebarOpen}
            >
              <svg
                className={`h-5 w-5 transition-transform ${sidebarOpen ? '' : 'rotate-180'}`}
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
              </svg>
            </button>
          </div>
        </header>
      )}

      <div className="flex flex-1">
        {/* Desktop Sidebar */}
        {!isLanding && (
          <aside
            className={`hidden lg:flex ${
              sidebarOpen ? 'w-56' : 'w-16'
            } flex-col border-r border-slate-700 bg-slate-800 transition-all duration-200`}
            role="navigation"
            aria-label="Main navigation"
          >
            <NavContent />
          </aside>
        )}

        {/* Mobile Drawer Overlay */}
        {!isLanding && mobileDrawerOpen && (
          <div
            className="fixed inset-0 z-40 bg-slate-900/80 backdrop-blur-sm lg:hidden"
            onClick={() => setMobileDrawerOpen(false)}
            aria-hidden="true"
          />
        )}

        {/* Mobile Drawer */}
        {!isLanding && (
          <aside
            className={`fixed inset-y-0 left-0 z-50 flex w-64 flex-col bg-slate-800 transition-transform duration-200 lg:hidden ${
              mobileDrawerOpen ? 'translate-x-0' : '-translate-x-full'
            }`}
            role="navigation"
            aria-label="Mobile navigation"
            aria-hidden={!mobileDrawerOpen}
          >
            {/* Drawer header */}
            <div className="flex h-14 items-center justify-between border-b border-slate-700 px-4">
              <div className="flex items-center gap-2">
                <span className="text-xl">🔭</span>
                <span className="font-semibold text-white">TSI</span>
              </div>
              <button
                onClick={() => setMobileDrawerOpen(false)}
                className="rounded-lg p-2 text-slate-400 hover:bg-slate-700 hover:text-white"
                aria-label="Close navigation menu"
              >
                <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            {/* Schedule indicator in drawer */}
            {scheduleId && (
              <div className="border-b border-slate-700 px-4 py-3">
                <span className="text-xs text-slate-400">Current Schedule</span>
                <p className="mt-0.5 font-medium text-white">
                  #{scheduleId}
                  {selectedSchedule?.schedule_name && (
                    <span className="ml-1 text-slate-400">• {selectedSchedule.schedule_name}</span>
                  )}
                </p>
              </div>
            )}

            <NavContent isMobile />
          </aside>
        )}

        {/* Main content */}
        <main
          id="main-content"
          className={`flex-1 overflow-auto ${isLanding ? '' : 'p-4 sm:p-6'}`}
          role="main"
          tabIndex={-1}
        >
          <Outlet />
        </main>
      </div>
    </div>
  );
}

export default Layout;
