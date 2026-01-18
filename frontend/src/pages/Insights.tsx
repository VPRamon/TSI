/**
 * Insights page - Analytics and key metrics.
 */
import { useParams } from 'react-router-dom';
import { useInsights } from '@/hooks';
import { Card, LoadingSpinner, ErrorMessage, MetricCard } from '@/components';

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
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Insights</h1>
        <p className="mt-1 text-slate-400">Key analytics and performance metrics</p>
      </div>

      {/* Key metrics */}
      <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
        <MetricCard label="Total Observations" value={metrics.total_observations} icon="🎯" />
        <MetricCard label="Scheduled" value={metrics.scheduled_count} icon="✅" />
        <MetricCard label="Unscheduled" value={metrics.unscheduled_count} icon="❌" />
        <MetricCard
          label="Scheduling Rate"
          value={`${(metrics.scheduling_rate * 100).toFixed(1)}%`}
          icon="📊"
        />
      </div>

      {/* Priority analysis */}
      <Card title="Priority Analysis">
        <div className="grid grid-cols-2 gap-4 md:grid-cols-4">
          <MetricCard label="Mean Priority" value={metrics.mean_priority.toFixed(2)} />
          <MetricCard label="Median Priority" value={metrics.median_priority.toFixed(2)} />
          <MetricCard label="Scheduled Mean" value={metrics.mean_priority_scheduled.toFixed(2)} />
          <MetricCard
            label="Unscheduled Mean"
            value={metrics.mean_priority_unscheduled.toFixed(2)}
          />
        </div>
      </Card>

      {/* Top by priority */}
      <Card title="Top Observations by Priority">
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-slate-700">
                <th className="px-4 py-3 text-left text-slate-400">Block ID</th>
                <th className="px-4 py-3 text-right text-slate-400">Priority</th>
                <th className="px-4 py-3 text-right text-slate-400">Visibility (h)</th>
                <th className="px-4 py-3 text-right text-slate-400">Requested (h)</th>
                <th className="px-4 py-3 text-center text-slate-400">Status</th>
              </tr>
            </thead>
            <tbody>
              {data.top_priority.slice(0, 10).map((obs) => (
                <tr key={obs.scheduling_block_id} className="border-b border-slate-700/50">
                  <td className="px-4 py-3 text-white">{obs.original_block_id}</td>
                  <td className="px-4 py-3 text-right text-white">{obs.priority.toFixed(2)}</td>
                  <td className="px-4 py-3 text-right text-slate-300">
                    {obs.total_visibility_hours.toFixed(1)}
                  </td>
                  <td className="px-4 py-3 text-right text-slate-300">
                    {obs.requested_hours.toFixed(1)}
                  </td>
                  <td className="px-4 py-3 text-center">
                    <span
                      className={`rounded px-2 py-1 text-xs ${
                        obs.scheduled
                          ? 'bg-green-500/20 text-green-400'
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
      </Card>

      {/* Correlations */}
      <Card title="Correlations">
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
          {data.correlations.map((corr) => (
            <div
              key={`${corr.variable1}-${corr.variable2}`}
              className="rounded-lg bg-slate-700/50 p-4"
            >
              <p className="text-sm text-slate-400">
                {corr.variable1} ↔ {corr.variable2}
              </p>
              <p
                className={`text-2xl font-bold ${
                  corr.correlation > 0 ? 'text-green-400' : 'text-red-400'
                }`}
              >
                {corr.correlation.toFixed(3)}
              </p>
            </div>
          ))}
        </div>
      </Card>

      {/* Conflicts */}
      {data.conflicts.length > 0 && (
        <Card title={`Scheduling Conflicts (${data.conflicts.length})`}>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-slate-700">
                  <th className="px-4 py-3 text-left text-slate-400">Block 1</th>
                  <th className="px-4 py-3 text-left text-slate-400">Block 2</th>
                  <th className="px-4 py-3 text-right text-slate-400">Overlap (h)</th>
                </tr>
              </thead>
              <tbody>
                {data.conflicts.slice(0, 10).map((conflict, index) => (
                  <tr key={index} className="border-b border-slate-700/50">
                    <td className="px-4 py-3 text-white">{conflict.block_id_1}</td>
                    <td className="px-4 py-3 text-white">{conflict.block_id_2}</td>
                    <td className="px-4 py-3 text-right text-red-400">
                      {conflict.overlap_hours.toFixed(2)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>
      )}
    </div>
  );
}

export default Insights;
