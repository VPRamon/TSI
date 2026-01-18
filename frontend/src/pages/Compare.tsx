/**
 * Compare page - Compare two schedules.
 */
import { useParams } from 'react-router-dom';
import { useCompare } from '@/hooks';
import { Card, LoadingSpinner, ErrorMessage, MetricCard } from '@/components';

function Compare() {
  const { scheduleId, otherId } = useParams();
  const currentId = parseInt(scheduleId ?? '0', 10);
  const comparisonId = parseInt(otherId ?? '0', 10);
  
  const { data, isLoading, error, refetch } = useCompare(currentId, comparisonId);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (error) {
    return (
      <ErrorMessage
        title="Failed to load comparison"
        message={(error as Error).message}
        onRetry={() => refetch()}
      />
    );
  }

  if (!data) {
    return <ErrorMessage message="No data available" />;
  }

  const changeTypeColors: Record<string, string> = {
    scheduled: 'bg-green-500/20 text-green-400',
    unscheduled: 'bg-red-500/20 text-red-400',
    unchanged: 'bg-slate-500/20 text-slate-400',
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Compare Schedules</h1>
        <p className="text-slate-400 mt-1">
          Comparing {data.current_name} vs {data.comparison_name}
        </p>
      </div>

      {/* Side by side stats */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Current schedule */}
        <Card title={data.current_name}>
          <div className="grid grid-cols-2 gap-4">
            <MetricCard
              label="Scheduled"
              value={data.current_stats.scheduled_count}
              icon="✅"
            />
            <MetricCard
              label="Unscheduled"
              value={data.current_stats.unscheduled_count}
              icon="❌"
            />
            <MetricCard
              label="Mean Priority"
              value={data.current_stats.mean_priority.toFixed(2)}
              icon="⭐"
            />
            <MetricCard
              label="Total Hours"
              value={`${data.current_stats.total_hours.toFixed(1)}h`}
              icon="⏱️"
            />
          </div>
        </Card>

        {/* Comparison schedule */}
        <Card title={data.comparison_name}>
          <div className="grid grid-cols-2 gap-4">
            <MetricCard
              label="Scheduled"
              value={data.comparison_stats.scheduled_count}
              icon="✅"
            />
            <MetricCard
              label="Unscheduled"
              value={data.comparison_stats.unscheduled_count}
              icon="❌"
            />
            <MetricCard
              label="Mean Priority"
              value={data.comparison_stats.mean_priority.toFixed(2)}
              icon="⭐"
            />
            <MetricCard
              label="Total Hours"
              value={`${data.comparison_stats.total_hours.toFixed(1)}h`}
              icon="⏱️"
            />
          </div>
        </Card>
      </div>

      {/* Overlap summary */}
      <Card title="Block Overlap">
        <div className="grid grid-cols-3 gap-4">
          <MetricCard
            label="Common Blocks"
            value={data.common_ids.length}
            icon="🔗"
          />
          <MetricCard
            label={`Only in ${data.current_name}`}
            value={data.only_in_current.length}
            icon="➡️"
          />
          <MetricCard
            label={`Only in ${data.comparison_name}`}
            value={data.only_in_comparison.length}
            icon="⬅️"
          />
        </div>
      </Card>

      {/* Scheduling changes */}
      {data.scheduling_changes.length > 0 && (
        <Card title={`Scheduling Changes (${data.scheduling_changes.length})`}>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-slate-700">
                  <th className="text-left py-3 px-4 text-slate-400">Block ID</th>
                  <th className="text-right py-3 px-4 text-slate-400">Priority</th>
                  <th className="text-center py-3 px-4 text-slate-400">Change</th>
                </tr>
              </thead>
              <tbody>
                {data.scheduling_changes.slice(0, 20).map((change) => (
                  <tr key={change.scheduling_block_id} className="border-b border-slate-700/50">
                    <td className="py-3 px-4 text-white">{change.scheduling_block_id}</td>
                    <td className="py-3 px-4 text-right text-slate-300">
                      {change.priority.toFixed(2)}
                    </td>
                    <td className="py-3 px-4 text-center">
                      <span
                        className={`px-2 py-1 rounded text-xs ${
                          changeTypeColors[change.change_type] || 'bg-slate-500/20 text-slate-400'
                        }`}
                      >
                        {change.change_type}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            {data.scheduling_changes.length > 20 && (
              <p className="text-center text-slate-400 mt-4">
                ... and {data.scheduling_changes.length - 20} more changes
              </p>
            )}
          </div>
        </Card>
      )}
    </div>
  );
}

export default Compare;
