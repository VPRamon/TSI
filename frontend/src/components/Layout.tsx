/**
 * Main layout component with navigation sidebar.
 */
import { Outlet, NavLink, useParams } from 'react-router-dom';
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

  return (
    <div className="flex min-h-screen bg-slate-900">
      {/* Sidebar */}
      <aside
        className={`${
          sidebarOpen ? 'w-64' : 'w-16'
        } transition-all duration-300 bg-slate-800 border-r border-slate-700 flex flex-col`}
      >
        {/* Header */}
        <div className="p-4 border-b border-slate-700 flex items-center justify-between">
          {sidebarOpen && (
            <h1 className="text-xl font-bold text-white">TSI</h1>
          )}
          <button
            onClick={toggleSidebar}
            className="p-2 rounded-lg hover:bg-slate-700 text-slate-400 hover:text-white"
          >
            {sidebarOpen ? '◀' : '▶'}
          </button>
        </div>

        {/* Navigation */}
        <nav className="flex-1 p-4 space-y-2">
          {/* Home link */}
          <NavLink
            to="/"
            className={({ isActive }) =>
              `flex items-center gap-3 px-3 py-2 rounded-lg transition-colors ${
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
              <div className="pt-4 pb-2">
                {sidebarOpen && (
                  <span className="text-xs font-semibold text-slate-500 uppercase">
                    Schedule #{scheduleId}
                  </span>
                )}
              </div>
              {scheduleNavItems.map((item) => (
                <NavLink
                  key={item.path}
                  to={`/schedules/${scheduleId}/${item.path}`}
                  className={({ isActive }) =>
                    `flex items-center gap-3 px-3 py-2 rounded-lg transition-colors ${
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
        <div className="p-4 border-t border-slate-700">
          <div className="flex items-center gap-2">
            <span
              className={`w-2 h-2 rounded-full ${
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

      {/* Main content */}
      <main className="flex-1 p-6 overflow-auto">
        <Outlet />
      </main>
    </div>
  );
}

export default Layout;
