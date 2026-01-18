/**
 * Main layout component with navigation sidebar.
 */
import { Outlet, NavLink, useParams, useLocation } from 'react-router-dom';
import { useAppStore } from '@/store';
import { useHealth } from '@/hooks';

// Navigation items for schedule-specific views
const scheduleNavItems = [
  { path: 'sky-map', label: 'Sky Map', icon: '🌌' },
  { path: 'distributions', label: 'Distributions', icon: '📊' },
  { path: 'timeline', label: 'Timeline', icon: '📅' },
  { path: 'insights', label: 'Insights', icon: '💡' },
  { path: 'trends', label: 'Trends', icon: '📈' },
  { path: 'validation', label: 'Validation', icon: '✅' },
];

function Layout() {
  const { scheduleId } = useParams();
  const { sidebarOpen, toggleSidebar } = useAppStore();
  const { data: health } = useHealth();
  const location = useLocation();
  const isLanding = location.pathname === '/';

  return (
    <div className="flex min-h-screen bg-slate-900">
      {/* Sidebar */}
      {!isLanding && (
        <aside
          className={`${
            sidebarOpen ? 'w-64' : 'w-16'
          } flex flex-col border-r border-slate-700 bg-slate-800 transition-all duration-300`}
        >
          {/* Header */}
          <div className="flex items-center justify-between border-b border-slate-700 p-4">
            {sidebarOpen && <h1 className="text-xl font-bold text-white">TSI</h1>}
            <button
              onClick={toggleSidebar}
              className="rounded-lg p-2 text-slate-400 hover:bg-slate-700 hover:text-white"
            >
              {sidebarOpen ? '◀' : '▶'}
            </button>
          </div>

          {/* Navigation */}
          <nav className="flex-1 space-y-2 p-4">
            {/* Home link */}
            <NavLink
              to="/"
              className={({ isActive }) =>
                `flex items-center gap-3 rounded-lg px-3 py-2 transition-colors ${
                  isActive
                    ? 'bg-primary-600 text-white'
                    : 'text-slate-400 hover:bg-slate-700 hover:text-white'
                }`
              }
            >
              <span>🏠</span>
              {sidebarOpen && <span>Home</span>}
            </NavLink>

            {/* Schedule-specific navigation */}
            {scheduleId && (
              <>
                <div className="pb-2 pt-4">
                  {sidebarOpen && (
                    <span className="text-xs font-semibold uppercase text-slate-500">
                      Schedule #{scheduleId}
                    </span>
                  )}
                </div>
                {scheduleNavItems.map((item) => (
                  <NavLink
                    key={item.path}
                    to={`/schedules/${scheduleId}/${item.path}`}
                    className={({ isActive }) =>
                      `flex items-center gap-3 rounded-lg px-3 py-2 transition-colors ${
                        isActive
                          ? 'bg-primary-600 text-white'
                          : 'text-slate-400 hover:bg-slate-700 hover:text-white'
                      }`
                    }
                  >
                    <span>{item.icon}</span>
                    {sidebarOpen && <span>{item.label}</span>}
                  </NavLink>
                ))}
              </>
            )}
          </nav>

          {/* Status footer */}
          <div className="border-t border-slate-700 p-4">
            <div className="flex items-center gap-2">
              <span
                className={`h-2 w-2 rounded-full ${
                  health?.database === 'connected' ? 'bg-green-500' : 'bg-red-500'
                }`}
              />
              {sidebarOpen && (
                <span className="text-xs text-slate-500">
                  {health?.database === 'connected' ? 'Connected' : 'Disconnected'}
                </span>
              )}
            </div>
          </div>
        </aside>
      )}

      {/* Main content */}
      <main className="flex-1 overflow-auto p-6">
        <Outlet />
      </main>
    </div>
  );
}

export default Layout;
