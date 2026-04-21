import { Suspense, lazy } from 'react';
import { Routes, Route, Navigate, useParams } from 'react-router-dom';
import { ErrorBoundary, LoadingSpinner } from './components';
import Layout from './components/Layout';

// Lazy load pages for better initial bundle size
const Landing = lazy(() => import('./pages/Landing'));
const SkyMap = lazy(() => import('./pages/SkyMap'));
const Distributions = lazy(() => import('./pages/Distributions'));
const Timeline = lazy(() => import('./pages/Timeline'));
const Insights = lazy(() => import('./pages/Insights'));
const Fragmentation = lazy(() => import('./pages/Fragmentation'));
const Trends = lazy(() => import('./pages/Trends'));
const Validation = lazy(() => import('./pages/Validation'));
const Compare = lazy(() => import('./pages/Compare'));
const VisibilityMap = lazy(() => import('./pages/VisibilityMap'));
const AltAz = lazy(() => import('./pages/AltAz'));
const ScheduleManagement = lazy(() => import('./pages/ScheduleManagement'));

// Loading fallback component
function PageLoader() {
  return (
    <div className="flex h-full min-h-[400px] items-center justify-center">
      <LoadingSpinner size="lg" />
    </div>
  );
}

function OldCompareRedirect() {
  const { scheduleId, otherId } = useParams<{ scheduleId: string; otherId: string }>();
  return <Navigate to={`/compare?ids=${scheduleId},${otherId}`} replace />;
}

function App() {
  return (
    <ErrorBoundary>
      <Suspense fallback={<PageLoader />}>
        <Routes>
          <Route path="/" element={<Layout />}>
            <Route index element={<Landing />} />
            <Route path="manage" element={<ScheduleManagement />} />
            <Route path="compare" element={<Compare />} />
            <Route path="schedules/:scheduleId">
              <Route path="sky-map" element={<SkyMap />} />
              <Route path="distributions" element={<Distributions />} />
              <Route path="visibility-map" element={<VisibilityMap />} />
              <Route path="timeline" element={<Timeline />} />
              <Route path="insights" element={<Insights />} />
              <Route path="fragmentation" element={<Fragmentation />} />
              <Route path="trends" element={<Trends />} />
              <Route path="alt-az" element={<AltAz />} />
              <Route path="validation" element={<Validation />} />
              <Route path="compare/:otherId" element={<OldCompareRedirect />} />
            </Route>
            <Route path="*" element={<Navigate to="/" replace />} />
          </Route>
        </Routes>
      </Suspense>
    </ErrorBoundary>
  );
}

export default App;
