/**
 * Insights page - Analytics and key metrics.
 * Redesigned with consistent layout primitives and improved table presentation.
 */
import { useParams } from 'react-router-dom';
import { useInsights } from '@/hooks';
import {
  LoadingSpinner,
  ErrorMessage,
  MetricCard,
  PageHeader,
  PageContainer,
  MetricsGrid,
} from '@/components';

function Insights() {
  const { scheduleId } = useParams();
  const id = parseInt(scheduleId ?? '0', 10);
  const { data, isLoading, error, refetch } = useInsights(id);

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (error) {
    return (
      <ErrorMessage
        title="Failed to load insights"
        message={(error as Error).message}
        onRetry={() => refetch()}
      />
    );
  }

  if (!data) {
    return <ErrorMessage message="No data available" />;
  }

  const { metrics } = data;

  return (
    <PageContainer>
      {/* Header */}
      <PageHeader
        title="Insights"
        description="Key analytics and performance metrics"
      />

      {/* Key metrics */}
      <MetricsGrid>
        <MetricCard label="Total Observations" value={metrics.total_observations} icon="🎯" />
        <MetricCard label="Scheduled" value={metrics.scheduled_count} icon="✅" />
        <MetricCard label="Unscheduled" value={metrics.unscheduled_count} icon="❌" />
        <MetricCard
          label="Scheduling Rate"
          value={`${(metrics.scheduling_rate * 100).toFixed(1)}%`}
          icon="📊"
        />
      </MetricsGrid>

      {/* Two-column layout for Priority Analysis and Correlations on desktop */}
      <div className="grid gap-6 lg:grid-cols-2">
        {/* Priority analysis */}
        <section className="rounded-lg border border-slate-700 bg-slate-800/30 p-4">
          <h2 className="mb-4 text-sm font-medium text-slate-300">Priority Analysis</h2>
          <div className="grid grid-cols-2 gap-3">
            <div className="rounded-md bg-slate-700/30 p-3">
              <p className="text-xs text-slate-400">Mean Priority</p>
              <p className="text-xl font-semibold text-white">
                {metrics.mean_priority.toFixed(2)}
              </p>
            </div>
            <div className="rounded-md bg-slate-700/30 p-3">
              <p className="text-xs text-slate-400">Median Priority</p>
              <p className="text-xl font-semibold text-white">
                {metrics.median_priority.toFixed(2)}
              </p>
            </div>
            <div className="rounded-md bg-slate-700/30 p-3">
              <p className="text-xs text-slate-400">Scheduled Mean</p>
              <p className="text-xl font-semibold text-white">
                {metrics.mean_priority_scheduled.toFixed(2)}
              </p>
            </div>
            <div className="rounded-md bg-slate-700/30 p-3">
              <p className="text-xs text-slate-400">Unscheduled Mean</p>
              <p className="text-xl font-semibold text-white">
                {metrics.mean_priority_unscheduled.toFixed(2)}
              </p>
            </div>
          </div>
        </section>

        {/* Correlations */}
        <section className="rounded-lg border border-slate-700 bg-slate-800/30 p-4">
          <h2 className="mb-4 text-sm font-medium text-slate-300">Correlations</h2>
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-1 xl:grid-cols-2">
            {data.correlations.map((corr) => (
              <div
                key={`${corr.variable1}-${corr.variable2}`}
                className="rounded-md bg-slate-700/30 p-3"
              >
                <p className="text-xs text-slate-400">
                  {corr.variable1} ↔ {corr.variable2}
                </p>
                <p
                  className={`text-xl font-bold ${
                    corr.correlation > 0 ? 'text-emerald-400' : 'text-red-400'
                  }`}
                >
                  {corr.correlation.toFixed(3)}
                </p>
              </div>
            ))}
          </div>
        </section>
      </div>

      {/* Top observations by priority */}
      <section className="rounded-lg border border-slate-700 bg-slate-800/30">
        <div className="border-b border-slate-700/50 px-4 py-3">
          <h2 className="text-sm font-medium text-slate-300">Top Observations by Priority</h2>
        </div>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-slate-700/50">
                <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wide text-slate-500">
                  Block ID
                </th>
                <th className="px-4 py-3 text-right text-xs font-medium uppercase tracking-wide text-slate-500">
                  Priority
                </th>
                <th className="px-4 py-3 text-right text-xs font-medium uppercase tracking-wide text-slate-500">
                  Visibility
                </th>
                <th className="px-4 py-3 text-right text-xs font-medium uppercase tracking-wide text-slate-500">
                  Requested
                </th>
                <th className="px-4 py-3 text-center text-xs font-medium uppercase tracking-wide text-slate-500">
                  Status
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-700/30">
              {data.top_priority.slice(0, 10).map((obs) => (
                <tr key={obs.scheduling_block_id} className="hover:bg-slate-700/20">
                  <td className="px-4 py-3 font-medium text-white">{obs.original_block_id}</td>
                  <td className="px-4 py-3 text-right tabular-nums text-white">
                    {obs.priority.toFixed(2)}
                  </td>
                  <td className="px-4 py-3 text-right tabular-nums text-slate-300">
                    {obs.total_visibility_hours.toFixed(1)}h
                  </td>
                  <td className="px-4 py-3 text-right tabular-nums text-slate-300">
                    {obs.requested_hours.toFixed(1)}h
                  </td>
                  <td className="px-4 py-3 text-center">
                    <span
                      className={`inline-flex rounded-full px-2 py-0.5 text-xs font-medium ${
                        obs.scheduled
                          ? 'bg-emerald-500/20 text-emerald-400'
                          : 'bg-red-500/20 text-red-400'
                      }`}
                    >
                      {obs.scheduled ? 'Scheduled' : 'Unscheduled'}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {/* Conflicts */}
      {data.conflicts.length > 0 && (
        <section className="rounded-lg border border-slate-700 bg-slate-800/30">
          <div className="border-b border-slate-700/50 px-4 py-3">
            <h2 className="text-sm font-medium text-slate-300">
              Scheduling Conflicts ({data.conflicts.length})
            </h2>
          </div>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-slate-700/50">
                  <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wide text-slate-500">
                    Block 1
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wide text-slate-500">
                    Block 2
                  </th>
                  <th className="px-4 py-3 text-right text-xs font-medium uppercase tracking-wide text-slate-500">
                    Overlap
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-700/30">
                {data.conflicts.slice(0, 10).map((conflict, index) => (
                  <tr key={index} className="hover:bg-slate-700/20">
                    <td className="px-4 py-3 font-medium text-white">{conflict.block_id_1}</td>
                    <td className="px-4 py-3 font-medium text-white">{conflict.block_id_2}</td>
                    <td className="px-4 py-3 text-right tabular-nums text-red-400">
                      {conflict.overlap_hours.toFixed(2)}h
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>
      )}
    </PageContainer>
  );
}

export default Insights;
