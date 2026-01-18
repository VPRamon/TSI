import { Routes, Route, Navigate } from 'react-router-dom';
import { ErrorBoundary } from './components';
import Layout from './components/Layout';
import Landing from './pages/Landing';
import SkyMap from './pages/SkyMap';
import Distributions from './pages/Distributions';
import Timeline from './pages/Timeline';
import Insights from './pages/Insights';
import Trends from './pages/Trends';
import Validation from './pages/Validation';
import Compare from './pages/Compare';

function App() {
  return (
    <ErrorBoundary>
      <Routes>
        <Route path="/" element={<Layout />}>
          <Route index element={<Landing />} />
          <Route path="schedules/:scheduleId">
            <Route path="sky-map" element={<SkyMap />} />
            <Route path="distributions" element={<Distributions />} />
            <Route path="timeline" element={<Timeline />} />
            <Route path="insights" element={<Insights />} />
            <Route path="trends" element={<Trends />} />
            <Route path="validation" element={<Validation />} />
            <Route path="compare/:otherId" element={<Compare />} />
          </Route>
          <Route path="*" element={<Navigate to="/" replace />} />
        </Route>
      </Routes>
    </ErrorBoundary>
  );
}

export default App;
